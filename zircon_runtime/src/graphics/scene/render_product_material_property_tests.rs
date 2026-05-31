use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager, ShaderAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialManagementIssueKind,
    RenderMaterialManagementPageRequest, RenderMaterialManagementQuery,
    RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
    RenderMaterialManagementSortOrder, RenderMaterialPropertyValue, RenderMaterialReadinessStatus,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

#[test]
fn render_product_material_properties_prepare_uniform_payload() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform.zshader"),
        )
        .expect("shader insert");
    let mut material = material_with_shader("res://shaders/runtime-property-uniform.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property values prepare");

    let material = streamer.material(&material_id).expect("runtime material");
    let payload = &material.shader_property_uniform_payload;
    assert_eq!(payload.layout.len(), 3);
    assert_eq!(payload.layout[0].name, "custom_gain");
    assert_eq!(payload.layout[0].offset, 0);
    assert_eq!(f32_at(&payload.bytes, 0), 2.5);
    assert_eq!(payload.layout[1].name, "rim_color");
    assert_eq!(payload.layout[1].offset, 16);
    assert_eq!(f32_at(&payload.bytes, 16), 0.25);
    assert_eq!(f32_at(&payload.bytes, 20), 0.5);
    assert_eq!(f32_at(&payload.bytes, 24), 0.75);
    assert_eq!(f32_at(&payload.bytes, 28), 1.0);
    assert_eq!(payload.layout[2].name, "use_rim");
    assert_eq!(payload.layout[2].offset, 32);
    assert_eq!(u32_at(&payload.bytes, 32), 1);
    assert_eq!(payload.bytes.len(), 48);
    assert_eq!(payload.unsupported.len(), 1);
    assert_eq!(payload.unsupported[0].name, "debug_label");
    assert_eq!(
        streamer.material_uniform_payload_byte_len(&material_id),
        Some(48)
    );
    assert_eq!(
        streamer.material_uniform_buffer_byte_len(&material_id),
        Some(64)
    );
}

#[test]
fn render_product_streamer_exposes_material_uniform_debug_counts() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-counts.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-counts.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform-counts.zshader"),
        )
        .expect("shader insert");
    let mut material =
        material_with_shader("res://shaders/runtime-property-uniform-counts.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    assert_eq!(streamer.material_uniform_field_count(&material_id), None);
    assert_eq!(
        streamer.material_uniform_unsupported_count(&material_id),
        None
    );
    assert_eq!(streamer.material_uniform_summary(&material_id), None);
    assert_eq!(streamer.material_uniform_fields(&material_id), None);
    assert_eq!(streamer.material_uniform_unsupported(&material_id), None);
    assert_eq!(streamer.material_property_value_summary(&material_id), None);
    assert_eq!(streamer.material_property_value_states(&material_id), None);
    assert_eq!(streamer.material_readiness_status(&material_id), None);
    assert_eq!(streamer.material_issue_state(&material_id), None);
    assert_eq!(streamer.material_management_snapshot(&material_id), None);
    assert_eq!(streamer.material_management_record(&material_id), None);
    assert!(streamer.material_management_records().is_empty());
    let empty_record_set = streamer.material_management_record_set();
    assert!(empty_record_set.is_empty());
    assert_eq!(empty_record_set.len(), 0);
    assert_eq!(
        empty_record_set.summary.status,
        RenderMaterialReadinessStatus::Ready
    );
    assert_eq!(empty_record_set.summary.total_count, 0);
    assert_eq!(empty_record_set.summary.degraded_count(), 0);
    let empty_overview = streamer.material_management_overview();
    assert!(empty_overview.is_empty());
    assert_eq!(empty_overview.len(), 0);
    assert_eq!(empty_overview.summary, empty_record_set.summary);
    assert_eq!(
        empty_overview.summary.status,
        RenderMaterialReadinessStatus::Ready
    );
    assert!(empty_overview.status_index.is_empty());
    assert_eq!(empty_overview.status_index.total_count(), 0);
    assert_eq!(empty_overview.status_index.degraded_count(), 0);
    let empty_status_index = streamer.material_management_status_index();
    assert!(empty_status_index.is_empty());
    assert_eq!(empty_status_index, empty_record_set.status_index);
    let empty_issue_index = streamer.material_management_issue_index();
    assert!(empty_issue_index.is_empty());
    assert_eq!(empty_issue_index, empty_record_set.issue_index);
    let empty_diagnostic_issue_view =
        streamer.material_management_issue_view(RenderMaterialManagementIssueKind::Diagnostic);
    assert!(empty_diagnostic_issue_view.is_empty());
    assert_eq!(empty_diagnostic_issue_view.len(), 0);
    assert_eq!(
        empty_diagnostic_issue_view.issue_kind,
        RenderMaterialManagementIssueKind::Diagnostic
    );
    assert!(empty_diagnostic_issue_view.material_ids.is_empty());
    assert!(empty_diagnostic_issue_view.records.is_empty());
    let empty_diagnostic_status_view =
        streamer.material_management_status_view(RenderMaterialReadinessStatus::Diagnostic);
    assert!(empty_diagnostic_status_view.is_empty());
    assert_eq!(empty_diagnostic_status_view.len(), 0);
    assert_eq!(
        empty_diagnostic_status_view.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert!(empty_diagnostic_status_view.material_ids.is_empty());
    assert!(empty_diagnostic_status_view.records.is_empty());
    let empty_query = streamer.material_management_query(
        RenderMaterialManagementQuery::new()
            .with_status(RenderMaterialReadinessStatus::Diagnostic)
            .with_text_filter("runtime"),
    );
    assert_eq!(empty_query.summary.total_count, 0);
    assert_eq!(empty_query.page.total_count, 0);
    assert_eq!(empty_query.page.returned_count, 0);
    assert!(!empty_query.page.has_previous_page);
    assert!(!empty_query.page.has_next_page);
    assert!(empty_query.issue_index.is_empty());
    assert!(empty_query.records.is_empty());
    assert_eq!(
        streamer.material_management_record_summary().status,
        RenderMaterialReadinessStatus::Ready
    );
    assert_eq!(streamer.material_management_record_summary().total_count, 0);
    assert_eq!(
        streamer
            .material_management_record_summary()
            .degraded_count(),
        0
    );
    assert_eq!(streamer.material_prepared_state(&material_id), None);
    assert!(streamer.material(&material_id).is_none());

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property debug counts prepare");

    assert_eq!(streamer.material_uniform_field_count(&material_id), Some(3));
    assert_eq!(
        streamer.material_uniform_unsupported_count(&material_id),
        Some(1)
    );
    let summary = streamer
        .material_uniform_summary(&material_id)
        .expect("material uniform summary");
    assert_eq!(summary.payload_byte_len, 48);
    assert_eq!(summary.field_count, 3);
    assert_eq!(summary.unsupported_count, 1);
    let report_summary = streamer
        .material_readiness_report(&material_id)
        .and_then(|report| report.uniform_summary)
        .expect("material readiness uniform summary");
    assert_eq!(report_summary, summary);
    let uniform_fields = streamer
        .material_uniform_fields(&material_id)
        .expect("material uniform fields");
    assert_eq!(
        uniform_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        vec!["custom_gain", "rim_color", "use_rim"]
    );
    assert_eq!(uniform_fields[0].kind.as_str(), "float");
    assert_eq!(uniform_fields[0].offset, 0);
    assert_eq!(uniform_fields[1].kind.as_str(), "vec4");
    assert_eq!(uniform_fields[1].offset, 16);
    assert_eq!(uniform_fields[2].kind.as_str(), "bool");
    assert_eq!(uniform_fields[2].offset, 32);
    let uniform_unsupported = streamer
        .material_uniform_unsupported(&material_id)
        .expect("material uniform unsupported entries");
    assert_eq!(uniform_unsupported.len(), 1);
    assert_eq!(uniform_unsupported[0].name, "debug_label");
    let report = streamer
        .material_readiness_report(&material_id)
        .expect("material readiness report");
    assert_eq!(report.status(), RenderMaterialReadinessStatus::Diagnostic);
    assert_eq!(report.uniform_fields, uniform_fields);
    assert_eq!(report.uniform_unsupported, uniform_unsupported);
    let value_summary = streamer
        .material_property_value_summary(&material_id)
        .expect("material value summary");
    let report_value_summary = streamer
        .material_readiness_report(&material_id)
        .and_then(|report| report.property_value_summary)
        .expect("material readiness value summary");
    assert_eq!(report_value_summary, value_summary);
    let value_states = streamer
        .material_property_value_states(&material_id)
        .expect("material value states");
    assert_eq!(
        value_states
            .iter()
            .map(|state| state.name.as_str())
            .collect::<Vec<_>>(),
        vec!["custom_gain", "debug_label", "rim_color", "use_rim"]
    );
    assert_eq!(
        value_states[0].value,
        RenderMaterialPropertyValue::Float { value: 2.5 }
    );
    assert!(value_states[0].is_uniform_eligible());
    assert_eq!(
        value_states[1].value,
        RenderMaterialPropertyValue::String {
            value: "author-only".to_string()
        }
    );
    assert!(!value_states[1].is_uniform_eligible());
    assert_eq!(
        value_states[2].value,
        RenderMaterialPropertyValue::Vec4 {
            value: [0.25, 0.5, 0.75, 1.0]
        }
    );
    assert!(value_states[2].is_uniform_eligible());
    assert_eq!(
        value_states[3].value,
        RenderMaterialPropertyValue::Bool { value: true }
    );
    assert!(value_states[3].is_uniform_eligible());
    assert_eq!(report.property_value_states, value_states);
    let issue_state = streamer
        .material_issue_state(&material_id)
        .expect("material issue state");
    assert_eq!(
        streamer.material_readiness_status(&material_id),
        Some(RenderMaterialReadinessStatus::Diagnostic)
    );
    assert_eq!(issue_state, report.issue_state());
    assert_eq!(
        issue_state.status(),
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert!(issue_state.is_ready());
    assert!(!issue_state.uses_fallback());
    assert!(issue_state.has_diagnostics());
    assert!(issue_state.validation_errors.is_empty());
    assert!(issue_state.fallback_usages.is_empty());
    assert_eq!(issue_state.diagnostics, report.diagnostics);
    let management_snapshot = streamer
        .material_management_snapshot(&material_id)
        .expect("material management snapshot");
    assert_eq!(management_snapshot, report.management_snapshot());
    assert_eq!(management_snapshot.summary, report.summary());
    assert_eq!(
        management_snapshot.summary.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert_eq!(management_snapshot.issue_state, issue_state);
    let management_record = streamer
        .material_management_record(&material_id)
        .expect("material management record");
    assert_eq!(management_record.material_id, material_id);
    assert_eq!(
        management_record.material_name.as_deref(),
        report.material_name.as_deref()
    );
    assert_eq!(management_record.snapshot, management_snapshot);
    assert_eq!(
        management_record.status(),
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert!(management_record.is_ready());
    assert_eq!(
        streamer.material_management_records(),
        vec![management_record.clone()]
    );
    let record_set = streamer.material_management_record_set();
    assert_eq!(record_set.records, vec![management_record.clone()]);
    assert_eq!(
        streamer.material_management_record_set_sorted(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Descending,
        )),
        record_set
    );
    assert_eq!(record_set.len(), 1);
    assert_eq!(record_set.status_index.total_count(), 1);
    assert_eq!(record_set.status_index.degraded_count(), 1);
    assert_eq!(record_set.issue_index.validation_errors, Vec::new());
    assert_eq!(record_set.issue_index.fallback_usages, Vec::new());
    assert_eq!(record_set.issue_index.diagnostics, vec![material_id]);
    assert_eq!(record_set.issue_index.bucket_entry_count(), 1);
    assert_eq!(
        streamer.material_management_issue_index(),
        record_set.issue_index
    );
    let diagnostic_issue_view =
        streamer.material_management_issue_view(RenderMaterialManagementIssueKind::Diagnostic);
    assert_eq!(
        streamer.material_management_issue_view_sorted(
            RenderMaterialManagementIssueKind::Diagnostic,
            RenderMaterialManagementSortOrder::new(
                RenderMaterialManagementSortKey::MaterialId,
                RenderMaterialManagementSortDirection::Descending,
            ),
        ),
        diagnostic_issue_view
    );
    assert_eq!(
        diagnostic_issue_view.issue_kind,
        RenderMaterialManagementIssueKind::Diagnostic
    );
    assert_eq!(diagnostic_issue_view.material_ids, vec![material_id]);
    assert_eq!(diagnostic_issue_view.records, record_set.overview().records);
    assert_eq!(diagnostic_issue_view.len(), 1);
    assert!(!diagnostic_issue_view.is_empty());
    assert!(streamer
        .material_management_issue_view(RenderMaterialManagementIssueKind::ValidationError)
        .is_empty());
    assert!(streamer
        .material_management_issue_view(RenderMaterialManagementIssueKind::FallbackUsage)
        .is_empty());
    assert_eq!(
        record_set
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Ready),
        &[]
    );
    assert_eq!(
        record_set
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Diagnostic),
        &[material_id]
    );
    assert_eq!(
        record_set
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Fallback),
        &[]
    );
    assert_eq!(
        record_set
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Invalid),
        &[]
    );
    let overview = streamer.material_management_overview();
    assert_eq!(overview, record_set.overview());
    assert_eq!(
        streamer.material_management_overview_sorted(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::Status,
            RenderMaterialManagementSortDirection::Ascending,
        )),
        overview
    );
    assert_eq!(overview.len(), 1);
    assert_eq!(overview.summary, record_set.summary);
    assert_eq!(overview.status_index, record_set.status_index);
    assert_eq!(overview.issue_index, record_set.issue_index);
    assert_eq!(
        streamer.material_management_status_index(),
        record_set.status_index
    );
    let diagnostic_status_view =
        streamer.material_management_status_view(RenderMaterialReadinessStatus::Diagnostic);
    assert_eq!(
        streamer.material_management_status_view_sorted(
            RenderMaterialReadinessStatus::Diagnostic,
            RenderMaterialManagementSortOrder::new(
                RenderMaterialManagementSortKey::MaterialId,
                RenderMaterialManagementSortDirection::Descending,
            ),
        ),
        diagnostic_status_view
    );
    assert_eq!(
        diagnostic_status_view.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert_eq!(diagnostic_status_view.material_ids, vec![material_id]);
    assert_eq!(diagnostic_status_view.records, overview.records);
    assert_eq!(diagnostic_status_view.len(), 1);
    assert!(!diagnostic_status_view.is_empty());
    assert!(streamer
        .material_management_status_view(RenderMaterialReadinessStatus::Ready)
        .is_empty());
    assert_eq!(
        overview.summary.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert_eq!(overview.records[0].material_id, material_id);
    assert_eq!(
        overview.records[0].material_name.as_deref(),
        report.material_name.as_deref()
    );
    assert_eq!(overview.records[0].summary, report.summary());
    assert_eq!(
        overview.records[0].status(),
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert!(overview.records[0].is_ready());
    let management_query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Diagnostic)
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter(material_id.to_string())
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)));
    let query_result = streamer.material_management_query(management_query.clone());
    assert_eq!(query_result.summary, record_set.summary);
    assert_eq!(query_result.status_index, record_set.status_index);
    assert_eq!(query_result.issue_index, record_set.issue_index);
    assert_eq!(query_result.page.total_count, 1);
    assert_eq!(query_result.page.returned_count, 1);
    assert_eq!(query_result.page.limit, Some(1));
    assert!(!query_result.page.has_previous_page);
    assert!(!query_result.page.has_next_page);
    assert_eq!(query_result.records, overview.records);
    let query_selection = streamer.material_management_query_selection(management_query.clone());
    assert_eq!(query_selection.query, management_query);
    assert_eq!(query_selection.query_result, query_result);
    assert_eq!(
        query_selection.selection.records,
        vec![management_record.clone()]
    );
    assert_eq!(query_selection.selection.requested_count, 1);
    assert!(query_selection.selection.is_complete());
    assert_eq!(query_selection.selection.summary, record_set.summary);
    assert_eq!(
        query_selection.selection.status_index,
        record_set.status_index
    );
    assert_eq!(
        query_selection.selection.issue_index,
        record_set.issue_index
    );
    let impossible_issue_query = streamer.material_management_query(
        RenderMaterialManagementQuery::new()
            .with_issue_kind(RenderMaterialManagementIssueKind::ValidationError)
            .with_text_filter(material_id.to_string()),
    );
    assert_eq!(impossible_issue_query.summary.total_count, 0);
    assert_eq!(impossible_issue_query.page.total_count, 0);
    assert!(impossible_issue_query.records.is_empty());
    assert!(impossible_issue_query.issue_index.is_empty());
    let missing_material_id = ResourceId::from_stable_label("material:missing-selection");
    let selection =
        streamer.material_management_selection([material_id, missing_material_id, material_id]);
    assert_eq!(selection.requested_count, 2);
    assert_eq!(selection.len(), 1);
    assert!(!selection.is_empty());
    assert_eq!(selection.missing_count(), 1);
    assert!(!selection.is_complete());
    assert_eq!(selection.records, vec![management_record.clone()]);
    assert_eq!(selection.summary, record_set.summary);
    assert_eq!(selection.status_index, record_set.status_index);
    assert_eq!(selection.issue_index, record_set.issue_index);
    assert_eq!(selection.missing_material_ids, vec![missing_material_id]);
    let record_summary = streamer.material_management_record_summary();
    assert_eq!(record_set.summary, record_summary);
    assert_eq!(
        record_summary.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert_eq!(record_summary.total_count, 1);
    assert_eq!(record_summary.ready_count, 0);
    assert_eq!(record_summary.diagnostic_count, 1);
    assert_eq!(record_summary.fallback_count, 0);
    assert_eq!(record_summary.invalid_count, 0);
    assert_eq!(record_summary.degraded_count(), 1);
    assert_eq!(record_summary.validation_error_count, 0);
    assert_eq!(record_summary.fallback_usage_count, 0);
    assert_eq!(
        record_summary.diagnostic_row_count,
        report.diagnostics.len()
    );
    assert_eq!(record_summary.issue_row_count(), report.diagnostics.len());
    assert!(record_summary.has_issue_rows());
    let prepared_state = streamer
        .material_prepared_state(&material_id)
        .expect("material prepared state");
    assert_eq!(prepared_state, report.prepared_state());
    assert_eq!(management_snapshot.prepared_state, prepared_state);
    assert_eq!(prepared_state.property_value_summary, Some(value_summary));
    assert_eq!(prepared_state.property_value_states, value_states);
    assert_eq!(prepared_state.uniform_summary, Some(summary));
    assert_eq!(prepared_state.uniform_fields, uniform_fields);
    assert_eq!(prepared_state.uniform_unsupported, uniform_unsupported);
    assert_eq!(
        prepared_state.standard_texture_slot_summary,
        Some(Default::default())
    );
    assert!(prepared_state.standard_texture_slot_states.is_empty());
    assert_eq!(
        prepared_state.texture_slot_summary,
        Some(Default::default())
    );
    assert!(prepared_state.non_standard_texture_slot_states.is_empty());
    assert_eq!(value_summary.total_count, 4);
    assert_eq!(value_summary.uniform_eligible_count(), 3);
    assert_eq!(value_summary.non_uniform_count(), 1);
    assert_eq!(value_summary.float_count, 1);
    assert_eq!(value_summary.bool_count, 1);
    assert_eq!(value_summary.vec4_count, 1);
    assert_eq!(value_summary.string_count, 1);
}

#[test]
fn render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-report.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-report.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_property_schema("res://shaders/runtime-property-uniform-report.zshader"),
        )
        .expect("shader insert");
    let mut material =
        material_with_shader("res://shaders/runtime-property-uniform-report.zshader");
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.5));
    material
        .property_values
        .insert("use_rim".to_string(), toml::Value::Boolean(true));
    material.property_values.insert(
        "debug_label".to_string(),
        toml::Value::String("author-only".to_string()),
    );
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property readiness prepare");

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("material readiness report");
    assert!(report.is_ready());
    assert!(report.validation_errors.is_empty());
    assert!(report.fallback_usages.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialUniform
    );
    assert_eq!(report.diagnostics[0].path, "uniform.debug_label");
    assert_eq!(
        report.diagnostics[0].diagnostic,
        "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type"
    );
}

#[test]
fn render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = locator("res://materials/runtime-property-uniform-default.zmaterial");
    let material_id = ResourceId::from_locator(&material_uri);
    let shader_uri = locator("res://shaders/runtime-property-uniform-default.zshader");
    asset_manager
        .assets::<ShaderAsset>()
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&shader_uri),
                ResourceKind::Shader,
                shader_uri.clone(),
            ),
            shader_with_string_default_schema(
                "res://shaders/runtime-property-uniform-default.zshader",
            ),
        )
        .expect("shader insert");
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material_with_shader("res://shaders/runtime-property-uniform-default.zshader"),
        )
        .expect("material insert");
    let mut streamer =
        ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);

    streamer
        .ensure_material(
            &device,
            &queue,
            &texture_layout,
            ResourceHandle::<MaterialMarker>::new(material_id),
        )
        .expect("shader property default readiness prepare");

    let material = streamer.material(&material_id).expect("runtime material");
    assert_eq!(material.shader_property_uniform_payload.layout.len(), 1);
    assert_eq!(
        material.shader_property_uniform_payload.unsupported[0].name,
        "debug_label"
    );

    let report = streamer
        .material_readiness_report(&material_id)
        .expect("material readiness report");
    assert!(report.is_ready());
    assert!(report.validation_errors.is_empty());
    assert!(report.fallback_usages.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialUniform
    );
    assert_eq!(report.diagnostics[0].path, "uniform.debug_label");
}

fn material_with_shader(shader_uri: &str) -> MaterialAsset {
    MaterialAsset {
        name: None,
        shader: asset_reference(shader_uri),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.5,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn shader_with_property_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", None),
        shader_property(
            "rim_color",
            "vec4",
            Some(toml::Value::Array(vec![
                toml::Value::Float(0.25),
                toml::Value::Float(0.5),
                toml::Value::Float(0.75),
                toml::Value::Float(1.0),
            ])),
        ),
        shader_property("use_rim", "bool", None),
        shader_property("debug_label", "string", None),
    ];
    shader
}

fn shader_with_string_default_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", Some(toml::Value::Float(1.0))),
        shader_property(
            "debug_label",
            "string",
            Some(toml::Value::String("schema-default".to_string())),
        ),
    ];
    shader
}

fn shader_property(
    name: &str,
    kind: &str,
    default: Option<toml::Value>,
) -> ShaderMaterialPropertyAsset {
    ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        required: default.is_none(),
        default,
        editor: Default::default(),
    }
}

fn wgsl_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: "".to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("test texture layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).expect("asset uri")
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
