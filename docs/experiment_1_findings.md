# Experiment 1: Heat Flow on a Rod — Results and Findings

**Status:** PARTIAL — P1.1 and P1.2 pass, P1.3 fails; and see the discriminating-power
finding below, which qualifies what P1.1 establishes
**Date:** June 2026; re-measured and re-analysed August 2026
**Metric:** `HeatFlowMetric { kappa: 1.0, lambda: 50.0 }` (anisotropic, sin²θ penalty)
**Configuration:** start (0.2, 0.9, 0.2) → end (0.433, 0.433, 0.433), N = 128

The rod carries three temperature probes; the triple (T₁, T₂, T₃) is a point in the cube,
and heat diffusion traces a trajectory through it. The friction makes movement along the
heat-equation direction h(p) cheap and movement across it expensive, so the geodesic
should reproduce the physics.

## Results

| Prediction | Threshold | Actual | Result |
|---|---|---|---|
| P1.1 Trajectory match | < 0.05 max deviation | 0.0437 | **PASS** (but see below) |
| P1.2 Monotonic equilibration (T₂ ↓, T₁ ↑) | qualitative | holds | **PASS** |
| P1.3 Energy conservation | sum = 1.3 ± 0.02 | 0.0713 max deviation | **FAIL** |

Earlier records described P1.3 as "qualified". The measured deviation is 3.5× the
pre-registered tolerance; it is a failure, and this document records it as one. The
underlying interpretation — that it is structural rather than a tuning problem — stands
and is argued below.

## Finding 1: the test configuration cannot discriminate

This is the most important finding, and it was not visible until Experiment 2 forced a
closer look at what the two experiments actually test.

The initial temperature profile (0.2, 0.9, 0.2) is symmetric. Subtract the mean and it
becomes proportional to (−1, 2, −1), which is an **eigenvector of the discrete Laplacian**
that the heat metric is built from:

    M(−1, 2, −1) = (3, −6, 3) = −3 · (−1, 2, −1)

A single excited mode decays as a whole: every component shrinks toward the mean by the
same factor at every instant. The trajectory through the cube is therefore an **exact
straight line** from (0.2, 0.9, 0.2) to the mean — no arc, no curvature, nothing that
distinguishes heat diffusion from uniform motion.

This is confirmed numerically. The `analytical_heat_path` curve that
`examples/experiment_1.rs` validates against deviates from the straight segment joining
its own endpoints by 2.7 × 10⁻¹⁶ — machine epsilon. It is a straight line.

The consequence: **P1.1 tests only that the geodesic is straight.** That is the same
property Experiment 0's P0.1 tests, under a metric where it is the trivially expected
answer. A Euclidean metric would pass P1.1 on this configuration just as well as the
heat metric does. The prediction cannot fail in a way that would be informative, and
passing it is correspondingly weak evidence.

The design document (§8.2.3) chose the symmetric hot-spot profile for its physical
clarity. That choice is what makes the geometry degenerate. Nothing was done wrong in
implementing it; the flaw is in the experiment design, and it was invisible until there
was a second experiment to compare against.

**An asymmetric initial condition would fix this.** A profile that excites more than one
Fourier mode — the modes decay at different rates, so the fast mode dies first and the
trajectory bends — produces a genuinely curved analytical path that a Euclidean metric
could not reproduce. This is the single highest-value change available to Experiment 1
and should precede any further use of it as a validation.

Two secondary defects in the same function, both minor next to the above:

- The decay rate is hard-coded as `exp(−2.5t)`. The eigenvalue derived above gives
  −3κ = −3.0. Since the trajectory is a straight line, the rate affects only the
  *parametrisation* of the comparison curve, not its geometry, so P1.1 is insensitive
  to it — which is itself a symptom of the degeneracy.
- The curve stops at t = 1 where `exp(−2.5) ≈ 0.082` is still non-zero, so it ends at
  (0.414, 0.472, 0.414) rather than at equilibrium (0.433, 0.433, 0.433). It never
  reaches the point the experiment declares as its endpoint.

## Finding 2: local friction cannot enforce global constraints

Total thermal energy T₁ + T₂ + T₃ must be conserved at 1.3 — insulated ends let no heat
escape. Along the geodesic the sum falls to 1.2287, a deviation of 0.0713.

Part of that is bookkeeping: the solver snaps the start point to the grid, so the path
begins at (0.1953, 0.8984, 0.1953) with a sum of 1.2891 rather than 1.3. That accounts
for 0.011. The remaining excursion, 1.2891 → 1.2287, is a real departure of the geodesic
from the conservation surface.

Raising λ does not fix it. This is the diagnostic result: if the violation were a matter
of insufficiently strong friction, a stronger penalty would suppress it. It does not,
because **a local cost function creates a preference, not a prohibition.** The metric
charges more for moving off the heat-equation direction; it cannot charge infinitely
more, and the geodesic will always accept a finite penalty in exchange for a shortcut.
Conservation is a global constraint — a statement about a surface the whole path must lie
on — and no per-point, per-direction cost can express it.

This is the project's most conceptually significant negative result. It marks a boundary
on what the friction formulation can encode, and it will recur for every domain with a
conservation law.

### Correction to the previously recorded diagnosis

`project_status.md` stated that conservation "drifts where the friction signal weakens
near equilibrium" — the reasoning being that h(p) → 0 at equilibrium, so the metric goes
isotropic and stops steering.

The measurement does not support this. The worst violation occurs at **9% along the
path**, at (0.198, 0.832, 0.198) — near the *start*, where the profile is still far from
equilibrium and h(p) is at its strongest. The sum then recovers, ending at 1.2875.

The vanishing-friction story is plausible and may still contribute near the end, but it
is not the mechanism behind the measured failure. The drift is an early excursion, not a
terminal one. Whatever the correct explanation, it has not been established, and this
should be treated as an open question rather than a settled one.

## Finding 3: P1.1 passed despite the isotropic solver, not because of it

Experiment 2 established that the FMM's eikonal update is inherently isotropic: it
evaluates the metric at the six axis directions and collapses the result to a single
scalar (`update_neighbours`, `src/solver.rs:178-182`), discarding all directional
information. Anisotropic metrics of the sin²θ form are therefore not properly supported.

`HeatFlowMetric` is exactly that form. The standing explanation for why it worked anyway
is that the heat-flow direction from (0.2, 0.9, 0.2) runs roughly diagonally through the
cube, so axis-aligned sampling happened to capture enough of the anisotropy.

Finding 1 offers a simpler explanation: on this configuration the correct answer is a
straight line, and the solver produces straight lines by default. The metric may have
contributed very little. Which of the two accounts is right is testable — run the same
configuration under `EuclideanMetric` and compare — and that test has not been run.

## Conclusion

Experiment 1 does not establish what it was recorded as establishing. Its qualitative
results are sound: the geodesic equilibrates monotonically (P1.2), lands on the correct
curve (P1.1), and the conservation failure (P1.3) taught the project something real and
durable about the limits of local friction. But the configuration is geometrically
degenerate, so P1.1 carries almost no discriminating power, and the analytical curve it
compares against is a straight line that never reaches the stated endpoint.

The analog-computer claim now rests substantially on **Experiment 2**, whose K-M mixing
curve is genuinely non-straight, arcs through green where the Euclidean path does not,
and therefore could have failed. Experiment 1 should be rebuilt on an asymmetric initial
condition before it is cited as independent corroboration.

## Open items

- **Rebuild with an asymmetric initial profile** so the analytical trajectory is curved
  and the prediction can discriminate.
- **Fix `analytical_heat_path`**: decay rate −3κ, and extend far enough to reach
  equilibrium.
- **Run the configuration under `EuclideanMetric`** to measure how much the heat metric
  actually contributes (Finding 3).
- **Establish the real mechanism of the conservation drift** — the recorded
  near-equilibrium explanation is contradicted by the measurement.
- **Encode P1.1–P1.3 as assertions** rather than printed output (hardening item H1).
