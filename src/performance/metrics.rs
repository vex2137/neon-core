//! Zero-allocation rolling frame-time and entity telemetry tracker.

const WINDOW_LEN: usize = 120;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct FrameSnapshot {
    pub avg_ms: f32,
    pub max_ms: f32,
    pub enemies: usize,
    pub bullets: usize,
    pub e_bullets: usize,
    pub particles: usize,
    pub grid_cells_used: usize,
}

pub struct PerfMetrics {
    samples_ms: [f32; WINDOW_LEN],
    cursor: usize,
    count: usize,
    last_snapshot: FrameSnapshot,
}

impl PerfMetrics {
    pub fn new() -> Self {
        Self {
            samples_ms: [0.0; WINDOW_LEN],
            cursor: 0,
            count: 0,
            last_snapshot: FrameSnapshot {
                avg_ms: 0.0,
                max_ms: 0.0,
                enemies: 0,
                bullets: 0,
                e_bullets: 0,
                particles: 0,
                grid_cells_used: 0,
            },
        }
    }

    pub fn record_frame(
        &mut self,
        dt_seconds: f32,
        enemies: usize,
        bullets: usize,
        e_bullets: usize,
        particles: usize,
        grid_cells_used: usize,
    ) {
        let ms = if dt_seconds.is_finite() {
            (dt_seconds * 1000.0).clamp(0.0, 1000.0)
        } else {
            0.0
        };
        self.samples_ms[self.cursor] = ms;
        self.cursor = (self.cursor + 1) % WINDOW_LEN;
        if self.count < WINDOW_LEN {
            self.count += 1;
        }

        let mut sum = 0.0_f32;
        let mut max_ms = 0.0_f32;
        for &s in &self.samples_ms[..self.count] {
            sum += s;
            if s > max_ms {
                max_ms = s;
            }
        }

        self.last_snapshot = FrameSnapshot {
            avg_ms: sum / self.count.max(1) as f32,
            max_ms,
            enemies,
            bullets,
            e_bullets,
            particles,
            grid_cells_used,
        };
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> FrameSnapshot {
        self.last_snapshot
    }
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_metrics_tracks_average_and_peak() {
        let mut m = PerfMetrics::new();
        m.record_frame(0.016, 10, 20, 5, 30, 8);
        m.record_frame(0.032, 12, 22, 6, 35, 9);
        let snap = m.snapshot();
        assert!((snap.avg_ms - 24.0).abs() < 0.01);
        assert!((snap.max_ms - 32.0).abs() < 0.01);
        assert_eq!(snap.enemies, 12);
        assert_eq!(snap.bullets, 22);
        assert_eq!(snap.e_bullets, 6);
        assert_eq!(snap.particles, 35);
        assert_eq!(snap.grid_cells_used, 9);
    }
}
