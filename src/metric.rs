use crate::types::{Point3, Vector3};

/// Direction-dependent cost of movement at a point.
/// This is the friction — the core abstraction of the framework.
pub trait Metric {
    /// The instantaneous cost of moving in direction `dir` at point `p`.
    /// Must be positive and continuous.
    fn cost(&self, p: &Point3, dir: &Vector3) -> f64;
}

/// Uniform cost everywhere. Geodesics are straight lines.
pub struct EuclideanMetric;

impl Metric for EuclideanMetric {
    fn cost(&self, _p: &Point3, dir: &Vector3) -> f64 {
        let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        norm
    }
}
