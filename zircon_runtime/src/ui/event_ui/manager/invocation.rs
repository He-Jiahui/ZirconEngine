use super::UiEventManager;
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventBinding};
use zircon_runtime_interface::ui::event_ui::{
    UiControlRequest, UiControlResponse, UiInvocationContext, UiInvocationError,
    UiInvocationResult, UiInvocationSource, UiNodePath, UiRouteId,
};

impl UiEventManager {
    /// 供可信宿主按 route_id 调用；空参数沿用注册绑定的默认参数，而不是显式清空参数。
    /// 此入口不进行节点动作的远程可调用性检查；远程节点动作应通过 call_action。
    ///
    /// # 参数与返回
    /// `route_id` 定位已登记记录；参数向量移入本次调用，返回拥有的成功值或定位/处理器错误。
    /// 处理器返回后广播调用结果；缺失路由或缺失执行处理器的早退不广播。
    pub fn invoke_route(
        &self,
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
    ) -> UiInvocationResult {
        self.invoke_route_internal(route_id, arguments, UiInvocationSource::Route)
    }

    /// 按传入绑定的原生路由键定位路由，并消费本次绑定中的参数。
    ///
    /// # 参数与返回
    /// `binding` 由本次调用拥有；查找成功后参数向量转交调用上下文。
    /// 找不到键时返回带原绑定的失败；成功定位后返回同一路由执行入口的结果。
    ///
    /// # 约束
    /// 传入参数为空时沿用注册绑定的默认参数；此入口不查询节点的远程动作权限。
    pub fn invoke_binding(&self, mut binding: UiEventBinding) -> UiInvocationResult {
        let key = binding.native_binding();
        let Some(route_id) = self.routes_by_binding.get(&key).copied() else {
            return UiInvocationResult::failure(
                None,
                Some(binding),
                UiInvocationError::UnknownBinding(key),
            );
        };
        let arguments = take_binding_arguments(&mut binding);
        self.invoke_route_internal(route_id, arguments, UiInvocationSource::Binding)
    }

    /// 先验证反射节点动作的远程权限和路由，再进入同一处理器；只有已发布的可调用动作能走此路径。
    ///
    /// # 参数与返回
    /// `node_path` 和 `action_id` 定位快照中已发布的动作；参数向量移入后续调用。
    /// 节点、动作、权限或路由描述缺失时返回失败，均不会执行处理器。
    /// 空参数仍沿用注册绑定的默认值；通过前置检查不保证处理器本身成功。
    pub fn call_action(
        &self,
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<UiBindingValue>,
    ) -> UiInvocationResult {
        let Some(node) = self.query_node(&node_path) else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::UnknownNode(node_path.0),
            );
        };
        let Some(action) = node.actions.get(&action_id) else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::UnknownAction {
                    node_path: node_path.0,
                    action_id,
                },
            );
        };
        if !action.callable_from_remote {
            return UiInvocationResult::failure(
                action.route_id,
                None,
                UiInvocationError::ActionNotCallable {
                    node_path: node.node_path.0,
                    action_id: action.action_id.clone(),
                },
            );
        }
        let Some(route_id) = action.route_id else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::ActionMissingRoute {
                    node_path: node.node_path.0,
                    action_id: action.action_id.clone(),
                },
            );
        };
        self.invoke_route_internal(
            route_id,
            arguments,
            UiInvocationSource::Action {
                node_path,
                action_id: action.action_id.clone(),
            },
        )
    }

    /// 消费一个控制请求，返回对应的查询、调用、确认或订阅编号应答。
    ///
    /// 查询和属性设置作用于管理器保存的快照；处理器调用沿用各入口的检查。
    /// 注册为宿主动作的路由须由宿主请求层接管；本方法不会找到该宿主的分派器。
    /// 通知接收端应通过 [`Self::subscribe`] 交付并持有；这里的订阅应答只含编号。
    pub fn handle_request(&mut self, request: UiControlRequest) -> UiControlResponse {
        match request {
            UiControlRequest::InvokeBinding { binding } => {
                UiControlResponse::Invocation(self.invoke_binding(binding))
            }
            UiControlRequest::InvokeRoute {
                route_id,
                arguments,
            } => UiControlResponse::Invocation(self.invoke_route(route_id, arguments)),
            UiControlRequest::QueryTree { tree_id } => {
                UiControlResponse::Tree(self.query_tree(&tree_id))
            }
            UiControlRequest::QueryNode { node_path } => {
                UiControlResponse::Node(self.query_node(&node_path))
            }
            UiControlRequest::QueryProperty {
                node_path,
                property_name,
            } => UiControlResponse::Property(self.query_property(&node_path, &property_name)),
            UiControlRequest::SetProperty {
                node_path,
                property_name,
                value,
            } => match self.set_property(node_path, property_name, value) {
                Ok(()) => UiControlResponse::Ack,
                Err(error) => {
                    UiControlResponse::Invocation(UiInvocationResult::failure(None, None, error))
                }
            },
            UiControlRequest::CallAction {
                node_path,
                action_id,
                arguments,
            } => UiControlResponse::Invocation(self.call_action(node_path, action_id, arguments)),
            UiControlRequest::SubscribeDiffs => {
                // The Receiver from subscribe() must be delivered to the caller so they can
                // consume notifications. The current UiControlResponse::Subscription variant
                // only carries the subscription ID; the Receiver is discarded here which means
                // diff/invocation notifications have no live recipient.
                // Tracked: CR-UI-COMP-0002 — add UiControlResponse::SubscriptionWithReceiver or
                // change the API to return (UiSubscriptionId, Receiver<UiNotification>) directly.
                let (subscription_id, _receiver) = self.subscribe();
                UiControlResponse::Subscription(subscription_id)
            }
            UiControlRequest::Unsubscribe { subscription_id } => {
                self.unsubscribe(subscription_id);
                UiControlResponse::Ack
            }
        }
    }

    // 统一选择默认参数和调用来源；只有实际处理器返回后的路径才形成广播结果。
    fn invoke_route_internal(
        &self,
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
        source: UiInvocationSource,
    ) -> UiInvocationResult {
        let Some(route) = self.routes_by_id.get(&route_id) else {
            return UiInvocationResult::failure(
                Some(route_id),
                None,
                UiInvocationError::UnknownRoute(route_id),
            );
        };
        let effective_arguments = if arguments.is_empty() {
            route
                .binding
                .action
                .as_ref()
                .map(|call| call.arguments.clone())
                .unwrap_or_default()
        } else {
            arguments
        };
        let context = UiInvocationContext {
            route_id: route.route_id,
            binding: route.binding.clone(),
            arguments: effective_arguments,
            source,
        };
        let Some(handler) = &route.handler else {
            return UiInvocationResult::failure(
                Some(route.route_id),
                Some(route.binding.clone()),
                UiInvocationError::HandlerFailed(
                    "route has no execution handler registered".to_string(),
                ),
            );
        };
        let result = match handler(context) {
            Ok(value) => UiInvocationResult::success(route.route_id, route.binding.clone(), value),
            Err(error) => UiInvocationResult::failure(
                Some(route.route_id),
                Some(route.binding.clone()),
                error,
            ),
        };
        self.broadcast(
            zircon_runtime_interface::ui::event_ui::UiNotification::Invocation(result.clone()),
        );
        result
    }
}

// 转交参数向量的原分配并清空原槽；无动作时返回空向量，由调用入口决定默认参数。
fn take_binding_arguments(binding: &mut UiEventBinding) -> Vec<UiBindingValue> {
    binding
        .action
        .as_mut()
        .map(|call| std::mem::take(&mut call.arguments))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "tests/owned_binding_arguments.rs"]
mod owned_binding_arguments_tests;
