# keamogetswe

**Forward-only learning through friction in continuous media.**

Modern machine learning learns by propagating error backward. This project explores an
alternative: encode a problem's structure as *friction* in a continuous medium, and let
the cheapest path through that friction be the learned representation. No backward pass.

Concretely, the medium is the RGB cube [0,1]³. Problem structure becomes a Riemannian-style
cost function `cost(p, v)` — the friction. The output is the **geodesic**, computed by
solving the eikonal equation |∇T| = 1/F with the Fast Marching Method and descending the
resulting distance field.

The design document is `chromatic_geodesics_v0.3.1.docx`; start there for the full
motivation and the three-perspective (CS / mathematics / colour theory) treatment.
`docs/design_notes_v0.4.0.md` records what implementation has since changed about it.

## Status

Milestones 1–2 complete. The "analog computer" claim — domain-derived friction produces
domain-faithful geodesics — is validated against **Kubelka-Munk pigment mixing**, a domain
with a known analytical solution and a genuinely non-straight answer.

| Experiment | Metric | Result |
|---|---|---|
| 0: Euclidean calibration | `EuclideanMetric` | **Partial** — path is straight (P0.1), but cost is 1.86% high and the path chatters at grid scale |
| 1: Heat flow | `HeatFlowMetric` | **Partial** — equilibrates correctly, but violates conservation, and the configuration is degenerate |
| 2: K-M colour mixing | `KubelkaMunkMetric` | **Pass** — all three predictions |

Full write-ups in `docs/`. Read `docs/project_status.md` first.

> **Testing caveat.** `cargo test` currently runs **zero tests**. All validation lives in
> the example binaries' printed PASS/FAIL output, which nothing captures or asserts —
> and `examples/experiment_0.rs` contains no validation code at all. Making these results
> reproducible is the project's top priority; see the hardening roadmap in
> `docs/project_status.md`.

## Project structure

```
keamogetswe/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Module declarations
│   ├── types.rs            # Point3, Vector3
│   ├── space.rs            # Space trait + CubeSpace
│   ├── metric.rs           # Metric trait + Euclidean / HeatFlow / KubelkaMunk
│   ├── solver.rs           # FMM: eikonal update, wavefront, path extraction
│   ├── path.rs             # GeodesicPath output type
│   └── export.rs           # JSON serialisation
├── examples/
│   ├── experiment_0.rs     # Euclidean calibration
│   ├── experiment_1.rs     # Heat flow on a rod
│   └── experiment_2.rs     # Kubelka-Munk colour mixing
├── viewer/
│   ├── index.html          # 3D geodesic viewer (Three.js)
│   └── sample_data/        # Experiment output
├── schemas/
│   └── geodesic_path.json  # JSON format contract
└── docs/                   # Findings, status, design delta
```

## Two independent components

**Rust crate** (`src/`, `examples/`): computes geodesics, outputs JSON.

**Viewer** (`viewer/`): renders JSON paths in a 3D RGB cube. No dependency on the crate,
no build step, no npm.

The JSON schema in `schemas/` is the contract between them.

## Building and running

```bash
cargo check                            # verify compilation
cargo run --release --example experiment_2   # run an experiment
```

Each example prints its predictions' PASS/FAIL and writes JSON to
`viewer/sample_data/`. Use `--release`: at N=128 the debug build is very slow.

## Running the viewer

Serve the viewer directory with any static file server:

```bash
cd viewer
python3 -m http.server 8000
# then open http://localhost:8000
```

The viewer loads Three.js from CDN. Drop any JSON file matching the schema onto it to
render it.
