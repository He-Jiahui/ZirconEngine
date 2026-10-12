#[path = "../../capability_summary/capability_summary.rs"]
mod capability_summary;
#[path = "../../capability_summary/device_diagnostics.rs"]
mod device_diagnostics;
#[path = "../../../capability_summary/flagship_baseline_supported.rs"]
mod flagship_baseline_supported;

pub(in crate::graphics::runtime::render_framework) use capability_summary::capability_summary;
pub(in crate::graphics::runtime::render_framework) use device_diagnostics::render_device_diagnostics;
