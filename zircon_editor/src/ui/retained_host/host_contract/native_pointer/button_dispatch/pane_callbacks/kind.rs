use super::super::super::{
    NativePointerButtonState, HOST_POINTER_DOWN, HOST_POINTER_UP, VIEWPORT_POINTER_DOWN,
    VIEWPORT_POINTER_UP,
};

pub(super) fn viewport_pointer_kind(state: NativePointerButtonState) -> i32 {
    match state {
        NativePointerButtonState::Pressed => VIEWPORT_POINTER_DOWN,
        NativePointerButtonState::Released => VIEWPORT_POINTER_UP,
    }
}

pub(super) fn host_pointer_kind(state: NativePointerButtonState) -> i32 {
    match state {
        NativePointerButtonState::Pressed => HOST_POINTER_DOWN,
        NativePointerButtonState::Released => HOST_POINTER_UP,
    }
}
