use serde::Serialize;
use crate::types::Point3;

/// A computed geodesic path through the cube.
#[derive(Debug, Serialize)]
pub struct GeodesicPath {
    /// Ordered sequence of points along the path.
    pub points: Vec<Point3>,

    /// Total cost of traversing the path under the metric.
    pub total_cost: f64,
}
