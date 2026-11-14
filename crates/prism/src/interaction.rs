//! Pointer normalization and easing, independent from GPU ownership.
//! Source: components/prism-background/runtime/interaction.ts.

use crate::clamp;
use crate::types::{CAMERA_ORBIT_LERP, LAMP_AIM_LERP, PRISM_DEFAULT_ARC};

type Pair = [f64; 2];

const AUTO_POINTER_PERIOD_SECONDS: f64 = 18.0;
const AUTO_POINTER_RADIUS: f64 = 0.34;

pub struct PrismInteraction {
    aim_target: Pair,
    aim_current: Pair,
    orbit_target: Pair,
    orbit_current: Pair,
}

impl Default for PrismInteraction {
    fn default() -> Self {
        Self {
            aim_target: [PRISM_DEFAULT_ARC, 0.5],
            aim_current: [PRISM_DEFAULT_ARC, 0.5],
            orbit_target: [0.0, 0.0],
            orbit_current: [0.0, 0.0],
        }
    }
}

impl PrismInteraction {
    /// Feeds the same normalized path as a real pointer without DOM events.
    pub fn set_normalized_pointer(&mut self, position: Pair) {
        let x = clamp(position[0], 0.0, 1.0);
        let y = clamp(position[1], 0.0, 1.0);
        // Height swings the source; width chooses the point of impact.
        self.aim_target = [y, x];
        // Hovering tilts only the camera, not the world-space light mesh.
        self.orbit_target = [x * 2.0 - 1.0, y * 2.0 - 1.0];
    }

    /// Returns whether the DOM adapter must invalidate the renderer.
    pub fn on_pointer_move(
        &mut self,
        is_primary: bool,
        client: Pair,
        canvas_origin: Pair,
        canvas_size: Pair,
    ) -> bool {
        if !is_primary || canvas_size[0] <= 0.0 || canvas_size[1] <= 0.0 {
            return false;
        }
        let x = clamp((client[0] - canvas_origin[0]) / canvas_size[0], 0.0, 1.0);
        let y = clamp((client[1] - canvas_origin[1]) / canvas_size[1], 0.0, 1.0);
        self.set_normalized_pointer([x, y]);
        true
    }

    pub fn on_pointer_leave(&mut self) {
        self.orbit_target = [0.0, 0.0];
    }

    /// Returns the eased value only when the lamp moved this frame.
    pub fn step_aim(&mut self) -> Option<Pair> {
        let next = eased_pair(self.aim_current, self.aim_target, LAMP_AIM_LERP);
        if let Some(next) = next {
            self.aim_current = next;
        }
        next
    }

    /// Returns the eased value only when the camera orbit moved this frame.
    pub fn step_orbit(&mut self) -> Option<Pair> {
        let next = eased_pair(self.orbit_current, self.orbit_target, CAMERA_ORBIT_LERP);
        if let Some(next) = next {
            self.orbit_current = next;
        }
        next
    }
}

/// A slow centered circle used as the virtual pointer on mobile layouts.
pub fn automatic_pointer_position(time_seconds: f64) -> Pair {
    let time = if time_seconds.is_finite() {
        time_seconds
    } else {
        0.0
    };
    let phase = (time / AUTO_POINTER_PERIOD_SECONDS) * std::f64::consts::PI * 2.0;
    [
        0.5 + phase.cos() * AUTO_POINTER_RADIUS,
        0.5 + phase.sin() * AUTO_POINTER_RADIUS,
    ]
}

fn eased_pair(current: Pair, target: Pair, amount: f64) -> Option<Pair> {
    let dx = target[0] - current[0];
    let dy = target[1] - current[1];
    if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
        if current == target {
            None
        } else {
            Some(target)
        }
    } else {
        Some([current[0] + dx * amount, current[1] + dy * amount])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Pair, b: Pair) {
        assert!((a[0] - b[0]).abs() < 1e-12);
        assert!((a[1] - b[1]).abs() < 1e-12);
    }

    #[test]
    fn mobile_pointer_follows_original_circle() {
        for (time, expected) in [
            (0.0, [0.84, 0.5]),
            (4.5, [0.5, 0.84]),
            (9.0, [0.16, 0.5]),
            (18.0, [0.84, 0.5]),
            (f64::NAN, [0.84, 0.5]),
            (f64::INFINITY, [0.84, 0.5]),
        ] {
            close(automatic_pointer_position(time), expected);
        }
    }

    #[test]
    fn automatic_and_real_pointers_share_easing() {
        let mut virtual_pointer = PrismInteraction::default();
        let mut real_pointer = PrismInteraction::default();
        virtual_pointer.set_normalized_pointer([0.84, 0.5]);
        assert!(real_pointer.on_pointer_move(true, [284.0, 130.0], [200.0, 80.0], [100.0, 100.0]));
        close(virtual_pointer.step_aim().unwrap(), [0.5, 0.5408]);
        close(virtual_pointer.step_orbit().unwrap(), [0.0544, 0.0]);
        close(real_pointer.step_aim().unwrap(), [0.5, 0.5408]);
        close(real_pointer.step_orbit().unwrap(), [0.0544, 0.0]);
    }

    #[test]
    fn easing_settles_and_leaving_resets_orbit_only() {
        let mut interaction = PrismInteraction::default();
        assert_eq!(interaction.step_aim(), None);
        assert_eq!(interaction.step_orbit(), None);
        assert!(!interaction.on_pointer_move(false, [1.0, 1.0], [0.0, 0.0], [100.0, 100.0]));
        assert!(!interaction.on_pointer_move(true, [1.0, 1.0], [0.0, 0.0], [0.0, 100.0]));
        interaction.set_normalized_pointer([2.0, -1.0]);
        for _ in 0..200 {
            interaction.step_aim();
            interaction.step_orbit();
        }
        assert_eq!(interaction.aim_current, [0.0, 1.0]);
        assert_eq!(interaction.orbit_current, [1.0, -1.0]);
        assert_eq!(interaction.step_orbit(), None);
        interaction.on_pointer_leave();
        assert_eq!(interaction.step_aim(), None);
        for _ in 0..200 {
            interaction.step_orbit();
        }
        assert_eq!(interaction.orbit_current, [0.0, 0.0]);
    }
}
