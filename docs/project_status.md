# keamogetswe — Project Status

**Date:** August 2026
**Version:** 0.1.0 (prototype)
**Last pass:** documentation reconciliation; all three experiments re-measured

## What exists

### Working code

- **Space module** (`src/space.rs`): `CubeSpace` with `contains` and `boundary_distance`.
  The `Space` trait is designed for future geometries — but the solver never calls it
  (see Known Limitations).
- **Metric module** (`src/metric.rs`): three implementations of the `Metric` trait —
  - `EuclideanMetric` — uniform cost, baseline reference
  - `HeatFlowMetric` — anisotropic sin²θ penalty against the heat-flow direction
  - `KubelkaMunkMetric` — isotropic, low-friction valley along the K-M mixing curve
- **Solver module** (`src/solver.rs`): complete FMM — eikonal update (`solve_local`),
  min-heap wavefront (`BinaryHeap<Reverse<FmmEntry>>`), gradient-descent path extraction,
  and cost-increment resampling (`resample_by_cost`).
- **Export module** (`src/export.rs`): JSON serialisation matching `schemas/geodesic_path.json`.
- **Viewer** (`viewer/index.html`): standalone Three.js viewer, no build step. Renders
  RGB-coloured paths in a wireframe cube, multiple paths per file, drag-and-drop JSON.

### Experiment results (measured August 2026)

| Experiment | Domain | P·1 | P·2 | P·3 | Overall |
|---|---|---|---|---|---|
| 0: Euclidean | none (calibration) | 0.0124 **PASS** | +1.86% **FAIL** | chatter **FAIL** | **PARTIAL** |
| 1: Heat flow | thermal diffusion | 0.0437 **PASS**¹ | **PASS** | 0.0713 **FAIL** | **PARTIAL** |
| 2: K-M mixing | subtractive colour | 0.0371 **PASS** | **PASS** | 9.06 **PASS** | **PASS** |

¹ P1.1 passes but has almost no discriminating power — the configuration is degenerate.

These numbers differ from what was recorded before this pass. Experiments 0 and 1 had
never been measured against their full prediction sets; Experiment 0 has no validation
code at all, and Experiment 1's P1.3 was recorded as "qualified" when it is a failure at
3.5× tolerance. Experiment 2's numbers are unchanged and reproduce exactly.

### Documents

All in `docs/`:

- `experiment_0_findings.md` — calibration; the two failures and what they mean
- `experiment_1_findings.md` — heat flow; conservation and the degeneracy finding
- `experiment_2_findings.md` — K-M mixing; the isotropic reformulation and the P2.3 fix
- `rca_report.md` — independent RCA of P2.3 (22 July 2026), with a resolution section
- `design_notes_v0.4.0.md` — delta against the v0.3.1 design document
- `../chromatic_geodesics_v0.3.1.docx` — the design document itself, unmodified
- `../schemas/geodesic_path.json` — JSON contract between crate and viewer

## What we've learned

### The framework works as an analog computer — on one solid domain

Domain-derived friction produces domain-faithful geodesics. Experiment 2 demonstrates
this convincingly: the K-M mixing curve is genuinely non-straight, the geodesic tracks it
to within 0.037, and it arcs through green where the Euclidean path passes through grey.
That result could have failed and did not.

Experiment 1 was believed to be independent corroboration. It is weaker than that. Its
initial profile (0.2, 0.9, 0.2) is an eigenvector of the discrete Laplacian, so the exact
heat trajectory is a straight line — and a straight line is what the solver produces by
default under any metric. P1.1 cannot fail informatively on this configuration. The core
claim is supported, but by one domain rather than two, until Experiment 1 is rebuilt on
an asymmetric profile.

### Local friction cannot enforce global constraints

Energy conservation (T₁ + T₂ + T₃ = 1.3) is violated by 0.0713 along the heat geodesic,
and raising λ does not reduce it. Local cost functions create preferences, not
prohibitions: the penalty for leaving the conservation surface is always finite, so the
geodesic trades some of it for a shortcut. This is structural, not a tuning problem.

*Correction to the earlier record:* this was attributed to the friction signal weakening
near equilibrium. The measurement contradicts that — the worst excursion is at 9% along
the path, where h(p) is strongest, and the sum partially recovers afterwards. The
conclusion stands; the mechanism is not established.

### The solver is isotropic

The eikonal update treats friction as a scalar per cell. Anisotropic metrics work only
when their preferred direction aligns with the grid axes. This forced the K-M metric to
be reformulated as isotropic position-dependent friction, and it means the sin²θ form the
design document prescribes for both domain experiments is not generally supported.
Supporting true anisotropy requires a tensor eikonal solver — a known research direction,
not a bug.

### Readout is a distinct source of error from the solver

Twice the distance field has been correct while the extracted path was not: P2.3 (uniform
arclength sampling erased the cost parametrization — fixed) and P0.3 (nearest-cell
snapping makes the gradient piecewise constant, so the descent chatters — not fixed).
Every experiment measures the extracted path and attributes what it finds to the
framework. Nothing currently distinguishes the two.

## What comes next

### Now: harden

The direction for this round. Every finding above was discovered by measuring things by
hand that no test measures — including two failures that sat unnoticed in recorded PASSes
for months. The work ahead (Experiment 3, learned friction, the tensor solver) is exactly
the regime where a silent regression is invisible, because there is no analytical answer
to check against. Harden while known answers still exist to test with.

- **H1. Regression tests.** `tests/experiments.rs`. Lift the validation logic out of the
  examples and assert the thresholds instead of printing them. Write P0.1–P0.3 for the
  first time. Gate full-resolution runs behind `#[ignore]`; run a coarser N by default.
- **H2. Expose the distance field.** `solve` builds the full field and drops it. Blocks
  multi-target extraction, field export, and all Milestone 4 tooling.
- **H3. Carry cost per point on `GeodesicPath`.** `extract_path` already builds
  `Vec<(Point3, f64)>` and discards the cost. Near-free, and the structural fix for the
  class of defect P2.3 belonged to.
- **H4. Interpolate the distance field in `extract_path`.** Replaces nearest-cell
  snapping with trilinear interpolation. Expected to fix P0.3 and improve P0.2.
- **H5. Fix the K-M inner loop.** `KubelkaMunkMetric::cost` scans 501 curve points per
  evaluation, once per neighbour update across a 128³ grid. Dominant cost of Experiment 2.
- **H6. Resolve the `_space` parameter.** Wire `Space` into the marching loop or drop it.
  Gates Milestone 7.

### Then: repair Experiment 1, and Milestones 3–4

- **Rebuild Experiment 1** on an asymmetric initial profile so the analytical trajectory
  is curved and P1.1 can discriminate. Add a `EuclideanMetric` control run.
- **Adopt Euclidean controls generally** — every domain experiment should report what the
  flat metric does on the same configuration.
- **Path analysis** (Milestone 4): per-point cost profiles, curvature, abstraction depth.
  Needs H2 and H3.
- **`BoundaryRepulsive` + Experiment 3** (Milestone 3): the first structured friction
  without a known-domain answer. Note P3.3 requires a 5% cost-ratio effect against a
  solver carrying a ~2% cost bias — resolve P0.2 first or the prediction is unmeasurable.

### Medium and long term

- **Learned friction** (M5) — blocked on defining "path quality". This is the gating
  unknown for the whole learning ambition, not an implementation detail.
- **Domain encoding beyond physics** (M6) — finite groups, DSL programs.
- **Anisotropic solver** (M7) — tensor eikonal. Note Milestone 3's `AnisotropicMetric`
  depends on this, a dependency the design document does not record.
- **Alternative geometries** (M7) — blocked on H6.
- **Higher dimensions** — O(Nⁿ log N) becomes prohibitive above roughly 5–6 dimensions.

## Known limitations

| Limitation | Impact | Status |
|---|---|---|
| `cargo test` runs zero tests | No result is reproducible; two failures went unnoticed in recorded PASSes | H1 |
| Experiment 0 has no validation code | The gate the other two stand on was never checked | H1 |
| P0.2: cost overestimated ~1.86% at N=128 | Any prediction turning on precise cost values is untrustworthy (e.g. Experiment 3's P3.3) | H4, then restate or raise N |
| P0.3: grid-scale path chatter | Extracted geometry is jagged at sub-cell amplitude | H4 |
| Experiment 1 configuration degenerate | P1.1 cannot discriminate; the analog-computer claim rests on Exp 2 alone | Rebuild with asymmetric profile |
| Isotropic solver | Anisotropic metrics need reformulation; sin²θ form unsupported | Encode physics positionally; tensor solver is M7 |
| Local friction only | Cannot enforce global constraints (conservation laws) | Accept as structural; constraint-augmented metrics are an open question |
| Distance field discarded after `solve` | Re-solving required for every additional target or analysis | H2 |
| `Space` unused by the solver | Non-cubic containers unimplementable | H6 |
| 3D only | Cannot encode domains needing more dimensions | Future |
| No learned friction | Analog computer, not a learning system | M5 |

## The foundation

The project set out to answer whether structured friction in a continuous medium produces
meaningful geodesics. The answer is yes, with understood limitations — and the limitations
are now better understood than the affirmative is. Experiment 2 is a genuine result.
Experiment 0 and Experiment 1 need repair before they carry the weight that has been put
on them.

Every failure has been diagnostic. The conservation violation taught us about local versus
global constraints. The K-M anisotropy failure taught us about the solver's structural
limits. P2.3 and P0.3 taught us that readout fidelity is a distinct concern from solver
accuracy. And this documentation pass taught us the most practical lesson of the three:
**a result that only exists as printed output is not a result the project can rely on.**
That is what the hardening round is for.
