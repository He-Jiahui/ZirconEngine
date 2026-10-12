#[path = "../union/optional.rs"]
mod optional;
#[path = "../../../../contract/pointer/union/pane_button_damage_visible.rs"]
mod visible;

pub(super) use self::optional::union_optional_frames;
pub(super) use self::visible::union_visible_frame;
