use winit::window::Window;
use zircon_runtime_interface::ui::dispatch::UiClipboardTransferFailure;

#[cfg(windows)]
#[path = "platform/windows.rs"]
mod windows;

#[cfg(windows)]
pub(super) fn read_text(window: Option<&dyn Window>) -> Result<String, UiClipboardTransferFailure> {
    windows::read_text(window)
}

#[cfg(windows)]
pub(super) fn write_text(
    window: Option<&dyn Window>,
    text: &str,
) -> Result<(), UiClipboardTransferFailure> {
    windows::write_text(window, text)
}

#[cfg(not(windows))]
pub(super) fn read_text(
    _window: Option<&dyn Window>,
) -> Result<String, UiClipboardTransferFailure> {
    Err(UiClipboardTransferFailure::Unsupported)
}

#[cfg(not(windows))]
pub(super) fn write_text(
    _window: Option<&dyn Window>,
    _text: &str,
) -> Result<(), UiClipboardTransferFailure> {
    Err(UiClipboardTransferFailure::Unsupported)
}
