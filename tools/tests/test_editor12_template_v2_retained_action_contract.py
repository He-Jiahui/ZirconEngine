from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


def require(text: str, needle: str, context: str) -> None:
    if needle not in text:
        raise AssertionError(f"{context}: missing {needle!r}")


def test_v2_retained_actions_preserve_typed_payloads_and_generation_boundary() -> None:
    model = source("zircon_editor/src/ui/template_runtime/model.rs")
    require(
        model,
        "pub template_action: Option<UiActionRef>",
        "V2 binding projection must retain the authored action",
    )

    host_nodes = source("zircon_editor/src/ui/template_runtime/host_nodes.rs")
    require(
        host_nodes,
        "pub template_action: Option<UiTemplateActionInvocation>",
        "host bindings must carry the resolved typed action",
    )

    projection = source("zircon_editor/src/ui/template_runtime/runtime/projection.rs")
    require(
        projection,
        "resolve_template_actions",
        "projection must resolve action expressions against pane state",
    )
    require(
        projection,
        "resolves_typed_action_payload_from_a_control_property_snapshot",
        "typed action payload resolution needs a focused Rust regression",
    )
    require(
        projection,
        "surface_node.attributes.insert",
        "pane component patches must override authored surface attributes",
    )

    registry = source(
        "zircon_editor/src/ui/template_runtime/runtime/template_action_registry.rs"
    )
    require(
        registry,
        "pub(super) struct TemplateActionRegistry",
        "registry must own token lifecycle",
    )
    require(
        registry,
        "plugin_owner",
        "plugin actions must be generation checked",
    )
    require(
        registry,
        "fn remove_pane",
        "reprojecting a pane must discard its stale action tokens",
    )
    require(
        registry,
        "fn remove_document",
        "retiring a plugin document must discard its action tokens",
    )
    require(
        registry,
        "fn update_control_attributes_for_pane",
        "a retained control-state update must refresh the pane-scoped action snapshot",
    )
    require(
        registry,
        "fn select_table_row",
        "table selection must validate the current pane snapshot before updating control state",
    )
    require(
        registry,
        "fn rebind_pane",
        "presentation rebuilds must retain only valid same-generation table selection state",
    )
    require(
        registry,
        "pane_binding_epochs",
        "every pane rebind must mint action tokens in a new local epoch",
    )
    require(
        registry,
        "g{owner_generation}/e{pane_epoch}",
        "plugin owner generation and pane epoch must be encoded in action tokens",
    )
    require(
        registry,
        "action_source_is_disabled",
        "disabled template controls must not resolve an action invocation",
    )
    require(
        registry,
        "disabled_action_source_does_not_resolve_an_invocation",
        "disabled action sources need a focused Rust regression",
    )
    require(
        registry,
        "explicitly_disabled_action_source_does_not_resolve_an_invocation",
        "enabled=false action sources need a focused Rust regression",
    )
    require(
        registry,
        "dynamic_disabled_state_blocks_an_already_bound_control_action",
        "dynamic disabled state must block an already-bound action",
    )

    runtime_host = source(
        "zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs"
    )
    require(
        runtime_host,
        "template_action_registry",
        "runtime must own action token state",
    )
    require(
        runtime_host,
        "bind_template_actions_for_pane",
        "pane projection must bind its own action tokens",
    )
    require(
        runtime_host,
        "update_template_action_control_state",
        "retained input must have a generic generation-scoped action-state refresh entry point",
    )
    require(
        runtime_host,
        "select_template_table_row",
        "retained table selection must update the current pane action snapshot",
    )
    require(
        runtime_host,
        "apply_pane_component_patches_to_surface",
        "body component patches must reach the native surface before retained projection",
    )
    require(
        runtime_host,
        "apply_template_control_attributes_to_surface",
        "the generation-scoped action snapshot must also update native control state",
    )
    require(
        runtime_host,
        "pane_control_state_projects_rows_selection_and_disabled_to_native_and_retained_models",
        "native and retained pane state needs a focused Rust regression",
    )
    require(
        runtime_host,
        "dispatch_template_action_for_token",
        "typed action lookup and dispatch must share one generation-stable boundary",
    )
    require(
        runtime_host,
        "plugin_v2_documents",
        "plugin owner lookup must be held stable through typed action dispatch",
    )
    require(
        runtime_host,
        "is_template_action_token",
        "unresolved template actions must be recognized without falling through legacy dispatch",
    )
    require(
        runtime_host,
        "remove_template_actions_for_pane",
        "pane teardown must discard generation-scoped action state",
    )

    pane_projection = source(
        "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs"
    )
    require(
        pane_projection,
        "bind_template_actions_for_pane",
        "retained pane conversion must register action tokens",
    )
    require(
        pane_projection,
        "apply_pane_component_patches_to_surface",
        "native surface state must be patched before layout and action binding",
    )
    require(
        pane_projection,
        "clear_template_actions_for_pane",
        "missing or non-V2 pane presentations must remove stale action state",
    )
    require(
        pane_projection,
        "Some(data.id.as_str())",
        "template action tokens must be scoped by the concrete retained pane identity",
    )

    pane_conversion = source(
        "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs"
    )
    require(
        pane_conversion,
        "remove_template_actions_for_pane(&pane_id)",
        "the non-V2 pane conversion branch must remove stale template action state",
    )

    hit = source(
        "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs"
    )
    require(hit, "disabled: node.disabled", "pointer hits must retain disabled state")
    require(
        hit,
        "table_row_source_index",
        "pointer hits must retain the table row's source index",
    )

    retained_app = source("zircon_editor/src/ui/retained_host/app.rs")
    click = source("zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs")
    require(
        retained_app,
        "UiPointerComponentEvent",
        "retained host must expose the typed pointer-component event contract",
    )
    require(
        click,
        "dispatch_pointer_component_event",
        "retained host must consume generic pointer component events",
    )
    require(
        click,
        "event.template_action",
        "pointer component events without a typed action must not submit an operation",
    )
    require(
        click,
        "dispatch_template_action_for_token",
        "host click dispatch must consume typed template-action tokens atomically",
    )
    require(
        click,
        "dispatch_template_action",
        "typed template action must use the generic operation dispatcher",
    )
    require(
        click,
        "is_template_action_token(action_id)",
        "an unresolved template action must not fall through to a legacy action route",
    )
    require(
        click,
        "dispatch_template_table_row_selected",
        "selected table rows must update retained template state through the host bridge",
    )
    require(
        click,
        "self.mark_presentation_dirty();",
        "a successful row selection must refresh the retained presentation",
    )

    activation = source(
        "zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/dispatch.rs"
    )
    require(
        activation,
        "if hit.disabled",
        "disabled controls must not submit operations",
    )
    require(
        activation,
        "invoke_template_table_row_selected",
        "table row hits must be handled before generic control activation",
    )

    activation_tests = source(
        "zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics_tests.rs"
    )
    require(
        activation_tests,
        "table_row_primary_press_emits_the_current_typed_selection",
        "the row selection callback needs a focused typed identity regression",
    )
    require(
        activation_tests,
        "disabled_table_row_primary_press_does_not_emit_selection",
        "disabled table rows must not update selection state",
    )

    pane_callbacks = source(
        "zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs"
    )
    require(
        pane_callbacks,
        "on_template_table_row_selected",
        "the retained host must expose a generic table row selection callback",
    )
    require(
        pane_callbacks,
        "on_pointer_component_event",
        "the pane host must expose the typed runtime pointer-event callback",
    )

    pane_callback_storage = source(
        "zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/pane.rs"
    )
    require(
        pane_callback_storage,
        "pointer_component_event",
        "the pane callback storage must retain the typed runtime pointer-event callback",
    )

    pane_wiring = source(
        "zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/pane_controls.rs"
    )
    require(
        pane_wiring,
        "on_pointer_component_event",
        "pane wiring must subscribe to typed runtime pointer events",
    )
    require(
        pane_wiring,
        "host.dispatch_pointer_component_event(event)",
        "typed runtime pointer events must reach the retained action dispatcher",
    )

    shared_sources = "\n".join(
        [model, host_nodes, projection, runtime_host, pane_projection, click, activation]
    )
    if "navigation.bake" in shared_sources or "NavigationBake" in shared_sources:
        raise AssertionError("shared retained-host action contract must not special-case Navigation")


if __name__ == "__main__":
    test_v2_retained_actions_preserve_typed_payloads_and_generation_boundary()
