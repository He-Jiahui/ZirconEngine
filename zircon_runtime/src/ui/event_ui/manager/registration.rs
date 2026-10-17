use std::sync::Arc;

use serde_json::Value;

use super::route_entry::RouteEntry;
use super::route_handler::RouteHandler;
use super::UiEventManager;
use zircon_runtime_interface::ui::binding::UiEventBinding;
use zircon_runtime_interface::ui::event_ui::{UiInvocationContext, UiInvocationError, UiRouteId};

impl UiEventManager {
    /// 注册可直接执行的处理器；重复 native binding 会令按绑定查询指向新路由，旧 route_id 仍可直接调用。
    /// 需要稳定去重的宿主应先查询 route_id_for_binding；仅发布宿主动作时使用 register_binding_route。
    ///
    /// # 参数与返回
    /// `binding` 由注册过程接管并保存其描述；`handler` 由共享回调持有，并在调用入口同步执行。
    /// 返回新分配的路由编号，此处不调用处理器。
    ///
    /// # 约束与生命周期
    /// 回调须满足线程共享约束且不借用短期数据；路由记录存续时会持有回调及其捕获。
    pub fn register_route<F>(&mut self, binding: UiEventBinding, handler: F) -> UiRouteId
    where
        F: Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static,
    {
        self.register_route_entry(binding, Some(Arc::new(handler)))
    }

    /// Registers a route whose typed binding is executed by an owning host dispatcher.
    ///
    /// The runtime manager intentionally stores no handler for this route. Callers that do not
    /// own a host dispatcher must use `register_route` instead.
    ///
    /// 创建仅登记绑定的路由，返回新编号；绑定随管理器中的路由记录保存。
    /// 通过管理器直接执行该编号会返回缺少处理器的错误，宿主须自行接管动作分派。
    /// 参见 [`Self::register_route`] 和 [`Self::route_binding`]。
    pub fn register_binding_route(&mut self, binding: UiEventBinding) -> UiRouteId {
        self.register_route_entry(binding, None)
    }

    /// 按路由编号返回绑定的独立副本，编号未登记时返回 `None`。
    ///
    /// 修改返回值不会更新注册表；此查询不执行绑定动作。
    pub fn route_binding(&self, route_id: UiRouteId) -> Option<UiEventBinding> {
        self.routes_by_id
            .get(&route_id)
            .map(|entry| entry.binding.clone())
    }

    /// 按绑定生成的原生文本键查询当前路由编号，缺失时返回 `None`。
    ///
    /// `binding` 仅在本次查找中借用；重注册同键时返回最近登记的编号。
    pub fn route_id_for_binding(&self, binding: &UiEventBinding) -> Option<UiRouteId> {
        self.routes_by_binding
            .get(&binding.native_binding())
            .copied()
    }

    // 绑定索引只保留当前映射，编号索引仍保留旧记录，因而重注册不使历史编号失效。
    fn register_route_entry(
        &mut self,
        binding: UiEventBinding,
        handler: Option<RouteHandler>,
    ) -> UiRouteId {
        self.next_route_id += 1;
        let route_id = UiRouteId::new(self.next_route_id);
        let entry = RouteEntry {
            route_id,
            binding: binding.clone(),
            handler,
        };
        self.routes_by_binding
            .insert(binding.native_binding(), route_id);
        self.routes_by_id.insert(route_id, entry);
        route_id
    }
}
