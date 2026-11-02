//! Animation utilities for smooth, physics-based motion.
//!
//! Provides timed animations with easing, spring physics, staggered
//! entrance effects, and looping support.

/// Easing functions for smooth animations
pub mod easing;
/// Interpolation trait for animating values
pub mod interpolate;
pub mod momentum;
pub mod staggered;
pub mod velocity_drag;

pub use easing::EasingFunction;
pub use interpolate::Interpolate;
pub use momentum::{
    ContinuousWithMomentum, MomentumBehavior, MomentumPosition, SnapToPageBoundaries,
};
pub use staggered::{AnimationSequence, LoopMode, LoopingAnimation, StaggeredAnimation};
pub use velocity_drag::{DoubleClickReset, DragMode, VelocityDrag, VelocityDragConfig};

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Animation hasn't started yet
    NotStarted,
    /// Animation is currently running
    Running,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
}

/// A generic animation that interpolates between two values over time
///
/// # Example
///
/// ```rust
/// use armas_basic::animation::{Animation, EasingFunction};
///
/// let mut anim = Animation::new(0.0_f32, 1.0, 0.3)
///     .easing(EasingFunction::EaseOut);
/// anim.start();
/// anim.update(0.15);
/// let value = anim.value(); // interpolated between 0.0 and 1.0
/// ```
#[derive(Debug, Clone)]
pub struct Animation<T: Interpolate> {
    /// Starting value
    pub start: T,
    /// Target value
    pub end: T,
    /// Duration of the animation in seconds
    pub duration: f32,
    /// Elapsed time in seconds
    pub elapsed: f32,
    /// Easing function to apply
    pub easing: EasingFunction,
    /// Current state of the animation
    pub state: AnimationState,
}

impl<T: Interpolate> Animation<T> {
    /// Create a new animation from start to end over duration
    pub const fn new(start: T, end: T, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing: EasingFunction::EaseInOut,
            state: AnimationState::NotStarted,
        }
    }

    /// Set the easing function
    #[must_use]
    pub const fn easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Start the animation
    pub const fn start(&mut self) {
        self.state = AnimationState::Running;
        self.elapsed = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    /// Reset the animation to the beginning
    pub const fn reset(&mut self) {
        self.elapsed = 0.0;
        self.state = AnimationState::NotStarted;
    }

    /// Update the animation with delta time
    pub fn update(&mut self, dt: f32) {
        if self.state != AnimationState::Running {
            return;
        }

        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = AnimationState::Completed;
        }
    }

    /// Get the current value of the animation
    pub fn value(&self) -> T {
        let t = if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        };

        let eased_t = self.easing.apply(t);
        self.start.interpolate(&self.end, eased_t)
    }

    /// Get the normalized progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.state == AnimationState::Completed
    }

    /// Check if the animation is running
    pub fn is_running(&self) -> bool {
        self.state == AnimationState::Running
    }
}

/// Spring-based animation for smooth, physics-based motion
///
/// This uses a damped spring physics model for natural-looking animations.
/// Unlike timed animations, springs don't have a fixed duration - they settle over time.
///
/// # Example
///
/// ```rust
/// use armas_basic::animation::SpringAnimation;
///
/// let mut spring = SpringAnimation::new(0.0, 1.0)
///     .params(200.0, 20.0);
/// spring.update(0.016); // one frame at 60fps
/// let current = spring.value;
/// ```
#[derive(Debug, Clone)]
pub struct SpringAnimation {
    /// Current value
    pub value: f32,
    /// Current velocity
    pub velocity: f32,
    /// Target value
    pub target: f32,
    /// Spring stiffness (higher = faster oscillation, typical: 100-300)
    pub stiffness: f32,
    /// Spring damping (higher = less oscillation, typical: 10-30)
    pub damping: f32,
}

/// Default stiffness — also the fallback for garbage `.params()` input
const DEFAULT_STIFFNESS: f32 = 200.0;
/// Default damping — also the fallback for garbage `.params()` input
const DEFAULT_DAMPING: f32 = 20.0;
/// `alpha * dt` at or beyond this decays to less than e^-80 of the
/// amplitude — below f32 resolution of any animated value. Snap to the
/// target instead of evaluating the map; the overdamped rate never
/// exceeds `alpha`, so this also keeps `cosh`/`sinh` finite (they
/// overflow past an argument of ~90).
const SETTLED_DECAY: f32 = 80.0;
/// Below this (rate * dt) both solution branches are arbitrarily close
/// to the critically-damped one, whose closed form divides by nothing.
const NEAR_CRITICAL: f32 = 1.0e-3;
/// Frame gaps longer than an hour are clamped: the exact map makes this
/// invisible except that it keeps every transcendental argument finite.
const MAX_DT: f32 = 3600.0;

/// Spring-parameter counterpart of the width-knob sanitizers: finite and
/// positive passes, anything else falls back to the default.
const fn sanitize_stiffness(raw: f32) -> f32 {
    if raw.is_finite() && raw > 0.0 {
        raw
    } else {
        DEFAULT_STIFFNESS
    }
}

/// Damping is finite and non-negative (zero is the legal undamped
/// oscillator); anything else falls back to the default.
const fn sanitize_damping(raw: f32) -> f32 {
    if raw.is_finite() && raw >= 0.0 {
        raw
    } else {
        DEFAULT_DAMPING
    }
}

impl SpringAnimation {
    /// Create a new spring animation
    #[must_use]
    pub const fn new(initial: f32, target: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target,
            stiffness: DEFAULT_STIFFNESS,
            damping: DEFAULT_DAMPING,
        }
    }

    /// Set spring parameters
    ///
    /// Sanitized at this boundary like every width knob: stiffness must be
    /// finite and positive, damping finite and non-negative — garbage
    /// (NaN, infinity, wrong sign) falls back to that parameter's default
    /// (200/20) rather than poisoning the simulation. Damping `0.0` stays
    /// legal: it is the undamped oscillator.
    #[must_use]
    pub const fn params(mut self, stiffness: f32, damping: f32) -> Self {
        self.stiffness = sanitize_stiffness(stiffness);
        self.damping = sanitize_damping(damping);
        self
    }

    /// Advance the simulation by `dt` seconds using the exact closed-form
    /// solution of the damped harmonic oscillator.
    ///
    /// The per-step map — oscillate by the discriminant rate times `dt`
    /// (or its hyperbolic counterpart), then decay by `e^(-alpha*dt)`
    /// with `alpha = damping / 2` — is exact, so it is unconditionally
    /// stable for any `dt` and any sanitized parameters: no sub-stepping,
    /// no dt clamp, and no stability bound that new parameters could
    /// silently outgrow. One big step lands on the same trajectory as
    /// many small ones.
    ///
    /// Three guards remain. A non-finite or non-positive `dt` advances
    /// nothing. A step whose decay is visually zero (an `alpha*dt` of 80
    /// or more, which is also where the hyperbolic terms would overflow)
    /// snaps straight to the target. And `dt` is capped at an hour to
    /// keep every transcendental argument finite.
    ///
    /// Parameters are sanitized by [`SpringAnimation::params`] and,
    /// the fields being public, re-sanitized here.
    pub fn update(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(MAX_DT);
        let stiffness = sanitize_stiffness(self.stiffness);
        let damping = sanitize_damping(self.damping);
        let alpha = 0.5 * damping;
        if alpha * dt >= SETTLED_DECAY {
            self.value = self.target;
            self.velocity = 0.0;
            return;
        }
        // Displacement from the target: it obeys u'' = -k*u - c*u'.
        let displacement = self.value - self.target;
        let velocity = self.velocity;
        let decay = (-alpha * dt).exp();
        let disc = alpha * alpha - stiffness; // sign picks the branch
        let rate = disc.abs().sqrt(); // omega (under) or beta (over)
        let (displacement_t, velocity_t) = if rate * dt < NEAR_CRITICAL {
            // Critically damped, or arbitrarily near either branch of it
            // where dividing by the rate is ill-conditioned: the limit
            // divides by nothing, with O((rate * dt)^2) error.
            let envelope_rate = velocity + alpha * displacement;
            (
                decay * (displacement + envelope_rate * dt),
                decay * (velocity - alpha * envelope_rate * dt),
            )
        } else if disc < 0.0 {
            // Underdamped: rotate by omega * dt, then shrink by decay.
            let (sin, cos) = (rate * dt).sin_cos();
            let sin_coeff = (velocity + alpha * displacement) / rate;
            (
                decay * (displacement * cos + sin_coeff * sin),
                decay
                    * ((sin_coeff * rate - alpha * displacement) * cos
                        - (displacement * rate + alpha * sin_coeff) * sin),
            )
        } else {
            // Overdamped: the hyperbolic counterpart of the rotation.
            let arg = rate * dt;
            let (sinh, cosh) = (arg.sinh(), arg.cosh());
            let sin_coeff = (velocity + alpha * displacement) / rate;
            (
                decay * (displacement * cosh + sin_coeff * sinh),
                decay
                    * ((sin_coeff * rate - alpha * displacement) * cosh
                        + (displacement * rate - alpha * sin_coeff) * sinh),
            )
        };
        self.value = self.target + displacement_t;
        self.velocity = velocity_t;
    }

    /// Set a new target value
    pub const fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Check if the spring has approximately settled at the target
    #[must_use]
    pub fn is_settled(&self, position_threshold: f32, velocity_threshold: f32) -> bool {
        let position_error = (self.value - self.target).abs();
        let velocity_mag = self.velocity.abs();

        position_error < position_threshold && velocity_mag < velocity_threshold
    }

    /// Reset the spring to a new position with zero velocity
    pub const fn reset(&mut self, value: f32, target: f32) {
        self.value = value;
        self.target = target;
        self.velocity = 0.0;
    }
}

#[cfg(test)]
mod spring_tests {
    use super::*;

    /// Kittest feeds 0.25s steps; the exact map stays on the true
    /// trajectory instead of exploding like a step-limited integrator.
    #[test]
    fn huge_dt_does_not_explode() {
        let mut spring = SpringAnimation::new(256.0, 240.0).params(300.0, 25.0);
        for _ in 0..50 {
            spring.update(0.25);
        }
        assert!(spring.value.is_finite());
        assert!(
            (spring.value - 240.0).abs() < 1.0,
            "settled at {}",
            spring.value
        );
    }

    /// The exact map is a flow: one big step must land on the same
    /// trajectory as many small ones — the property semi-implicit Euler
    /// violates once its step exceeds the stability bound.
    #[test]
    fn one_big_step_matches_many_small_steps() {
        let mut big = SpringAnimation::new(0.0, 1.0).params(300.0, 25.0);
        big.update(0.25);
        let mut small = SpringAnimation::new(0.0, 1.0).params(300.0, 25.0);
        for _ in 0..250 {
            small.update(0.001);
        }
        assert!(
            (big.value - small.value).abs() < 1.0e-3,
            "{} vs {}",
            big.value,
            small.value
        );
        assert!(
            (big.velocity - small.velocity).abs() < 5.0e-2,
            "{} vs {}",
            big.velocity,
            small.velocity
        );
    }
}

#[cfg(test)]
mod spring_stability_tests {
    use super::*;

    /// The checkbox spring (k=2000, c=55) diverged at dt=1/30 under a
    /// single semi-implicit step — the exact map has no bound to violate.
    #[test]
    fn stiff_spring_survives_frame_spikes() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(2000.0, 55.0);
        for _ in 0..200 {
            spring.update(1.0 / 30.0);
        }
        assert!(
            (spring.value - 1.0).abs() < 0.01,
            "settled at {}",
            spring.value
        );
    }

    /// Overdamped springs have the tightest semi-implicit bound
    /// (h < 2/c): (300, 200) diverged under the old sub-step band even
    /// though both parameters are perfectly legitimate.
    #[test]
    fn overdamped_spring_settles() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300.0, 200.0);
        for _ in 0..300 {
            spring.update(1.0 / 60.0);
        }
        assert!(spring.value.is_finite());
        assert!(
            (spring.value - 1.0).abs() < 0.01,
            "settled at {}",
            spring.value
        );
    }

    /// Extreme stiffness once forced sub-steps below any fixed floor;
    /// the exact map is O(1) for any parameters.
    #[test]
    fn extreme_stiffness_settles() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300_000.0, 55.0);
        for _ in 0..300 {
            spring.update(1.0 / 60.0);
        }
        assert!(
            (spring.value - 1.0).abs() < 0.01,
            "settled at {}",
            spring.value
        );
    }

    /// A step whose decay is visually zero lands exactly on the target.
    #[test]
    fn long_gap_snaps_to_the_target() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300.0, 200.0);
        spring.update(1.0);
        assert_eq!(spring.value, 1.0);
        assert_eq!(spring.velocity, 0.0);
    }

    /// NaN dt passes through arithmetic and would poison the spring
    /// forever; negative dt must likewise advance nothing.
    #[test]
    fn nan_dt_does_not_poison() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300.0, 25.0);
        spring.update(f32::NAN);
        spring.update(-1.0);
        assert!(spring.value.is_finite());
        for _ in 0..300 {
            spring.update(1.0 / 60.0);
        }
        assert!(
            (spring.value - 1.0).abs() < 0.01,
            "settled at {}",
            spring.value
        );
    }

    /// An undamped spring never settles but must stay bounded — the exact
    /// map conserves energy instead of injecting it.
    #[test]
    fn undamped_spring_stays_bounded() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300.0, 0.0);
        for _ in 0..500 {
            spring.update(1.0 / 30.0);
        }
        assert!(spring.value.abs() < 2.0, "bounded, got {}", spring.value);
    }
}

#[cfg(test)]
mod spring_param_tests {
    use super::*;

    /// `params` sanitizes at the boundary like every width knob.
    #[test]
    fn garbage_params_fall_back_to_defaults() {
        let spring = |k: f32, c: f32| SpringAnimation::new(0.0, 1.0).params(k, c);
        assert_eq!(spring(f32::NAN, 20.0).stiffness, 200.0);
        assert_eq!(spring(f32::INFINITY, 20.0).stiffness, 200.0);
        assert_eq!(spring(0.0, 20.0).stiffness, 200.0);
        assert_eq!(spring(-300.0, 20.0).stiffness, 200.0);
        assert_eq!(spring(2000.0, f32::NAN).damping, 20.0);
        assert_eq!(spring(2000.0, f32::NEG_INFINITY).damping, 20.0);
        assert_eq!(spring(2000.0, -5.0).damping, 20.0);
        // Legal values pass through untouched — including damping 0.
        assert_eq!(spring(2000.0, 55.0).stiffness, 2000.0);
        assert_eq!(spring(2000.0, 0.0).damping, 0.0);
    }

    /// The fields are public, so `update` re-sanitizes: direct mutation
    /// cannot poison the simulation either.
    #[test]
    fn poisoned_fields_do_not_poison_the_simulation() {
        let mut spring = SpringAnimation::new(0.0, 1.0);
        spring.stiffness = f32::NAN;
        spring.damping = f32::NEG_INFINITY;
        for _ in 0..300 {
            spring.update(1.0 / 60.0);
        }
        assert!(spring.value.is_finite());
        assert!(
            (spring.value - 1.0).abs() < 0.01,
            "settled at {}",
            spring.value
        );
    }
}
