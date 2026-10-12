#[path = "schema/frame.rs"]
mod frame;
#[path = "schema/geometry.rs"]
mod geometry;
#[path = "schema/hit.rs"]
mod hit;
#[path = "schema/named.rs"]
mod named;
#[path = "../schema/native_hierarchy.rs"]
mod native_hierarchy;
#[path = "schema/rounded.rs"]
mod rounded;
#[path = "schema/tab.rs"]
mod tab;
#[path = "schema/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract) use frame::{
    UiProfileFrame, UiProfileLayout, UiProfilePoint, UiProfileSize,
};
pub(in crate::ui::retained_host::host_contract) use geometry::{
    UiProfileGeometry, UiProfileLogicalSize,
};
pub(in crate::ui::retained_host::host_contract) use hit::UiProfileHitSample;
pub(in crate::ui::retained_host::host_contract) use named::UiProfileNamedFrame;
pub(in crate::ui::retained_host::host_contract) use native_hierarchy::{
    UiProfileNativeHierarchy, UiProfileNativeHierarchyRename, UiProfileNativeHierarchyRow,
    UiProfileNativePopupFocus,
};
pub(in crate::ui::retained_host::host_contract) use rounded::UiProfileRoundedShape;
pub(in crate::ui::retained_host::host_contract) use tab::UiProfileTabFrame;
pub(in crate::ui::retained_host::host_contract) use text::UiProfileTextRun;
