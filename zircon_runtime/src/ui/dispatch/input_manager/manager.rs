#[path = "manager/toast_timer_queue.rs"]
mod toast_timer_queue;

use std::time::Duration;

use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchPhase, UiDispatchReply, UiImeInputEvent, UiImeInputEventKind,
        UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiInputSequence, UiInputTimestamp, UiPointerId, UiPointerInputEvent, UiPointerSource,
        UiSubmenuHoverTimerInputEvent, UiToastTimerInputEvent, UiTooltipTimerInputEvent,
        UiTooltipTimerInputEventKind, UiTypeaheadTimerInputEvent,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind},
    tree::UiTreeError,
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{input, UiSurface},
};

use super::{
    bound_text_model_updates::UiTextModelUpdateState,
    clipboard_host_requests::UiClipboardHostRequestQueue,
    ime_host_requests::{
        append_ime_host_requests_for_input_method_requests, append_ime_host_requests_for_result,
    },
    number_model_updates::UiNumberModelUpdateState,
    outcome::UiInputDispatchOutcome,
    pointer_table::UiActivePointerTable,
    text_document_session::UiTextDocumentSession,
    text_focus_lifecycle::finish_pending_text_focus_loss,
    timers::UiInputTimerState,
};
use crate::core::framework::input::ImeHostRequest;
use toast_timer_queue::toast_timer_from_queue_value;

/// 为宿主持有的输入流维护分发器、跨事件状态与待取请求。
/// # 使用与生命周期
/// 复用实例以延续指针、文本及计时状态；派发时独占借用管理器和传入表面。
/// # 参数与返回
/// 派发入口消费事件并返回结果；查询接口借用状态，取出接口转移队列内容的所有权。
/// # 错误与约束
/// 单事件与批次错误不回滚已发生的状态变化；切换表面不表示重置所有状态。
/// # 参考
/// 见同模块的指针表、计时器、文本会话与结果聚合实现。
pub struct UiInputManager {
    // 回调注册与跨事件服务分开持有，管理器负责串联它们的执行时机。
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
    diagnostics_mode: UiInputDiagnosticsMode,
    pointers: UiActivePointerTable,
    timers: UiInputTimerState,
    // 请求保留到宿主取出，单次结果的返回不会自动清空这些队列。
    ime_host_requests: Vec<ImeHostRequest>,
    clipboard_host_requests: UiClipboardHostRequestQueue,
    // 会话及模型更新各自保留状态，派发前按传入表面协调文本所有者。
    pub(super) text_documents: UiTextDocumentSession,
    pub(super) text_model_updates: UiTextModelUpdateState,
    pub(super) number_model_updates: UiNumberModelUpdateState,
}

impl Default for UiInputManager {
    fn default() -> Self {
        Self {
            pointer: UiPointerDispatcher::default(),
            navigation: UiNavigationDispatcher::default(),
            diagnostics_mode: UiInputDiagnosticsMode::Full,
            pointers: UiActivePointerTable::default(),
            timers: UiInputTimerState::default(),
            ime_host_requests: Vec::new(),
            clipboard_host_requests: UiClipboardHostRequestQueue::default(),
            text_documents: UiTextDocumentSession::default(),
            text_model_updates: UiTextModelUpdateState::default(),
            number_model_updates: UiNumberModelUpdateState::default(),
        }
    }
}

impl UiInputManager {
    /// 创建摘要诊断实例；保留输入行为所需状态，但不采集完整路由跟踪。
    pub fn summary() -> Self {
        Self {
            diagnostics_mode: UiInputDiagnosticsMode::Summary,
            ..Self::default()
        }
    }

    /// 读取随后派发所用的诊断粒度，不改变活动状态。
    pub fn diagnostics_mode(&self) -> UiInputDiagnosticsMode {
        self.diagnostics_mode
    }

    /// 设置随后派发的诊断粒度；已经生成的结果不会随之更新。
    pub fn set_diagnostics_mode(&mut self, diagnostics_mode: UiInputDiagnosticsMode) {
        self.diagnostics_mode = diagnostics_mode;
    }

    /// 借用指针回调注册表；引用的有效期受当前管理器借用限制。
    pub fn pointer_dispatcher(&self) -> &UiPointerDispatcher {
        &self.pointer
    }

    /// 可变借用指针注册表供回调注册和移除；借用期间不能再次派发。
    pub fn pointer_dispatcher_mut(&mut self) -> &mut UiPointerDispatcher {
        &mut self.pointer
    }

    /// 借用导航回调注册表；引用的有效期受当前管理器借用限制。
    pub fn navigation_dispatcher(&self) -> &UiNavigationDispatcher {
        &self.navigation
    }

    /// 可变借用导航注册表供回调注册和移除；借用期间不能再次派发。
    pub fn navigation_dispatcher_mut(&mut self) -> &mut UiNavigationDispatcher {
        &mut self.navigation
    }

    /// 借用跨事件指针状态，供宿主检查按压、悬停及捕获结果。
    pub fn active_pointers(&self) -> &UiActivePointerTable {
        &self.pointers
    }

    /// 可变借用指针状态；调用方需协调手动修改与表面捕获状态。
    pub fn active_pointers_mut(&mut self) -> &mut UiActivePointerTable {
        &mut self.pointers
    }

    /// 借用当前计时状态；后续输入或时钟推进可更新其中候选。
    pub fn timers(&self) -> &UiInputTimerState {
        &self.timers
    }

    /// 按微秒时间戳返回最早可见计时工作的剩余时长；双击分类不触发帧唤醒。
    pub fn next_frame_visible_delay(&self, now: UiInputTimestamp) -> Option<Duration> {
        self.timers.next_frame_visible_delay(now)
    }

    /// 查询提示框进入动画的归一化进度；没有活动进入动画时返回空值。
    pub fn tooltip_intro_progress(&self, now: UiInputTimestamp) -> Option<f32> {
        self.timers.tooltip_intro_progress(now)
    }

    /// Arms a tooltip whose owner and text were resolved by a host-specific presentation layer.
    ///
    /// The input manager remains the timing and transient-state authority even when the host uses
    /// richer metadata than the generic surface tooltip contract can express.
    /// 外部解析的所有者与文本进入同一计时状态；起点为微秒时间戳，延迟单位为毫秒。
    pub fn arm_tooltip_candidate(
        &mut self,
        surface: &mut UiSurface,
        started_at: UiInputTimestamp,
        owner: UiNodeId,
        tooltip_id: impl Into<String>,
        delay_ms: u64,
    ) {
        let tooltip_id = tooltip_id.into();
        self.dismiss_tooltip(surface);
        surface.input.arm_tooltip(tooltip_id.clone(), Some(owner));
        self.timers
            .arm_tooltip_expiration(owner, tooltip_id, started_at, delay_ms);
    }

    /// Clears every pending or visible tooltip owned by this input manager.
    /// 同时清除候选到期、进入动画及表面提示框状态，供宿主主动撤销提示。
    pub fn dismiss_tooltip(&mut self, surface: &mut UiSurface) {
        self.timers.clear_tooltip_expirations();
        self.timers.clear_tooltip_intro();
        surface.input.dismiss_transient_ui(
            zircon_runtime_interface::ui::dispatch::UiTransientDismissalTarget::Tooltip,
        );
    }

    /// 取出并清空待处理输入法请求；返回队列由宿主负责执行。
    pub fn drain_ime_host_requests(&mut self) -> Vec<ImeHostRequest> {
        std::mem::take(&mut self.ime_host_requests)
    }

    /// 把待处理剪贴板请求追加到宿主容器并清空内部队列，保留容器已有内容。
    pub(crate) fn drain_clipboard_host_requests_into(
        &mut self,
        output: &mut Vec<zircon_runtime_interface::ui::dispatch::UiClipboardRequest>,
    ) {
        self.clipboard_host_requests.drain_into(output);
    }

    /// 消费一条输入并更新传入表面，返回派发结果或树错误。
    /// 所有者同步、触摸分类及提示框清理在底层派发前发生，失败不会撤销这些变化。
    pub fn dispatch_input_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        self.dispatch_input_event_with_query(surface, event, None)
    }

    /// 内部查询可替换指针命中空间；原始事件仍保留物理位置，省略时使用默认查询。
    pub(crate) fn dispatch_input_event_with_query(
        &mut self,
        surface: &mut UiSurface,
        mut event: UiInputEvent,
        pointer_query: Option<UiHitTestQuery>,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        self.synchronize_text_document_owners(surface);
        let active_pointer_event = self.active_pointer_event_for_input(&event);
        apply_primary_touch_mouse_semantics(&mut event, active_pointer_event);
        self.clear_tooltip_for_activity(surface, &event);
        let pointer_release = self.prepare_double_click_pointer_release(surface, &mut event);
        let timestamp = input_event_timestamp(&event);
        let diagnostics_mode = self.diagnostics_mode;
        let mut result = input::dispatch_input_event(
            surface,
            &self.pointer,
            &self.navigation,
            event,
            pointer_query,
            Some(&mut self.text_documents),
            diagnostics_mode,
        )?;
        self.arm_double_click_from_pointer_release(pointer_release);
        self.update_active_pointer_table(surface, &result, active_pointer_event);
        self.arm_timers_from_component_events(surface, timestamp, &result);
        self.record_text_service_lifecycle_from_result(surface, &mut result);
        Ok(result)
    }

    /// 窗口事件直接走表面窗口入口；普通输入才走本管理器的完整指针与计时处理。
    pub(crate) fn cancel_input(
        &mut self,
        surface: &mut UiSurface,
        metadata: UiInputEventMetadata,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        use zircon_runtime_interface::ui::{
            dispatch::{UiDispatchHostRequest, UiDispatchHostRequestKind},
            window::{UiWindowEvent, UiWindowEventMetadata},
        };

        let high_precision_owner = surface.input.high_precision_owner;
        let pointer_lock_owner = surface.input.pointer_lock_owner;
        let pointer_lock_policy = surface.input.pointer_lock_policy;
        surface.clear_focus();
        self.pointers.clear();
        self.timers.cancel_interaction_timers();
        let window_metadata = UiWindowEventMetadata {
            timestamp: metadata.timestamp,
            sequence: metadata.sequence,
            synthetic: true,
            ..UiWindowEventMetadata::default()
        };
        let mut result = match self.dispatch_window_input_pump_event(
            surface,
            UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(window_metadata, false)),
        ) {
            Ok(result) => result,
            Err(error) => {
                // Keep undelivered host-release ownership across a fallible hover clear.
                // Capture, presses and drags stay retired; the Session blocks new input.
                surface.input.high_precision_owner = high_precision_owner;
                surface.input.pointer_lock_owner = pointer_lock_owner;
                surface.input.pointer_lock_policy = pointer_lock_policy;
                return Err(error);
            }
        };
        surface.input.high_precision_owner = None;
        surface.input.pointer_lock_owner = None;
        surface.input.pointer_lock_policy = None;
        if let Some(target) = high_precision_owner {
            if !result.host_requests.iter().any(|row| matches!(
                &row.request,
                UiDispatchHostRequestKind::HighPrecisionPointer { target: owner, enabled: false }
                    if *owner == target
            )) {
                result.host_requests.push(UiDispatchHostRequest {
                    effect_index: result.applied_effects.len(),
                    request: UiDispatchHostRequestKind::HighPrecisionPointer { target, enabled: false },
                    reason: "runtime input ownership ended".to_string(),
                });
            }
        }
        if let Some(policy) = pointer_lock_owner.and(pointer_lock_policy) {
            if !result.host_requests.iter().any(|row| matches!(
                &row.request,
                UiDispatchHostRequestKind::PointerUnlock { policy: existing } if *existing == policy
            )) {
                result.host_requests.push(UiDispatchHostRequest {
                    effect_index: result.applied_effects.len(),
                    request: UiDispatchHostRequestKind::PointerUnlock { policy },
                    reason: "runtime input ownership ended".to_string(),
                });
            }
        }
        Ok(result)
    }

    pub fn dispatch_window_input_pump_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        match event {
            UiWindowInputPumpEvent::Input(input) => self.dispatch_input_event(surface, input),
            UiWindowInputPumpEvent::Window(window) => {
                self.synchronize_text_document_owners(surface);
                let mut result = input::dispatch_window_event(
                    surface,
                    &self.pointer,
                    &self.navigation,
                    window,
                    self.diagnostics_mode,
                )?;
                self.record_text_service_lifecycle_from_result(surface, &mut result);
                Ok(result)
            }
        }
    }

    /// 按顺序消费批次，成功时返回拥有逐项结果和克隆请求的汇总。
    /// 首个树错误终止批次并舍弃局部结果列表；此前事件造成的状态变化仍保留。
    pub fn dispatch_window_input_pump_batch(
        &mut self,
        surface: &mut UiSurface,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError> {
        let mut results = Vec::with_capacity(batch.events.len());
        for event in batch.events {
            results.push(self.dispatch_window_input_pump_event(surface, event)?);
        }
        Ok(UiInputDispatchOutcome::from_results(surface, results))
    }

    /// 由宿主传入微秒时间戳推进时钟，依次取出键入检索、子菜单、提示框和通知到期项。
    /// 到期项生成序号为零的合成输入；最后收取焦点生命周期事件与输入法请求。
    pub fn tick(
        &mut self,
        surface: &mut UiSurface,
        now: UiInputTimestamp,
    ) -> Result<Vec<UiInputDispatchResult>, UiTreeError> {
        self.synchronize_text_document_owners(surface);
        self.timers.record_tick(now);
        self.timers.expire_double_click_candidate(now);
        self.timers.expire_tooltip_intro(now);
        let mut results = Vec::new();
        for target in self.timers.drain_expired_typeahead(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TypeaheadTimer(UiTypeaheadTimerInputEvent { metadata, target }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            results.push(result);
        }
        for (target, option_id) in self.timers.drain_expired_submenu_hover(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent {
                    metadata,
                    target,
                    option_id,
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            results.push(result);
        }
        for (target, tooltip_id) in self.timers.drain_expired_tooltips(now) {
            let intro_tooltip_id = tooltip_id.clone();
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
                    metadata,
                    kind: UiTooltipTimerInputEventKind::Elapsed,
                    tooltip_id,
                    owner: Some(target),
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            if surface.input.tooltip.as_ref().is_some_and(|tooltip| {
                tooltip.visible
                    && tooltip.owner == Some(target)
                    && tooltip.tooltip_id == intro_tooltip_id
            }) {
                self.timers.arm_tooltip_intro(now);
            }
            results.push(result);
        }
        for (target, toast_id) in self.timers.drain_expired_toasts(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::ToastTimer(UiToastTimerInputEvent {
                    metadata,
                    target,
                    toast_id,
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            results.push(result);
        }
        let lifecycle = surface.input.take_deferred_focus_input_lifecycle();
        append_ime_host_requests_for_input_method_requests(
            lifecycle.input_method_requests,
            &mut self.ime_host_requests,
        );
        if !lifecycle.component_events.is_empty() {
            let result = focus_input_method_lifecycle_result(now, lifecycle.component_events);
            self.arm_timers_from_component_events(surface, now, &result);
            results.push(result);
        }
        Ok(results)
    }

    fn record_text_service_lifecycle_from_result(
        &mut self,
        surface: &mut UiSurface,
        result: &mut UiInputDispatchResult,
    ) {
        finish_pending_text_focus_loss(
            &mut self.text_documents,
            &mut self.text_model_updates,
            surface,
        );
        append_ime_host_requests_for_result(result, &mut self.ime_host_requests);
        self.clipboard_host_requests.record_result(surface, result);
    }

    fn arm_timers_from_component_events(
        &mut self,
        surface: &mut UiSurface,
        timestamp: UiInputTimestamp,
        result: &UiInputDispatchResult,
    ) {
        // 只有已投递给组件的报告才能驱动后续计时候选。
        for report in &result.component_events {
            if !report.delivered {
                continue;
            }
            if matches!(report.event, UiComponentEvent::KeyboardText { .. }) {
                if let Some(timeout_ms) =
                    surface.typeahead_timeout_ms_for_component_node(report.target)
                {
                    self.timers
                        .arm_typeahead_expiration(report.target, timestamp, timeout_ms);
                }
            }
            self.arm_submenu_hover_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
            self.arm_tooltip_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
            self.arm_toast_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
        }
    }

    fn arm_submenu_hover_timer_from_component_event(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        let UiComponentEvent::ValueChanged { property, value } = event else {
            return;
        };
        if property != "hovered_option_id" {
            return;
        }

        let Some(delay_ms) = surface.submenu_hover_delay_ms_for_component_node(target) else {
            self.timers.clear_submenu_hover_expiration(target);
            return;
        };
        let option_id = match value {
            UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => value,
            _ => {
                self.timers.clear_submenu_hover_expiration(target);
                return;
            }
        };
        self.timers
            .arm_submenu_hover_expiration(target, option_id.as_str(), timestamp, delay_ms);
    }

    fn arm_tooltip_timer_from_component_event(
        &mut self,
        surface: &mut UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        match event {
            UiComponentEvent::Hover { hovered: true } => {
                let Some((tooltip_id, delay_ms)) = surface.tooltip_timer_for_component_node(target)
                else {
                    self.timers.clear_tooltip_expiration(target);
                    clear_tooltip_candidate_for_owner(surface, target);
                    return;
                };
                surface.input.arm_tooltip(tooltip_id.clone(), Some(target));
                self.timers
                    .arm_tooltip_expiration(target, tooltip_id, timestamp, delay_ms);
            }
            UiComponentEvent::Hover { hovered: false } => {
                self.timers.clear_tooltip_expiration(target);
                clear_tooltip_candidate_for_owner(surface, target);
            }
            _ => {}
        }
    }

    fn arm_toast_timer_from_component_event(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        match event {
            UiComponentEvent::ValueChanged { property, value }
                if matches!(property.as_str(), "toast_queue" | "queue") =>
            {
                if let Some((toast_id, timeout_ms)) = toast_timer_from_queue_value(value) {
                    self.timers
                        .arm_toast_expiration_ref(target, toast_id, timestamp, timeout_ms);
                    return;
                }
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::ValueChanged { property, .. }
                if matches!(
                    property.as_str(),
                    "current_toast_id"
                        | "auto_hide_duration_ms"
                        | "autoHideDuration"
                        | "message"
                        | "text"
                        | "open"
                        | "popup_open"
                ) =>
            {
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::OpenPopup => {
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::ClosePopup => {
                self.timers.clear_toast_expiration(target);
            }
            _ => {}
        }
    }

    fn arm_toast_timer_from_surface_state(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
    ) {
        let Some((toast_id, timeout_ms)) = surface.toast_timer_for_component_node_ref(target)
        else {
            self.timers.clear_toast_expiration(target);
            return;
        };
        self.timers
            .arm_toast_expiration_ref(target, toast_id, timestamp, timeout_ms);
    }

    fn clear_tooltip_for_activity(&mut self, surface: &mut UiSurface, event: &UiInputEvent) {
        if !input_event_cancels_tooltip(event) {
            return;
        }
        self.dismiss_tooltip(surface);
    }

    fn prepare_double_click_pointer_release(
        &self,
        surface: &UiSurface,
        event: &mut UiInputEvent,
    ) -> Option<UiDoubleClickPointerRelease> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        let click_target = pointer_release_click_target(surface, pointer)?;
        let click_count = self.timers.double_click_count_for_release(
            click_target,
            pointer.metadata.pointer_id,
            pointer.metadata.pointer_source,
            pointer.event.button,
            pointer.metadata.timestamp,
        );
        pointer.event.click_count = pointer.event.click_count.max(click_count);
        Some(UiDoubleClickPointerRelease {
            target: click_target,
            pointer_id: pointer.metadata.pointer_id,
            pointer_source: pointer.metadata.pointer_source,
            button: pointer.event.button,
            timestamp: pointer.metadata.timestamp,
            click_count: pointer.event.click_count,
        })
    }

    fn arm_double_click_from_pointer_release(
        &mut self,
        pointer_release: Option<UiDoubleClickPointerRelease>,
    ) {
        let Some(pointer_release) = pointer_release else {
            return;
        };
        self.timers.arm_double_click_candidate(
            pointer_release.target,
            pointer_release.pointer_id,
            pointer_release.pointer_source,
            pointer_release.button,
            pointer_release.click_count,
            pointer_release.timestamp,
        );
    }

    fn active_pointer_event_for_input(
        &self,
        event: &UiInputEvent,
    ) -> Option<UiActivePointerInputEvent> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        let pointer_id = pointer.metadata.pointer_id.unwrap_or_default();
        Some(UiActivePointerInputEvent {
            pointer_id,
            source: pointer.metadata.pointer_source,
            kind: pointer.event.kind,
            point: pointer.event.point,
            button: pointer.event.button,
            is_primary: self.active_pointer_is_primary(pointer_id, pointer.metadata.pointer_source),
        })
    }

    fn update_active_pointer_table(
        &mut self,
        surface: &UiSurface,
        result: &UiInputDispatchResult,
        active_pointer_event: Option<UiActivePointerInputEvent>,
    ) {
        let Some(active_pointer_event) = active_pointer_event else {
            return;
        };
        let pointer_id = active_pointer_event.pointer_id;
        if matches!(active_pointer_event.kind, UiPointerEventKind::Cancel) {
            self.pointers.remove(pointer_id);
            return;
        }
        // 悬停取物理命中路径；捕获路由不应把物理悬停路径替换为捕获祖先链。

        self.pointers.upsert(
            pointer_id,
            active_pointer_event.source,
            active_pointer_event.is_primary,
        );
        self.pointers
            .record_point(pointer_id, active_pointer_event.point);
        let routing = result.pointer_routing.as_ref();
        if let Some(routing) = routing {
            self.pointers
                .set_hovered_path_iter(pointer_id, routing.physical_bubble_route());
        } else {
            self.pointers.set_hovered_path(pointer_id, []);
        }
        match active_pointer_event.kind {
            UiPointerEventKind::Down => {
                self.pointers.press_button(
                    pointer_id,
                    active_pointer_event.button,
                    routing.and_then(|routing| routing.route_target),
                );
            }
            UiPointerEventKind::Up => {
                self.pointers
                    .release_button(pointer_id, active_pointer_event.button);
            }
            UiPointerEventKind::Move | UiPointerEventKind::Scroll | UiPointerEventKind::Cancel => {}
        }
        let capture_target = surface
            .input
            .pointer_capture_owner(pointer_id)
            .or_else(|| routing.and_then(|routing| routing.capture_target));
        self.pointers.set_capture_target(pointer_id, capture_target);
    }

    fn active_pointer_is_primary(&self, pointer_id: UiPointerId, source: UiPointerSource) -> bool {
        if let Some(entry) = self.pointers.entry(pointer_id) {
            return entry.is_primary;
        }
        !source.is_touch_like() || !self.pointers.has_primary_for_source(source)
    }
}

fn focus_input_method_lifecycle_result(
    now: UiInputTimestamp,
    component_events: Vec<zircon_runtime_interface::ui::dispatch::UiComponentEventReport>,
) -> UiInputDispatchResult {
    let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
    metadata.synthetic = true;
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Cancel,
            text: String::new(),
            cursor_range: None,
            preedit_clauses: Vec::new(),
            delete_surrounding: None,
        }),
        UiDispatchReply::handled().in_phase(UiDispatchPhase::DefaultAction),
    );
    result.diagnostics.routed = true;
    result.diagnostics.handled_phase = Some("focus.input_method_lifecycle".to_string());
    result.component_events = component_events;
    result
}

/// 派发前复制的指针事实，供成功派发后更新活动表时使用。
#[derive(Clone, Copy, Debug, PartialEq)]
struct UiActivePointerInputEvent {
    pointer_id: UiPointerId,
    source: UiPointerSource,
    kind: UiPointerEventKind,
    point: UiPoint,
    button: Option<UiPointerButton>,
    is_primary: bool,
}

/// 松开派发前解析的双击候选，成功后才写入计时状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiDoubleClickPointerRelease {
    target: UiNodeId,
    pointer_id: Option<UiPointerId>,
    pointer_source: UiPointerSource,
    button: Option<UiPointerButton>,
    timestamp: UiInputTimestamp,
    click_count: u8,
}

fn apply_primary_touch_mouse_semantics(
    event: &mut UiInputEvent,
    active_pointer_event: Option<UiActivePointerInputEvent>,
) {
    let Some(active_pointer_event) = active_pointer_event else {
        return;
    };
    if active_pointer_event.is_primary || !active_pointer_event.source.is_touch_like() {
        return;
    }
    let UiInputEvent::Pointer(pointer) = event else {
        return;
    };
    pointer.event.button = None;
}

fn input_event_cancels_tooltip(event: &UiInputEvent) -> bool {
    matches!(
        event,
        UiInputEvent::Pointer(_)
            | UiInputEvent::Keyboard(_)
            | UiInputEvent::Text(_)
            | UiInputEvent::Ime(_)
            | UiInputEvent::Clipboard(_)
            | UiInputEvent::Navigation(_)
            | UiInputEvent::Analog(_)
            | UiInputEvent::MouseMotion(_)
            | UiInputEvent::DragDrop(_)
            | UiInputEvent::Accessibility(_)
    )
}

fn pointer_release_click_target(
    surface: &UiSurface,
    pointer: &UiPointerInputEvent,
) -> Option<UiNodeId> {
    if !matches!(pointer.event.kind, UiPointerEventKind::Up)
        || pointer.event.button != Some(UiPointerButton::Primary)
    {
        return None;
    }
    let pressed = surface.focus.pressed?;
    let hit = surface.hit_test(pointer.event.point);
    hit.stacked.contains(&pressed).then_some(pressed)
}

fn clear_tooltip_candidate_for_owner(surface: &mut UiSurface, target: UiNodeId) {
    let Some(tooltip_id) = surface
        .input
        .tooltip
        .as_ref()
        .filter(|tooltip| tooltip.owner == Some(target))
        .map(|tooltip| tooltip.tooltip_id.clone())
    else {
        return;
    };
    surface.input.clear_tooltip(tooltip_id.as_str());
}

#[cfg(test)]
#[path = "manager/tests/cases.rs"]
mod tests;

fn input_event_timestamp(event: &UiInputEvent) -> UiInputTimestamp {
    match event {
        UiInputEvent::Pointer(pointer) => pointer.metadata.timestamp,
        UiInputEvent::Keyboard(keyboard) => keyboard.metadata.timestamp,
        UiInputEvent::Text(text) => text.metadata.timestamp,
        UiInputEvent::Ime(ime) => ime.metadata.timestamp,
        UiInputEvent::Clipboard(clipboard) => clipboard.metadata.timestamp,
        UiInputEvent::Navigation(navigation) => navigation.metadata.timestamp,
        UiInputEvent::Analog(analog) => analog.metadata.timestamp,
        UiInputEvent::MouseMotion(motion) => motion.metadata.timestamp,
        UiInputEvent::DragDrop(drag_drop) => drag_drop.metadata.timestamp,
        UiInputEvent::Popup(popup) => popup.metadata.timestamp,
        UiInputEvent::TooltipTimer(tooltip) => tooltip.metadata.timestamp,
        UiInputEvent::TypeaheadTimer(typeahead) => typeahead.metadata.timestamp,
        UiInputEvent::SubmenuHoverTimer(submenu_hover) => submenu_hover.metadata.timestamp,
        UiInputEvent::ToastTimer(toast) => toast.metadata.timestamp,
        UiInputEvent::Accessibility(accessibility) => accessibility.metadata.timestamp,
    }
}
