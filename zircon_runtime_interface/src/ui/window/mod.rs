//! 平台无关的窗口事件、输入规范化和有序泵批次契约。
//!
//! 宿主提供窗口身份与时间序号，平台事件经规范化进入共享分发器；本模块只构造数据，
//! 不拥有窗口、事件循环或分发状态。固定 ABI 载荷须先建立有效借用，转换结果拥有其文本。
//! 参见 [`UiWindowInputContext`]、[`UiWindowInputPumpBatch`] 与 [`UiRuntimeEvent`]。
#[path = "event.rs"]
mod event;
#[path = "impact.rs"]
mod impact;
#[path = "input.rs"]
mod input;
#[path = "metadata.rs"]
mod metadata;
#[path = "metrics.rs"]
mod metrics;
#[path = "pump.rs"]
mod pump;
#[path = "runtime_event.rs"]
mod runtime_event;
#[path = "runtime_event_abi.rs"]
mod runtime_event_abi;
#[path = "runtime_event_adapter.rs"]
mod runtime_event_adapter;

pub use event::{
    UiWindowAction, UiWindowActivation, UiWindowEvent, UiWindowEventKind, UiWindowRedrawReason,
};
pub use impact::UiWindowEventImpact;
pub use input::{
    UiWindowInputContext, UiWindowPlatformInputEvent, UiWindowPlatformInputEventKind,
    UiWindowTouchPhase,
};
pub use metadata::UiWindowEventMetadata;
pub use metrics::{UiWindowMetrics, UiWindowPixelPosition, UiWindowPixelSize};
pub use pump::{UiWindowInputPumpBatch, UiWindowInputPumpEvent};
pub use runtime_event_adapter::{
    runtime_event_to_window_input_pump_event, runtime_events_to_window_input_pump_batch,
    UiRuntimeEventAdapterContext, UiRuntimeEventAdapterError, UiRuntimeEventAdapterResult,
};

pub use runtime_event::UiRuntimeEvent;
pub use runtime_event_abi::{
    runtime_abi_event_to_window_input_pump_event, runtime_abi_events_to_window_input_pump_batch,
};
