use crate::metric::Metric;
use crate::path::GeodesicPath;
use crate::space::Space;
use crate::types::Point3;
use ordered_float::NotNan;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Far,
    Considered,
    Accepted,
}
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
    let n = _resolution;
    let h: f64 = 1.0 / n as f64;
    let total = n * n * n;

    let idx = |x: usize, y: usize, z: usize| z * n * n + y * n + x;
    let coords = |i: usize| (i % n, (i / n) % n, i / (n * n));
    let to_grid = |p: &Point3| -> usize {
        let x = ((p[0] / h).floor() as usize).min(n - 1);
        let y = ((p[1] / h).floor() as usize).min(n - 1);
        let z = ((p[2] / h).floor() as usize).min(n - 1);
        idx(x, y, z)
    };

    // ── Allocate grid ──
    let mut cost = vec![f64::INFINITY; total];
    let mut state = vec![CellState::Far; total];
    let mut heap: BinaryHeap<Reverse<FmmEntry>> = BinaryHeap::new();

    // ── Initialize source ──
    let start_idx = to_grid(&_start);
    cost[start_idx] = 0.0;
    state[start_idx] = CellState::Accepted;

    // Seed the heap with the source's neighbours
    let (sx, sy, sz) = coords(start_idx);
    update_neighbours(sx, sy, sz, n, h, _metric, &mut cost, &mut state, &mut heap);

    // ── Main loop ──
    while let Some(Reverse(entry)) = heap.pop() {
        let i = entry.index;

        // Stale entry check: this cell may have been Accepted
        // since we pushed this entry. The heap can contain
        // multiple entries for the same cell (from different
        // neighbour updates). We skip any that are already final.
        if state[i] == CellState::Accepted {
            continue;
        }

        state[i] = CellState::Accepted;
        cost[i] = entry.cost.into_inner();

        let (x, y, z) = coords(i);
        update_neighbours(x, y, z, n, h, _metric, &mut cost, &mut state, &mut heap);
    }

    // ── Extract geodesic ──
    let end_idx = to_grid(&_end);
    let points = extract_path(&cost, end_idx, n, h);
    let total_cost = cost[end_idx];
    GeodesicPath { points, total_cost }
}

/// Solve the eikonal equation locally at one grid cell.
///
/// Given the minimum accepted costs along each axis and the local
/// friction, compute the minimum cost to reach this cell.
///
/// axis_costs: up to 3 values, one per axis. f64::INFINITY means
///             no accepted neighbor exists on that axis.
/// h:          grid spacing (1.0 / resolution)
/// friction:   local cost of traversal at this cell
pub fn solve_local(axis_costs: &[f64], h: f64, friction: f64) -> f64 {
    let mut vals: Vec<f64> = axis_costs
        .iter()
        .copied()
        .filter(|&cost| cost.is_finite())
        .collect();

    if vals.is_empty() {
        return f64::INFINITY;
    }

    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let hf: f64 = h * friction;
    let mut t = vals[0] + hf;

    for k in 1..vals.len() {
        if t <= vals[k] {
            break;
        }

        let n: f64 = (k + 1) as f64;
        let sum: f64 = vals[..=k].iter().sum();
        let sum_sq: f64 = vals[..=k].iter().map(|val| val * val).sum();

        let a: f64 = n;
        let b: f64 = -2.0 * sum;
        let c: f64 = sum_sq - (hf * hf);

        let discriminant = (b * b) - 4.0 * (a * c);

        if discriminant >= 0.0 {
            let candidate: f64 = (-b + discriminant.sqrt()) / (2.0 * a);
            if candidate > vals[k] {
                t = candidate;
            }
        }
    }
    t
}

fn update_neighbours(
    x: usize,
    y: usize,
    z: usize,
    n: usize,
    h: f64,
    metric: &impl Metric,
    cost: &mut [f64],
    state: &mut [CellState],
    heap: &mut BinaryHeap<Reverse<FmmEntry>>,
) {
    let idx = |x: usize, y: usize, z: usize| z * n * n + y * n + x;

    // The 6 face-neighbours in 3D: ±x, ±y, ±z
    let neighbours: [(isize, isize, isize); 6] = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

    for (dx, dy, dz) in &neighbours {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        let nz = z as isize + dz;

        // Bounds check
        if nx < 0 || ny < 0 || nz < 0 {
            continue;
        }
        let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
        if nx >= n || ny >= n || nz >= n {
            continue;
        }

        let ni = idx(nx, ny, nz);
        if state[ni] == CellState::Accepted {
            continue;
        }

        // Gather the minimum accepted cost along each axis
        let axis_costs = [
            axis_min(nx, ny, nz, 0, n, cost, state), // x-axis
            axis_min(nx, ny, nz, 1, n, cost, state), // y-axis
            axis_min(nx, ny, nz, 2, n, cost, state), // z-axis
        ];

        let point = [nx as f64 * h, ny as f64 * h, nz as f64 * h];
        let direction = [*dx as f64, *dy as f64, *dz as f64];
        let friction = metric.cost(&point, &direction);

        let new_cost = solve_local(&axis_costs, h, friction);

        if new_cost < cost[ni] {
            cost[ni] = new_cost;
            state[ni] = CellState::Considered;
            heap.push(Reverse(FmmEntry {
                cost: NotNan::new(new_cost).unwrap(),
                index: ni,
            }));
        }
    }
}

fn axis_min(
    x: usize,
    y: usize,
    z: usize,
    axis: usize,
    n: usize,
    cost: &[f64],
    state: &[CellState],
) -> f64 {
    let idx = |x: usize, y: usize, z: usize| z * n * n + y * n + x;
    let mut min_cost = f64::INFINITY;

    // Check both directions along this axis
    for &delta in &[-1isize, 1isize] {
        let (nx, ny, nz) = match axis {
            0 => (x as isize + delta, y as isize, z as isize),
            1 => (x as isize, y as isize + delta, z as isize),
            2 => (x as isize, y as isize, z as isize + delta),
            _ => unreachable!(),
        };

        if nx < 0 || ny < 0 || nz < 0 {
            continue;
        }
        let (nx, ny, nz) = (nx as usize, ny as usize, nz as usize);
        if nx >= n || ny >= n || nz >= n {
            continue;
        }

        let ni = idx(nx, ny, nz);
        if state[ni] == CellState::Accepted && cost[ni] < min_cost {
            min_cost = cost[ni];
        }
    }

    min_cost
}
fn extract_path(cost: &[f64], end_idx: usize, n: usize, h: f64) -> Vec<Point3> {
    let idx = |x: usize, y: usize, z: usize| z * n * n + y * n + x;
    let coords = |i: usize| (i % n, (i / n) % n, i / (n * n));

    // Raw descent trajectory: (world point, cost T at that point).
    // The cost parametrization is what carries the domain's nonlinearity;
    // we resample by it after the descent.
    let mut trajectory: Vec<(Point3, f64)> = Vec::new();

    // Start at the end point as continuous coordinates
    let (ex, ey, ez) = coords(end_idx);
    let mut px = ex as f64;
    let mut py = ey as f64;
    let mut pz = ez as f64;

    let step_size = 0.5; // half a grid cell per step
    let max_steps = n * n * n;

    for _ in 0..max_steps {
        // Snap to nearest grid cell to read the distance field
        let gx = (px.round() as usize).min(n - 1);
        let gy = (py.round() as usize).min(n - 1);
        let gz = (pz.round() as usize).min(n - 1);

        // Record the current position with its cost value
        trajectory.push(([px * h, py * h, pz * h], cost[idx(gx, gy, gz)]));

        if cost[idx(gx, gy, gz)] < h {
            // Close enough to the source
            trajectory.push(([gx as f64 * h, gy as f64 * h, gz as f64 * h], 0.0));
            break;
        }

        // Estimate gradient using central differences
        // For each axis: (cost at +1) - (cost at -1) / (2h)
        // At boundaries, use one-sided differences

        let grad_x = if gx == 0 {
            cost[idx(gx + 1, gy, gz)] - cost[idx(gx, gy, gz)]
        } else if gx == n - 1 {
            cost[idx(gx, gy, gz)] - cost[idx(gx - 1, gy, gz)]
        } else {
            (cost[idx(gx + 1, gy, gz)] - cost[idx(gx - 1, gy, gz)]) / 2.0
        };

        let grad_y = if gy == 0 {
            cost[idx(gx, gy + 1, gz)] - cost[idx(gx, gy, gz)]
        } else if gy == n - 1 {
            cost[idx(gx, gy, gz)] - cost[idx(gx, gy - 1, gz)]
        } else {
            (cost[idx(gx, gy + 1, gz)] - cost[idx(gx, gy - 1, gz)]) / 2.0
        };

        let grad_z = if gz == 0 {
            cost[idx(gx, gy, gz + 1)] - cost[idx(gx, gy, gz)]
        } else if gz == n - 1 {
            cost[idx(gx, gy, gz)] - cost[idx(gx, gy, gz - 1)]
        } else {
            (cost[idx(gx, gy, gz + 1)] - cost[idx(gx, gy, gz - 1)]) / 2.0
        };

        // Normalise the gradient
        let mag = (grad_x * grad_x + grad_y * grad_y + grad_z * grad_z).sqrt();

        if mag < 1e-10 {
            break; // flat region, no direction to follow
        }

        // Step downhill (negative gradient direction)
        px -= step_size * grad_x / mag;
        py -= step_size * grad_y / mag;
        pz -= step_size * grad_z / mag;

        // Clamp to grid bounds
        px = px.max(0.0).min((n - 1) as f64);
        py = py.max(0.0).min((n - 1) as f64);
        pz = pz.max(0.0).min((n - 1) as f64);
    }

    trajectory.reverse();
    resample_by_cost(&trajectory)
}

/// Resample a (point, cost) trajectory at equal cost increments.
///
/// The descent produces points at uniform arclength, which erases the
/// cost parametrization. Emitting points at uniform ΔT restores it:
/// spacing shrinks where friction is high and stretches where it is low.
fn resample_by_cost(trajectory: &[(Point3, f64)]) -> Vec<Point3> {
    if trajectory.len() < 2 {
        return trajectory.iter().map(|(p, _)| *p).collect();
    }

    let num_points = trajectory.len();
    let t_start = trajectory[0].1;
    let t_end = trajectory[trajectory.len() - 1].1;
    let span = t_end - t_start;

    if span.abs() < 1e-12 {
        return trajectory.iter().map(|(p, _)| *p).collect();
    }

    let mut path = Vec::with_capacity(num_points);
    let mut cursor = 0;

    for i in 0..num_points {
        let target = t_start + span * (i as f64 / (num_points - 1) as f64);

        // Advance to the segment containing the target cost.
        // Costs along the reversed trajectory increase from source to
        // target, but grid snapping can produce small non-monotonic
        // wiggles, so we only ever move the cursor forward.
        while cursor + 1 < num_points - 1 && trajectory[cursor + 1].1 < target {
            cursor += 1;
        }

        let (p0, t0) = trajectory[cursor];
        let (p1, t1) = trajectory[cursor + 1];
        let frac = if (t1 - t0).abs() < 1e-12 {
            0.0
        } else {
            ((target - t0) / (t1 - t0)).max(0.0).min(1.0)
        };

        path.push([
            p0[0] + frac * (p1[0] - p0[0]),
            p0[1] + frac * (p1[1] - p0[1]),
            p0[2] + frac * (p1[2] - p0[2]),
        ]);
    }

    path
}
struct FmmEntry {
    cost: NotNan<f64>,
    index: usize,
}

impl Ord for FmmEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}
impl PartialOrd for FmmEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for FmmEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Eq for FmmEntry {}
