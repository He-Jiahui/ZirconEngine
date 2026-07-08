use super::support::read_repo_file;

#[test]
fn runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt() {
    let navigation_input = read_repo_file("zircon_runtime/src/ui/surface/input/navigation.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending";

    assert!(
        navigation_input.contains("routed_reply"),
        "Runtime 09 M1.2 should use semantic navigation route reply naming"
    );
    assert!(
        !navigation_input.contains("legacy"),
        "Runtime 09 M1.2 should remove legacy wording from navigation route reply code"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        assert!(
            doc_source.contains(status_anchor),
            "{doc_name} should record Runtime 09 M1.2 navigation legacy reply rename status"
        );
    }
}

#[test]
fn runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt() {
    let pointer_input = read_repo_file("zircon_runtime/src/ui/surface/input/pointer.rs");
    let pointer_reply = read_repo_file("zircon_runtime/src/ui/surface/input/pointer_reply.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../docs/zircon_runtime/ui/surface/input.md");
    let status_anchor = "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt";

    for (file_name, file_source) in [
        ("pointer.rs", pointer_input.as_str()),
        ("pointer_reply.rs", pointer_reply.as_str()),
    ] {
        assert!(
            file_source.contains("routed_result"),
            "Runtime 09 M1.2 should use semantic routed_result naming in {file_name}"
        );
        assert!(
            !file_source.contains("legacy"),
            "Runtime 09 M1.2 should remove legacy wording from {file_name}"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("surface input doc", input_doc),
    ] {
        for required_anchor in [status_anchor, guard_anchor, "routed_result"] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 pointer legacy reply rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt() {
    let pointer_capture =
        read_repo_file("zircon_runtime/src/ui/surface/input/state/pointer_capture.rs");
    let focus_pointer =
        read_repo_file("zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../docs/zircon_runtime/ui/surface/input.md");
    let status_anchor =
        "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt";
    let semantic_name = "has_pointer_capture_for_owner";
    let stale_unindexed_api = concat!("has_pointer_capture_or_", "unindexed_fallback_for_owner");

    for (file_name, file_source) in [
        ("state/pointer_capture.rs", pointer_capture.as_str()),
        ("effect/focus_pointer.rs", focus_pointer.as_str()),
    ] {
        assert!(
            file_source.contains(semantic_name),
            "Runtime 09 M1.2 should use semantic pointer capture fallback naming in {file_name}"
        );
        assert!(
            !file_source.contains("has_legacy_or_indexed_pointer_capture_for_owner"),
            "Runtime 09 M1.2 should remove legacy wording from the pointer capture fallback API in {file_name}"
        );
        assert!(
            !file_source.contains(stale_unindexed_api),
            "Runtime 09 M4 should remove the unindexed pointer capture fallback API from {file_name}"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("surface input doc", input_doc),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 pointer capture fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt() {
    let table_rows =
        read_repo_file("zircon_runtime/src/ui/surface/render/collection_rows/table.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt";
    let semantic_name = "split_row_label_table_text";

    assert!(
        table_rows.contains(semantic_name),
        "Runtime 09 M1.2 should use semantic row-label fallback table splitting"
    );
    assert!(
        !table_rows.contains("split_legacy_table_text"),
        "Runtime 09 M1.2 should remove legacy wording from table row-label fallback splitting"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 table row-label fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt() {
    let interaction = read_repo_file("zircon_runtime/src/ui/template/build/interaction.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor =
        "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt";
    let semantic_name = "component_name_interaction_fallback";

    assert!(
        interaction.contains(semantic_name),
        "Runtime 09 M1.2 should name the template fallback after component-name inference"
    );
    for retired_name in [
        "legacy_component_interaction_fallback",
        "legacy_interactive",
    ] {
        assert!(
            !interaction.contains(retired_name),
            "Runtime 09 M1.2 should remove `{retired_name}` from template interaction inference"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 template component-name fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt() {
    let property_mutation = read_repo_file("zircon_runtime/src/ui/surface/property_mutation.rs");
    let property_mutation_doc =
        include_str!("../../../../../docs/zircon_runtime/ui/surface/property_mutation.md");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt";
    let semantic_name = "state_visible_flag";

    assert!(
        property_mutation.contains(semantic_name),
        "Runtime 09 M1.2 should name the visibility transition input after the state visible flag"
    );
    assert!(
        !property_mutation.contains("legacy_visible"),
        "Runtime 09 M1.2 should remove legacy wording from property mutation visibility transition"
    );

    for (doc_name, doc_source) in [
        ("property mutation doc", property_mutation_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 property visibility flag rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt() {
    let responsive_mui = read_repo_file("zircon_runtime/src/ui/layout/pass/responsive_mui.rs");
    let layout_pass_doc = include_str!("../../../../../docs/zircon_runtime/ui/layout/pass.md");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt";
    let semantic_name = "state_visible_flag";

    assert!(
        responsive_mui.contains(semantic_name),
        "Runtime 09 M1.2 should name responsive visible input after the state visible flag"
    );
    assert!(
        !responsive_mui.contains("legacy_visible"),
        "Runtime 09 M1.2 should remove legacy wording from responsive MUI visibility DTO"
    );

    for (doc_name, doc_source) in [
        ("layout pass doc", layout_pass_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 responsive MUI visibility flag rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt() {
    let accessibility_extract = read_repo_file("zircon_runtime/src/ui/accessibility/extract.rs");
    let accessibility_doc = include_str!("../../../../../docs/zircon_runtime/ui/accessibility.md");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt";
    let semantic_name = "fallback_properties";

    assert!(
        accessibility_extract.contains(semantic_name),
        "Runtime 09 M1.2 should name accessibility open-state alternatives as fallback properties"
    );
    assert!(
        !accessibility_extract.contains("legacy_properties"),
        "Runtime 09 M1.2 should remove legacy wording from accessibility open-state fallback properties"
    );
    assert!(
        !accessibility_extract.contains("legacy_property"),
        "Runtime 09 M1.2 should remove legacy wording from accessibility open-state fallback locals"
    );

    for (doc_name, doc_source) in [
        ("accessibility doc", accessibility_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 accessibility fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt() {
    let layout_engine_contract = read_repo_file("zircon_runtime_interface/src/ui/layout/engine.rs");
    let layout_pass_engine = read_repo_file("zircon_runtime/src/ui/layout/pass/engine.rs");
    let layout_pass_doc = include_str!("../../../../../docs/zircon_runtime/ui/layout/pass.md");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt";

    for forbidden_name in ["LegacyZircon", "legacy_zircon", "legacy_selected_count"] {
        for (file_name, file_source) in [
            ("layout engine contract", layout_engine_contract.as_str()),
            ("layout pass engine", layout_pass_engine.as_str()),
        ] {
            assert!(
                !file_source.contains(forbidden_name),
                "Runtime 09 M1.2 should remove old layout engine backend name `{forbidden_name}` from {file_name}"
            );
        }
    }

    assert!(
        layout_engine_contract.contains("UiLayoutEngineBackend::Zircon")
            && layout_engine_contract.contains("pub fn zircon()"),
        "Runtime 09 M1.2 layout engine contract should retain the Zircon backend and constructor"
    );
    assert!(
        layout_pass_engine.contains("UiLayoutEngineBackend::Zircon")
            && layout_pass_engine.contains("UiLayoutEngineCapability::zircon()"),
        "Runtime 09 M1.2 layout pass should consume the Zircon backend constructor"
    );
    assert!(
        layout_engine_contract.contains("zircon_selected_count"),
        "Runtime 09 M1.2 layout engine report should expose zircon_selected_count"
    );

    for (doc_name, doc_source) in [
        ("layout pass doc", layout_pass_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            guard_anchor,
            "UiLayoutEngineBackend::Zircon",
            "UiLayoutEngineCapability::zircon",
            "zircon_selected_count",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 layout engine backend cutover anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt() {
    let default_interactions =
        read_repo_file("zircon_runtime/src/ui/surface/surface/default_interactions.rs");
    let default_interactions_doc =
        include_str!("../../../../../docs/zircon_runtime/ui/surface/default_interactions.md");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor =
        "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt";
    let semantic_name = "fallback_properties";

    assert!(
        default_interactions.contains("fn default_open_boolean_value("),
        "Runtime 09 M1.2 should keep default open-state fallback lookup in default_interactions"
    );
    assert!(
        default_interactions.contains(semantic_name)
            && default_interactions.contains("fallback_property"),
        "Runtime 09 M1.2 should name default interaction open-state alternatives as fallback properties"
    );
    for retired_name in ["legacy_properties", "legacy_property"] {
        assert!(
            !default_interactions.contains(retired_name),
            "Runtime 09 M1.2 should remove `{retired_name}` from default interaction open-state fallback lookup"
        );
    }

    for (doc_name, doc_source) in [
        ("default interactions doc", default_interactions_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            guard_anchor,
            "default_open_boolean_value",
            semantic_name,
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 default interaction fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_ui_input_events_route_through_single_dispatch_authority() {
    let dispatch_input = read_repo_file("zircon_runtime/src/ui/surface/input/dispatch.rs");
    let input_mod = read_repo_file("zircon_runtime/src/ui/surface/input/mod.rs");
    let route_authority = read_repo_file("zircon_runtime/src/ui/surface/input/route_authority.rs");
    let input_manager_routing =
        read_repo_file("zircon_runtime/src/ui/dispatch/input_manager/routing.rs");
    let surface = read_repo_file("zircon_runtime/src/ui/surface/surface.rs");
    let runtime_manager =
        read_repo_file("zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_manager.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor = "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending";
    let bypass_verdict = "runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers";

    for dispatch_anchor in [
        "mod route_authority;",
        "annotate_authoritative_input_dispatch",
        "dispatch_pointer_input",
        "dispatch_navigation_input",
        "dispatch_keyboard_input",
        "dispatch_drag_drop_input",
        "Ok(result)",
    ] {
        assert!(
            dispatch_input.contains(dispatch_anchor) || input_mod.contains(dispatch_anchor),
            "unified UiInputEvent dispatch should retain `{dispatch_anchor}`"
        );
    }

    for authority_anchor in [
        "runtime_09_m1_1_ui_input_route_authority",
        "route_authority=",
        "route_authority_stage_names_for_policy",
        "route_stage_names_for_policy",
    ] {
        assert!(
            route_authority.contains(authority_anchor),
            "Runtime 09 M1.1 route authority module should retain `{authority_anchor}`"
        );
    }
    for routing_anchor in [
        "UI_INPUT_ROUTE_ORDER",
        "UiInputRouteStage::PointerCapture",
        "UiInputRouteStage::PopupStack",
        "UiInputRouteStage::PreviewTunnel",
        "UiInputRouteStage::DirectTarget",
        "UiInputRouteStage::BubblePath",
        "UiInputRouteStage::FocusPath",
        "UiInputRouteStage::DefaultAction",
        "route_policy_uses_stage",
        "route_stage_name",
        "route_stage_names_for_policy",
    ] {
        assert!(
            input_manager_routing.contains(routing_anchor),
            "input_manager routing authority should retain `{routing_anchor}`"
        );
    }

    assert!(
        surface.contains("pub fn dispatch_input_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_input_event("),
        "surface/runtime_ui_support should keep UiInputEvent dispatch as the normalized input entry"
    );
    assert!(
        surface.contains("pub fn dispatch_pointer_event(")
            && surface.contains("pub fn dispatch_navigation_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_pointer_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_navigation_event("),
        "direct pointer/navigation helpers remain visible and need the documented owner verdict"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, bypass_verdict] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.1 route authority anchor `{required_anchor}`"
            );
        }
    }
}
