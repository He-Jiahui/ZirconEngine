use crate::script::CapabilitySet;

use super::capability_fixture::{
    assert_registered_capability_descriptor, bridge_capability_case, fixed_capability_cases,
    registered_bridge_exports, registered_builtin_exports,
};

#[test]
fn host_capability_representatives_are_declared_on_registered_modules() {
    let exports = registered_builtin_exports();

    for case in fixed_capability_cases() {
        assert_registered_capability_descriptor(&exports, &case);
    }

    let bridge_exports = registered_bridge_exports();
    assert_registered_capability_descriptor(&bridge_exports, &bridge_capability_case());
}

#[test]
fn host_function_without_required_capability_is_rejected_with_explicit_error() {
    let exports = registered_builtin_exports();

    for case in fixed_capability_cases() {
        let error = exports
            .call_with_capabilities(
                case.module,
                case.function,
                case.arguments.clone(),
                &CapabilitySet::default(),
            )
            .unwrap_err();

        assert!(
            format!("{error}").contains(&format!("missing capability {}", case.capability)),
            "call to {}.{} should reject missing capability `{}`",
            case.module,
            case.function,
            case.capability
        );
    }

    let bridge_exports = registered_bridge_exports();
    let bridge_case = bridge_capability_case();
    let error = bridge_exports
        .call_with_capabilities(
            bridge_case.module,
            bridge_case.function,
            bridge_case.arguments,
            &CapabilitySet::default(),
        )
        .unwrap_err();

    assert!(
        format!("{error}").contains(&format!("missing capability {}", bridge_case.capability)),
        "bridge host call should reject missing capability `{}`",
        bridge_case.capability
    );
}
