# Experiment 0: Euclidean Validation — Results and Findings

**Status:** PARTIAL — P0.1 passes, P0.2 and P0.3 fail
**Date:** August 2026 (first measured run; previously recorded PASS without measurement)
**Metric:** `EuclideanMetric` (uniform cost = 1 everywhere)
**Configuration:** start (0.2, 0.3, 0.4) → end (0.8, 0.6, 0.7), N = 128

Experiment 0 is the calibration gate. Its purpose is not to test the framework but to
test the *solver*: under a flat metric the geodesic must be a straight line of the
correct length. The design document is explicit that "if any prediction fails, the
solver has a bug. Do not proceed."

## Results

| Prediction | Threshold | Actual | Result |
|---|---|---|---|
| P0.1 Deviation from straight line | < 2/N = 0.0156 | 0.0124 | **PASS** |
| P0.2 Path cost accuracy | √0.54 = 0.7348 ± 1% | 0.7485 (+1.86%) | **FAIL** |
| P0.3 Path curvature | < 0.1 per unit length | ≈ 7.5 | **FAIL** |

## Provenance note: this result was previously recorded without being measured

`project_status.md` and `rca_report.md` both recorded Experiment 0 as **PASS** with no
numbers attached. `examples/experiment_0.rs` explains why: the example solves, writes
viewer JSON, and prints the total cost — it contains **no validation code at all**. There
is no check of P0.1, P0.2, or P0.3 anywhere in the repository. The recorded PASS appears
to have been a visual judgement from the viewer ("it looks straight"), which P0.1 confirms
was correct as far as it went.

The numbers above are the first actual measurement of all three predictions, computed
externally from `viewer/sample_data/experiment_0.json`. They are not reproducible by
`cargo test`; making them so is item H1 of the hardening roadmap.

## P0.2: cost overestimated by 1.86%

The computed cost is 0.7485 against a true Euclidean distance of √0.54 ≈ 0.7348 — an
overestimate of 1.86%, nearly double the 1% tolerance.

The direction of the error is diagnostic. The path runs diagonally through the grid, and
the first-order upwind eikonal update in `solve_local` (`src/solver.rs:88-127`) is known
to overestimate distances along directions that are not axis-aligned: the scheme
approximates the wavefront with axis-aligned finite differences, so diagonal travel is
charged slightly more than it should be. The error is one-sided and shrinks with
resolution, which is consistent with `project_status.md`'s note that the distance field
error halves when N doubles.

**This is discretisation, not a bug** — but P0.2 was pre-registered at ±1% and N = 128,
and at that setting the solver does not meet it. Two honest responses, both open:

1. Raise the resolution until the error falls inside the band, and record the N required.
2. Restate P0.2 as a convergence claim (error → 0 as O(1/N)) rather than an absolute
   tolerance, which is what a first-order scheme can actually promise.

Option 2 is the better prediction, but changing a pre-registered threshold after seeing
the result needs to be recorded as exactly that. The convergence order has not been
measured in the current tree; that measurement is a prerequisite for either option.

## P0.3: grid-scale chatter in the extracted path

Measured maximum discrete curvature along the path is roughly 167 per unit length at full
sampling — two orders of magnitude above the 0.1 noise floor. This is not a measurement
artefact of closely-spaced points: decimating the path 10× still leaves a maximum
curvature of about 7.5.

Yet P0.1 passes. The path never wanders more than 0.0124 from the true line — under one
grid cell. The two facts together characterise the defect precisely: **small-amplitude,
high-frequency wobble**. The path goes to the right place by a visibly jagged route.

The mechanism is in `extract_path` (`src/solver.rs:250-254`). Each descent step snaps the
current continuous position to the *nearest grid cell* before reading the distance field,
so the gradient is piecewise constant across each cell. As the descent crosses a cell
boundary the gradient estimate jumps discontinuously, and the step direction chatters at
the cell scale. Under a flat metric, where the true gradient is constant everywhere, all
of the observed curvature is this artefact.

This is the same *class* of defect as P2.3 (see `docs/experiment_2_findings.md`): the
distance field is correct and the readout degrades it. It is a different instance —
P2.3 was about the path's parametrization, this is about its geometry — but the lesson
generalises: **extraction fidelity is a separate concern from solver accuracy, and the
project has now been bitten by it twice.**

The fix is trilinear interpolation of the distance field at the continuous position
rather than nearest-cell snapping. This is a contained change to `extract_path` and is
expected to help P0.2 as well, since a smoother descent accumulates less excess length.

## Implication: Gate 0 is not passed as written

The design document's gate structure makes Experiment 0 a strict precondition for
Experiments 1 and 2. As literally specified, that gate is not met.

This does **not** invalidate Experiments 1 and 2, and the framework's core claim is not
in question. What P0.1 establishes — the geodesic lands on the correct curve to within a
grid cell — is precisely the property that P1.1 (trajectory match < 0.05) and P2.1 (K-M
curve deviation < 0.05) depend on, and both of those tolerances are three times larger
than the deviation measured here. The two failures are a systematic ~2% cost bias and a
sub-cell geometric wobble; neither is capable of producing a qualitatively wrong path.

What it does mean is that the solver is **less accurate than the project has been
assuming**, in ways that were never measured because nothing measured them. Any future
result that turns on precise cost values — cost-ratio comparisons between metrics, which
Experiment 3's P3.3 (ratio > 1.05) requires at a tolerance smaller than the bias
measured here — is not currently trustworthy.

## Open items

- **P0.1–P0.3 are not checked by any code.** (Hardening item H1.)
- **Interpolate the distance field in `extract_path`** instead of nearest-cell snapping.
  Expected to fix P0.3 and improve P0.2.
- **Measure the convergence order** of the distance field and of the total cost, and use
  it to settle whether P0.2 is restated or the resolution raised.
- **Re-run Experiments 1 and 2 after the extraction fix** as a regression check. Both are
  expected to be unaffected in geometry; P1.3's conservation deviation may change, since
  a chattering path accumulates spurious excursions.
