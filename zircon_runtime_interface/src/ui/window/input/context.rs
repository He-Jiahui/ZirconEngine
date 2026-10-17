use serde::{Deserialize, Serialize};

use crate::ui::{
    dispatch::{
        UiDeviceId, UiInputEventMetadata, UiInputModifiers, UiPointerId, UiPointerSource,
        UiSurfaceId, UiUserId,
    },
    layout::UiPoint,
};

use super::super::{UiWindowEventMetadata, UiWindowMetrics};

/// 身份、时间序号和缓存由宿主维护；构造和链式设置只保存值，
/// 不验证窗口归属、不换算坐标，也不自动推进事件序号。
/// Platform event adapters use this context to attach stable window/user/device
/// identity before handing normalized events to the shared UI dispatcher.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowInputContext {
    /// 规范化时移入共享输入事件的路由元数据；触摸分支会改写指针身份和来源。
    pub metadata: UiInputEventMetadata,
    /// Most recently applied surface metrics, when the platform event source
    /// needs to preserve DPI while translating a physical-size-only resize.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_metrics: Option<UiWindowMetrics>,
    /// Last known cursor position in logical pixels.  Platform adapters set this
    /// from the most recent PointerMoved event so that events without an explicit
    /// position (e.g. MouseWheel) can route to the correct hit-test target rather
    /// than always hitting the origin.
    // TODO: [CR-ZCA-R038-0001] 确认缓存与命中坐标的单位约定：旧说明为逻辑像素，
    // winit 的移动位置当前仅从物理像素转为浮点；尚缺生产缓存赋值与 DPI 不为 1 的贯通证据。
    // 下一步追踪宿主缓存写入、滚轮消费与命中测试，补充含缩放的端到端夹具再决定换算责任。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_cursor_position: Option<UiPoint>,
}

// 可选浮点缓存按位比较：相同 NaN 位模式可相等，正零与负零不相等。
impl PartialEq for UiWindowInputContext {
    fn eq(&self, other: &Self) -> bool {
        self.metadata == other.metadata
            && window_metrics_equal(self.window_metrics, other.window_metrics)
            && eq_point(self.last_cursor_position, other.last_cursor_position)
    }
}

impl Eq for UiWindowInputContext {}

impl UiWindowInputContext {
    /// 复制窗口时间、序号和合成标记，并克隆窗口身份；即使身份为空也保留为 `Some`。
    /// 用户、设备等其余字段使用输入元数据默认值，两个平台缓存初始为 `None`。
    pub fn from_window_metadata(metadata: &UiWindowEventMetadata) -> Self {
        let mut input = UiInputEventMetadata::new(metadata.timestamp, metadata.sequence);
        input.window_id = Some(metadata.window_id.clone());
        input.synthetic = metadata.synthetic;
        Self {
            metadata: input,
            window_metrics: None,
            last_cursor_position: None,
        }
    }

    /// 设置用户身份并返回拥有的新上下文。
    pub fn with_user_id(mut self, user_id: UiUserId) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    /// 设置设备身份并返回上下文，不推断平台设备。
    pub fn with_device_id(mut self, device_id: UiDeviceId) -> Self {
        self.metadata.device_id = Some(device_id);
        self
    }

    /// 设置目标 surface 身份，不验证其与窗口的关联。
    pub fn with_surface_id(mut self, surface_id: UiSurfaceId) -> Self {
        self.metadata.surface_id = Some(surface_id);
        self
    }

    /// 设置指针身份；触摸规范化可按触摸事件再次覆盖。
    pub fn with_pointer_id(mut self, pointer_id: UiPointerId) -> Self {
        self.metadata.pointer_id = Some(pointer_id);
        self
    }

    /// 设置指针来源；触摸规范化固定使用触摸来源。
    pub fn with_pointer_source(mut self, pointer_source: UiPointerSource) -> Self {
        self.metadata.pointer_source = pointer_source;
        self
    }

    /// 保存来源提供的修饰键快照，不维护按键状态。
    pub fn with_modifiers(mut self, modifiers: UiInputModifiers) -> Self {
        self.metadata.modifiers = modifiers;
        self
    }

    /// 缓存当前窗口度量，供平台适配器使用；规范化不会把此缓存传入分发 DTO。
    pub fn with_window_metrics(mut self, metrics: UiWindowMetrics) -> Self {
        self.window_metrics = Some(metrics);
        self
    }

    /// 缓存来源提供的光标位置，供缺少位置的后续平台事件使用；此方法不换算坐标。
    pub fn with_last_cursor_position(mut self, position: UiPoint) -> Self {
        self.last_cursor_position = Some(position);
        self
    }
}

// 用浮点位模式比较窗口度量，避免 NaN 破坏本类型声明的 Eq 等价关系。
fn window_metrics_equal(left: Option<UiWindowMetrics>, right: Option<UiWindowMetrics>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.logical_size.width.to_bits() == right.logical_size.width.to_bits()
                && left.logical_size.height.to_bits() == right.logical_size.height.to_bits()
                && left.physical_size == right.physical_size
                && left.scale_factor.to_bits() == right.scale_factor.to_bits()
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}

fn eq_point(left: Option<UiPoint>, right: Option<UiPoint>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(l), Some(r)) => l.x.to_bits() == r.x.to_bits() && l.y.to_bits() == r.y.to_bits(),
        _ => false,
    }
}
