use keamogetswe::export::{GeodesicOutput, Metadata, PathEntry};
use keamogetswe::metric::HeatFlowMetric;
use keamogetswe::solver::solve;
use keamogetswe::space::CubeSpace;

fn main() {
    let space = CubeSpace;
    let metric = HeatFlowMetric {
        kappa: 1.0,
        lambda: 50.0,
    };

    let start = [0.2, 0.9, 0.2];
    let end = [0.433, 0.433, 0.433];
    let resolution = 128;

    println!("Running heat flow experiment at N={}...", resolution);
    let path = solve(&space, &metric, start, end, resolution);
    println!("Total cost: {:.4}", path.total_cost);

    let analytical = analytical_heat_path(200);

    // Validation checks
    validate(&path.points, &analytical);

    let output = GeodesicOutput {
        metadata: Metadata {
            name: String::from("Experiment 1: Heat flow"),
            metric: String::from("HeatFlowMetric"),
            resolution,
        },
        paths: vec![
            PathEntry {
                label: String::from("computed"),
                points: path.points,
            },
            PathEntry {
                label: String::from("analytical"),
                points: analytical,
            },
        ],
    };

    let json = output.to_json().unwrap();
    std::fs::write("viewer/sample_data/experiment_1.json", json).unwrap();
    println!("Written to viewer/sample_data/experiment_1.json");
}

/// Analytical solution: exponential decay toward thermal equilibrium.
///
/// The mean temperature is conserved (insulated ends).
/// The dominant Fourier mode decays as exp(−αt) where α
/// controls the rate of equilibration.
fn analytical_heat_path(num_points: usize) -> Vec<[f64; 3]> {
    let mean = (0.2 + 0.9 + 0.2) / 3.0;
    (0..num_points)
        .map(|i| {
            let t = i as f64 / (num_points - 1) as f64;
            let decay = (-2.5 * t).exp();
            [
                mean + (0.2 - mean) * decay,
                mean + (0.9 - mean) * decay,
                mean + (0.2 - mean) * decay,
            ]
        })
        .collect()
}

/// Print validation checks against the experiment predictions.
fn validate(computed: &[[f64; 3]], analytical: &[[f64; 3]]) {
    // P1.2: Monotonic equilibration
    // T₂ should decrease, T₁ and T₃ should increase along the path
    let mut t2_monotonic = true;
    let mut t1_monotonic = true;
    for i in 1..computed.len() {
        if computed[i][1] > computed[i - 1][1] + 1e-6 {
            t2_monotonic = false;
        }
        if computed[i][0] < computed[i - 1][0] - 1e-6 {
            t1_monotonic = false;
        }
    }
    println!(
        "P1.2 T₂ monotonically decreasing: {}",
        if t2_monotonic { "PASS" } else { "FAIL" }
    );
    println!(
        "P1.2 T₁ monotonically increasing: {}",
        if t1_monotonic { "PASS" } else { "FAIL" }
    );

    // P1.3: Conservation — T₁ + T₂ + T₃ ≈ 1.3 throughout
    let initial_sum = 0.2 + 0.9 + 0.2;
    let mut max_deviation = 0.0f64;
    for p in computed {
        let sum = p[0] + p[1] + p[2];
        let dev = (sum - initial_sum).abs();
        max_deviation = max_deviation.max(dev);
    }
    println!(
        "P1.3 Conservation (max deviation from {:.1}): {:.4} — {}",
        initial_sum,
        max_deviation,
        if max_deviation < 0.02 { "PASS" } else { "FAIL" }
    );

    // P1.1: Trajectory match against analytical
    // Sample 10 evenly spaced points along the computed path
    // and find the closest point on the analytical curve
    let mut max_distance = 0.0f64;
    let sample_count = 10;
    for s in 0..sample_count {
        let ci = s * (computed.len() - 1) / (sample_count - 1);
        let cp = computed[ci];

        let min_dist = analytical
            .iter()
            .map(|ap| {
                let dx = cp[0] - ap[0];
                let dy = cp[1] - ap[1];
                let dz = cp[2] - ap[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .fold(f64::INFINITY, f64::min);

        max_distance = max_distance.max(min_dist);
    }
    println!(
        "P1.1 Max trajectory deviation: {:.4} — {}",
        max_distance,
        if max_distance < 0.05 { "PASS" } else { "FAIL" }
    );
}