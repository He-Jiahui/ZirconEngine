mod execution;
mod execution_resources;
mod frame;
mod materialization;
mod post_process;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    frame::record(store, stats);
    post_process::record(store, stats);
}
