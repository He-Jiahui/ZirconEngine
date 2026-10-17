//! 指针回调注册表与单次路由分派的执行实现。
//! 每次调用创建独立累积器；回调只借用路由，结束后路由与累积效果一起移入结果。

use std::collections::HashMap;
use std::sync::Arc;

use crate::ui::dispatch::visited_node_set::UiDispatchVisitedNodeSet;
use crate::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiPointerDispatchContext, UiPointerDispatchEffect,
    UiPointerDispatchInvocation, UiPointerDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiPointerEventKind, UiPointerRoute};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};

/// 注册表持有的共享回调；闭包捕获须能独立存活并满足跨线程约束。
/// 路由借用只对当前调用有效，闭包自身的静态生命周期不会延长该借用。
type PointerHandler = Arc<
    dyn for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
        + Send
        + Sync
        + 'static,
>;

/// 单次分派的效果累积器；遍历多个命中候选时共用，结束后被结果消费。
/// 阻断某个候选与后来由另一个候选处理可以同时留下记录，并非互斥状态。
#[derive(Default)]
struct UiPointerDispatchState {
    // 只记录非 Unhandled 回应；同一节点的多个回调可以留下多条记录。
    invocations: Vec<UiPointerDispatchInvocation>,
    handled_by: Option<UiNodeId>,
    blocked_by: Option<UiNodeId>,
    passthrough: Vec<UiNodeId>,
    // 捕获与焦点字段是待提交请求；这里不直接改变表面状态。
    captured_by: Option<UiNodeId>,
    released_capture: Option<UiNodeId>,
    focus_changed_to: Option<UiNodeId>,
    focus_cleared: bool,
    // 可继续传播的失效请求在已执行的回调之间累积。
    requested_dirty: UiDirtyFlags,
    requested_damage: Vec<UiFrame>,
}

impl UiPointerDispatchState {
    /// 消费累积器和路由，保留结果构造器生成的基础诊断，再转移本次回调效果。
    fn finish(self, route: UiPointerRoute) -> UiPointerDispatchResult {
        let mut result = UiPointerDispatchResult::new(route);
        result.invocations = self.invocations;
        result.handled_by = self.handled_by;
        result.blocked_by = self.blocked_by;
        result.passthrough = self.passthrough;
        result.captured_by = self.captured_by;
        result.released_capture = self.released_capture;
        result.focus_changed_to = self.focus_changed_to;
        result.focus_cleared = self.focus_cleared;
        result.requested_dirty = self.requested_dirty;
        result.requested_damage = self.requested_damage;
        result
    }
}

/// 按节点、事件种类及可选阶段保存指针回调，并对已解析路由执行分派。
///
/// # 调用时机
/// 表面生成指针路由后使用此注册表；输入管理器也可长期持有它并按需注册回调。
/// 同一键的回调按注册顺序执行，阶段回调先于普通回调。
///
/// # 生命周期与约束
/// 注册表持有回调直到分派器释放；每次分派另建效果状态，不保留路由借用。
/// 分派只收集捕获、焦点及失效请求，表面状态由调用方随后应用。
/// 默认实例没有回调，分派时直接生成基础结果并跳过路由遍历。
///
/// # 参考
/// [`UiPointerDispatchContext`] 描述回调借用，[`UiPointerDispatchEffect`] 描述单次回应。
#[derive(Default)]
pub struct UiPointerDispatcher {
    handlers: HashMap<(UiNodeId, UiPointerEventKind), Vec<PointerHandler>>,
    phase_handlers: HashMap<(UiNodeId, UiPointerEventKind, UiDispatchPhase), Vec<PointerHandler>>,
}

impl UiPointerDispatcher {
    /// 为指定节点和事件种类追加普通回调，在目标或冒泡阶段执行。
    ///
    /// # 参数
    /// `node_id` 是路由节点标识；注册时不查询树，也不验证节点是否仍存在。
    /// `kind` 选择指针事件种类；`handler` 借用本次上下文并返回一个分派效果。
    ///
    /// # 调用时机与生命周期
    /// 在需要处理该事件前注册；同一键允许多条回调，依注册顺序尝试。
    /// 闭包由注册表持有，捕获必须满足静态生命周期及线程约束；上下文不能随闭包保存。
    /// 普通回调不进入预览阶段；指定阶段请使用 [`Self::register_phase`]。
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiPointerEventKind, handler: F)
    where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.handlers
            .entry((node_id, kind))
            .or_default()
            .push(Arc::new(handler));
    }

    /// 为指定节点、事件种类和阶段追加回调，并在该阶段先于普通回调执行。
    ///
    /// # 参数
    /// `node_id`、`kind` 与 `handler` 的所有权及有效期约束同 [`Self::register`]。
    /// `phase` 选择阶段；本分派器只遍历 PreviewTunnel、Target 和 Bubble，
    /// 其他阶段键可以被保存，但不会由本实现触发。
    ///
    /// # 调用时机与约束
    /// 回调是否到达取决于路由和先前回调的停止效果；预览还受事件种类与捕获状态限制。
    /// 同一阶段键依注册顺序执行，当前上下文只在本次回调调用期间有效。
    pub fn register_phase<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        phase: UiDispatchPhase,
        handler: F,
    ) where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.phase_handlers
            .entry((node_id, kind, phase))
            .or_default()
            .push(Arc::new(handler));
    }

    /// 分派调用方提供的路由，按树的父链构造各候选的目标及冒泡路径。
    ///
    /// # 参数
    /// `tree` 与路由须属于同一棵树，参与查询的父链须完整且无环；此处只借用树。
    /// `route` 按值交入，保留事件数据、预览路径和命中候选，成功时由结果接管。
    ///
    /// # 返回与错误
    /// 返回原路由、回调的非 Unhandled 回应及累积请求；本方法不提交表面状态。
    /// 需要查询的候选或祖先缺失时返回树错误；无回调时跳过查询，不据此验证路由。
    ///
    /// # 调用时机与约束
    /// 适用于自行提供路由的调用方；表面同次命中后的缓存复用走内部入口。
    /// 回调结束后不能继续持有上下文中的路由借用。
    ///
    /// # 参考
    /// [`crate::ui::surface::UiSurface::dispatch_pointer_event`] 负责表面路由及后续状态应用。
    pub fn dispatch(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.dispatch_route(tree, route, false)
    }

    /// 供表面在同次路由计算后立即分派，复用未捕获目标的已计算冒泡路径。
    /// `tree` 与按值交入的 `route` 必须保持同次路由的一致性；捕获及其他候选仍查询树。
    /// 返回值和树查询错误同 [`Self::dispatch`]，此入口本身不提交表面状态。
    pub(crate) fn dispatch_surface_route(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.dispatch_route(tree, route, true)
    }

    /// 先执行独立预览，再按捕获、命中堆栈或根回退选择候选。
    /// 缓存可信标志只允许表面复用当前未捕获目标的路径；其他路径仍从树解析。
    fn dispatch_route(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
        trust_cached_target_route: bool,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let mut state = UiPointerDispatchState::default();
        // 空注册表不需要验证或遍历路由，直接保留基础诊断。
        if self.handlers.is_empty() && self.phase_handlers.is_empty() {
            return Ok(state.finish(route));
        }

        let mut propagation_stopped = false;
        if route_uses_preview_tunnel(&route) {
            for node_id in route.routing_root_to_leaf().iter().copied() {
                if self.dispatch_phase_handlers(
                    &route,
                    &mut state,
                    node_id,
                    UiDispatchPhase::PreviewTunnel,
                    false,
                )? {
                    propagation_stopped = true;
                    break;
                }
            }
        }
        if propagation_stopped {
            return Ok(state.finish(route));
        }

        // 捕获独占候选；普通目标只在命中堆栈未包含它时补入，避免复制候选列表。
        // 无目标时才使用根回退列表，顺序由路由生产者决定。
        let (injected_candidate, candidate_slice): (Option<UiNodeId>, &[UiNodeId]) =
            if let Some(captured) = route.captured {
                (Some(captured), &[])
            } else if let Some(target) = route.target {
                (
                    (!route.stacked.contains(&target)).then_some(target),
                    route.stacked.as_slice(),
                )
            } else {
                (None, route.root_targets.as_slice())
            };
        let expected_visited_len = route
            .routing_root_to_leaf()
            .len()
            .saturating_add(candidate_slice.len())
            .saturating_add(injected_candidate.is_some() as usize);
        // 此集合只覆盖目标及冒泡，跨候选抑制共享祖先的重复调用；预览阶段独立执行。
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(expected_visited_len);

        'candidate: for candidate in injected_candidate
            .into_iter()
            .chain(candidate_slice.iter().copied())
        {
            // 只有表面刚计算的普通目标可复用缓存；捕获或其他堆叠候选必须解析自己的父链。
            let stop_dispatch = if trust_cached_target_route
                && route.captured.is_none()
                && route.target == Some(candidate)
            {
                self.dispatch_candidate_route(
                    &route,
                    &mut state,
                    &mut visited,
                    candidate,
                    route.bubble_route(),
                )?
            } else {
                self.dispatch_candidate_route(
                    &route,
                    &mut state,
                    &mut visited,
                    candidate,
                    tree.bubble_route(candidate)?,
                )?
            };
            if stop_dispatch {
                break 'candidate;
            }
        }

        Ok(state.finish(route))
    }

    /// 沿单个候选向根遍历，候选自身使用目标阶段，祖先使用冒泡阶段。
    /// `visited` 由整次分派共享，节点无回调也会占位；返回 true 才结束全部候选。
    /// 阻断当前候选返回 false，让外层继续尝试后续候选；父链已在进入前由调用方解析。
    fn dispatch_candidate_route(
        &self,
        route: &UiPointerRoute,
        state: &mut UiPointerDispatchState,
        visited: &mut UiDispatchVisitedNodeSet,
        candidate: UiNodeId,
        bubble_route: impl IntoIterator<Item = UiNodeId>,
    ) -> Result<bool, UiTreeError> {
        for node_id in bubble_route {
            if !visited.insert(node_id) {
                continue;
            }
            let phase = if node_id == candidate {
                UiDispatchPhase::Target
            } else {
                UiDispatchPhase::Bubble
            };
            if self.dispatch_phase_handlers(route, state, node_id, phase, true)? {
                return Ok(true);
            }
            // 普通阶段的阻断只结束这条候选父链，不能把它误作整次分派已处理。
            if state.blocked_by == Some(node_id) {
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// 查询当前节点与阶段的精确键；阶段回调先于可选的普通回调。
    /// `include_unqualified_handlers` 在预览时为 false，避免普通回调提前执行。
    /// 上下文仅在存在回调时建立一次，并借给这一节点阶段的每个回调。
    /// 返回 true 表示应停止整次分派；非预览阻断则通过状态由候选层截断父链。
    fn dispatch_phase_handlers(
        &self,
        route: &UiPointerRoute,
        state: &mut UiPointerDispatchState,
        node_id: UiNodeId,
        phase: UiDispatchPhase,
        include_unqualified_handlers: bool,
    ) -> Result<bool, UiTreeError> {
        let phase_handlers = self.phase_handlers.get(&(node_id, route.kind, phase));
        let unqualified_handlers = include_unqualified_handlers
            .then(|| self.handlers.get(&(node_id, route.kind)))
            .flatten();

        if phase_handlers.is_none() && unqualified_handlers.is_none() {
            return Ok(false);
        }

        let context = UiPointerDispatchContext {
            node_id,
            phase,
            route,
        };
        let phase_handlers = phase_handlers
            .into_iter()
            .flat_map(|handlers| handlers.iter());
        let unqualified_handlers = unqualified_handlers
            .into_iter()
            .flat_map(|handlers| handlers.iter());
        for handler in phase_handlers.chain(unqualified_handlers) {
            let effect = handler(&context);
            // 未处理回应既不进入调用记录，也不改变累积状态。
            if effect == UiPointerDispatchEffect::Unhandled {
                continue;
            }
            state.invocations.push(UiPointerDispatchInvocation {
                node_id,
                phase,
                effect,
            });
            if apply_pointer_effect(state, node_id, effect, phase) {
                return Ok(true);
            }
            if effect == UiPointerDispatchEffect::Blocked {
                return Ok(false);
            }
        }
        Ok(false)
    }
}

/// 只有未捕获的按下、释放及滚动事件沿非空根到叶路径执行预览。
/// 捕获路由直接进入候选阶段；移动等其他种类也不执行此预览。
fn route_uses_preview_tunnel(route: &UiPointerRoute) -> bool {
    route.captured.is_none()
        && matches!(
            route.kind,
            UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll
        )
        && !route.routing_root_to_leaf().is_empty()
}

/// 将一个回应归入当前状态，返回是否结束整次分派。
/// 捕获、释放和焦点请求同时标记已处理；脏域及损伤帧只累积并继续传播。
/// 阻断是否结束全部候选由阶段决定，表面状态在结果返回后另行应用。
fn apply_pointer_effect(
    state: &mut UiPointerDispatchState,
    node_id: UiNodeId,
    effect: UiPointerDispatchEffect,
    phase: UiDispatchPhase,
) -> bool {
    match effect {
        UiPointerDispatchEffect::Unhandled => false,
        UiPointerDispatchEffect::Handled => {
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::Blocked => {
            state.blocked_by = Some(node_id);
            phase == UiDispatchPhase::PreviewTunnel
        }
        UiPointerDispatchEffect::Passthrough => {
            state.passthrough.push(node_id);
            false
        }
        UiPointerDispatchEffect::CapturePointer => {
            state.captured_by = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ReleasePointerCapture => {
            state.released_capture = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::SetFocus => {
            state.focus_changed_to = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ClearFocus => {
            state.focus_cleared = true;
            state.handled_by = Some(node_id);
            true
        }
        // 逐字段合并，保留前面回调已经请求的其他失效域。
        UiPointerDispatchEffect::RequestDirty(flags) => {
            state.requested_dirty.layout |= flags.layout;
            state.requested_dirty.hit_test |= flags.hit_test;
            state.requested_dirty.render |= flags.render;
            state.requested_dirty.style |= flags.style;
            state.requested_dirty.text |= flags.text;
            state.requested_dirty.input |= flags.input;
            state.requested_dirty.visible_range |= flags.visible_range;
            false
        }
        UiPointerDispatchEffect::RequestDamage(frame) => {
            state.requested_damage.push(frame);
            false
        }
    }
}
