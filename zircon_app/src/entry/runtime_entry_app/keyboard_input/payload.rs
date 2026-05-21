use winit::event::KeyEvent;
use zircon_runtime_interface::ZrByteSlice;

pub(super) fn keyboard_text_payload(event: &KeyEvent) -> ZrByteSlice {
    event
        .text
        .as_ref()
        .map(|text| ZrByteSlice {
            data: text.as_bytes().as_ptr(),
            len: text.len(),
        })
        .unwrap_or_else(ZrByteSlice::empty)
}
