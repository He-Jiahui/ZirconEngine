mod counters;
mod overlay;
mod planned_present;
mod summary;

pub(in crate::ui::retained_host::host_contract) use self::counters::record_chrome_command_stream_counters;
pub(in crate::ui::retained_host::host_contract) use self::overlay::damage_with_debug_overlay;
pub(in crate::ui::retained_host::host_contract) use self::planned_present::plan_present_for_diagnostics;
pub(in crate::ui::retained_host::host_contract) use self::summary::{
    frame_summary, presentation_summary,
};
