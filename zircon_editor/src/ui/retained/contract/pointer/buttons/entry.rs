#[path = "entry/body_routes.rs"]
mod body_routes;
#[path = "entry/input.rs"]
mod input;
#[path = "entry/primary_overlays.rs"]
mod primary_overlays;
#[path = "entry/release_capture.rs"]
mod release_capture;
#[path = "entry/sequence.rs"]
mod sequence;

pub(in crate::ui::retained_host::host_contract) use self::sequence::dispatch_native_pointer_button;
