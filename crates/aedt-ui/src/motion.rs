//! Motion tokens: one small set of durations, used with `duration(..)` and
//! the app's entrances. Reduced motion shortens every transition to zero.

use nsh_core::{
    element::{LayoutAnimation, LayoutCurve, LayoutTransition},
    prelude::*,
};

/// A hover wash or a color change.
pub const INSTANT: f32 = 120.0;

/// Moving into place: an echo panel entering, a selection following.
pub const BASE: f32 = 240.0;

/// How long a control's or a row's hover and selection wash lasts, honoring
/// reduced motion.
pub fn row_duration() -> impl Fn() -> f32 + Copy {
    || if reduce_motion() { 0.0 } else { INSTANT }
}

/// A conversation opening: fade in only, `BASE` with ease-out.
pub fn fade() -> MotionSpec {
    MotionSpec::new(KeyframeTiming::new(BASE).easing(Easing::EaseOut))
        .values(MotionChannel::Opacity, [0.0, 1.0])
}

/// Hides or shows the side panel `open` governs: the panel fades out of the
/// room it took while the pane beside it eases into that room, and fades back
/// in as the pane gives the room up. Nothing when the user asked for less
/// motion.
pub fn set_panel(open: Signal<bool>, shown: bool) {
    if untracked(|| open.get()) == shown {
        return;
    }

    if !untracked(reduce_motion) {
        let transition = LayoutTransition::new(LayoutCurve::EaseOut, BASE);

        animate_layout(LayoutAnimation::all(transition));
    }

    open.set(shown);
}
