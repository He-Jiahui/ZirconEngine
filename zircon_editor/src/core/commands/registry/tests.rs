use std::sync::Arc;
use std::time::Instant;

use crate::core::asset::AssetWriteAccess;
use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::EditorI18nService;

use super::super::{
    CommandEvalCtx, EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandExecutionContract, EditorCommandPaletteMru, EditorCommandResourceBudget,
    EditorCommandResultCodecId, WhenClause,
};
use super::{EditorCommandRegistry, EditorCommandRegistryError};

#[test]
fn command_descriptor_validation_streams_path_segments() {
    let source = include_str!("../registry.rs");
    let menu_collect = ["split('/')", ".collect::<Vec<_>>()"].concat();
    let schema_collect = ["split('.')", ".collect::<Vec<_>>()"].concat();
    assert!(!source.contains(&menu_collect));
    assert!(!source.contains(&schema_collect));
}

#[test]
fn command_registry_preserves_execution_contract_without_materializing_executor() {
    let contract = EditorCommandExecutionContract::new(
        EditorCommandResultCodecId::parse("zircon.editor.command-result.v1")
            .expect("versioned result codec id should be valid"),
        EditorCommandResourceBudget::new(4096, 8192, 250)
            .expect("bounded command resource budget should be valid"),
    );
    let descriptor = EditorCommandDescriptor::operation(
        EditorOperationPath::parse("test.command.execution_contract")
            .expect("test command id should be valid"),
    )
    .with_execution_contract(contract.clone());

    let encoded = serde_json::to_vec(&descriptor).expect("descriptor should serialize");
    let decoded: EditorCommandDescriptor =
        serde_json::from_slice(&encoded).expect("descriptor should deserialize");
    assert_eq!(decoded.execution_contract(), Some(&contract));

    let registry = EditorCommandRegistry::new(vec![decoded]).expect("descriptor should register");
    assert!(registry
        .operation_factory(&EditorOperationPath::parse("test.command.execution_contract").unwrap())
        .is_none());
}

#[test]
fn native_command_requires_contract_before_executor_admission() {
    let command_id = EditorOperationPath::parse("test.command.native_endpoint").unwrap();
    let descriptor = EditorCommandDescriptor::native(command_id.clone());

    assert!(matches!(
        EditorCommandRegistry::new(vec![descriptor]),
        Err(EditorCommandRegistryError::InvalidExecutionContract { command_id: id, .. })
            if id == command_id
    ));

    let contract = EditorCommandExecutionContract::new(
        EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
        EditorCommandResourceBudget::new(1024, 4096, 1000).unwrap(),
    );
    let registry =
        EditorCommandRegistry::new(vec![
            EditorCommandDescriptor::native(command_id.clone()).with_execution_contract(contract)
        ])
        .expect("native command with a valid contract should register");

    assert!(matches!(
        registry
            .command(&command_id)
            .map(EditorCommandDescriptor::action),
        Some(EditorCommandAction::NativeEndpoint)
    ));
    assert_eq!(registry.native_executor_count(), 0);
}

#[test]
fn native_executor_invocation_reports_missing_admission_without_a_fallback_route() {
    let command_id = EditorOperationPath::parse("test.command.native_missing").unwrap();
    let registry = EditorCommandRegistry::default();

    assert!(matches!(
        registry.invoke_native_executor(&command_id, b"{}"),
        Err(super::super::EditorCommandExecutorRegistryError::MissingExecutor { command_id: id })
            if id == command_id
    ));
}

#[test]
fn headless_commandlets_require_unique_typed_routes() {
    let descriptor = EditorCommandDescriptor::new(
        EditorOperationPath::parse("test.commandlet.plugin_list")
            .expect("test command id should be valid"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessPluginList,
    )
    .with_payload_schema_id("editor.commandlet.plugin-list")
    .with_required_capabilities(["plugin.catalog.read"]);
    let mut registry = EditorCommandRegistry::default();

    assert!(matches!(
        registry.register(descriptor.clone()),
        Err(EditorCommandRegistryError::HeadlessCommandletMissingRoute(
            _
        ))
    ));

    let route = EditorOperationPath::parse("commandlet.route.plugin_list")
        .expect("test commandlet route should be valid");
    let descriptor = descriptor.with_headless_commandlet_route(route.clone());
    assert!(matches!(
        registry.register(descriptor.clone()),
        Err(EditorCommandRegistryError::HeadlessCommandletMissingName(_))
    ));
    let descriptor = descriptor.with_headless_commandlet_name("plugin-list");
    registry
        .register(descriptor.clone())
        .expect("a routed headless commandlet should register");
    assert_eq!(
        registry
            .command_for_headless_commandlet_route(&route)
            .map(EditorCommandDescriptor::id),
        Some(descriptor.id())
    );

    let duplicate = EditorCommandDescriptor::new(
        EditorOperationPath::parse("test.commandlet.plugin_list_copy")
            .expect("test command id should be valid"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessPluginList,
    )
    .with_payload_schema_id("editor.commandlet.plugin-list-copy")
    .with_required_capabilities(["plugin.catalog.read"])
    .with_headless_commandlet_route(route.clone())
    .with_headless_commandlet_name("plugin-list-copy");

    assert!(matches!(
        registry.register(duplicate),
        Err(EditorCommandRegistryError::DuplicateHeadlessCommandletRoute(route_error))
            if route_error == route
    ));

    let duplicate_name = EditorCommandDescriptor::new(
        EditorOperationPath::parse("test.commandlet.plugin_list_name_copy")
            .expect("test command id should be valid"),
        EditorCommandCategory::Command,
        EditorCommandAction::HeadlessPluginList,
    )
    .with_payload_schema_id("editor.commandlet.plugin-list-name-copy")
    .with_required_capabilities(["plugin.catalog.read"])
    .with_headless_commandlet_route(
        EditorOperationPath::parse("commandlet.route.plugin_list_name_copy")
            .expect("test commandlet route should be valid"),
    )
    .with_headless_commandlet_name("plugin-list");

    assert!(matches!(
        registry.register(duplicate_name),
        Err(EditorCommandRegistryError::DuplicateHeadlessCommandletName(name))
            if name == "plugin-list"
    ));
}

#[test]
fn stable_command_catalog_is_shared_until_registry_generation_changes() {
    let mut registry = EditorCommandRegistry::default_workbench();

    let first = registry.command_palette_catalog();
    let second = registry.command_palette_catalog();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.generation(), registry.generation());

    registry
        .register(test_command(10_000))
        .expect("new command should advance the catalog generation");
    let next = registry.command_palette_catalog();

    assert!(!Arc::ptr_eq(&first, &next));
    assert_eq!(next.generation(), first.generation() + 1);
    assert_eq!(next.len(), first.len() + 1);
}

#[test]
fn palette_query_retains_only_the_requested_window_without_truncating_matches() {
    let mut registry = EditorCommandRegistry::default();
    for index in 0..1_000 {
        registry
            .register(test_command(index))
            .expect("generated command ids should be unique");
    }

    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let window = catalog.query_window(
        &i18n,
        &locale,
        &CommandEvalCtx::interactive(),
        "palette command",
        480,
        24,
    );

    assert_eq!(window.total_match_count(), 1_000);
    assert_eq!(window.offset(), 480);
    assert_eq!(window.len(), 24);
    assert_eq!(window.metrics().retained_handles, 24);
    assert_eq!(window.metrics().visited_entries, 1_000);
    assert_eq!(window.metrics().enablement_evaluations, 1_000);
    assert_eq!(window.metrics().candidate_handles, 504);
    assert_eq!(window.metrics().owned_buffers, 4);
    assert_eq!(
        window.entries().next().map(|entry| entry.id.as_str()),
        Some("test.palette.command_0480")
    );
    assert_eq!(
        window.entries().last().map(|entry| entry.id.as_str()),
        Some("test.palette.command_0503")
    );
}

#[test]
fn palette_mru_precedes_catalog_order_and_breaks_fuzzy_score_ties() {
    let mut registry = EditorCommandRegistry::default();
    for index in 0..4 {
        registry
            .register(test_command(index))
            .expect("generated command ids should be unique");
    }
    let mru = EditorCommandPaletteMru::new([
        EditorOperationPath::parse("test.palette.command_0003")
            .expect("the recent command id should be valid"),
        EditorOperationPath::parse("test.palette.command_0001")
            .expect("the recent command id should be valid"),
    ])
    .expect("the bounded MRU list should be valid");
    let context = CommandEvalCtx::interactive();

    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let unfiltered = catalog.query_window_with_mru(&i18n, &locale, &context, "", 0, 4, &mru);
    let fuzzy =
        catalog.query_window_with_mru(&i18n, &locale, &context, "palette command", 0, 4, &mru);
    let expected = vec![
        "test.palette.command_0003",
        "test.palette.command_0001",
        "test.palette.command_0000",
        "test.palette.command_0002",
    ];

    assert_eq!(
        unfiltered
            .entries()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        fuzzy
            .entries()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn palette_query_index_visits_only_documents_with_the_rarest_query_byte() {
    let mut commands = (0..1_000).map(test_command).collect::<Vec<_>>();
    commands.push(EditorCommandDescriptor::new(
        EditorOperationPath::parse("test.palette.unique_zanzibar")
            .expect("unique command id should be valid"),
        EditorCommandCategory::Command,
        EditorCommandAction::Emit(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette,
        )),
    ));
    let registry =
        EditorCommandRegistry::new(commands).expect("generated command ids should be unique");

    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let window = registry.command_palette_catalog().query_window(
        &i18n,
        &locale,
        &CommandEvalCtx::interactive(),
        "zanzibar",
        0,
        12,
    );

    assert_eq!(window.total_match_count(), 1);
    assert_eq!(window.metrics().visited_entries, 1);
    assert_eq!(window.metrics().enablement_evaluations, 1);
    assert_eq!(
        window.entries().next().map(|entry| entry.id.as_str()),
        Some("test.palette.unique_zanzibar")
    );
}

#[test]
fn palette_catalog_enablement_slot_preserves_descriptor_requirements() {
    let descriptor = test_command(0)
        .with_when(WhenClause::SelectionNonEmpty)
        .with_required_capabilities(["palette.execute"])
        .with_asset_write_target_arguments("asset_type", "asset_locator");
    let registry = EditorCommandRegistry::new(vec![descriptor])
        .expect("the descriptor should satisfy the registry contract");
    let selected = CommandEvalCtx::interactive()
        .with_selection_count(1)
        .with_capabilities(["palette.execute"]);

    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    assert!(catalog
        .query_window(&i18n, &locale, &selected, "palette", 0, 16)
        .is_empty());
    assert_eq!(
        catalog
            .query_window(
                &i18n,
                &locale,
                &selected.with_asset_write_access(AssetWriteAccess::Writable),
                "palette",
                0,
                16,
            )
            .total_match_count(),
        1
    );
}

#[test]
fn one_thousand_query_updates_emit_current_source_burst_metrics() {
    let mut registry = EditorCommandRegistry::default();
    for index in 0..1_000 {
        registry
            .register(test_command(index))
            .expect("generated command ids should be unique");
    }
    let context = CommandEvalCtx::interactive();
    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();

    let mut elapsed_micros = Vec::with_capacity(1_000);
    let mut maximum_visited_entries = 0;
    let mut maximum_document_byte_visits = 0;
    let mut maximum_text_comparisons = 0;
    let mut maximum_enablement_evaluations = 0;
    let mut maximum_candidate_handles = 0;
    let mut maximum_retained_handles = 0;
    let mut maximum_owned_buffers = 0;
    for index in 0..1_000 {
        let query = format!("command {:02}", index % 100);
        let started_at = Instant::now();
        let metrics = catalog
            .query_window(&i18n, &locale, &context, &query, 0, 16)
            .metrics();
        elapsed_micros.push(started_at.elapsed().as_micros());
        maximum_visited_entries = maximum_visited_entries.max(metrics.visited_entries);
        maximum_document_byte_visits =
            maximum_document_byte_visits.max(metrics.document_byte_visits);
        maximum_text_comparisons = maximum_text_comparisons.max(metrics.text_comparisons);
        maximum_enablement_evaluations =
            maximum_enablement_evaluations.max(metrics.enablement_evaluations);
        maximum_candidate_handles = maximum_candidate_handles.max(metrics.candidate_handles);
        maximum_retained_handles = maximum_retained_handles.max(metrics.retained_handles);
        maximum_owned_buffers = maximum_owned_buffers.max(metrics.owned_buffers);
    }
    elapsed_micros.sort_unstable();
    let p95_index = elapsed_micros
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    let p95_micros = elapsed_micros[p95_index];

    println!(
        "EDITOR08_PALETTE_QUERY_BURST samples=1000 p95_us={p95_micros} max_visits={maximum_visited_entries} max_document_byte_visits={maximum_document_byte_visits} max_text_comparisons={maximum_text_comparisons} max_enablement_evaluations={maximum_enablement_evaluations} max_candidate_handles={maximum_candidate_handles} max_retained_handles={maximum_retained_handles} max_owned_buffers={maximum_owned_buffers}"
    );
    assert_eq!(maximum_visited_entries, 1_000);
    assert!(maximum_document_byte_visits > 0);
    assert!(maximum_text_comparisons > 0);
    assert_eq!(maximum_enablement_evaluations, 1_000);
    assert!(maximum_candidate_handles <= 16);
    assert!(maximum_retained_handles <= 16);
    assert_eq!(maximum_owned_buffers, 4);
}

fn test_command(index: usize) -> EditorCommandDescriptor {
    EditorCommandDescriptor::new(
        EditorOperationPath::parse(&format!("test.palette.command_{index:04}"))
            .expect("generated command id should be valid"),
        EditorCommandCategory::Command,
        EditorCommandAction::Emit(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette,
        )),
    )
}
