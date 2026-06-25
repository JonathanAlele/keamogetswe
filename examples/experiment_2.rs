use keamogetswe::export::{GeodesicOutput, Metadata, PathEntry};
use keamogetswe::metric::KubelkaMunkMetric;
use keamogetswe::solver::solve;
use keamogetswe::space::CubeSpace;

fn main() {
    let phthalo_blue = [0.05, 0.15, 0.55];
    let cadmium_yellow = [0.95, 0.85, 0.15];

    let metric = KubelkaMunkMetric::new(phthalo_blue, cadmium_yellow, 50.0);
    let space = CubeSpace;
    let resolution = 128;

    println!("Running K-M mixing experiment at N={}...", resolution);
    let path = solve(&space, &metric, phthalo_blue, cadmium_yellow, resolution);
    println!("Total cost: {:.4}", path.total_cost);

    // The analytical curve comes directly from the K-M theory
    let analytical = metric.mixing_curve_points();

    // Also compute the straight line for comparison
    let straight: Vec<[f64; 3]> = (0..100)
        .map(|i| {
            let t = i as f64 / 99.0;
            [
                phthalo_blue[0] + t * (cadmium_yellow[0] - phthalo_blue[0]),
                phthalo_blue[1] + t * (cadmium_yellow[1] - phthalo_blue[1]),
                phthalo_blue[2] + t * (cadmium_yellow[2] - phthalo_blue[2]),
            ]
        })
        .collect();

    validate(&path.points, &analytical);

    let output = GeodesicOutput {
        metadata: Metadata {
            name: String::from("Experiment 2: K-M colour mixing"),
            metric: String::from("KubelkaMunkMetric"),
            resolution,
        },
        paths: vec![
            PathEntry {
                label: String::from("computed"),
                points: path.points,
            },
            PathEntry {
                label: String::from("analytical (K-M)"),
                points: analytical,
            },
            PathEntry {
                label: String::from("straight line"),
                points: straight,
            },
        ],
    };

    let json = output.to_json().unwrap();
    std::fs::write("viewer/sample_data/experiment_2.json", json).unwrap();
    println!("Written to viewer/sample_data/experiment_2.json");
}

fn validate(computed: &[[f64; 3]], analytical: &[[f64; 3]]) {
    // P2.1: Path follows K-M curve
    let mut max_distance = 0.0f64;
    let sample_count = 11;
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
        "P2.1 Max K-M curve deviation: {:.4} — {}",
        max_distance,
        if max_distance < 0.05 { "PASS" } else { "FAIL" }
    );

    // P2.2: Green intermediates — G > R and G > B at some point
    let has_green = computed.iter().any(|p| p[1] > p[0] && p[1] > p[2]);
    println!(
        "P2.2 Green intermediates (G > R and G > B): {}",
        if has_green { "PASS" } else { "FAIL" }
    );

    // P2.3: Nonlinear spacing — ratio of max to min inter-point distance
    if computed.len() > 2 {
        let distances: Vec<f64> = computed
            .windows(2)
            .map(|w| {
                let dx = w[1][0] - w[0][0];
                let dy = w[1][1] - w[0][1];
                let dz = w[1][2] - w[0][2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .filter(|&d| d > 1e-10)
            .collect();

        if let (Some(&min_d), Some(&max_d)) = (
            distances.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
            distances.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
        ) {
            let ratio = max_d / min_d;
            println!(
                "P2.3 Spacing ratio (max/min): {:.2} — {}",
                ratio,
                if ratio > 1.5 { "PASS" } else { "FAIL" }
            );
        }
    }
}