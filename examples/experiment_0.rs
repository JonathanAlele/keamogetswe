use keamogetswe::space::CubeSpace;
use keamogetswe::metric::EuclideanMetric;

fn main() {
    let _space = CubeSpace;
    let _metric = EuclideanMetric;
    let _start = [0.2, 0.3, 0.4];
    let _end = [0.8, 0.6, 0.7];
    let _resolution = 128;

    // TODO: Once the solver is implemented:
    // let path = keamogetswe::solver::solve(&space, &metric, start, end, resolution);
    // let output = GeodesicOutput { ... };
    // std::fs::write("viewer/sample_data/experiment_0.json", output.to_json().unwrap());

    println!("Experiment 0: solver not yet implemented");
    println!("Using synthetic sample data in viewer/sample_data/");
}
