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

/// Friction derived from the heat equation on a 3-point rod.
///
/// At each point p = (T₁, T₂, T₃), the heat equation defines
/// a preferred direction of change. Movement aligned with this
/// direction is cheap; movement against it is expensive.
///
/// cost(p, v) = ‖v‖ × (1 + λ × sin²θ)
///
/// where θ is the angle between v and the heat flow direction h(p),
/// and λ controls the anisotropy strength.
pub struct HeatFlowMetric {
    pub kappa: f64,   // thermal diffusivity
    pub lambda: f64,  // anisotropy strength
}

impl Metric for HeatFlowMetric {
    fn cost(&self, p: &Point3, dir: &Vector3) -> f64 {
        let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if norm < 1e-12 {
            return 0.0;
        }

        // Heat equation preferred direction at this point
        let h = [
            self.kappa * (p[1] - p[0]),
            self.kappa * (p[0] - 2.0 * p[1] + p[2]),
            self.kappa * (p[1] - p[2]),
        ];

        let h_norm = (h[0] * h[0] + h[1] * h[1] + h[2] * h[2]).sqrt();

        // At equilibrium, h is zero — no preferred direction, isotropic cost
        if h_norm < 1e-12 {
            return norm;
        }

        // cos²θ = (v · h)² / (‖v‖² × ‖h‖²)
        let dot = dir[0] * h[0] + dir[1] * h[1] + dir[2] * h[2];
        let cos2 = (dot * dot) / (norm * norm * h_norm * h_norm);
        let sin2 = 1.0 - cos2;

        norm * (1.0 + self.lambda * sin2)
    }
}

/// Friction derived from Kubelka-Munk subtractive colour mixing.
///
/// Two pigments define a mixing curve through RGB space.
/// Movement along the curve is cheap; movement away from it is expensive.
/// The preferred direction at any point is the tangent to the mixing
/// curve at the nearest point on the curve.
pub struct KubelkaMunkMetric {
    pub lambda: f64,
    /// Precomputed mixing curve: (point, tangent) pairs
    curve: Vec<([f64; 3], [f64; 3])>,
}

impl KubelkaMunkMetric {
    /// Construct from two pigment colours (as RGB in [0,1]³).
    pub fn new(pigment_a: [f64; 3], pigment_b: [f64; 3], lambda: f64) -> Self {
        // Convert RGB reflectances to K/S ratios
        let to_ks = |r: f64| -> f64 {
            let r = r.max(0.001).min(0.999); // clamp to avoid division issues
            (1.0 - r) * (1.0 - r) / (2.0 * r)
        };

        let ks_a: [f64; 3] = [to_ks(pigment_a[0]), to_ks(pigment_a[1]), to_ks(pigment_a[2])];
        let ks_b: [f64; 3] = [to_ks(pigment_b[0]), to_ks(pigment_b[1]), to_ks(pigment_b[2])];

        // K/S back to reflectance
        let to_rgb = |ks: f64| -> f64 {
            1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
        };

        // Sample the mixing curve at fine resolution
        let num_samples = 500;
        let points: Vec<[f64; 3]> = (0..=num_samples)
            .map(|i| {
                let t = i as f64 / num_samples as f64;
                // Linear blend in K/S space
                let ks_mix = [
                    (1.0 - t) * ks_a[0] + t * ks_b[0],
                    (1.0 - t) * ks_a[1] + t * ks_b[1],
                    (1.0 - t) * ks_a[2] + t * ks_b[2],
                ];
                // Convert back to RGB
                [to_rgb(ks_mix[0]), to_rgb(ks_mix[1]), to_rgb(ks_mix[2])]
            })
            .collect();

        // Compute tangent at each point via central differences
        let mut curve = Vec::with_capacity(points.len());
        for i in 0..points.len() {
            let tangent = if i == 0 {
                [
                    points[1][0] - points[0][0],
                    points[1][1] - points[0][1],
                    points[1][2] - points[0][2],
                ]
            } else if i == points.len() - 1 {
                let j = points.len() - 2;
                [
                    points[i][0] - points[j][0],
                    points[i][1] - points[j][1],
                    points[i][2] - points[j][2],
                ]
            } else {
                [
                    (points[i + 1][0] - points[i - 1][0]) / 2.0,
                    (points[i + 1][1] - points[i - 1][1]) / 2.0,
                    (points[i + 1][2] - points[i - 1][2]) / 2.0,
                ]
            };
            curve.push((points[i], tangent));
        }

        KubelkaMunkMetric { lambda, curve }
    }

    /// The mixing curve points, for use in analytical comparison.
    pub fn mixing_curve_points(&self) -> Vec<[f64; 3]> {
        self.curve.iter().map(|(p, _)| *p).collect()
    }
}

impl Metric for KubelkaMunkMetric {
    fn cost(&self, p: &Point3, dir: &Vector3) -> f64 {
        let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if norm < 1e-12 {
            return 0.0;
        }

        // Find nearest point on the mixing curve
        let mut best_dist_sq = f64::INFINITY;
        let mut best_tangent = [0.0, 0.0, 0.0];

        for (cp, ct) in &self.curve {
            let dx = p[0] - cp[0];
            let dy = p[1] - cp[1];
            let dz = p[2] - cp[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_tangent = *ct;
            }
        }

        let t_norm = (best_tangent[0] * best_tangent[0]
            + best_tangent[1] * best_tangent[1]
            + best_tangent[2] * best_tangent[2])
            .sqrt();

        // Far from curve or degenerate tangent — isotropic
        if t_norm < 1e-12 {
            return norm;
        }

        let dot = dir[0] * best_tangent[0]
            + dir[1] * best_tangent[1]
            + dir[2] * best_tangent[2];
        let cos2 = (dot * dot) / (norm * norm * t_norm * t_norm);
        let sin2 = 1.0 - cos2;

        norm * (1.0 + self.lambda * sin2)
    }
}