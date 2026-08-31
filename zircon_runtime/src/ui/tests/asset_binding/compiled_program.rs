use super::*;

use std::hint::black_box;
use std::time::Instant;

use crate::ui::template::{UiRuntimeCompiledAssetArtifact, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::{UiNodeId, UiTreeId},
    template::{
        UiBindingDiagnosticCode, UiBindingId, UiBindingMissingValuePolicy, UiBindingMode,
        UiBindingTriggerTiming, UiBindingWritePermissions, UiCompiledActionPayloadValue,
        UiCompiledBindingExpression, UiCompiledBindingProgram, UiCompiledBindingTargetKind,
        UiCompiledNodeId, UiPropertyId, UI_BINDING_EXPRESSION_MAX_DEPTH,
        UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
    },
};

#[test]
fn compiler_cooks_event_binding_mode_contract_into_binding_program() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.binding_mode.event"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onClick"
event = "Click"
mode = "Event"
route = "runtime74.binding_mode.event"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let binding = program.binding(handle).unwrap();

    assert_eq!(binding.mode, UiBindingMode::Event);
    assert_eq!(
        binding.mode.trigger_timing(),
        UiBindingTriggerTiming::EventDispatch
    );
    assert_eq!(
        binding.mode.write_permissions(),
        UiBindingWritePermissions::TARGET_ONLY
    );
}

#[test]
fn compiler_rejects_unimplemented_binding_modes_with_stable_diagnostic() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let compiler = UiDocumentCompiler::default().with_component_registry(registry.clone());

    for mode in ["OneTime", "OneWay", "TwoWay", "Command"] {
        let source = format!(
            r#"
[asset]
kind = "layout"
id = "runtime74.binding_mode.unsupported"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onClick"
event = "Click"
mode = "{mode}"
route = "runtime74.binding_mode.unsupported"
"#
        );
        let document = UiAssetLoader::load_toml_str(&source).unwrap();
        let report = collect_asset_binding_report(&document, &registry);

        assert_eq!(report.diagnostics.len(), 1, "{mode}");
        assert_eq!(
            report.diagnostics[0].code,
            UiBindingDiagnosticCode::UnsupportedBindingMode,
            "{mode}"
        );
        assert!(report.diagnostics[0]
            .message
            .contains("does not have a runtime executor"));
        assert!(compiler.compile(&document).is_err(), "{mode}");
    }
}

#[test]
fn compiler_interns_binding_identity_and_target_endpoints() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();

    assert!(!program.generation().is_invalid());
    assert_eq!(program.asset_id(), Some("editor.binding.valid"));
    assert_eq!(program.node_count(), 1);
    assert_eq!(program.binding_count(), 1);
    assert!(program.is_well_formed());

    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .expect("root binding should have a compiled handle");
    assert_eq!(handle.binding_id, UiBindingId::new(0));
    assert_eq!(program.binding_name(handle), Some("Root/onClick"));

    let binding = program.binding(handle).expect("handle should resolve");
    assert_eq!(binding.targets.len(), 5);
    assert_eq!(
        program.route_name(binding.route_id.unwrap()),
        Some("Route.Valid")
    );
    assert_eq!(program.property_name(UiPropertyId::new(0)), Some("text"));
    assert_eq!(
        binding.targets[0].kind,
        UiCompiledBindingTargetKind::Property
    );
    assert_eq!(binding.targets[0].property, Some(UiPropertyId::new(0)));
    assert_eq!(
        binding.targets[0].expression,
        UiCompiledBindingExpression::Literal(UiValue::String("Bound".to_string()))
    );
    for (target_index, target) in binding.targets.iter().enumerate() {
        assert_eq!(target.endpoint.generation, program.generation());
        assert_eq!(target.endpoint.node_id, UiCompiledNodeId::new(0));
        assert_eq!(target.endpoint.binding_id, UiBindingId::new(0));
        assert_eq!(target.endpoint.target_index.get() as usize, target_index);
    }
}

#[test]
fn compiler_retains_explicit_missing_value_policy_and_defaults_legacy_targets_to_required() {
    let mut document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let targets = &mut document.root.as_mut().unwrap().bindings[0].targets;
    assert_eq!(
        targets[0].target.missing_policy,
        UiBindingMissingValuePolicy::Required
    );
    targets[0].target.missing_policy = UiBindingMissingValuePolicy::Fallback {
        value: UiValue::String("fallback".to_string()),
    };
    document.root.as_mut().unwrap().bindings[0]
        .action
        .as_mut()
        .unwrap()
        .payload_missing_policy = UiBindingMissingValuePolicy::Optional;

    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let target = &program.binding(handle).unwrap().targets[0];

    assert_eq!(
        target.missing_policy,
        UiBindingMissingValuePolicy::Fallback {
            value: UiValue::String("fallback".to_string()),
        }
    );
    assert_eq!(
        program.binding(handle).unwrap().payload_missing_policy,
        UiBindingMissingValuePolicy::Optional
    );
    let round_trip: UiCompiledBindingProgram =
        toml::Value::try_from(program).unwrap().try_into().unwrap();
    assert_eq!(
        round_trip.binding(handle).unwrap().targets[0].missing_policy,
        target.missing_policy
    );
}

#[test]
fn compiled_program_accepts_legacy_payload_without_asset_identity() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let mut serialized = toml::Value::try_from(compiled.template_instance().binding_program())
        .expect("binding program should serialize");
    serialized
        .as_table_mut()
        .expect("binding program should serialize as a table")
        .remove("asset_id");

    let legacy: UiCompiledBindingProgram = serialized.try_into().unwrap();

    assert_eq!(legacy.asset_id(), None);
    assert!(legacy.is_well_formed());
}

#[test]
fn compiler_interns_direct_action_identity() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.compiled_action"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onSave"
event = "Click"

[root.bindings.action]
action = "editor.save"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let binding = program.binding(handle).unwrap();

    assert_eq!(
        program.action_name(binding.action_id.unwrap()),
        Some("editor.save")
    );
    assert!(binding.route_id.is_none());
}

#[test]
fn compiler_cooks_typed_component_event_into_binding_program() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.compiled_component_event"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onOpen"
event = "Click"
component_event = "OpenPopup"
route = "editor.popup.open"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();

    assert_eq!(
        program.binding(handle).unwrap().component_event,
        Some(zircon_runtime_interface::ui::component::UiComponentEventKind::OpenPopup)
    );
}

#[test]
fn compiler_cooks_action_payload_values_into_binding_program() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.compiled_payload"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onRoute"
event = "Click"

[root.bindings.action]
route = "editor.route"

[root.bindings.action.payload]
fixed = "compiled"
current = "=prop.text"
preview_only = "=concat(self.text, \"!\")"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let binding = program.binding(handle).unwrap();

    assert_eq!(binding.payload_fields.len(), 3);
    let payload = binding
        .payload_fields
        .iter()
        .map(|field| (program.property_name(field.property).unwrap(), &field.value))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        payload.get("fixed").copied(),
        Some(&UiCompiledActionPayloadValue::Literal(UiValue::String(
            "compiled".to_string()
        )))
    );
    assert!(matches!(
        payload.get("current").copied(),
        Some(UiCompiledActionPayloadValue::Expression(
            UiCompiledBindingExpression::Property(_)
        ))
    ));
    assert_eq!(
        payload.get("preview_only").copied(),
        Some(&UiCompiledActionPayloadValue::Unavailable)
    );
    assert!(program.is_well_formed());
}

#[test]
fn compiler_rejects_non_finite_compiled_payload_literals() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.non_finite_payload"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onRoute"
event = "Click"

[root.bindings.action]
route = "editor.route"

[root.bindings.action.payload]
invalid = nan
"#,
    )
    .unwrap();

    let error = UiDocumentCompiler::default()
        .compile(&document)
        .expect_err("non-finite payloads must not enter compiled binding IR");

    assert!(error
        .to_string()
        .contains("compiled binding program is malformed"));
}

#[test]
fn compiled_program_rejects_reordered_source_binding_slots() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.binding_order"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/first"
event = "Click"
route = "Route.First"

[[root.bindings]]
id = "Root/second"
event = "Click"
route = "Route.Second"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let mut serialized = toml::Value::try_from(compiled.template_instance().binding_program())
        .expect("binding program should serialize");

    serialized
        .get_mut("nodes")
        .and_then(toml::Value::as_array_mut)
        .and_then(|nodes| nodes.first_mut())
        .and_then(toml::Value::as_table_mut)
        .and_then(|node| node.get_mut("binding_ids"))
        .and_then(toml::Value::as_array_mut)
        .expect("root binding ids should serialize as an array")
        .reverse();
    let bindings = serialized
        .get_mut("bindings")
        .and_then(toml::Value::as_array_mut)
        .expect("compiled bindings should serialize as an array");
    bindings[0]
        .as_table_mut()
        .unwrap()
        .insert("source_binding_index".to_string(), toml::Value::Integer(1));
    bindings[1]
        .as_table_mut()
        .unwrap()
        .insert("source_binding_index".to_string(), toml::Value::Integer(0));

    let reordered: UiCompiledBindingProgram = serialized.try_into().unwrap();
    assert!(!reordered.is_well_formed());
}

#[test]
fn compiled_program_rejects_reordered_action_payload_fields() {
    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.payload_order"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onRoute"
event = "Click"

[root.bindings.action]
route = "Route.Payload"

[root.bindings.action.payload]
alpha = "first"
middle = "second"
omega = "third"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let mut serialized = toml::Value::try_from(compiled.template_instance().binding_program())
        .expect("binding program should serialize");

    serialized
        .get_mut("bindings")
        .and_then(toml::Value::as_array_mut)
        .and_then(|bindings| bindings.first_mut())
        .and_then(toml::Value::as_table_mut)
        .and_then(|binding| binding.get_mut("payload_fields"))
        .and_then(toml::Value::as_array_mut)
        .expect("payload fields should serialize as an array")
        .reverse();

    let reordered: UiCompiledBindingProgram = serialized.try_into().unwrap();
    assert!(!reordered.is_well_formed());
}

#[test]
fn compiled_artifact_round_trip_preserves_binding_program_generation() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let artifact = UiDocumentCompiler::default()
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let expected = artifact.compiled.binding_program().clone();

    let bytes = artifact.to_bytes().unwrap();
    let restored = UiRuntimeCompiledAssetArtifact::from_bytes(&bytes).unwrap();

    assert_eq!(restored.compiled.binding_program(), &expected);
    assert_eq!(restored.compiled.binding_program().binding_count(), 1);
}

#[test]
fn compiler_rejects_over_budget_binding_expression_before_program_publication() {
    let mut document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    document.root.as_mut().unwrap().bindings[0].targets[0].expression =
        "x".repeat(UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES + 1);

    let error = UiDocumentCompiler::default()
        .compile(&document)
        .unwrap_err();

    assert!(
        error.to_string().contains("source bytes budget"),
        "unexpected compile error: {error}"
    );
}

#[test]
fn stale_compiled_binding_handle_fails_closed_without_mutating_target() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let mut stale_handle = compiled
        .template_instance()
        .binding_program()
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    stale_handle.generation =
        zircon_runtime_interface::ui::template::UiCompiledBindingGeneration::new(
            stale_handle.generation.get().wrapping_add(1),
        );
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime74.compiled_binding.stale"),
        &compiled,
    )
    .unwrap();
    let node_id = UiNodeId::new(1);
    surface
        .tree
        .node_mut(node_id)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("text".to_string(), toml::Value::String("Ready".to_string()));

    let event = UiPointerComponentEvent::new(
        &surface.tree.tree_id,
        node_id,
        "BindingRoot",
        "Root/onClick",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_compiled_binding(stale_handle);
    let mut events = vec![event];
    let reports = surface.apply_pointer_binding_targets(&mut events).unwrap();

    assert!(events.is_empty());
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].rejected_count, 1);
    let receipt = reports[0]
        .execution_receipt
        .as_ref()
        .expect("stale endpoint should publish a bounded execution receipt");
    assert_eq!(receipt.asset_id, "editor.binding.valid");
    assert_eq!(receipt.binding_id, "Root/onClick");
    assert_eq!(receipt.generation, stale_handle.generation.get());
    assert_eq!(receipt.execution_count, 0);
    assert_eq!(receipt.miss_count, 1);
    assert_eq!(receipt.error_count, 0);
    assert_eq!(
        surface
            .tree
            .node(node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("text"),
        Some(&toml::Value::String("Ready".to_string()))
    );
}

#[test]
fn mismatched_compiled_target_endpoint_fails_closed_without_mutating_target() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let mut serialized = toml::Value::try_from(program).unwrap();
    let target_endpoint = serialized
        .get_mut("bindings")
        .and_then(toml::Value::as_array_mut)
        .and_then(|bindings| bindings.first_mut())
        .and_then(toml::Value::as_table_mut)
        .and_then(|binding| binding.get_mut("targets"))
        .and_then(toml::Value::as_array_mut)
        .and_then(|targets| targets.first_mut())
        .and_then(toml::Value::as_table_mut)
        .and_then(|target| target.get_mut("endpoint"))
        .and_then(toml::Value::as_table_mut)
        .expect("compiled target endpoint should serialize as a table");
    target_endpoint.insert("target_index".to_string(), toml::Value::Integer(99));
    let corrupted_program: UiCompiledBindingProgram = serialized.try_into().unwrap();
    assert!(!corrupted_program.is_well_formed());

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime74.compiled_binding.target_mismatch"),
        &compiled,
    )
    .unwrap();
    surface.install_compiled_binding_program(corrupted_program);
    let node_id = UiNodeId::new(1);
    surface
        .tree
        .node_mut(node_id)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("text".to_string(), toml::Value::String("Ready".to_string()));

    let event = UiPointerComponentEvent::new(
        &surface.tree.tree_id,
        node_id,
        "BindingRoot",
        "Root/onClick",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_compiled_binding(handle);
    let mut events = vec![event];
    let reports = surface.apply_pointer_binding_targets(&mut events).unwrap();

    assert!(events.is_empty());
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].rejected_count, 1);
    assert_eq!(
        surface
            .tree
            .node(node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("text"),
        Some(&toml::Value::String("Ready".to_string()))
    );
}

#[test]
fn over_budget_compiled_expression_fails_closed_without_mutating_target() {
    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let mut over_budget = UiCompiledBindingExpression::Literal(UiValue::Bool(true));
    for _ in 0..UI_BINDING_EXPRESSION_MAX_DEPTH {
        over_budget = UiCompiledBindingExpression::Not(Box::new(over_budget));
    }
    let mut serialized = toml::Value::try_from(program).unwrap();
    let target = serialized
        .get_mut("bindings")
        .and_then(toml::Value::as_array_mut)
        .and_then(|bindings| bindings.first_mut())
        .and_then(toml::Value::as_table_mut)
        .and_then(|binding| binding.get_mut("targets"))
        .and_then(toml::Value::as_array_mut)
        .and_then(|targets| targets.first_mut())
        .and_then(toml::Value::as_table_mut)
        .expect("compiled target should serialize as a table");
    target.insert(
        "expression".to_string(),
        toml::Value::try_from(over_budget).unwrap(),
    );
    let corrupted_program: UiCompiledBindingProgram = serialized.try_into().unwrap();
    assert!(!corrupted_program.is_well_formed());

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime74.compiled_binding.expression_budget"),
        &compiled,
    )
    .unwrap();
    surface.install_compiled_binding_program(corrupted_program);
    let node_id = UiNodeId::new(1);
    surface
        .tree
        .node_mut(node_id)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("text".to_string(), toml::Value::String("Ready".to_string()));

    let event = UiPointerComponentEvent::new(
        &surface.tree.tree_id,
        node_id,
        "BindingRoot",
        "Root/onClick",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_compiled_binding(handle);
    let mut events = vec![event];
    let reports = surface.apply_pointer_binding_targets(&mut events).unwrap();

    assert!(events.is_empty());
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].rejected_count, 1);
    assert_eq!(
        surface
            .tree
            .node(node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("text"),
        Some(&toml::Value::String("Ready".to_string()))
    );
}

#[test]
fn compiled_binding_endpoint_lookup_improves_nearest_rank_p95_by_at_least_twenty_five_percent() {
    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_TARGET: usize = 4_000;

    let document = UiAssetLoader::load_toml_str(VALID_BINDING_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let source_binding = &document.root.as_ref().unwrap().bindings[0];
    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut compiled_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut checksum = 0u64;

    for sample_index in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if sample_index % 2 == 0 {
            let legacy = measure_legacy_target_lookup(source_binding, LOOKUPS_PER_TARGET);
            let optimized = measure_compiled_target_lookup(program, handle, LOOKUPS_PER_TARGET);
            (legacy, optimized)
        } else {
            let optimized = measure_compiled_target_lookup(program, handle, LOOKUPS_PER_TARGET);
            let legacy = measure_legacy_target_lookup(source_binding, LOOKUPS_PER_TARGET);
            (legacy, optimized)
        };
        legacy_samples_us.push(legacy.0);
        compiled_samples_us.push(optimized.0);
        checksum ^= legacy.1 ^ optimized.1;
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let compiled_p95_us = nearest_rank_p95(&compiled_samples_us);
    assert_ne!(black_box(checksum), u64::MAX);
    assert!(
        u128::from(compiled_p95_us) * 4 <= u128::from(legacy_p95_us) * 3,
        "compiled endpoint P95 {compiled_p95_us}us must improve legacy parse/string lookup P95 {legacy_p95_us}us by at least 25%"
    );
    println!(
        "PERF-RUNTIME74-COMPILED-ENDPOINT sample_pairs={SAMPLE_PAIRS} lookups_per_target={LOOKUPS_PER_TARGET} target_count={} legacy_samples_us={} compiled_samples_us={} legacy_p95_us={legacy_p95_us} compiled_p95_us={compiled_p95_us} improvement_threshold_percent=25",
        source_binding.targets.len(),
        join_samples(&legacy_samples_us),
        join_samples(&compiled_samples_us),
    );
}

#[test]
fn compiled_action_payload_ir_improves_nearest_rank_p95_by_at_least_twenty_five_percent() {
    const SAMPLE_PAIRS: usize = 21;
    const DISPATCHES_PER_SAMPLE: usize = 4_000;

    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "runtime74.compiled_payload_perf"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onRoute"
event = "Click"

[root.bindings.action]
route = "editor.route"

[root.bindings.action.payload]
ready = '=prop.text == "Ready"'
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();
    let compiled_payload = &program.binding(handle).unwrap().payload_fields[0].value;
    let legacy_payload = document.root.as_ref().unwrap().bindings[0]
        .action
        .as_ref()
        .unwrap()
        .payload
        .get("ready")
        .unwrap();
    let surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime74.compiled_payload_perf"),
        &compiled,
    )
    .unwrap();
    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut compiled_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut checksum = 0u64;

    for sample_index in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if sample_index % 2 == 0 {
            let legacy =
                measure_legacy_payload_dispatch(&surface, legacy_payload, DISPATCHES_PER_SAMPLE);
            let optimized = measure_compiled_payload_dispatch(
                &surface,
                compiled_payload,
                DISPATCHES_PER_SAMPLE,
            );
            (legacy, optimized)
        } else {
            let optimized = measure_compiled_payload_dispatch(
                &surface,
                compiled_payload,
                DISPATCHES_PER_SAMPLE,
            );
            let legacy =
                measure_legacy_payload_dispatch(&surface, legacy_payload, DISPATCHES_PER_SAMPLE);
            (legacy, optimized)
        };
        legacy_samples_us.push(legacy.0);
        compiled_samples_us.push(optimized.0);
        checksum ^= legacy.1 ^ optimized.1;
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let compiled_p95_us = nearest_rank_p95(&compiled_samples_us);
    assert_ne!(black_box(checksum), u64::MAX);
    assert!(
        u128::from(compiled_p95_us) * 4 <= u128::from(legacy_p95_us) * 3,
        "compiled payload P95 {compiled_p95_us}us must improve legacy parse/evaluate P95 {legacy_p95_us}us by at least 25%"
    );
    println!(
        "PERF-RUNTIME74-COMPILED-PAYLOAD sample_pairs={SAMPLE_PAIRS} dispatches_per_sample={DISPATCHES_PER_SAMPLE} legacy_samples_us={} compiled_samples_us={} legacy_p95_us={legacy_p95_us} compiled_p95_us={compiled_p95_us} improvement_threshold_percent=25",
        join_samples(&legacy_samples_us),
        join_samples(&compiled_samples_us),
    );
}

fn measure_legacy_payload_dispatch(
    surface: &crate::ui::surface::UiSurface,
    payload: &toml::Value,
    repeats: usize,
) -> (u64, u64) {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..repeats {
        let value = black_box(surface)
            .template_action_payload_value(UiNodeId::new(1), black_box(payload))
            .unwrap();
        checksum = checksum.wrapping_add(u64::from(value == UiValue::Bool(true)));
    }
    (elapsed_us(started), black_box(checksum))
}

fn measure_compiled_payload_dispatch(
    surface: &crate::ui::surface::UiSurface,
    payload: &UiCompiledActionPayloadValue,
    repeats: usize,
) -> (u64, u64) {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..repeats {
        let value = black_box(surface)
            .resolve_compiled_action_payload_value(UiNodeId::new(1), black_box(payload))
            .unwrap();
        checksum = checksum.wrapping_add(u64::from(value == UiValue::Bool(true)));
    }
    (elapsed_us(started), black_box(checksum))
}

fn measure_legacy_target_lookup(
    binding: &zircon_runtime_interface::ui::template::UiBindingRef,
    repeats: usize,
) -> (u64, u64) {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..repeats {
        for target in black_box(&binding.targets) {
            let expression = UiBindingExpression::parse(black_box(&target.expression)).unwrap();
            checksum = checksum.wrapping_add(match expression {
                UiBindingExpression::Literal(_) => 1,
                UiBindingExpression::PropRef(ref property) => property.len() as u64 + 2,
                UiBindingExpression::Equals(_, _) => 3,
                UiBindingExpression::NotEquals(_, _) => 5,
                _ => 7,
            });
            checksum = checksum.wrapping_add(
                target
                    .target
                    .name
                    .as_deref()
                    .map_or(1, |name| name.len() as u64),
            );
        }
    }
    (elapsed_us(started), black_box(checksum))
}

fn measure_compiled_target_lookup(
    program: &zircon_runtime_interface::ui::template::UiCompiledBindingProgram,
    handle: zircon_runtime_interface::ui::template::UiCompiledBindingHandle,
    repeats: usize,
) -> (u64, u64) {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..repeats {
        let binding = black_box(program).binding(black_box(handle)).unwrap();
        for target in black_box(&binding.targets) {
            checksum = checksum.wrapping_add(match &target.expression {
                UiCompiledBindingExpression::Literal(_) => 1,
                UiCompiledBindingExpression::Property(property) => {
                    program.property_name(*property).unwrap().len() as u64 + 2
                }
                UiCompiledBindingExpression::Equals(_, _) => 3,
                UiCompiledBindingExpression::NotEquals(_, _) => 5,
                _ => 7,
            });
            checksum = checksum.wrapping_add(
                target
                    .property
                    .and_then(|property| program.property_name(property))
                    .map_or(1, |name| name.len() as u64),
            );
        }
    }
    (elapsed_us(started), black_box(checksum))
}

fn elapsed_us(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn nearest_rank_p95(samples: &[u64]) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

fn join_samples(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
