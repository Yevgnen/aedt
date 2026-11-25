//! Motion tokens: one small set of durations, used with `duration(..)` and
//! the app's entrances. Reduced motion shortens every transition to zero.

use nsh_core::prelude::*;

/// A hover wash or a color change.
pub(crate) const INSTANT: f32 = 120.0;

/// A control's press and focus feedback.
pub(crate) const FAST: f32 = 180.0;

/// Moving into place: an echo panel entering, a selection following.
pub(crate) const BASE: f32 = 240.0;

/// How long a control's transition lasts, honoring reduced motion.
pub(crate) fn control_duration() -> impl Fn() -> f32 + Copy {
    || if reduce_motion() { 0.0 } else { FAST }
}

/// How long a row's hover and selection wash lasts, honoring reduced motion.
pub(crate) fn row_duration() -> impl Fn() -> f32 + Copy {
    || if reduce_motion() { 0.0 } else { INSTANT }
}

/// An entrance: fade in while rising four points, `BASE` with ease-out.
pub(crate) fn fade_rise() -> MotionSpec {
    MotionSpec::new(KeyframeTiming::new(BASE).easing(Easing::EaseOut))
        .values(MotionChannel::Opacity, [0.0, 1.0])
        .values(MotionChannel::TranslateY, [4.0, 0.0])
}

/// A conversation opening: fade in only, `BASE` with ease-out.
pub(crate) fn fade() -> MotionSpec {
    MotionSpec::new(KeyframeTiming::new(BASE).easing(Easing::EaseOut))
        .values(MotionChannel::Opacity, [0.0, 1.0])
}
