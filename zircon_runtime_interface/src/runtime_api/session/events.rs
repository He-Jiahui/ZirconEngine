//! Runtime 事件的固定布局载体与字段填充构造器。
//! 构造器不执行能力协商或载荷解码；借用型载荷由同步事件入口消费。

use crate::buffer::ZrByteSlice;
use crate::handles::ZrRuntimeViewportHandle;

use super::super::constants::*;
use super::editor_transform::ZrRuntimeEditorTransformWriteV1;
use super::ime_composition_capability_v2::ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE;
use super::viewport::{ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// 宿主输入、窗口状态和编辑器命令共用的固定布局事件载体。
/// 字段含义由 kind 决定；payload 借用调用方内存，只能在同步事件处理期间读取或复制。
pub struct ZrRuntimeEventV1 {
    /// 事件信封版本，构造器原样保存，由事件 ABI 入口检查。
    pub abi_version: u32,
    /// 选择其余字段解释方式的事件标签；未使用字段由 [`Self::new`] 填默认值。
    pub kind: u32,
    /// 事件目标视口数值身份，不持有视口；当前动态消费另核对默认视口。
    pub viewport: ZrRuntimeViewportHandle,
    /// 调整视口时的像素尺寸，IME 光标区域事件中复用为宽高。
    pub size: ZrRuntimeViewportSizeV1,
    /// 视口尺寸、缩放等度量；仅相应构造器填入，本值不自动同步 host 窗口。
    pub metrics: ZrRuntimeViewportMetricsV1,
    /// 按 kind 表示横向位置或横向增量；带位置滚轮的增量另存于 key_code 位模式。
    pub x: f32,
    /// 按 kind 表示纵向位置或纵向增量；带位置滚轮的增量另存于 scan_code 位模式。
    pub y: f32,
    /// 按 kind 保存纵向滚量、按钮/轴值或窗口缩放系数，构造器不检查有限性。
    pub delta: f32,
    /// 按 kind 保存按钮、键动作、主题、布尔值或滚轮位置存在标记。
    pub button: u32,
    /// 按 kind 保存阶段、生命周期、IME/窗口子状态或滚轮单位。
    pub state: u32,
    /// 触点或游戏手柄的数值身份，不建立远端资源所有权。
    pub pointer_id: u64,
    /// 按 kind 保存键码、范围起点/光标前删除字节数、vendor ID 或滚轮横量位模式。
    pub key_code: u32,
    /// 按 kind 保存扫描码、范围终点/光标后删除字节数、product ID 或滚轮纵量位模式。
    pub scan_code: u32,
    /// 由事件 kind 解释的借用字节；非空区域须在同步事件处理返回前保持可读。
    pub payload: ZrByteSlice,
}

/// 各构造器返回按值事件，不派发、不验证版本/标签/数值，也不复制 payload 字节。
/// 传入的 ABI 版本和视口原样保存；带载荷事件须保持源字节地址与内容有效直至同步事件调用返回。
/// 字段名相同的参数按值填入；下方文档重点说明跨 kind 复用和与验证入口的分工。
impl ZrRuntimeEventV1 {
    /// 保存版本、kind、视口并初始化其余字段；metrics 默认缩放为 1、尺寸为零，payload 为空。
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

    /// 同时保存完整 metrics，并把其 physical_size 写入 size；不验证两种尺寸或缩放关系。
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

    /// 构造按行计量的纯纵向滚轮事件，水平滚量为零；参见 [`Self::mouse_wheel_delta`]。
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

    /// unit 写入 state，水平/纵向滚量写入 x/y，delta 同时保存纵向值；不设置指针位置标记。
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

    /// 借用调用方编码的相机 payload，不在构造时解析 JSON 或校验相机；消费格式见视口相机 DTO。
    pub const fn viewport_camera(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            payload,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1,
                viewport,
            )
        }
    }

    /// 将固定布局写入请求临时借给事件；调用方须持有 request 直至 handle_event 返回。
    pub fn editor_transform_write(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        request: &ZrRuntimeEditorTransformWriteV1,
    ) -> Self {
        Self {
            payload: request.as_payload(),
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1,
                viewport,
            )
        }
    }

    /// 带指针位置的滚轮事件把水平、垂直滚量的位模式放入键码字段，以保留旧事件布局。
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

    /// 把有符号窗口坐标转换为 f32 保存，可能舍入；消费方再转换回整数，不能据此要求全 i32 精确保值。
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

    pub const fn window_surface_recreated(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1,
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

    /// action 写入 button，键码与扫描码原样保存；key_text 只借用字节，不检查 UTF-8。
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

    /// value 为借用的预编辑 UTF-8 字节，cursor_start/end 写入键码字段；隐藏游标使用公开哨兵，构造不校验范围。
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

    /// 使用既有 IME kind 携带状态 10 和组合 V2 编码字节。
    /// 本构造器只填充字段，不验证载荷或协商结果；调用方须保持借用字节可读至同步处理返回。
    pub const fn ime_composition_v2(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            state: ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE,
            payload,
            ..Self::new(abi_version, ZR_RUNTIME_EVENT_KIND_IME_V1, viewport)
        }
    }

    /// 借用本次提交的 UTF-8 字节并设置旧版提交状态；不检查编码或当前焦点/组合。
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

    /// 把前后删除的 UTF-8 字节数保存到 key_code/scan_code，不是字符数，也不在此执行删除。
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

    /// x/y 保存区域位置，width/height 写入 size；当前消费另检查有限位置及非零宽高。
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

    /// 借用周围 UTF-8 文本，cursor/anchor 是其字节偏移；当前消费另检查范围与字符边界。
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

    /// 借用编码后的剪贴板结果；构造不检查 JSON、传输身份或目标，消费路径另行验证。
    pub const fn clipboard_result(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        payload: ZrByteSlice,
    ) -> Self {
        Self {
            payload,
            ..Self::new(
                abi_version,
                ZR_RUNTIME_EVENT_KIND_CLIPBOARD_RESULT_V1,
                viewport,
            )
        }
    }

    /// 借用编码后的无障碍动作；构造不查动作目标或解析 JSON。
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

    /// 委托带设备 ID 的构造器，vendor/product 均为零，表示未提供这两项设备信息。
    pub const fn gamepad_connection(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        gamepad_id: u64,
        state: u32,
        name: ZrByteSlice,
    ) -> Self {
        Self::gamepad_connection_with_ids(abi_version, viewport, gamepad_id, state, 0, 0, name)
    }

    /// gamepad_id 存入 pointer_id，vendor/product 存入两键码字段，name 只借用；构造不检查编码或 ID 范围。
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

    /// V2 payload borrowing lasts through synchronous dispatch. V1 constructors keep V1 kinds.
    pub const fn gamepad_connection_v2(
        abi_version: u32, viewport: ZrRuntimeViewportHandle, gamepad_id: u64,
        state: u32, vendor_id: u32, product_id: u32, payload: ZrByteSlice,
    ) -> Self {
        Self { state, pointer_id: gamepad_id, key_code: vendor_id, scan_code: product_id, payload,
            ..Self::new(abi_version, super::ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V2, viewport) }
    }

    pub const fn gamepad_button_v2(
        abi_version: u32, viewport: ZrRuntimeViewportHandle, gamepad_id: u64,
        button: u32, state: u32, value: f32, payload: ZrByteSlice,
    ) -> Self {
        Self { button, state, delta: value, pointer_id: gamepad_id, payload,
            ..Self::new(abi_version, super::ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V2, viewport) }
    }

    pub const fn gamepad_axis_v2(
        abi_version: u32, viewport: ZrRuntimeViewportHandle, gamepad_id: u64,
        axis: u32, value: f32, payload: ZrByteSlice,
    ) -> Self {
        Self { button: axis, delta: value, pointer_id: gamepad_id, payload,
            ..Self::new(abi_version, super::ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V2, viewport) }
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
