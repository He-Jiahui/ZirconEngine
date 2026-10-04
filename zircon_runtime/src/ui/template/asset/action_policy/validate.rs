use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyDiagnosticSeverity,
    UiActionPolicyReport, UiActionSideEffectClass, UiAssetDocument,
};

// TODO: [CR-UI-TEMPLATE-CONTRACT-0002] 确认宿主策略报告是否必须阻断被拒动作的运行时包；目前 compile_package_artifact 仍返回包含该诊断的产物，缺少消费端准入约定；后续核对打包与宿主执行入口。
/// 为打包配置和编辑器预览汇总动作副作用诊断；调用方负责检查报告的准入结果。
/// 有效路由与模板事件派发一致：嵌套动作路由优先于绑定上的备用路由。
pub fn validate_document_action_policy(
    document: &UiAssetDocument,
    policy: &UiActionHostPolicy,
) -> UiActionPolicyReport {
    let mut diagnostics = Vec::new();
    for node in document.iter_nodes() {
        for binding in &node.bindings {
            let action_ref = binding.action.as_ref();
            let route = action_ref
                .and_then(|action| action.route.as_deref())
                .or(binding.route.as_deref());
            let action = action_ref.and_then(|action| action.action.as_deref());
            let side_effect = UiActionSideEffectClass::infer(route, action);
            if policy.allows(side_effect) {
                continue;
            }

            diagnostics.push(UiActionPolicyDiagnostic {
                severity: UiActionPolicyDiagnosticSeverity::Error,
                node_id: node.node_id.clone(),
                binding_id: binding.id.clone(),
                route: route.map(str::to_string),
                action: action.map(str::to_string),
                side_effect,
                message: format!(
                    "binding {} on node {} requires {:?} side effects not allowed by host policy",
                    binding.id, node.node_id, side_effect
                ),
            });
        }
    }
    UiActionPolicyReport { diagnostics }
}

#[cfg(test)]
#[path = "validate/tests/action_ref_tests.rs"]
mod action_ref_tests;
