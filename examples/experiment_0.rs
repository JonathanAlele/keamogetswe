use keamogetswe::export::{GeodesicOutput, Metadata, PathEntry};
use keamogetswe::metric::EuclideanMetric;
use keamogetswe::space::CubeSpace;

fn main() {
    let _space = CubeSpace;
    let _metric = EuclideanMetric;
    let _start = [0.2, 0.3, 0.4];
    let _end = [0.8, 0.6, 0.7];
    let _resolution = 128;

    let path = keamogetswe::solver::solve(&_space, &_metric, _start, _end, _resolution);
    let output = GeodesicOutput {
        metadata: Metadata {
            name: String::from("Experiment 0: Euclidean validation"),
            metric: String::from("EuclideanMetric"),
            resolution: _resolution,
        },
        paths: vec![PathEntry {
            label: String::from("computed"),
            points: path.points,
        }],
    };

    let json = output.to_json().unwrap();
    std::fs::write("viewer/sample_data/experiment_0.json", json).unwrap();

    println!("Experiment 0 complete. Total cost: {:.4}", path.total_cost);
}
