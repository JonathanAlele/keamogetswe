use crate::types::Point3;
use crate::space::Space;
use crate::metric::Metric;
use crate::path::GeodesicPath;

/// Compute the geodesic between two points using the Fast Marching Method.
///
/// This is the core computation. It builds a distance field from `start`
/// using the FMM, then backtracks from `end` to extract the shortest path.
pub fn solve(
    _space: &impl Space,
    _metric: &impl Metric,
    _start: Point3,
    _end: Point3,
    _resolution: usize,
) -> GeodesicPath {
    // TODO: Implement FMM solver
    todo!("FMM solver not yet implemented")
}
