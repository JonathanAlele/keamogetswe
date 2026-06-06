# keamogetswe

Forward-only learning through friction in continuous media.

## Project structure

```
keamogetswe/
├── Cargo.toml              # Rust crate definition
├── src/
│   ├── lib.rs              # Module declarations
│   ├── types.rs            # Point3, Vector3
│   ├── space.rs            # Space trait + CubeSpace
│   ├── metric.rs           # Metric trait + EuclideanMetric
│   ├── solver.rs           # FMM solver (stub)
│   ├── path.rs             # GeodesicPath output type
│   └── export.rs           # JSON serialisation
├── examples/
│   └── experiment_0.rs     # Euclidean validation (stub)
├── viewer/
│   ├── index.html          # 3D geodesic viewer (Three.js)
│   └── sample_data/        # Synthetic test paths
│       ├── experiment_0.json
│       ├── experiment_1.json
│       └── experiment_2.json
└── schemas/
    └── geodesic_path.json  # JSON format contract
```

## Two independent components

**Rust crate** (`src/`, `examples/`): computes geodesics, outputs JSON.

**Viewer** (`viewer/`): renders JSON paths in a 3D RGB cube. No dependency on the crate.

The JSON schema in `schemas/` is the contract between them.

## Running the viewer

Serve the viewer directory with any static file server:

```bash
cd viewer
python3 -m http.server 8000
# then open http://localhost:8000
```

Or use any other static server. The viewer loads Three.js from CDN.

Drop any JSON file matching the schema onto the viewer to render it.

## Building the crate

```bash
cargo check        # verify compilation
cargo test         # run tests
cargo run --example experiment_0   # run an experiment
```
