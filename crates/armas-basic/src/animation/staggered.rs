//! Staggered animations for choreographed entrance effects
//!
//! Provides tools for creating aceternity-style staggered animations where
//! multiple elements animate with sequential delays.

use super::{
    non_negative_dt, sanitize_duration, Animation, AnimationState, EasingFunction, Interpolate,
};

/// A non-finite delay collapses to "no delay" (the items animate
/// together) instead of poisoning every item window.
const fn sanitize_delay(raw: f32) -> f32 {
    crate::sanitize::finite_or(raw, 0.0)
}

/// Staggered animation system for multiple items
///
/// Creates a choreographed animation where items animate in sequence
/// with configurable delays between them.
///
/// # Example
///
/// ```rust
/// use armas_basic::animation::StaggeredAnimation;
///
/// // Fade in 5 items, each delayed by 0.1s, taking 0.3s each
/// let mut anim = StaggeredAnimation::new(0.0_f32, 1.0, 5, 0.1, 0.3);
/// anim.update(0.2);
/// let opacity_0 = anim.opacity(0); // mostly visible
/// let opacity_4 = anim.opacity(4); // not started yet
/// ```
#[derive(Debug, Clone)]
pub struct StaggeredAnimation<T: Interpolate> {
    /// Base delay before first item starts (seconds)
    pub base_delay: f32,
    /// Delay between each item (seconds)
    pub stagger_delay: f32,
    /// Duration of each item's animation (seconds)
    pub duration: f32,
    /// Easing function for each item
    pub easing: EasingFunction,
    /// Total elapsed time
    elapsed: f32,
    /// Number of items
    item_count: usize,
    /// Start and end values
    start: T,
    end: T,
}

impl<T: Interpolate> StaggeredAnimation<T> {
    /// Create a new staggered animation
    ///
    /// Knobs are sanitized at this boundary like every other animator: a
    /// non-finite duration falls back to the 0.3s default, a non-finite
    /// delay collapses to "no delay".
    pub const fn new(
        start: T,
        end: T,
        item_count: usize,
        stagger_delay: f32,
        duration: f32,
    ) -> Self {
        Self {
            base_delay: 0.0,
            stagger_delay: sanitize_delay(stagger_delay),
            duration: sanitize_duration(duration),
            easing: EasingFunction::EaseOut,
            elapsed: 0.0,
            item_count,
            start,
            end,
        }
    }

    /// Set the base delay before first item
    ///
    /// A non-finite delay collapses to "no delay" — the same policy as
    /// the constructor, so a caller bug lands somewhere visible instead
    /// of being silently ignored.
    #[must_use]
    pub const fn base_delay(mut self, delay: f32) -> Self {
        self.base_delay = sanitize_delay(delay);
        self
    }

    /// Set the easing function
    #[must_use]
    pub const fn easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Update the animation
    ///
    /// A non-finite or negative dt advances nothing (the shared dt
    /// policy of every timed animator). The knob fields stay as written:
    /// every read path sanitizes through [`Self::sane_knobs`].
    pub fn update(&mut self, dt: f32) {
        if !non_negative_dt(dt) {
            return;
        }
        self.elapsed += dt;
    }

    /// Reset the animation
    pub const fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Sane knob values for the read paths (the fields are public).
    const fn sane_knobs(&self) -> (f32, f32, f32) {
        (
            sanitize_delay(self.base_delay),
            sanitize_delay(self.stagger_delay),
            sanitize_duration(self.duration),
        )
    }

    /// Get the value for a specific item index
    pub fn value(&self, index: usize) -> T {
        let (base_delay, stagger_delay, duration) = self.sane_knobs();
        let item_start_time = base_delay + (index as f32 * stagger_delay);
        let item_end_time = item_start_time + duration;

        if self.elapsed < item_start_time {
            // Not started yet
            return self.start.clone();
        }

        if self.elapsed >= item_end_time {
            // Completed
            return self.end.clone();
        }

        // In progress
        let item_elapsed = self.elapsed - item_start_time;
        let t = (item_elapsed / duration).clamp(0.0, 1.0);
        let eased_t = self.easing.apply(t);

        self.start.interpolate(&self.end, eased_t)
    }

    /// Get the progress (0.0 to 1.0) for a specific item
    pub fn progress(&self, index: usize) -> f32 {
        let (base_delay, stagger_delay, duration) = self.sane_knobs();
        let item_start_time = base_delay + (index as f32 * stagger_delay);
        let item_end_time = item_start_time + duration;

        if self.elapsed < item_start_time {
            0.0
        } else if self.elapsed >= item_end_time {
            1.0
        } else {
            ((self.elapsed - item_start_time) / duration).clamp(0.0, 1.0)
        }
    }

    /// Check if all items have completed
    ///
    /// An animation with no items is trivially complete (and
    /// `item_count - 1` would underflow).
    pub fn is_complete(&self) -> bool {
        if self.item_count == 0 {
            return true;
        }
        let (base_delay, stagger_delay, duration) = self.sane_knobs();
        let last_item = self.item_count.saturating_sub(1) as f32;
        let last_item_end = base_delay + (last_item * stagger_delay) + duration;
        self.elapsed >= last_item_end
    }

    /// Get opacity for item (useful for fade-in effects)
    pub fn opacity(&self, index: usize) -> f32 {
        self.progress(index)
    }

    /// Get scale for item (useful for scale-in effects)
    pub fn scale(&self, index: usize) -> f32 {
        let t = self.progress(index);
        0.8 + 0.2 * t // Scale from 0.8 to 1.0
    }

    /// Get Y offset for item (useful for slide-in effects)
    pub fn y_offset(&self, index: usize, distance: f32) -> f32 {
        let t = self.progress(index);
        distance * (1.0 - t)
    }
}

/// Animation sequence that runs animations one after another
///
/// # Example
///
/// ```rust
/// use armas_basic::animation::{Animation, AnimationSequence, EasingFunction};
///
/// let seq = AnimationSequence::new()
///     .then(Animation::new(0.0_f32, 1.0, 0.3), 0.0)
///     .then(Animation::new(1.0_f32, 0.5, 0.2), 0.1);
/// ```
#[derive(Debug)]
pub struct AnimationSequence<T: Interpolate> {
    animations: Vec<SequenceStep<T>>,
    current_step: usize,
    elapsed: f32,
}

#[derive(Debug)]
struct SequenceStep<T: Interpolate> {
    delay: f32,
    animation: Animation<T>,
}

impl<T: Interpolate> AnimationSequence<T> {
    /// Create a new empty sequence
    #[must_use]
    pub const fn new() -> Self {
        Self {
            animations: Vec::new(),
            current_step: 0,
            elapsed: 0.0,
        }
    }

    /// Add an animation step with optional delay
    ///
    /// A non-finite delay collapses to "no delay" (a NaN delay would
    /// otherwise compare false against the elapsed time forever).
    #[must_use]
    pub fn then(mut self, animation: Animation<T>, delay: f32) -> Self {
        self.animations.push(SequenceStep {
            delay: sanitize_delay(delay),
            animation,
        });
        self
    }

    /// Update the sequence
    ///
    /// A non-finite or negative dt advances nothing — the sequence drives
    /// its steps by absolute time (writing the inner `elapsed` directly),
    /// so one garbage frame would otherwise poison the inner animation
    /// past its own guard.
    pub fn update(&mut self, dt: f32) {
        if self.current_step >= self.animations.len() {
            return;
        }
        if !non_negative_dt(dt) {
            return;
        }

        self.elapsed += dt;
        let step = &mut self.animations[self.current_step];

        // Wait for delay
        if self.elapsed < step.delay {
            return;
        }

        // Update animation
        let animation_dt = self.elapsed - step.delay;
        step.animation.elapsed = animation_dt;

        if animation_dt >= step.animation.sane_duration() {
            // Move to next step
            self.current_step += 1;
            self.elapsed = 0.0;
        }
    }

    /// Get the current value
    ///
    /// # Panics
    ///
    /// If the sequence has no steps there is no value to hold; check
    /// [`Self::is_complete`] (true for an empty sequence) before reading.
    #[must_use]
    pub fn value(&self) -> T {
        if let Some(step) = self.animations.get(self.current_step) {
            return step.animation.value();
        }
        // Finished: hold the last animation's end value.
        let last = self
            .animations
            .last()
            .expect("value() on an empty AnimationSequence");
        last.animation.end.clone()
    }

    /// Check if the entire sequence is complete
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.current_step >= self.animations.len()
    }

    /// Reset the sequence
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.elapsed = 0.0;
        for step in &mut self.animations {
            step.animation.reset();
        }
    }
}

impl<T: Interpolate> Default for AnimationSequence<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Loop mode for repeating animations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// Play once and stop
    Once,
    /// Loop infinitely
    Loop,
    /// Ping-pong (forward then backward)
    PingPong,
}

/// Animation with loop support
///
/// # Example
///
/// ```rust
/// use armas_basic::animation::{LoopingAnimation, LoopMode};
///
/// let mut anim = LoopingAnimation::new(0.0_f32, 1.0, 1.0, LoopMode::PingPong);
/// anim.update(0.5);
/// let value = anim.value();
/// ```
#[derive(Debug, Clone)]
pub struct LoopingAnimation<T: Interpolate> {
    animation: Animation<T>,
    mode: LoopMode,
    forward: bool,
}

impl<T: Interpolate> LoopingAnimation<T> {
    /// Create a new looping animation
    pub const fn new(start: T, end: T, duration: f32, mode: LoopMode) -> Self {
        Self {
            animation: Animation::new(start, end, duration),
            mode,
            forward: true,
        }
    }

    /// Set easing function
    #[must_use]
    pub const fn easing(mut self, easing: EasingFunction) -> Self {
        self.animation.easing = easing;
        self
    }

    /// Update the animation
    pub fn update(&mut self, dt: f32) {
        self.animation.update(dt);

        if self.animation.is_complete() {
            match self.mode {
                LoopMode::Once => {
                    // Stay at end
                }
                LoopMode::Loop => {
                    // Restart from beginning
                    self.animation.reset();
                    self.animation.start();
                }
                LoopMode::PingPong => {
                    // Reverse direction
                    self.forward = !self.forward;
                    std::mem::swap(&mut self.animation.start, &mut self.animation.end);
                    self.animation.reset();
                    self.animation.start();
                }
            }
        } else if self.animation.state == AnimationState::NotStarted {
            self.animation.start();
        }
    }

    /// Get current value
    pub fn value(&self) -> T {
        self.animation.value()
    }

    /// Get progress
    pub fn progress(&self) -> f32 {
        self.animation.progress()
    }

    /// Reset the animation
    pub const fn reset(&mut self) {
        self.animation.reset();
        self.forward = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staggered_animation() {
        let mut anim = StaggeredAnimation::new(0.0_f32, 1.0, 3, 0.1, 0.3);

        // At start, all items should be at 0
        assert_eq!(anim.value(0), 0.0);
        assert_eq!(anim.value(1), 0.0);
        assert_eq!(anim.value(2), 0.0);

        // After 0.15s, first item should be halfway, second just started
        anim.elapsed = 0.15;
        assert!(anim.progress(0) > 0.0);
        assert!(anim.progress(1) < 0.5);
        assert_eq!(anim.progress(2), 0.0);
    }

    #[test]
    fn test_looping_animation() {
        let mut anim = LoopingAnimation::new(0.0_f32, 1.0, 1.0, LoopMode::Loop);

        // First update starts the animation
        anim.update(0.0);
        anim.update(0.5);
        assert!(anim.progress() > 0.0 && anim.progress() < 1.0);

        anim.update(0.6); // Should loop
        assert!(anim.progress() < 0.5);
    }

    /// One NaN dt frame must not make every item's value NaN forever and
    /// pin `is_complete` at false.
    #[test]
    fn nan_dt_does_not_poison_staggered() {
        let mut anim = StaggeredAnimation::new(0.0_f32, 1.0, 5, 0.1, 0.3);
        anim.update(f32::NAN);
        anim.update(-1.0);
        assert!(anim.value(0).is_finite());
        for _ in 0..60 {
            anim.update(1.0 / 60.0);
        }
        assert!(anim.is_complete());
        assert!((anim.value(4) - 1.0).abs() < 1.0e-6);
    }

    /// Constructor knobs are sanitized at the boundary; poisoned pub
    /// fields fall back at the read sites.
    #[test]
    fn nan_knobs_do_not_poison() {
        let mut anim = StaggeredAnimation::new(0.0_f32, 1.0, 5, f32::NAN, f32::INFINITY);
        assert_eq!(anim.stagger_delay, 0.0);
        assert_eq!(anim.duration, 0.3);
        for step in 0..30 {
            anim.update(1.0 / 60.0);
            assert!(anim.progress(4 - step % 5).is_finite());
        }
        assert!(anim.is_complete());

        // Direct field poisoning: reads stay sane even before a heal
        // (elapsed 0.5s against the healed 0.3s duration = complete).
        anim.base_delay = f32::NAN;
        anim.duration = f32::NAN;
        assert!(anim.value(0).is_finite());
        assert!(anim.progress(0).is_finite());
        assert!(anim.is_complete());
    }

    /// The sequence drives inner animations by absolute time, so one NaN
    /// dt would write NaN straight past the inner guard.
    #[test]
    fn nan_dt_does_not_poison_sequence() {
        let mut seq = AnimationSequence::new().then(Animation::new(0.0_f32, 1.0, 0.3), 0.0);
        seq.update(f32::NAN);
        assert!(seq.value().is_finite());
        for _ in 0..30 {
            seq.update(1.0 / 60.0);
        }
        assert!(seq.is_complete());
        assert!((seq.value() - 1.0).abs() < 1.0e-6);
    }

    /// `item_count - 1` underflowed (panic in debug, a wrapped ~1e18
    /// window in release pinning `is_complete` false forever).
    #[test]
    fn empty_staggered_is_trivially_complete() {
        let mut anim = StaggeredAnimation::new(0.0_f32, 1.0, 0, 0.1, 0.3);
        assert!(anim.is_complete());
        anim.update(1.0 / 60.0);
        assert!(anim.is_complete());
    }

    /// An empty sequence has no value to hold; the panic is an explicit,
    /// documented contract rather than an accidental index.
    #[test]
    #[should_panic(expected = "empty AnimationSequence")]
    fn empty_sequence_value_panics_explicitly() {
        let seq: AnimationSequence<f32> = AnimationSequence::new();
        let _ = seq.value();
    }

    /// The `base_delay` builder collapses garbage like the constructor
    /// does, instead of silently keeping the previous value.
    #[test]
    fn base_delay_garbage_collapses_to_no_delay() {
        let anim = StaggeredAnimation::new(0.0_f32, 1.0, 5, 0.1, 0.3)
            .base_delay(f32::NAN)
            .base_delay(f32::INFINITY);
        assert_eq!(anim.base_delay, 0.0);
    }
}
