use crate::buffer::ZrByteSlice;
use crate::handles::ZrRuntimeViewportHandle;

use super::constants::*;
use super::viewport::{ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeEventV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub metrics: ZrRuntimeViewportMetricsV1,
    pub x: f32,
    pub y: f32,
    pub delta: f32,
    pub button: u32,
    pub state: u32,
    pub pointer_id: u64,
    pub key_code: u32,
    pub scan_code: u32,
    pub payload: ZrByteSlice,
}

impl ZrRuntimeEventV1 {
    pub const fn new(abi_version: u32, kind: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            abi_version,
            kind,
            viewport,
            size: ZrRuntimeViewportSizeV1::new(0, 0),
            metrics: ZrRuntimeViewportMetricsV1::new(
                ZrRuntimeViewportSizeV1::new(0, 0),
                1.0,
                ZrRuntimeViewportSizeV1::new(0, 0),
            ),
            x: 0.0,
            y: 0.0,
            delta: 0.0,
            button: 0,
            state: 0,
            pointer_id: 0,
            key_code: 0,
            scan_code: 0,
            payload: ZrByteSlice::empty(),
        }
    }

    pub const fn viewport_resized(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            size,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
                viewport,
            )
        }
    }

    pub const fn viewport_metrics(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        metrics: ZrRuntimeViewportMetricsV1,
    ) -> Self {
        Self {
            size: metrics.physical_size,
            metrics,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
                viewport,
            )
        }
    }

    pub const fn pointer_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
                viewport,
            )
        }
    }

    pub const fn mouse_button(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        button: u32,
        state: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            button,
            state,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, viewport)
        }
    }

    pub const fn mouse_wheel(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        delta: f32,
    ) -> Self {
        Self::mouse_wheel_delta(
            abi_version,
            viewport,
            ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1,
            0.0,
            delta,
        )
    }

    pub const fn mouse_wheel_delta(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        unit: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            delta: y,
            state: unit,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, viewport)
        }
    }

    pub fn mouse_wheel_delta_at(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        unit: u32,
        point_x: f32,
        point_y: f32,
        delta_x: f32,
        delta_y: f32,
    ) -> Self {
        Self {
            x: point_x,
            y: point_y,
            delta: delta_y,
            button: ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1,
            state: unit,
            key_code: delta_x.to_bits(),
            scan_code: delta_y.to_bits(),
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, viewport)
        }
    }

    pub const fn cursor_entered(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self::new(
            abi_version,
            ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
            viewport,
        )
    }

    pub const fn cursor_left(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, viewport)
    }

    pub const fn file_hovered(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        path: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
            payload: path,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn file_dropped(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        path: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_DROPPED_V1,
            payload: path,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn file_drag_cancelled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
                viewport,
            )
        }
    }

    pub const fn window_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            state: ZR_RUNTIME_WINDOW_STATUS_MOVED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_occluded(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        occluded: bool,
    ) -> Self {
        Self {
            button: if occluded {
                ZR_RUNTIME_WINDOW_BOOL_TRUE_V1
            } else {
                ZR_RUNTIME_WINDOW_BOOL_FALSE_V1
            },
            state: ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_theme_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        theme: u32,
    ) -> Self {
        Self {
            button: theme,
            state: ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_scale_factor_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        scale_factor: f32,
    ) -> Self {
        Self {
            delta: scale_factor,
            state: ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_backend_scale_factor_changed(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        scale_factor: f32,
    ) -> Self {
        Self {
            delta: scale_factor,
            state: ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_close_requested(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn window_destroyed(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                viewport,
            )
        }
    }

    pub const fn mouse_motion(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        delta_x: f32,
        delta_y: f32,
    ) -> Self {
        Self {
            x: delta_x,
            y: delta_y,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1, viewport)
        }
    }

    pub const fn lifecycle(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        state: u32,
    ) -> Self {
        Self {
            state,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, viewport)
        }
    }

    pub const fn touch(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        pointer_id: u64,
        phase: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            x,
            y,
            state: phase,
            pointer_id,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_TOUCH_V1, viewport)
        }
    }

    pub const fn keyboard(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        action: u32,
        key_code: u32,
        scan_code: u32,
        key_text: ZrByteSlice,
    ) -> Self {
        Self {
            button: action,
            key_code,
            scan_code,
            payload: key_text,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, viewport)
        }
    }

    pub const fn ime_enabled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_ENABLED_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_disabled(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_DISABLED_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_preedit(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
        cursor_start: u32,
        cursor_end: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_PREEDIT_V1,
            payload: value,
            key_code: cursor_start,
            scan_code: cursor_end,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_commit(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_COMMIT_V1,
            payload: value,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_delete_surrounding(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        before_bytes: u32,
        after_bytes: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
            key_code: before_bytes,
            scan_code: after_bytes,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_request_enable(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_request_disable(abi_version: u32, viewport: ZrRuntimeViewportHandle) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_cursor_area(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            x,
            y,
            size: ZrRuntimeViewportSizeV1::new(width, height),
            state: ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn ime_surrounding_text(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        value: ZrByteSlice,
        cursor: u32,
        anchor: u32,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1,
            payload: value,
            key_code: cursor,
            scan_code: anchor,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    pub const fn accessibility_action(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            payload,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_connection(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        state: u32,
        name: ZrByteSlice,
    ) -> Self {
        Self::gamepad_connection_with_ids(abi_version, viewport, gamepad_id, state, 0, 0, name)
    }

    pub const fn gamepad_connection_with_ids(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        state: u32,
        vendor_id: u32,
        product_id: u32,
        name: ZrByteSlice,
    ) -> Self {
        Self {
            state,
            pointer_id: gamepad_id,
            key_code: vendor_id,
            scan_code: product_id,
            payload: name,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_button(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        button: u32,
        state: u32,
        value: f32,
    ) -> Self {
        Self {
            button,
            state,
            delta: value,
            pointer_id: gamepad_id,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
                viewport,
            )
        }
    }

    pub const fn gamepad_axis(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        axis: u32,
        value: f32,
    ) -> Self {
        Self {
            button: axis,
            delta: value,
            pointer_id: gamepad_id,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, viewport)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTranslatedEventV1 {
    pub abi_version: u32,
    pub event: ZrRuntimeEventV1,
    pub host_reason: ZrByteSlice,
}

impl ZrRuntimeTranslatedEventV1 {
    pub const fn new(abi_version: u32, event: ZrRuntimeEventV1, host_reason: ZrByteSlice) -> Self {
        Self {
            abi_version,
            event,
            host_reason,
        }
    }

    pub const fn viewport_metrics(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        metrics: ZrRuntimeViewportMetricsV1,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::viewport_metrics(abi_version, viewport, metrics),
            ZrByteSlice::from_static(b"viewport_metrics"),
        )
    }

    pub const fn touch_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        pointer_id: u64,
        x: f32,
        y: f32,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::touch(
                abi_version,
                viewport,
                pointer_id,
                ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
                x,
                y,
            ),
            ZrByteSlice::from_static(b"touch_moved"),
        )
    }

    pub const fn keyboard_text(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        key_text: ZrByteSlice,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::keyboard(
                abi_version,
                viewport,
                ZR_RUNTIME_KEY_ACTION_TEXT_V1,
                0,
                0,
                key_text,
            ),
            ZrByteSlice::from_static(b"keyboard_text"),
        )
    }
}
