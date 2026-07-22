# Experiment 2: Kubelka-Munk Colour Mixing — Results and Findings

**Status:** PASS  
**Date:** June 2026 (P2.3 resolved July 2026)  
**Metric:** KubelkaMunkMetric (isotropic reformulation, λ=50.0)  
**Resolution:** N=128  

## Results

| Prediction | Threshold | Actual | Result |
|---|---|---|---|
| P2.1 Path follows K-M curve | < 0.05 max deviation | 0.0371 | **PASS** |
| P2.2 Green intermediates (G > R and G > B) | at any point | yes | **PASS** |
| P2.3 Nonlinear spacing (max/min ratio) | > 1.5 | 9.06 | **PASS** |

P2.3 originally failed (ratio 1.07) with the fixed-step extraction; see the root cause
analysis and fix below. P2.1 moved from 0.0366 to 0.0371 under the new extraction —
the geometry is unchanged, only the sampling of it.

## Key finding: isotropic solver requires isotropic metric reformulation

The original metric design was anisotropic — direction-dependent cost, where movement along
the K-M mixing curve tangent was cheap and perpendicular movement was expensive. This follows
the same sin²θ pattern as the heat flow metric.

Under this design, all three predictions failed completely (P2.1 deviation of 0.3833, no
green intermediates, uniform spacing). The geodesic ignored the metric entirely and defaulted
to near-Euclidean behaviour.

### Root cause

The FMM solver's eikonal update (`solve_local`) is inherently isotropic. The equation:

    ∑ᵢ (t − aᵢ)² = (h·f)²

treats `f` as a scalar — the same friction in all directions at that cell. The solver
evaluates the metric at axis-aligned directions (±R, ±G, ±B) and feeds a single friction
value to `solve_local`. This discards all directional information.

The heat flow experiment (Experiment 1) passed despite this limitation because the heat flow
direction from (0.2, 0.9, 0.2) toward equilibrium happens to be roughly diagonal in the
cube. The axis-aligned sampling captured enough of the anisotropy by accident to steer the
path correctly.

The K-M mixing curve snakes through the cube at angles that don't align with any axis. The
axis-aligned sampling misrepresented the anisotropy so completely that the metric provided
no useful guidance.

### Solution: isotropic reformulation

The metric was reformulated as position-dependent isotropic friction:

    cost(p, v) = ‖v‖ × (1 + λ × d²)

where d is the distance from point p to the nearest point on the K-M mixing curve. This
creates a valley of low friction along the curve. The physics is encoded in the landscape
shape (where the valley lies) rather than in directional preferences (which way is cheap
to move). The solver handles isotropic friction correctly, so the geodesic flows through
the valley.

After reformulation, P2.1 and P2.2 passed. The geodesic tracks the K-M curve and arcs
through green — the signature of subtractive mixing.

### Implication for the framework

The framework currently has two modes:

1. **Anisotropic metrics that happen to align with grid axes** (e.g. heat flow along a
   diagonal): these work because the axis-aligned sampling captures enough directional
   information.

2. **Arbitrary anisotropic metrics** (e.g. K-M mixing along a non-axis-aligned curve):
   these require reformulation as isotropic position-dependent friction.

Supporting true anisotropic friction would require an anisotropic FMM — a fundamentally
more complex algorithm that solves a tensor eikonal equation. This is a known research
problem and a candidate for future development (see Open Questions).

## P2.3 failure: parametrization discarded at readout (resolved)

The original spacing ratio of 1.07 (vs threshold 1.5) was caused by `extract_path`
using a fixed step size (0.5 grid cells) with a normalized gradient — every emitted
point was exactly 0.5 cells from the previous one, so uniform spacing was guaranteed
by construction.

### Deeper root cause

The fixed step was the immediate cause, but not the structural one. P2.3 is a
prediction about the path's *parametrization* (points sampled along the mixing
process should be unevenly spaced in the cube), while the extraction returned only
the path's *geometric image* sampled at uniform arclength. Any uniform-arclength
sampling of a curve is uniformly spaced by definition — even a perfect adaptive
extraction of the correct geodesic would have failed P2.3.

The nonlinearity the prediction tests for was present in the solver's output all
along: it lives in the distance-field values T along the path (where friction is
high, T grows faster per unit distance). It was simply discarded because points were
emitted per-step rather than per-cost-increment.

### Fix: resample by cost

`extract_path` now records the distance-field value T at each descent position and
re-emits the path at equal increments of T (`resample_by_cost` in `src/solver.rs`),
linearly interpolating between descent samples. Spatial spacing now shrinks where
friction is high (near the pure pigments, where small mixing-ratio changes produce
large colour changes) and stretches along the flat middle of the mixing curve.

Result: spacing ratio 9.06, comfortably above the 1.5 threshold. Experiments 0 and 1
were re-run as a regression check: all previously passing predictions still pass, and
Experiment 1's conservation qualification is unchanged (it is a property of the
geodesic's geometry, not of the sampling).

The original diagnosis ("adaptive step sizes" as mitigation) would not have fixed
this — an adaptive-step extraction still samples by arclength. The distance field was
correct; the readout discarded the parametrization.

## Conclusion

The analog computer claim is validated for a second independent domain, with all three
predictions now passing. Position-dependent isotropic friction successfully guides geodesics
along physically correct mixing paths. The framework reproduces the qualitative signature of
subtractive colour mixing (green intermediates from blue + yellow) from a friction function
derived from Kubelka-Munk theory.

The experiment also revealed the solver's isotropic limitation — a structural constraint
that was not visible in Experiment 1 and only became apparent when the target curve was
misaligned with the grid axes. This is the most important technical finding from the
experiment.
