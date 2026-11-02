//! Shared boundary policy for user-supplied float knobs.
//!
//! Every public knob that takes a raw `f32` derived from caller state
//! (container widths, spring parameters — NaN long before first measure,
//! infinity from a divide-by-zero) sanitizes at the builder boundary with
//! the same rule, so use sites only ever see sane values.

/// A finite value strictly above `min` passes through; anything else
/// (NaN, ±∞, or at/below `min`) falls back to `default`.
pub const fn finite_above_or(raw: f32, min: f32, default: f32) -> f32 {
    if raw.is_finite() && raw > min {
        raw
    } else {
        default
    }
}

/// [`finite_above_or`] with an inclusive floor — for knobs whose zero is
/// legal (e.g. spring damping, the undamped oscillator).
pub const fn finite_at_least_or(raw: f32, min: f32, default: f32) -> f32 {
    if raw.is_finite() && raw >= min {
        raw
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_above_min_passes_through() {
        assert_eq!(finite_above_or(150.0, 0.0, 280.0), 150.0);
        assert_eq!(finite_above_or(0.5, 0.0, 280.0), 0.5);
    }

    #[test]
    fn garbage_falls_back_to_default() {
        assert_eq!(finite_above_or(f32::NAN, 0.0, 280.0), 280.0);
        assert_eq!(finite_above_or(f32::INFINITY, 0.0, 280.0), 280.0);
        assert_eq!(finite_above_or(f32::NEG_INFINITY, 0.0, 280.0), 280.0);
        assert_eq!(finite_above_or(0.0, 0.0, 280.0), 280.0);
        assert_eq!(finite_above_or(-4.0, 0.0, 280.0), 280.0);
    }

    #[test]
    fn inclusive_floor_admits_zero() {
        assert_eq!(finite_at_least_or(0.0, 0.0, 20.0), 0.0);
        assert_eq!(finite_at_least_or(5.0, 0.0, 20.0), 5.0);
        assert_eq!(finite_at_least_or(f32::NAN, 0.0, 20.0), 20.0);
        assert_eq!(finite_at_least_or(-0.1, 0.0, 20.0), 20.0);
        assert_eq!(finite_at_least_or(f32::INFINITY, 0.0, 20.0), 20.0);
    }
}
