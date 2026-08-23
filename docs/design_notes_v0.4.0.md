# Design Notes v0.4.0 — Delta Against the v0.3.1 Document

**Date:** August 2026
**Supersedes:** nothing. `chromatic_geodesics_v0.3.1.docx` remains the project's design
document and is unmodified. This file records what the implementation taught that the
design document does not contain, and where the two have diverged.

The v0.3.1 document was written before Milestone 1. Two milestones of implementation and
three experiments later, parts of it describe a system that was not built, and it has no
record of the findings that only came out of building. Rather than rewrite a document
whose value is partly as a dated statement of intent, this delta sits alongside it.

---

## A. Naming reconciliation

The design document's §4.1 module table names types that do not exist. Nothing here is a
regression — the code took a simpler route — but a reader coming from the document to the
code will not find what it describes.

| v0.3.1 §4.1 | In code | Notes |
|---|---|---|
| `RiemannianMetric` (trait) | `Metric` (`src/metric.rs:5`) | Renamed. Same signature: `cost(&self, p: &Point3, dir: &Vector3) -> f64` |
| `EuclideanMetric` | `EuclideanMetric` | As specified |
| `DomainMetric` | `HeatFlowMetric`, `KubelkaMunkMetric` | Split into two concrete metrics rather than one parameterised type |
| `BoundaryRepulsive` | — | Not implemented. Required by Experiment 3 |
| `CustomMetric` | — | Not implemented. Users implement `Metric` directly, which serves the same purpose |
| `AnisotropicMetric` (§7 M3) | — | Not implemented, and **blocked** — see §B.1 |
| `Space` (trait), `CubeSpace` | `Space`, `CubeSpace` (`src/space.rs`) | As specified, but see §C.3 |
| `Point3`, `Vector3` | `pub type Point3 = [f64; 3]` (`src/types.rs`) | Type aliases rather than structs |
| `Region` (interior/face/edge/corner) | — | Not implemented. `CubeSpace` offers only `boundary_distance` |
| `Grid3D`, `DistanceField`, `FastMarcher`, `GeodesicExtractor` | — | See §A.1 |
| `GeodesicPath` | `GeodesicPath { points, total_cost }` (`src/path.rs`) | Present but minimal |
| `PathSegment`, `PathAnalysis` | — | Not implemented. Milestone 4 |
| `JsonExporter` | `GeodesicOutput::to_json` (`src/export.rs`) | Function rather than type |
| `CsvExporter` | — | Not implemented; no demand for it has arisen |

### A.1 The solver is functions, not objects

The document proposes a solver built from four types. The code is four free functions over
local `Vec`s: `solve`, `solve_local`, `update_neighbours`, `axis_min`, plus `extract_path`
and `resample_by_cost` (`src/solver.rs`).

This has cost nothing so far and is easier to read. It has one real consequence: **the
distance field is a local variable.** `solve` allocates a full `Vec<f64>` of N³ costs,
extracts one path from it, and drops it (`src/solver.rs:72-76`). Nothing else can reach
it. That blocks re-extracting to a second target without re-running the FMM, exporting the
field for visualisation, and all of the Milestone 4 analysis tooling. Hardening item H2
addresses it; whether the fix is a returned struct or a `FastMarcher` object as the
document envisaged is an open design choice.

---

## B. Findings the design document does not contain

Three results came out of implementation. All three are constraints, and all three
generalise beyond the experiment that produced them.

### B.1 The solver is isotropic — and this inverts a roadmap dependency

`update_neighbours` (`src/solver.rs:172-182`) gathers the minimum accepted cost along each
axis, evaluates `metric.cost(&point, &direction)` at the single axis direction it is
stepping in, and passes **one scalar** into `solve_local`. The eikonal update

    ∑ᵢ (t − aᵢ)² = (h·f)²

treats `f` as a scalar — the same friction in every direction at that cell. All
directional information in the metric is discarded.

This matters because the design document prescribes the anisotropic form

    cost(p, v) = ‖v‖ × (1 + λ · sin²θ)

for **both** Experiment 1 (§8.2.2) and Experiment 2 (§8.3.2). That form is not generally
supported by the solver that was built. Anisotropic metrics work only when their preferred
direction happens to align with the grid axes. `KubelkaMunkMetric` had to be reformulated
as position-dependent *isotropic* friction

    cost(p, v) = ‖v‖ × (1 + λ · d²)

where d is the distance to the K-M mixing curve — encoding the physics in the shape of the
landscape rather than in directional preference. (Experiment 1 was left in the sin²θ form
and passed, but `experiment_1_findings.md` §3 now questions how much its metric was
contributing at all.)

**The roadmap dependency this creates is not recorded in the document.** §7 places
`AnisotropicMetric` in Milestone 3 and the tensor eikonal solver in Milestone 7. Milestone
3 cannot be done without Milestone 7. Either the solver work moves forward, or Milestone 3
is restricted to isotropic metrics — which is the natural reading, since
`BoundaryRepulsive` (§8.4) is isotropic and Experiment 3 needs nothing more.

### B.2 Local friction cannot enforce global constraints

Experiment 1's conservation prediction fails (deviation 0.0713 against ±0.02), and raising
λ does not reduce it. A local, per-point, per-direction cost creates a *preference*, not a
*prohibition*: the penalty for leaving the conservation surface is always finite, so the
geodesic will always trade some of it for a shortcut.

This is a boundary on the formulation itself, not on the implementation, and it will
recur in every domain with a conservation law. The design document's framing — friction
encodes the domain — needs the qualifier that it encodes the domain's *local* structure
only. Whether a constraint-augmented metric can recover global constraints is now an open
question (§D).

### B.3 The readout is a separate source of error from the solver

Twice now the distance field has been correct while the extracted path was not:

- **P2.3**: `extract_path` emitted points at uniform arclength, which erases the cost
  parametrization by construction. The nonlinearity the prediction tested for was present
  in the T values all along. Fixed by `resample_by_cost`.
- **P0.3**: `extract_path` snaps to the nearest grid cell before reading the gradient
  (`src/solver.rs:250-254`), so the gradient is piecewise constant per cell and the descent
  direction chatters at cell scale. Under a flat metric, where the true gradient is
  constant, *all* measured curvature is this artefact. Not yet fixed.

The generalisable form: **solver accuracy and readout fidelity are independent, and the
project has no instrumentation that distinguishes them.** Every experiment measures the
extracted path and attributes what it finds to the framework. Milestone 4's path analysis
must be built on the cost parametrization (hardening item H3) rather than on the geometric
image alone, or it will inherit the same class of error a third time.

---

## C. Corrections to the experiment design (§8)

### C.1 Experiment 1's configuration is degenerate

The initial profile (0.2, 0.9, 0.2) is symmetric, and once the mean is subtracted it is
proportional to (−1, 2, −1) — an eigenvector of the discrete Laplacian. A single excited
mode decays as a whole, so the exact trajectory is a **straight line** to the mean.

P1.1 ("trajectory match < 0.05") therefore tests only straightness, which is what
Experiment 0 already tests and what the solver produces by default. A Euclidean metric
would pass it. The prediction cannot fail informatively.

The fix is an asymmetric initial profile exciting more than one Fourier mode: the modes
decay at different rates, the fast one dies first, and the trajectory bends. Full argument
in `docs/experiment_1_findings.md` §1.

### C.2 Gate 0 is not met as pre-registered

§8.6 makes Experiment 0 a strict precondition. On first actual measurement, P0.2 fails
(cost overestimated 1.86% against a ±1% band) and P0.3 fails (curvature chatter). P0.1
passes at 0.0124 < 0.0156.

Neither failure is capable of producing a qualitatively wrong path — the deviation that
matters for P1.1 and P2.1 is three times inside their tolerances — so Experiments 1 and 2
are not invalidated. But the solver is less accurate than assumed, and P0.2's ±1%
tolerance is a promise a first-order upwind scheme cannot keep at fixed resolution. A
convergence-rate prediction would be the honest restatement; changing a pre-registered
threshold after seeing the result must be recorded as such.

Consequence worth flagging early: **Experiment 3's P3.3 requires a cost ratio > 1.05**, a
5% effect measured against a solver carrying a ~2% systematic cost bias. That prediction
is not currently measurable at the precision it assumes.

### C.3 `Space` is decorative

`solve` takes `_space: &impl Space` and never uses it (`src/solver.rs:20`). Boundaries come
entirely from grid index bounds. `CubeSpace::contains` and `boundary_distance` are dead
code as far as the solver is concerned.

This matters for Milestone 7: a non-cubic container is unimplementable while the solver
assumes its grid *is* the domain. The trait exists; the seam does not.

---

## D. Open questions, revised

The document's §9 list stands. Implementation added these:

- **Can a constraint-augmented metric enforce a global constraint?** Or is conservation
  inherently outside what any local friction can express (§B.2)? If the latter, the
  framework has a characterisable class of domains it cannot encode faithfully, and
  naming that class is valuable in itself.
- **What is "path quality"?** Milestone 5's learned friction needs an objective function
  to update the metric against, and the project has no definition of one. This is the
  gating unknown for the entire learning ambition, not an implementation detail.
- **How much did the metric contribute?** No experiment has been run against
  `EuclideanMetric` as a control. Given §C.1, this is not a rhetorical question: the
  cheapest informative experiment available is re-running Experiment 1's configuration
  under a flat metric and measuring the difference.

The last of these suggests a general practice the design document does not require:
**every domain experiment should report its Euclidean control.** The framework's claim is
that domain-derived friction produces domain-faithful geodesics; without the control, a
passing prediction cannot distinguish that from the geodesic being straight anyway.

---

## E. Milestone status

| Milestone | Document status | Actual |
|---|---|---|
| M1 Foundation | ● | **Complete.** Gate 0 passes only partially — see §C.2 |
| M2 Analog computer validation | ● | **Complete, with qualification.** Gate 2 passed. Experiment 2 is sound; Experiment 1 is degenerate (§C.1), so the claim rests on one strong domain rather than two |
| M3 Structured friction | ○ | Next. Restrict to isotropic metrics: `BoundaryRepulsive` + Experiment 3. `AnisotropicMetric` blocked on M7 solver work (§B.1) |
| M4 Path analysis | ○ | Needs H2 (distance field) and H3 (per-point cost) first |
| M5 Learned friction | ○ | Blocked on defining path quality (§D) |
| M6 Domain encoding | ○ | Unchanged |
| M7 Alternative geometries | ○ | Also now the home of the tensor eikonal solver, which M3 depends on. Blocked on §C.3 |

Ahead of all of these sits the hardening work — see `docs/project_status.md`. Every result
above was discovered by measuring things by hand that no test measures. That is the gap to
close first.
