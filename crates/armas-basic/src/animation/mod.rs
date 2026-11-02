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
    ///
    /// A non-finite dt (whose NaN would make `elapsed` — and therefore
    /// every later `value()` — NaN forever) advances nothing; a negative
    /// dt likewise never rewinds the clock.
    pub fn update(&mut self, dt: f32) {
        if self.state != AnimationState::Running {
            return;
        }
        if !dt.is_finite() || dt < 0.0 {
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
/// Upper bound of the sanitized stiffness range. A stiffness of 1e6
/// oscillates at ~159 Hz — beyond any perceivable UI spring and past
/// Nyquist at 60 fps — and the bound keeps squared parameters far below
/// f32 overflow and the oscillation argument inside f64 range-reduction
/// precision.
const MAX_STIFFNESS: f32 = 1.0e6;
/// Upper bound of the sanitized damping range: at 60 fps a damping of
/// 1e6 is a hard snap, and the bound keeps `alpha` squared finite.
const MAX_DAMPING: f32 = 1.0e6;
/// `alpha * dt` at or beyond this decays to less than e^-80 of the
/// amplitude — below f32 resolution of any animated value. Snap to the
/// target instead of evaluating the map, so the spring lands exactly and
/// `is_settled` can turn off the caller's repaint loop.
const SETTLED_DECAY: f32 = 80.0;
/// Below this (rate * dt) both solution branches are arbitrarily close
/// to the critically-damped one, whose closed form divides by nothing.
const NEAR_CRITICAL: f32 = 1.0e-3;
/// Frame gaps longer than an hour are clamped: the exact map makes this
/// invisible except that it bounds the oscillation argument (rate * dt
/// stays below ~3.6e6) within f64 range-reduction precision.
const MAX_DT: f32 = 3600.0;

/// Spring parameters share the width-knob boundary policy (finite
/// positive passes, garbage falls back to the default) and are
/// additionally capped: values beyond the cap are indistinguishable from
/// it in a UI spring, and the cap keeps every intermediate of the exact
/// map finite.
const fn sanitize_stiffness(raw: f32) -> f32 {
    let raw = crate::sanitize::finite_above_or(raw, 0.0, DEFAULT_STIFFNESS);
    if raw > MAX_STIFFNESS {
        MAX_STIFFNESS
    } else {
        raw
    }
}

/// Damping is finite and non-negative (zero is the legal undamped
/// oscillator); anything else falls back to the default, and sane values
/// are capped.
const fn sanitize_damping(raw: f32) -> f32 {
    if raw.is_finite() && raw >= 0.0 {
        if raw > MAX_DAMPING {
            MAX_DAMPING
        } else {
            raw
        }
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
    /// legal: it is the undamped oscillator. Valid-but-extreme values are
    /// capped at 1e6 (beyond any perceivable UI spring).
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
    /// (or decay along its two real exponentials), then shrink by
    /// `e^(-alpha*dt)` with `alpha = damping / 2` — is exact and
    /// unconditionally stable for any `dt` and any sanitized parameters:
    /// no sub-stepping and no stability bound that new parameters could
    /// silently outgrow. One big step lands on the same trajectory as
    /// many small ones, and every decay factor is at most 1, so no
    /// intermediate grows beyond the state itself.
    ///
    /// Guards. A non-finite or non-positive `dt` advances nothing. A
    /// step whose decay is visually zero (an `alpha*dt` of 80 or more)
    /// snaps straight to the target. `dt` is capped at an hour so the
    /// oscillation argument stays within range-reduction precision. And
    /// non-finite state — the fields are public, and callers derive
    /// targets from unmeasured rects — is healed by snapping to the
    /// target. Parameters are sanitized by [`SpringAnimation::params`]
    /// and, the fields being public, re-sanitized here.
    pub fn update(&mut self, dt: f32) {
        // Heal poisoned state before anything else: a NaN target adopts
        // the current position, then value/velocity reset onto it, so
        // one garbage step can never pin is_settled at false forever.
        if !self.target.is_finite() {
            self.target = if self.value.is_finite() {
                self.value
            } else {
                0.0
            };
        }
        if !self.value.is_finite() || !self.velocity.is_finite() {
            self.value = self.target;
            self.velocity = 0.0;
            return;
        }
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
            // divides by nothing. Per-step displacement error is
            // O((rate*dt)^2); the velocity of the limit form carries an
            // additional O(rate^2*dt) local term, but in the regime this
            // branch guards (rate*dt < 1e-3 at real frame rates) the
            // induced spring-constant error rate^2 is invisible.
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
            // Overdamped: decay along the two real exponentials
            // e^(-(alpha -/+ beta)*dt). Each factor lies in [0, 1], so no
            // intermediate grows beyond the state itself; the slow
            // exponent alpha - beta is computed as k/(alpha + beta),
            // which survives the catastrophic cancellation when
            // beta ~= alpha.
            let slow = (-(stiffness / (alpha + rate)) * dt).exp();
            let fast = (-(alpha + rate) * dt).exp();
            let decay_cosh = 0.5 * (slow + fast);
            let decay_sinh = 0.5 * (slow - fast);
            let sin_coeff = (velocity + alpha * displacement) / rate;
            (
                displacement * decay_cosh + sin_coeff * decay_sinh,
                (sin_coeff * rate - alpha * displacement) * decay_cosh
                    + (displacement * rate - alpha * sin_coeff) * decay_sinh,
            )
        };
        self.value = self.target + displacement_t;
        self.velocity = velocity_t;
    }

    /// Set a new target value
    ///
    /// A non-finite target is ignored — it would poison every later
    /// step (`update` also heals one that arrived by direct field write).
    pub const fn set_target(&mut self, target: f32) {
        if target.is_finite() {
            self.target = target;
        }
    }

    /// Check if the spring has approximately settled at the target
    #[must_use]
    pub fn is_settled(&self, position_threshold: f32, velocity_threshold: f32) -> bool {
        let position_error = (self.value - self.target).abs();
        let velocity_mag = self.velocity.abs();

        position_error < position_threshold && velocity_mag < velocity_threshold
    }

    /// Reset the spring to a new position with zero velocity
    ///
    /// Non-finite values are ignored (same policy as [`Self::set_target`]).
    pub const fn reset(&mut self, value: f32, target: f32) {
        if value.is_finite() {
            self.value = value;
        }
        if target.is_finite() {
            self.target = target;
        }
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

    /// A large displacement once overflowed inside the overdamped branch
    /// (displacement * cosh hit inf before the decay factor could shrink
    /// it): a ~100k-pixel carousel offset with a heavy-damped spring and a
    /// frame hitch must settle, not explode.
    #[test]
    fn overdamped_big_displacement_does_not_overflow() {
        let mut spring = SpringAnimation::new(0.0, 110_000.0).params(300.0, 200.0);
        for step in 0..200 {
            spring.update(0.79);
            assert!(spring.value.is_finite(), "overflowed at step {step}");
        }
        assert!(
            (spring.value - 110_000.0).abs() < 1.0,
            "settled at {}",
            spring.value
        );
    }

    /// A `f32::MAX` damping once overflowed alpha*alpha to inf and wrote
    /// NaN into the value on the first frame; the 1e6 cap keeps the map
    /// finite at any dt.
    #[test]
    fn huge_damping_tiny_dt_stays_finite() {
        let mut spring = SpringAnimation::new(0.0, 1.0).params(300.0, f32::MAX);
        assert_eq!(spring.damping, 1.0e6);
        spring.update(1.0e-30);
        spring.update(1.0e-30);
        assert!(spring.value.is_finite());
        for _ in 0..60 {
            spring.update(1.0 / 60.0);
        }
        assert_eq!(spring.value, 1.0);
        assert_eq!(spring.velocity, 0.0);
    }

    /// Capped extreme stiffness (1e30 clamps to 1e6, omega = 1000 rad/s)
    /// keeps the oscillation argument inside range-reduction precision,
    /// so the flow property — one big step equals many small ones —
    /// still holds instead of phase-wrapping to garbage.
    #[test]
    fn capped_stiffness_flow_stays_precise() {
        let mut big = SpringAnimation::new(0.0, 1.0).params(1.0e30, 1.0e-6);
        assert_eq!(big.stiffness, 1.0e6);
        big.update(0.5);
        let mut small = SpringAnimation::new(0.0, 1.0).params(1.0e30, 1.0e-6);
        for _ in 0..500 {
            small.update(0.001);
        }
        assert!(
            (big.value - small.value).abs() < 1.0e-2,
            "{} vs {}",
            big.value,
            small.value
        );
        assert!(
            (big.velocity - small.velocity).abs() < 5.0,
            "{} vs {}",
            big.velocity,
            small.velocity
        );
    }

    /// The critically-damped branch (rate = 0): the envelope form must be
    /// a flow too.
    #[test]
    fn critically_damped_flow() {
        let mut big = SpringAnimation::new(0.0, 1.0).params(2500.0, 100.0);
        big.update(0.1);
        let mut small = SpringAnimation::new(0.0, 1.0).params(2500.0, 100.0);
        for _ in 0..1000 {
            small.update(1.0e-4);
        }
        assert!(
            (big.value - small.value).abs() < 1.0e-3,
            "{} vs {}",
            big.value,
            small.value
        );
        assert!(
            (big.velocity - small.velocity).abs() < 1.0e-2,
            "{} vs {}",
            big.velocity,
            small.velocity
        );
    }

    /// In the regime the near-critical branch actually guards — a rate
    /// that is genuinely small, so `rate * dt` falls under the cutoff at
    /// real frame rates — the big step takes the exact overdamped branch
    /// while the small steps take the limit form, and they must agree.
    /// (Far from critical damping the limit form integrates a slightly
    /// different spring, which is why the cutoff is on `rate * dt`, not on
    /// `dt` alone.)
    #[test]
    fn near_critical_matches_exact_at_small_rates() {
        let mut big = SpringAnimation::new(0.0, 1.0).params(1.0e-4, 0.021);
        big.update(1.0);
        let mut small = SpringAnimation::new(0.0, 1.0).params(1.0e-4, 0.021);
        for _ in 0..1000 {
            small.update(0.001);
        }
        assert!(
            (big.value - small.value).abs() < 1.0e-4,
            "{} vs {}",
            big.value,
            small.value
        );
        assert!(
            (big.velocity - small.velocity).abs() < 1.0e-4,
            "{} vs {}",
            big.velocity,
            small.velocity
        );
    }
}

#[cfg(test)]
mod spring_param_tests {
    use super::*;

    /// `params` sanitizes at the boundary like every width knob and caps
    /// valid-but-absurd values.
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
        // Valid-but-extreme values clamp to the caps, not the defaults.
        assert_eq!(spring(1.0e30, 20.0).stiffness, 1.0e6);
        assert_eq!(spring(f32::MAX, 20.0).stiffness, 1.0e6);
        assert_eq!(spring(2000.0, 1.0e30).damping, 1.0e6);
        assert_eq!(spring(1.0e6, 1.0e6).stiffness, 1.0e6);
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

#[cfg(test)]
mod spring_state_tests {
    use super::*;

    /// `set_target` ignores a non-finite target instead of poisoning every
    /// later step (carousel derives targets from unmeasured rects).
    #[test]
    fn set_target_ignores_non_finite() {
        let mut spring = SpringAnimation::new(0.0, 1.0);
        spring.set_target(f32::NAN);
        assert_eq!(spring.target, 1.0);
        spring.set_target(5.0);
        assert_eq!(spring.target, 5.0);
    }

    /// A NaN value or velocity (the fields are public) is healed onto the
    /// target instead of pinning `is_settled` — and the caller's repaint
    /// loop — open forever.
    #[test]
    fn poisoned_value_is_healed() {
        let mut spring = SpringAnimation::new(0.0, 1.0);
        spring.value = f32::NAN;
        spring.velocity = f32::INFINITY;
        spring.update(1.0 / 60.0);
        assert_eq!(spring.value, 1.0);
        assert_eq!(spring.velocity, 0.0);
        assert!(spring.is_settled(0.01, 0.01));
    }

    /// A NaN target adopts the current position, so the spring settles
    /// where it is instead of chasing NaN forever.
    #[test]
    fn poisoned_target_adopts_position() {
        let mut spring = SpringAnimation::new(0.3, 1.0);
        spring.target = f32::NAN;
        spring.update(1.0 / 60.0);
        assert_eq!(spring.target, 0.3);
        assert_eq!(spring.value, 0.3);
        assert!(spring.is_settled(0.01, 0.01));
    }

    /// `reset` ignores non-finite values like `set_target` does.
    #[test]
    fn reset_ignores_non_finite() {
        let mut spring = SpringAnimation::new(0.0, 1.0);
        spring.reset(f32::NAN, f32::INFINITY);
        assert_eq!(spring.value, 0.0);
        assert_eq!(spring.target, 1.0);
        assert_eq!(spring.velocity, 0.0);
    }
}

#[cfg(test)]
mod timed_animation_dt_tests {
    use super::*;

    /// The timed animator shares the spring's dt policy: one NaN dt must
    /// not make `elapsed` — and every later `value()` — NaN forever.
    #[test]
    fn nan_dt_does_not_poison_timed_animation() {
        let mut anim = Animation::new(0.0_f32, 1.0, 0.3);
        anim.start();
        anim.update(f32::NAN);
        anim.update(-0.5);
        anim.update(0.15);
        assert!(anim.value().is_finite());
        anim.update(0.15);
        assert!(anim.is_complete());
        assert!((anim.value() - 1.0).abs() < 1.0e-6);
    }
}
