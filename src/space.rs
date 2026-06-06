use crate::types::Point3;

/// A bounded container geometry.
/// The solver and metric operate within this space.
pub trait Space {
    /// Whether a point lies inside the space.
    fn contains(&self, p: &Point3) -> bool;

    /// Minimum distance from the point to the nearest boundary face.
    fn boundary_distance(&self, p: &Point3) -> f64;
}

/// The unit RGB cube [0,1]³.
pub struct CubeSpace;

impl Space for CubeSpace {
    fn contains(&self, p: &Point3) -> bool {
        p.iter().all(|&c| c >= 0.0 && c <= 1.0)
    }

    fn boundary_distance(&self, p: &Point3) -> f64 {
        p.iter()
            .flat_map(|&c| [c, 1.0 - c])
            .fold(f64::INFINITY, f64::min)
    }
}
