mod diagnostics;
mod reasons;
mod requests;
mod transaction;

use super::HostInvalidationMask;
pub(in crate::ui::retained_host::app) use transaction::{
    HostInvalidationScope, HostInvalidationTransaction,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct HostInvalidationRoot {
    pending_recompute: HostInvalidationTransaction,
    total_requests: u64,
    layout_requests: u64,
    presentation_requests: u64,
    render_requests: u64,
    paint_only_requests: u64,
    hit_test_requests: u64,
    window_metrics_requests: u64,
    slow_path_rebuilds: u64,
    render_rebuilds: u64,
}

#[cfg(test)]
mod tests;
