//! Fixed-capacity intrusive linked-list spatial hash grid for O(1) local queries.
//!
//! Arena coordinates span `[-ARENA_RADIUS, +ARENA_RADIUS]` (`[-2800, 2800]`).
//! With `CELL_SIZE = 128.0`, `GRID_DIM = 48`, and `GRID_HALF = 24`, the grid
//! covers `[-3072.0, +3072.0]` on both axes with zero heap allocations per frame.

use macroquad::math::Vec2;

pub const CELL_SIZE: f32 = 128.0;
pub const GRID_DIM: usize = 48;
pub const GRID_HALF: i32 = 24;
pub const GRID_CELLS: usize = GRID_DIM * GRID_DIM;

#[inline]
pub fn cell_coord(v: f32) -> usize {
    if !v.is_finite() {
        return GRID_HALF as usize;
    }
    let c = (v / CELL_SIZE).floor() as i32 + GRID_HALF;
    c.clamp(0, (GRID_DIM - 1) as i32) as usize
}

pub struct SpatialGrid {
    cell_head: Vec<i32>,
    next: Vec<i32>,
    occupied_cells: usize,
    indexed_count: usize,
}

impl SpatialGrid {
    pub fn with_capacity(max_entities: usize) -> Self {
        Self {
            cell_head: vec![-1; GRID_CELLS],
            next: vec![-1; max_entities],
            occupied_cells: 0,
            indexed_count: 0,
        }
    }

    /// Rebuilds the grid in-place from an iterator of `(index, alive, pos)`.
    pub fn rebuild<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (usize, bool, Vec2)>,
    {
        self.cell_head.fill(-1);
        self.occupied_cells = 0;
        self.indexed_count = 0;

        for (i, alive, pos) in items {
            if !alive || i >= self.next.len() || !pos.x.is_finite() || !pos.y.is_finite() {
                continue;
            }
            let cx = cell_coord(pos.x);
            let cy = cell_coord(pos.y);
            let cell = cy * GRID_DIM + cx;
            if self.cell_head[cell] == -1 {
                self.occupied_cells += 1;
            }
            self.next[i] = self.cell_head[cell];
            self.cell_head[cell] = i as i32;
            self.indexed_count += 1;
        }
    }

    /// Visits every indexed entity inside the AABB `[origin - half_extent, origin + half_extent]`.
    /// Returning `false` from `visitor` stops iteration immediately.
    #[inline]
    pub fn for_each_in_aabb<F>(&self, origin: Vec2, half_extent: f32, mut visitor: F)
    where
        F: FnMut(usize) -> bool,
    {
        if !origin.x.is_finite() || !origin.y.is_finite() || !half_extent.is_finite() {
            return;
        }
        let span = half_extent.max(0.0);
        let cx0 = cell_coord(origin.x - span);
        let cx1 = cell_coord(origin.x + span);
        let cy0 = cell_coord(origin.y - span);
        let cy1 = cell_coord(origin.y + span);

        for cy in cy0..=cy1 {
            let row = cy * GRID_DIM;
            for cx in cx0..=cx1 {
                let mut j = self.cell_head[row + cx];
                while j != -1 {
                    let idx = j as usize;
                    j = self.next[idx];
                    if !visitor(idx) {
                        return;
                    }
                }
            }
        }
    }

    /// Finds the nearest indexed entity within `max_range` satisfying `predicate`.
    pub fn find_nearest<F>(&self, origin: Vec2, max_range: f32, mut get_pos: F) -> Option<usize>
    where
        F: FnMut(usize) -> Option<Vec2>,
    {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || !max_range.is_finite()
            || max_range <= 0.0
        {
            return None;
        }
        let mut best = None;
        let mut best_d2 = max_range * max_range;
        self.for_each_in_aabb(origin, max_range, |idx| {
            if let Some(pos) = get_pos(idx) {
                let d2 = pos.distance_squared(origin);
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = Some(idx);
                }
            }
            true
        });
        best
    }

    pub fn occupied_cells(&self) -> usize {
        self.occupied_cells
    }

    #[cfg(test)]
    pub fn indexed_count(&self) -> usize {
        self.indexed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::math::vec2;

    #[test]
    fn cell_coord_clamps_boundaries_and_handles_nan() {
        assert_eq!(cell_coord(-10_000.0), 0);
        assert_eq!(cell_coord(10_000.0), GRID_DIM - 1);
        assert_eq!(cell_coord(0.0), GRID_HALF as usize);
        assert_eq!(cell_coord(f32::NAN), GRID_HALF as usize);
    }

    #[test]
    fn query_finds_neighbors_across_cell_boundaries() {
        let mut grid = SpatialGrid::with_capacity(16);
        let pts = [
            (true, vec2(126.0, 0.0)),   // cell x = 24
            (true, vec2(130.0, 0.0)),   // cell x = 25 (just across boundary)
            (false, vec2(127.0, 0.0)),  // dead, must be excluded
            (true, vec2(900.0, 900.0)), // far away
        ];
        grid.rebuild(pts.iter().enumerate().map(|(i, &(a, p))| (i, a, p)));

        assert_eq!(grid.indexed_count(), 3);
        let mut found = Vec::new();
        grid.for_each_in_aabb(vec2(128.0, 0.0), 20.0, |idx| {
            found.push(idx);
            true
        });
        found.sort_unstable();
        assert_eq!(found, vec![0, 1]);

        let nearest = grid.find_nearest(vec2(129.0, 0.0), 50.0, |idx| Some(pts[idx].1));
        assert_eq!(nearest, Some(1));
    }

    #[test]
    fn rebuild_clears_previous_frame_entries() {
        let mut grid = SpatialGrid::with_capacity(8);
        grid.rebuild([(0, true, vec2(10.0, 10.0))]);
        assert_eq!(grid.indexed_count(), 1);

        grid.rebuild(std::iter::empty());
        assert_eq!(grid.indexed_count(), 0);
        assert_eq!(grid.occupied_cells(), 0);
        let mut hits = 0;
        grid.for_each_in_aabb(vec2(10.0, 10.0), 100.0, |_| {
            hits += 1;
            true
        });
        assert_eq!(hits, 0);
    }
}
