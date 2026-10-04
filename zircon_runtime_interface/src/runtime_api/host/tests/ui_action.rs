use std::collections::BTreeMap;

use super::*;
use crate::ui::component::UiValue;

#[test]
fn secure_ui_action_delivery_serializes_identity_without_plaintext() {
    let tree_id = UiTreeId::new("secure.auth");
    let reference = UiSecureTextValueRef::issue(tree_id.clone(), UiNodeId::new(7), "value");
    let request = ZrRuntimeUiActionHostRequestV1::new(
        ZrRuntimeViewportHandle::new(3),
        5,
        11,
        0,
        tree_id,
        UiNodeId::new(7),
        UiTemplateActionInvocation::route(
            "woc.shell.auth.submit",
            BTreeMap::from([("credential".to_string(), UiValue::Null)]),
        ),
        Some(reference),
    );

    let encoded = serde_json::to_string(&request).expect("serialize secure UI action");
    assert!(encoded.contains("woc.shell.auth.submit"));
    assert!(!encoded.contains("correct horse battery staple"));

    let debug = format!("{request:?}");
    assert!(debug.contains("woc.shell.auth.submit"));
    assert!(!debug.contains("credential"));
}
