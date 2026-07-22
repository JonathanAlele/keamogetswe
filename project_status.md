# keamogetswe — Project Status

**Date:** June 2026  
**Version:** 0.1.0 (prototype)

## What exists

### Working code

- **Space module:** `CubeSpace` with boundary detection. `Space` trait designed for
  future alternative geometries.
- **Metric module:** Three implementations of the `Metric` trait:
  - `EuclideanMetric` — uniform cost, baseline reference
  - `HeatFlowMetric` — anisotropic, heat equation physics
  - `KubelkaMunkMetric` — isotropic, subtractive mixing physics
- **Solver module:** Complete FMM implementation with eikonal update, priority queue
  (min-heap via `Reverse<FmmEntry>`), gradient-based path extraction.
- **Export module:** JSON serialisation matching the viewer schema.
- **Viewer:** Standalone Three.js viewer rendering RGB-coloured geodesic paths in a
  wireframe cube. Loads JSON, supports multiple paths per file, orbit/zoom controls.

### Validated experiments

| Experiment | Domain | Core result | Status |
|---|---|---|---|
| 0: Euclidean | None (calibration) | Straight lines, correct distance | **PASS** |
| 1: Heat flow | Thermal diffusion | Path matches analytical solution | **PASS** (conservation qualified) |
| 2: K-M mixing | Subtractive colour | Path follows mixing curve through green | **PASS** (all predictions, incl. nonlinear spacing) |

### Project documents

- `docs/experiment_1_findings.md` — full writeup with root cause analysis of
  conservation failure
- `docs/experiment_2_findings.md` — full writeup with isotropic reformulation finding
- `schemas/geodesic_path.json` — JSON contract between crate and viewer
- Main project document (v0.3.1) — living document with orientation, three perspectives,
  milestones, and original experiment design

## What we've learned

### The framework works as an analog computer

Domain-derived friction produces domain-faithful geodesics. This is validated across two
independent domains (heat diffusion, pigment mixing) with known analytical solutions. The
core claim — that structured friction in a continuous medium can substitute for
domain-specific simulation — is supported.

### Local friction cannot enforce global constraints

Experiment 1 showed that energy conservation (T₁ + T₂ + T₃ = const) drifts where the
friction signal weakens near equilibrium. Local cost functions create preferences, not
prohibitions. Increasing the penalty strength (λ) did not reduce the violation, confirming
it's a structural property of local friction, not a parameter tuning issue.

### The solver is isotropic

The eikonal update treats friction as a scalar per cell. Anisotropic metrics work when
their preferred direction happens to align with grid axes (Experiment 1) but fail when
the preferred direction is arbitrarily oriented (Experiment 2, original formulation).
This required reformulating the K-M metric as isotropic position-dependent friction.

Supporting true anisotropy requires an anisotropic FMM — a tensor eikonal solver. This
is a known research direction, not a bug.

### Path extraction limits visual fidelity but not accuracy

The distance field (FMM output) is more accurate than the extracted path. The original
axis-stepping extraction produced staircase artefacts. The gradient-descent extraction
fixed the smoothness but initially produced uniform spacing that discarded the cost
parametrization; the extraction now resamples the descent trajectory at equal cost
increments, restoring the curve's nonlinearity in the output (Experiment 2 P2.3 now
passes). The distance field itself converges correctly with increasing resolution
(verified by halving the error when doubling N).

## What comes next

### Near-term (Milestones 3–4)

- **Path analysis tooling:** per-point cost profiles, curvature computation, abstraction
  depth measurement. Currently only total cost is reported; the experiment validations
  had to compute their own metrics.
- **BoundaryRepulsive metric:** the exploratory experiment from the original design
  (Experiment 3 in the document). Now that the analog computer is validated, structured
  friction without known-domain backing is the next frontier.
- **Additional Euclidean regression tests:** encode the validation as `#[test]` functions
  in the crate so they run automatically with `cargo test`.

### Medium-term (Milestones 5–6)

- **Learned friction:** the transition from friction-as-given (analog computer) to
  friction-as-learned (self-organising system). Requires defining what "path quality"
  means and building a meta-learning loop. This is the bridge to the machine learning
  ambition.
- **Domain encoding beyond physics:** finite groups via character tables, DSL programs
  via type-directed embedding. Tests whether the framework generalises beyond domains
  with continuous physics.

### Long-term (Milestone 7+)

- **Anisotropic solver:** tensor eikonal equation. Would unlock true directional friction
  and remove the isotropic reformulation requirement.
- **Alternative container geometries:** hexagonal prism, dodecahedron. Requires
  generalising the FMM to non-cubic grids.
- **Higher dimensions:** extending from [0,1]³ to [0,1]ⁿ. Computational cost scales
  as O(Nⁿ log N), which becomes prohibitive above about 5–6 dimensions with current
  methods.

## Known limitations

| Limitation | Impact | Mitigation |
|---|---|---|
| Isotropic solver | Anisotropic metrics require reformulation | Encode physics in position-dependent cost |
| Local friction only | Cannot enforce global constraints (conservation laws) | Accept as structural; explore constraint-augmented metrics |
| ~~Grid-based extraction~~ | ~~Uniform spacing~~ — resolved: extraction now resamples by cost, restoring the nonlinear parametrization | Fixed July 2026 (`resample_by_cost`) |
| 3D only | Cannot encode domains requiring more dimensions | Future: extend to [0,1]ⁿ |
| No learned friction | Currently analog computer only, not a learning system | Milestone 5 |

## The foundation

The project set out to answer: does structured friction in a continuous medium produce
meaningful geodesics? The answer is yes, with understood limitations. The Euclidean test
proved the solver is transparent. The heat flow test proved the framework reproduces
physics. The K-M test proved it generalises across domains and revealed the solver's
isotropic constraint.

Every failure was diagnostic. The conservation violation taught us about local vs global
constraints. The K-M anisotropy failure taught us about the solver's structural limitations.
The staircase artefact taught us about extraction fidelity. Each failure sharpened the
understanding of what the framework can and cannot do.

The analog computer mode is validated. The path forward is toward learned friction and
domain encoding — where the framework stops reproducing known physics and starts
discovering new structure.
