#[path = "builtin.rs"]
mod builtin;
#[path = "live.rs"]
mod live;
#[path = "native.rs"]
mod native;
#[path = "native_load_state.rs"]
mod native_load_state;
#[path = "project_snapshot.rs"]
mod project_snapshot;

pub(in crate::ui::host) use self::project_snapshot::ProjectPluginStatusSnapshot;
