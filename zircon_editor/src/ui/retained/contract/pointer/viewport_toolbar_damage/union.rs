#[path = "../../../../contract/pointer/union/viewport_toolbar_damage_optional.rs"]
mod optional;
#[path = "../../../../contract/pointer/union/viewport_toolbar_damage_visible.rs"]
mod visible;

pub(super) use self::optional::union_optional_frames;
pub(super) use self::visible::{union_visible_frame, visible_frame};
