use winit::event::Ime;
use zircon_runtime_interface::ui::window::{
    UiWindowInputContext, UiWindowInputPumpEvent, UiWindowPlatformInputEvent,
};

use super::window::input_event;

// 组合会话交给文本输入 owner；保留平台给出的 UTF-8 字节范围，避免按字符序号重解释光标。
// Enabled 是宿主能力状态；Disabled 必须作为 Cancel 路由，以结束表面上尚未提交的组合。
pub(super) fn translate_ime_event(
    context: UiWindowInputContext,
    event: &Ime,
) -> Option<UiWindowInputPumpEvent> {
    match event {
        Ime::Preedit(text, cursor_range) => Some(input_event(
            UiWindowPlatformInputEvent::ime_with_cursor_range(
                context,
                zircon_runtime_interface::ui::dispatch::UiImeInputEventKind::Preedit,
                text.clone(),
                cursor_range.map(|(start, end)| {
                    zircon_runtime_interface::ui::dispatch::UiTextByteRange::new(
                        clamp_byte_index(start),
                        clamp_byte_index(end),
                    )
                }),
            ),
        )),
        Ime::Commit(text) => Some(input_event(UiWindowPlatformInputEvent::ime(
            context,
            zircon_runtime_interface::ui::dispatch::UiImeInputEventKind::Commit,
            text.clone(),
        ))),
        Ime::Disabled => Some(input_event(UiWindowPlatformInputEvent::ime(
            context,
            zircon_runtime_interface::ui::dispatch::UiImeInputEventKind::Cancel,
            "",
        ))),
        Ime::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } => Some(input_event(
            UiWindowPlatformInputEvent::ime_delete_surrounding(
                context,
                clamp_byte_index(*before_bytes),
                clamp_byte_index(*after_bytes),
            ),
        )),
        Ime::Enabled => None,
    }
}

fn clamp_byte_index(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}
