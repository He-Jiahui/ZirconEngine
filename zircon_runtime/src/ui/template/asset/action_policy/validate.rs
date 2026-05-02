use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyDiagnosticSeverity,
    UiActionPolicyReport, UiActionSideEffectClass, UiAssetDocument,
};

pub fn validate_document_action_policy(
    document: &UiAssetDocument,
    policy: &UiActionHostPolicy,
) -> UiActionPolicyReport {
    let mut diagnostics = Vec::new();
    for node in document.iter_nodes() {
        for binding in &node.bindings {
            let route = binding
                .action
                .as_ref()
                .and_then(|action| action.route.as_deref())
                .or(binding.route.as_deref());
            let action = binding
                .action
                .as_ref()
                .and_then(|action| action.action.as_deref());
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
