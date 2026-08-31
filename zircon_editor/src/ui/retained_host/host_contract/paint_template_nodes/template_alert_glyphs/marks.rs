use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INFO_MARK_ASSET: &str = "zircon_editor_shell/status/alert-info.svg";
const SUCCESS_MARK_ASSET: &str = "zircon_editor_shell/status/alert-success.svg";
const WARNING_MARK_ASSET: &str = "zircon_editor_shell/status/alert-warning.svg";
const ERROR_MARK_ASSET: &str = "zircon_editor_shell/status/alert-error.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    color: [u8; 4],
    opacity: f32,
) {
    let asset = match tone {
        AlertTone::Info => INFO_MARK_ASSET,
        AlertTone::Success => SUCCESS_MARK_ASSET,
        AlertTone::Warning => WARNING_MARK_ASSET,
        AlertTone::Error => ERROR_MARK_ASSET,
    };
    push_icon_asset_pixels(commands, asset, rect, clip, order, Some(color), opacity);
}
