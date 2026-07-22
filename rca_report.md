# Root Cause Analysis Report — keamogetswe (Chromatic Geodesics)

**Date:** 22 July 2026
**Scope:** Study of project goals, code, and recorded experiment results; root cause
analysis of the outstanding failure. No code changes were made.

## 1. Project understanding

The project prototypes **forward-only learning through friction**: instead of
backpropagation, learning is framed as navigating a structured resistant medium. The
medium is the RGB cube [0,1]³; the problem structure is encoded as a friction function
(a Riemannian-style metric `cost(p, v)`); the learned representation is the geodesic —
the cheapest path through the friction landscape (main document, v0.3.1).

The implementation is a Rust crate:

- `src/space.rs` — `Space` trait + `CubeSpace` container.
- `src/metric.rs` — `Metric` trait with `EuclideanMetric`, `HeatFlowMetric`
  (anisotropic, sin²θ penalty against the heat-flow direction), and
  `KubelkaMunkMetric` (isotropic valley of low friction along the K-M mixing curve).
- `src/solver.rs` — Fast Marching Method: eikonal update (`solve_local`), min-heap
  wavefront propagation (`solve`), and gradient-descent path extraction
  (`extract_path`).
- `examples/experiment_{0,1,2}.rs` — the three validation experiments from the design
  document, each printing PASS/FAIL against pre-registered predictions and writing
  JSON for the Three.js viewer.

The stated validation ladder: Experiment 0 calibrates the solver (Euclidean straight
lines), Experiments 1 (heat diffusion) and 2 (Kubelka-Munk pigment mixing) test the
core "analog computer" claim against domains with known analytical solutions.

## 2. Recorded results

| Experiment | Prediction | Threshold | Actual | Result |
|---|---|---|---|---|
| 0 Euclidean | straight line, correct distance | — | — | PASS |
| 1 Heat flow | P1.1 trajectory match | < 0.05 | pass | PASS |
| 1 Heat flow | P1.3 conservation | ± 0.02 | drift near equilibrium | qualified |
| 2 K-M mixing | P2.1 curve deviation | < 0.05 | 0.0366 | PASS |
| 2 K-M mixing | P2.2 green intermediates | any point | yes | PASS |
| 2 K-M mixing | **P2.3 spacing ratio (max/min)** | **> 1.5** | **1.07** | **FAIL** |

P2.3 is the one prediction still failing after the isotropic reformulation of the K-M
metric. It is the subject of the root cause analysis below.

## 3. Root cause analysis: P2.3 (nonlinear spacing) failure

### 3.1 What the prediction expects

P2.3 encodes real K-M physics: near the pure pigments, small changes in mixing ratio t
produce large colour changes; near the middle the curve flattens. So points sampled
along the mixing process should be unevenly spaced, with max/min inter-point distance
exceeding 1.5. The validation (`examples/experiment_2.rs:95-119`) measures the
Euclidean distance between consecutive points of the extracted path.

### 3.2 Immediate cause — extraction emits constant-length steps

`extract_path` (`src/solver.rs:232-310`) backtracks from the target by gradient
descent on the distance field with:

- a **fixed step size of 0.5 grid cells** (`solver.rs:244`), and
- a **normalized gradient** (`solver.rs:291-300`), so every step has exactly the same
  length regardless of how steep the cost landscape is.

One path point is pushed per step, so consecutive points are 0.5·h apart *by
construction*. The measured ratio of 1.07 is just noise from the final snap-to-cell
and boundary clamping. Uniform spacing is guaranteed no matter what curve the geodesic
follows — the test could never pass with this extraction.

This matches the project's own diagnosis in `experiment_2_findings.md` ("fixed step
size... produces roughly uniform spacing").

### 3.3 Structural root cause — the readout discards the parametrization

The documented diagnosis is correct but incomplete. Replacing the fixed step with an
adaptive one would **not** fix P2.3, because the failure is not a step-size problem —
it is a **parametrization problem**:

- The FMM produces two things: the geodesic's *geometric image* (which points in the
  cube it passes through) and, implicitly, its *cost parametrization* (the distance
  field value T at each point — "how far along the mixing process are we").
- P2.3 is a prediction about the **parametrization**: points sampled at equal
  increments of the mixing process should be unequally spaced in the cube.
- `extract_path` returns only the geometric image, sampled at uniform *arclength*.
  Any uniform-arclength sampling of a curve is uniformly spaced by definition — even a
  perfect, artefact-free extraction of the correct geodesic would fail P2.3.

The nonlinearity the prediction looks for is present in the solver's output — it lives
in the distance field values along the path (where friction is higher, T grows faster
per unit distance) — but it is discarded at readout because points are emitted
per-step rather than per-cost-increment. This is why the findings document correctly
observes "the distance field is correct; the readout is limited," yet the proposed
mitigation ("adaptive step sizes") would not by itself recover the signal.

Chain of causation:

1. P2.3 asks about spacing under the mixing-process (cost) parametrization.
2. `extract_path` reparametrizes the geodesic by arclength (constant-length steps).
3. Arclength parametrization makes spacing uniform for *any* curve.
4. Measured ratio ≈ 1 → FAIL, independent of the geodesic's correctness (P2.1/P2.2
   confirm the geometry itself is right).

### 3.4 Suggested corrections (not implemented)

- **Emit points at equal cost increments:** during backtracking, record the distance
  field value T at each descent position and output interpolated points at uniform ΔT.
  Spatial spacing then shrinks where friction is high (curve ends) and stretches where
  it is low — exactly the nonlinearity P2.3 tests.
- **Or attach T per point** to `GeodesicPath` (e.g. `(point, cost_so_far)`) and let
  the validation resample by cost. This also serves the "per-point cost profiles"
  item already on the Milestone 3–4 roadmap.
- **Or redefine P2.3** to measure d(position)/d(cost) variation along the path rather
  than raw inter-point distances, acknowledging that spatial spacing of an
  arclength-sampled path is not a meaningful observable.

## 4. Secondary findings

1. **Original anisotropic K-M failure (historical):** confirmed in code. The solver
   samples the metric only at the six axis directions and passes a single scalar
   friction into the eikonal update (`update_neighbours`, `src/solver.rs:178-182`),
   discarding all directional information. The documented conclusion — anisotropic
   metrics only work when their preferred direction happens to align with the grid,
   and true anisotropy needs a tensor eikonal solver — is consistent with the code.
2. **Stale build failure in `target/flycheck0/`:** the recorded E0308 error
   ("`solve_local` ... expected f64, found ()") is from an earlier stub version of
   `solve_local`; the current `src/solver.rs:88-127` has a complete implementation.
   Not a live failure.
3. **`cargo test` runs zero tests:** there are no `#[test]` functions anywhere in the
   crate. All validation lives in the example binaries' printed PASS/FAIL output,
   which is not captured or asserted anywhere. (Already acknowledged as near-term
   work in `project_status.md`.)
4. **Documentation drift:** `project_status.md` references
   `docs/experiment_1_findings.md`, but no `docs/` directory exists (only
   `experiment_2_findings.md` at the repo root); the README still labels `solver.rs`
   and `experiment_0.rs` as stubs although both are complete.

## 5. Conclusion

The framework's core claim holds: the geodesic geometry is validated in two
independent domains (P1.1, P2.1, P2.2 pass). The remaining failure, P2.3, is not a
defect in the solver or the metric — it is a **readout defect**: the path extraction
reparametrizes the geodesic by arclength, which structurally erases the cost
parametrization that the prediction measures. The fix is to expose cost-along-path in
the extracted output (or to restate the prediction in cost terms), not to tune the
step size.
