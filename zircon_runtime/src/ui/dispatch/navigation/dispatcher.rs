use std::collections::HashMap;
use std::sync::Arc;

use crate::ui::dispatch::visited_node_set::UiDispatchVisitedNodeSet;
use zircon_runtime_interface::ui::dispatch::{
    UiNavigationDispatchContext, UiNavigationDispatchEffect, UiNavigationDispatchInvocation,
    UiNavigationDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{UiNavigationEventKind, UiNavigationRoute};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

/// 注册表以 `Arc` 持有回调；每次调用临时借用当前路由，决议本身不借用该路由。
/// 高阶生命周期约束使同一回调可接收不同派发轮次的路由视图。
type NavigationHandler = Arc<
    dyn for<'route> Fn(&UiNavigationDispatchContext<'route>) -> UiNavigationDispatchEffect
        + Send
        + Sync
        + 'static,
>;

/// 按节点和导航事件保存处理器，并同步派发已构造的导航路由。
///
/// `Default` 创建空注册表；同一键下的回调按注册顺序保留，由注册表中的 `Arc` 持有。
/// 派发仅借用树与处理器，返回拥有原路由的结果；焦点状态由
/// [`crate::ui::surface::UiSurface::dispatch_navigation_event`] 在派发后提交。
#[derive(Default)]
pub struct UiNavigationDispatcher {
    handlers: HashMap<(UiNodeId, UiNavigationEventKind), Vec<NavigationHandler>>,
}

impl UiNavigationDispatcher {
    /// 为指定节点和导航事件追加处理器，供后续匹配路由调用。
    ///
    /// # 参数
    /// - `node_id`：回调所属节点的标识，作为注册键使用；此处不检查节点是否存在。
    /// - `kind`：需要响应的导航事件种类。
    /// - `handler`：借用当前节点及整条路由视图，返回继续派发、接管或请求焦点的决议。
    ///
    /// # 约束与生命周期
    /// 同一键下的回调按追加顺序执行。回调须适用于任意路由借用生命周期，并满足
    /// `Send + Sync + 'static`；捕获值不借用短期局部数据，注册表释放时也释放其 `Arc` 引用。
    /// 回调在 [`Self::dispatch`] 的当前调用线程同步执行。
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiNavigationEventKind, handler: F)
    where
        F: for<'route> Fn(&UiNavigationDispatchContext<'route>) -> UiNavigationDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.handlers
            .entry((node_id, kind))
            .or_default()
            .push(Arc::new(handler));
    }

    /// 按给定路由执行导航处理器链，收集第一个接管决议。
    ///
    /// # 调用时机与参数
    /// Surface 准备路由后调用本方法；其他调用方也须先提供候选节点序列。
    /// - `tree`：只读树视图，用于检查 `Focus` 决议中的目标节点是否存在。
    /// - `route`：转入结果的路由；有目标时按冒泡列表遍历，无目标时按根候选列表遍历。
    ///
    /// # 返回与错误
    /// 成功结果拥有原路由、零或一次接管记录及可选焦点目标。空候选或空注册表返回空决议。
    /// `Focus` 指向不存在的节点时返回 [`UiTreeError::MissingNode`]。
    ///
    /// # 约束与生命周期
    /// 候选按首次出现去重，同节点回调按注册顺序调用；`Unhandled` 继续，
    /// `Handled` 或有效 `Focus` 终止整条处理器链。此处不重建路由，也不提交焦点状态；
    /// Surface 可在派发后执行默认导航并提交焦点。回调仅在同步调用期间借用路由。
    ///
    /// # 参考
    /// [`crate::ui::surface::UiSurface::dispatch_navigation_event`] 负责后续默认导航和焦点提交。
    pub fn dispatch(
        &self,
        tree: &UiTree,
        route: UiNavigationRoute,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let route_kind = route.kind;
        let candidates: &[UiNodeId] = if route.target.is_some() {
            &route.bubbled
        } else {
            &route.root_targets
        };
        if candidates.is_empty() || self.handlers.is_empty() {
            return Ok(UiNavigationDispatchResult::new(route));
        }
        // 候选数是记录数的保守上界；首个接管决议终止整条路由，实际只记录零或一次调用。
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(candidates.len());
        let mut invocations = Vec::with_capacity(candidates.len());
        let mut handled_by = None;
        let mut focus_changed_to = None;

        'route: for node_id in candidates.iter().copied() {
            if !visited.insert(node_id) {
                continue;
            }
            let Some(handlers) = self.handlers.get(&(node_id, route_kind)) else {
                continue;
            };
            let context = UiNavigationDispatchContext {
                node_id,
                route: &route,
            };
            for handler in handlers {
                let effect = handler(&context);
                match effect {
                    UiNavigationDispatchEffect::Unhandled => continue,
                    UiNavigationDispatchEffect::Handled => {
                        invocations.push(UiNavigationDispatchInvocation {
                            node_id,
                            effect: UiNavigationDispatchEffect::Handled,
                        });
                        handled_by = Some(node_id);
                        break 'route;
                    }
                    UiNavigationDispatchEffect::Focus(target_node_id) => {
                        tree.node(target_node_id)
                            .ok_or(UiTreeError::MissingNode(target_node_id))?;
                        invocations.push(UiNavigationDispatchInvocation {
                            node_id,
                            effect: UiNavigationDispatchEffect::Focus(target_node_id),
                        });
                        handled_by = Some(node_id);
                        focus_changed_to = Some(target_node_id);
                        break 'route;
                    }
                }
            }
        }

        let mut result = UiNavigationDispatchResult::new(route);
        result.invocations = invocations;
        result.handled_by = handled_by;
        result.focus_changed_to = focus_changed_to;
        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests/dispatcher_optimization_tests.rs"]
mod optimization_tests;
