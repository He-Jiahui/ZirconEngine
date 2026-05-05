use crate::{
    math::{is_finite_vec3, perspective, transform_to_mat4, Transform, UVec2, Vec3},
    resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord},
    ui::{
        binding::{UiEventBinding, UiEventKind, UiEventPath},
        component::{
            UiComponentCategory, UiComponentDescriptor, UiComponentEventKind, UiComponentState,
            UiDragSourceMetadata, UiDropPolicy, UiHostCapability, UiPropSchema, UiRenderCapability,
            UiSlotSchema, UiValue, UiValueKind,
        },
        dispatch::{UiPointerDispatchContext, UiPointerEvent},
        event_ui::{
            UiBindingCodec, UiControlRequest, UiInvocationContext, UiNodeId, UiNodePath,
            UiPropertyInvalidationReason, UiReflectedProperty, UiReflectedPropertySource,
            UiReflectorHitContext, UiReflectorNode, UiReflectorSnapshot, UiTreeId,
            UiWidgetLifecycleState,
        },
        layout::{BoxConstraints, UiFrame, UiPoint},
        surface::{
            UiArrangedNode, UiArrangedTree, UiHitGridDebugStats, UiHitPath, UiHitTestCell,
            UiHitTestEntry, UiHitTestGrid, UiHitTestQuery, UiMaterialBatchDebugStat,
            UiOverdrawDebugStats, UiPointerEventKind, UiRenderCommand, UiRenderCommandKind,
            UiRenderDebugStats, UiRenderExtract, UiRenderList, UiResolvedStyle,
            UiSurfaceDebugOptions, UiSurfaceDebugSnapshot, UiSurfaceFrame, UiTextAlign, UiTextWrap,
            UiVirtualPointerPosition, UiWidgetReflectorNode,
        },
        template::{
            UiActionHostPolicy, UiActionPolicyReport, UiActionSideEffectClass, UiAssetDocument,
            UiAssetFingerprint, UiAssetHeader, UiAssetImports, UiAssetKind, UiAssetMigrationReport,
            UiAssetMigrationStep, UiAssetSchemaDiagnostic, UiAssetSchemaDiagnosticSeverity,
            UiAssetSchemaSourceKind, UiBindingExpression, UiCompileCacheKey,
            UiCompiledAssetDependencyManifest, UiCompiledAssetHeader,
            UiCompiledAssetPackageProfile, UiCompiledAssetPackageSection,
            UiCompiledAssetPackageValidationReport, UiLocalizationReport, UiLocalizedTextRef,
            UiNodeDefinition, UiResourceKind, UiResourceRef, UiSelector, UiSelectorCombinator,
            UiSelectorSegment, UiSelectorToken, UiTemplateDocument, UiTemplateNode,
            UiTextDirection, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION, UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
        },
        tree::{UiDirtyFlags, UiInputPolicy, UiTree, UiTreeNode, UiVisibility},
    },
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV1, ZrRuntimeHostFetchRequestV1, ZrRuntimeSessionHandle,
    ZrRuntimeTranslatedEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1, ZR_RUNTIME_EVENT_KIND_TOUCH_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1, ZR_RUNTIME_FETCH_FLAG_STREAMING_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
};

#[test]
fn math_contract_exposes_shared_transform_and_glam_aliases() {
    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let matrix = transform_to_mat4(transform);

    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert!(is_finite_vec3(transform.forward()));
    assert_eq!(UVec2::new(0, 2).x, 0);
    assert!(matrix
        .w_axis
        .truncate()
        .abs_diff_eq(transform.translation, f32::EPSILON));
    assert!(perspective(1.0, 16.0 / 9.0, 0.1, 100.0).is_finite());
}

#[test]
fn ui_surface_frame_contract_carries_arranged_render_and_hit_state() {
    let node_id = UiNodeId::new(7);
    let arranged = UiArrangedNode {
        node_id,
        node_path: UiNodePath::new("root/button"),
        parent: None,
        children: Vec::new(),
        frame: UiFrame::new(4.0, 8.0, 64.0, 20.0),
        clip_frame: UiFrame::new(0.0, 0.0, 128.0, 64.0),
        z_index: 3,
        paint_order: 9,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Receive,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        clip_to_bounds: false,
        control_id: Some("primary".to_string()),
    };
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("ui.surface"),
        roots: vec![node_id],
        nodes: vec![arranged.clone()],
        draw_order: vec![node_id],
    };
    let hit_grid = UiHitTestGrid {
        bounds: UiFrame::new(4.0, 8.0, 64.0, 20.0),
        cell_size: 64.0,
        columns: 1,
        rows: 1,
        entries: vec![UiHitTestEntry {
            node_id,
            frame: arranged.frame,
            clip_frame: arranged.frame,
            z_index: arranged.z_index,
            paint_order: arranged.paint_order,
            control_id: arranged.control_id.clone(),
        }],
        cells: vec![UiHitTestCell { entries: vec![0] }],
    };
    let frame = UiSurfaceFrame {
        tree_id: UiTreeId::new("ui.surface"),
        arranged_tree,
        render_extract: UiRenderExtract {
            tree_id: UiTreeId::new("ui.surface"),
            list: UiRenderList::default(),
        },
        hit_grid,
        focus_state: Default::default(),
        last_rebuild: Default::default(),
    };
    let hit_path = UiHitPath {
        target: Some(node_id),
        root_to_leaf: vec![node_id],
        bubble_route: vec![node_id],
        virtual_pointer: None,
    };
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(12.0, 6.0), UiPoint::new(10.0, 4.0));
    let query = UiHitTestQuery::new(UiPoint::new(300.0, 200.0))
        .with_cursor_radius(4.0)
        .with_virtual_pointer(virtual_pointer);

    assert!(!UiVisibility::Collapsed.occupies_layout());
    assert!(UiVisibility::SelfHitTestInvisible.allows_child_hit_test());
    assert!(!UiVisibility::HitTestInvisible.allows_child_hit_test());
    assert_eq!(frame.arranged_tree.get(node_id), Some(&arranged));
    assert_eq!(
        frame.hit_grid.entries[0].control_id.as_deref(),
        Some("primary")
    );
    assert_eq!(hit_path.target, Some(node_id));
    assert_eq!(query.hit_point(), UiPoint::new(12.0, 6.0));
    assert_eq!(query.sanitized_cursor_radius(), 4.0);
}

#[test]
fn ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats() {
    let node_id = UiNodeId::new(9);
    let snapshot = UiSurfaceDebugSnapshot {
        tree_id: UiTreeId::new("ui.surface.debug"),
        roots: vec![node_id],
        nodes: vec![UiWidgetReflectorNode {
            node_id,
            node_path: UiNodePath::new("root/debug_button"),
            parent: None,
            children: Vec::new(),
            frame: UiFrame::new(8.0, 8.0, 48.0, 20.0),
            clip_frame: UiFrame::new(0.0, 0.0, 64.0, 32.0),
            z_index: 2,
            paint_order: 3,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            control_id: Some("debug.button".to_string()),
            render_command_count: 1,
            hit_entry_count: 1,
            hit_cell_count: 2,
        }],
        rebuild: Default::default(),
        render: UiRenderDebugStats {
            command_count: 1,
            quad_count: 1,
            material_batch_count: 1,
            estimated_draw_calls: 1,
            material_batches: vec![UiMaterialBatchDebugStat {
                key: "kind=Quad;bg=#224466".to_string(),
                break_reason: "same material".to_string(),
                command_kind: UiRenderCommandKind::Quad,
                command_count: 1,
                clipped_command_count: 0,
                node_ids: vec![node_id],
            }],
            ..UiRenderDebugStats::default()
        },
        hit_test: UiHitGridDebugStats {
            entry_count: 1,
            cell_count: 2,
            occupied_cell_count: 2,
            max_entries_per_cell: 1,
            average_entries_per_occupied_cell: 1.0,
        },
        overdraw: UiOverdrawDebugStats {
            sample_cell_size: UiSurfaceDebugOptions::default().overdraw_sample_cell_size,
            bounds: UiFrame::new(8.0, 8.0, 48.0, 20.0),
            columns: 2,
            rows: 1,
            covered_cells: 2,
            overdrawn_cells: 0,
            max_layers: 1,
            total_layer_samples: 2,
        },
        focus_state: Default::default(),
    };

    let serialized = serde_json::to_string(&snapshot).unwrap();

    assert!(serialized.contains("material_batches"));
    assert!(serialized.contains("debug.button"));
    assert_eq!(
        snapshot.nodes[0].node_path,
        UiNodePath::new("root/debug_button")
    );
}

#[test]
fn ui_visibility_contract_separates_layout_render_and_hit_policy() {
    assert!(UiVisibility::Visible.occupies_layout());
    assert!(UiVisibility::Visible.is_render_visible());
    assert!(UiVisibility::Visible.allows_self_hit_test());
    assert!(UiVisibility::Visible.allows_child_hit_test());

    assert!(UiVisibility::Hidden.occupies_layout());
    assert!(!UiVisibility::Hidden.is_render_visible());
    assert!(!UiVisibility::Hidden.allows_self_hit_test());
    assert!(!UiVisibility::Hidden.allows_child_hit_test());

    assert!(!UiVisibility::Collapsed.occupies_layout());
    assert!(!UiVisibility::Collapsed.is_render_visible());
    assert!(!UiVisibility::Collapsed.allows_self_hit_test());
    assert!(!UiVisibility::Collapsed.allows_child_hit_test());

    assert!(UiVisibility::HitTestInvisible.occupies_layout());
    assert!(UiVisibility::HitTestInvisible.is_render_visible());
    assert!(!UiVisibility::HitTestInvisible.allows_self_hit_test());
    assert!(!UiVisibility::HitTestInvisible.allows_child_hit_test());

    assert!(UiVisibility::SelfHitTestInvisible.occupies_layout());
    assert!(UiVisibility::SelfHitTestInvisible.is_render_visible());
    assert!(!UiVisibility::SelfHitTestInvisible.allows_self_hit_test());
    assert!(UiVisibility::SelfHitTestInvisible.allows_child_hit_test());

    let mut legacy_hidden = UiTreeNode::new(UiNodeId::new(8), UiNodePath::new("root/hidden"));
    legacy_hidden.state_flags.visible = false;
    assert_eq!(legacy_hidden.effective_visibility(), UiVisibility::Hidden);
    assert_eq!(UiVisibility::Visible.effective(false), UiVisibility::Hidden);
    assert_eq!(
        UiVisibility::Collapsed.effective(true),
        UiVisibility::Collapsed
    );
    assert_eq!(
        UiVisibility::Collapsed.effective(false),
        UiVisibility::Collapsed
    );
}

#[test]
fn resource_contract_exposes_stable_identity_and_status_records() {
    let locator = ResourceLocator::parse("res://materials/hero.mat#surface").unwrap();
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Material, locator.clone())
        .with_source_hash("source-hash")
        .with_importer_version(2);

    assert_eq!(locator.to_string(), "res://materials/hero.mat#surface");
    assert_eq!(record.id(), id);
    assert_eq!(record.kind, ResourceKind::Material);
    assert_eq!(record.primary_locator(), &locator);
    assert_eq!(record.source_hash, "source-hash");
    assert_eq!(record.importer_version, 2);
}

#[test]
fn runtime_abi_version_starts_at_v1() {
    assert_eq!(ZIRCON_RUNTIME_ABI_VERSION_V1, 1);
}

#[test]
fn opaque_handles_reserve_zero_as_invalid() {
    assert!(!ZrRuntimeSessionHandle::invalid().is_valid());
    assert!(!ZrRuntimeViewportHandle::invalid().is_valid());
    assert!(ZrRuntimeSessionHandle(7).is_valid());
    assert!(ZrRuntimeViewportHandle(9).is_valid());
}

#[test]
fn byte_slices_can_be_empty_or_static() {
    let empty = ZrByteSlice::empty();
    assert!(empty.is_empty());

    let bytes = ZrByteSlice::from_static(b"runtime");
    assert_eq!(bytes.len, 7);
    assert_eq!(unsafe { bytes.as_slice() }, b"runtime");
}

#[test]
fn owned_buffer_empty_has_no_free_callback() {
    let buffer = ZrOwnedByteBuffer::empty();
    assert!(buffer.is_empty());
    assert!(buffer.free.is_none());
}

#[test]
fn status_preserves_raw_codes_and_diagnostics() {
    let status = ZrStatus::new(
        ZrStatusCode::UnsupportedVersion,
        ZrByteSlice::from_static(b"bad abi"),
    );

    assert!(!status.is_ok());
    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    assert_eq!(unsafe { status.diagnostics.as_slice() }, b"bad abi");
}

#[test]
fn runtime_api_table_records_size_and_version() {
    let api = ZrRuntimeApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    assert_eq!(api.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(api.size_bytes, core::mem::size_of::<ZrRuntimeApiV1>());
    assert!(api.create_session.is_none());
    assert!(api.capture_frame.is_none());
}

#[test]
fn runtime_events_use_fixed_repr_payload_fields() {
    let event = ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(5),
        12.0,
        34.0,
    );

    assert_eq!(event.kind, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1);
    assert_eq!(event.viewport.raw(), 5);
    assert_eq!(event.x, 12.0);
    assert_eq!(event.y, 34.0);
    assert!(event.payload.is_empty());
}

#[test]
fn runtime_abi_events_cover_lifecycle_touch_keyboard_and_canvas_metrics() {
    let lifecycle = ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::invalid(),
        ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    );
    let touch = ZrRuntimeEventV1::touch(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(2),
        42,
        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
        13.0,
        21.0,
    );
    let keyboard = ZrRuntimeEventV1::keyboard(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(2),
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
        65,
        30,
        ZrByteSlice::from_static(b"KeyA"),
    );
    let metrics = ZrRuntimeViewportMetricsV1::new(
        ZrRuntimeViewportSizeV1::new(1280, 720),
        2.0,
        ZrRuntimeViewportSizeV1::new(2560, 1440),
    );

    assert_eq!(lifecycle.kind, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1);
    assert_eq!(lifecycle.state, ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1);
    assert_eq!(touch.kind, ZR_RUNTIME_EVENT_KIND_TOUCH_V1);
    assert_eq!(touch.pointer_id, 42);
    assert_eq!(touch.state, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1);
    assert_eq!(keyboard.kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
    assert_eq!(keyboard.button, ZR_RUNTIME_KEY_ACTION_PRESSED_V1);
    assert_eq!(keyboard.key_code, 65);
    assert_eq!(unsafe { keyboard.payload.as_slice() }, b"KeyA");
    assert_eq!(metrics.logical_size.width, 1280);
    assert_eq!(metrics.device_scale_factor, 2.0);
    assert_eq!(metrics.physical_size.height, 1440);
}

#[test]
fn runtime_host_fetch_request_declares_streaming_resource_contract() {
    let request = ZrRuntimeHostFetchRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrByteSlice::from_static(b"res://assets/zircon-project.toml"),
        ZR_RUNTIME_FETCH_FLAG_STREAMING_V1,
    );

    assert_eq!(request.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(request.flags, ZR_RUNTIME_FETCH_FLAG_STREAMING_V1);
    assert_eq!(
        unsafe { request.uri.as_slice() },
        b"res://assets/zircon-project.toml"
    );
}

#[test]
fn runtime_abi_translated_event_helpers_cover_mobile_and_browser_host_callbacks() {
    let viewport = ZrRuntimeViewportHandle::new(2);
    let metrics = ZrRuntimeViewportMetricsV1::new(
        ZrRuntimeViewportSizeV1::new(640, 360),
        2.0,
        ZrRuntimeViewportSizeV1::new(1280, 720),
    );
    let resize = ZrRuntimeTranslatedEventV1::viewport_metrics(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        metrics,
    );
    let touch = ZrRuntimeTranslatedEventV1::touch_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        9,
        10.0,
        20.0,
    );
    let keyboard = ZrRuntimeTranslatedEventV1::keyboard_text(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        ZrByteSlice::from_static(b"A"),
    );

    assert_eq!(resize.event.kind, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1);
    assert_eq!(resize.event.size.width, 1280);
    assert_eq!(resize.event.metrics.device_scale_factor, 2.0);
    assert_eq!(
        unsafe { resize.host_reason.as_slice() },
        b"viewport_metrics"
    );
    assert_eq!(touch.event.kind, ZR_RUNTIME_EVENT_KIND_TOUCH_V1);
    assert_eq!(touch.event.pointer_id, 9);
    assert_eq!(touch.event.state, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1);
    assert_eq!(keyboard.event.kind, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1);
    assert_eq!(keyboard.event.button, crate::ZR_RUNTIME_KEY_ACTION_TEXT_V1);
    assert_eq!(unsafe { keyboard.event.payload.as_slice() }, b"A");
}

#[test]
fn runtime_frame_request_and_frame_carry_size_and_owned_rgba() {
    let request = ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(7),
        ZrRuntimeViewportSizeV1::new(640, 360),
    );
    let frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    assert_eq!(request.viewport.raw(), 7);
    assert_eq!(request.size.width, 640);
    assert_eq!(request.size.height, 360);
    assert!(frame.is_empty());
}

#[test]
fn ui_mod_is_not_runtime_source_path_inclusion() {
    let ui_mod_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/mod.rs");
    let ui_mod = std::fs::read_to_string(ui_mod_path).expect("read interface ui root");

    assert!(!ui_mod.contains("#[path ="));
    assert!(!ui_mod.contains("zircon_runtime/src/ui"));
}

#[test]
fn ui_reflector_contract_carries_lifecycle_property_sources_and_dirty_reasons() {
    let node_id = UiNodeId::new(42);
    let property_dirty = UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        style: false,
        text: true,
        input: false,
        visible_range: false,
    };
    let label_property = UiReflectedProperty::new(
        "label",
        UiReflectedPropertySource::Authored,
        UiValue::String("Launch".to_string()),
    )
    .writable(true)
    .authored_value(UiValue::String("Launch".to_string()))
    .descriptor_default_value(UiValue::String("Button".to_string()))
    .invalidation(UiPropertyInvalidationReason::with_dirty(property_dirty));
    let visibility_property = UiReflectedProperty::new(
        "visibility",
        UiReflectedPropertySource::RuntimeState,
        UiValue::Enum("Visible".to_string()),
    )
    .writable(true)
    .visibility_affecting(true)
    .invalidation(UiPropertyInvalidationReason::with_dirty(UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        style: false,
        text: false,
        input: true,
        visible_range: false,
    }));
    let node = UiReflectorNode::new(
        node_id,
        UiNodePath::new("root/launch"),
        "button",
        "Launch Button",
    )
    .with_property(label_property)
    .with_property(visibility_property);
    let mut snapshot =
        UiReflectorSnapshot::new(UiTreeId::new("ui.reflector"), vec![node_id], vec![node]);
    snapshot.focused = Some(node_id);
    snapshot.hovered = vec![node_id];
    snapshot.hit_context = Some(UiReflectorHitContext {
        query_point: UiPoint::new(10.0, 12.0),
        hit_target: Some(node_id),
        hit_stack: vec![node_id],
        rejected: vec!["outside clip".to_string()],
    });

    let serialized = serde_json::to_string(&snapshot).unwrap();
    let round_trip: UiReflectorSnapshot = serde_json::from_str(&serialized).unwrap();
    let reflected_node = round_trip.node(node_id).expect("reflected node");
    let label = reflected_node
        .properties
        .get("label")
        .expect("label property");
    let visibility = reflected_node
        .properties
        .get("visibility")
        .expect("visibility property");

    assert_eq!(round_trip.focused, Some(node_id));
    assert_eq!(
        round_trip
            .hit_context
            .as_ref()
            .expect("hit context")
            .hit_target,
        Some(node_id)
    );
    assert_eq!(reflected_node.lifecycle, UiWidgetLifecycleState::Declared);
    assert_eq!(label.source, UiReflectedPropertySource::Authored);
    assert_eq!(label.value_kind, UiValueKind::String);
    assert_eq!(
        label.descriptor_default_value,
        Some(UiValue::String("Button".to_string()))
    );
    assert!(label.invalidation.dirty.layout);
    assert!(label.invalidation.any());
    assert!(visibility.visibility_affecting);
    assert!(visibility.invalidation.dirty.hit_test);
}

#[test]
fn ui_binding_and_event_contracts_construct_and_serialize() {
    let path = UiEventPath::new("panel", "submit", UiEventKind::Click);
    let binding = UiEventBinding::new(
        path.clone(),
        crate::ui::binding::UiBindingCall::new("submit_form"),
    );
    let context = UiInvocationContext {
        route_id: crate::ui::event_ui::UiRouteId::new(1),
        binding: binding.clone(),
        arguments: Vec::new(),
        source: crate::ui::event_ui::UiInvocationSource::Binding,
    };
    let request = UiControlRequest::InvokeBinding {
        binding: binding.clone(),
    };

    assert_eq!(binding.path.event_kind, UiEventKind::Click);
    assert_eq!(path.native_prefix(), "panel/submit:onClick");
    assert!(UiBindingCodec::format(&binding).contains("submit_form"));
    assert!(serde_json::to_string(&request)
        .unwrap()
        .contains("InvokeBinding"));
    assert_eq!(context.route_id, crate::ui::event_ui::UiRouteId::new(1));
}

#[test]
fn ui_layout_surface_dispatch_and_tree_contracts_construct_and_serialize() {
    let tree_id = crate::ui::event_ui::UiTreeId::new("main-tree");
    let node_id = crate::ui::event_ui::UiNodeId::new(7);
    let frame = UiFrame::new(0.0, 0.0, 320.0, 180.0);
    let editable = crate::ui::surface::UiEditableTextState {
        text: "hello".to_string(),
        caret: crate::ui::surface::UiTextCaret {
            offset: 5,
            affinity: crate::ui::surface::UiTextCaretAffinity::Downstream,
        },
        selection: Some(crate::ui::surface::UiTextSelection {
            anchor: 0,
            focus: 5,
        }),
        composition: Some(crate::ui::surface::UiTextComposition {
            range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
            text: "hello".to_string(),
        }),
        read_only: false,
    };
    let command = UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: Some(frame),
        z_index: 1,
        style: UiResolvedStyle {
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::Word,
            text_direction: crate::ui::surface::UiTextDirection::Auto,
            text_overflow: crate::ui::surface::UiTextOverflow::Ellipsis,
            rich_text: true,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(crate::ui::surface::UiResolvedTextLayout {
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::Word,
            direction: crate::ui::surface::UiTextDirection::LeftToRight,
            overflow: crate::ui::surface::UiTextOverflow::Ellipsis,
            font_size: 16.0,
            line_height: 20.0,
            measured_width: 40.0,
            measured_height: 20.0,
            source_range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
            lines: vec![crate::ui::surface::UiResolvedTextLine {
                text: "hello".to_string(),
                frame,
                source_range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
                visual_range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
                measured_width: 40.0,
                baseline: 12.0,
                direction: crate::ui::surface::UiTextDirection::LeftToRight,
                runs: vec![crate::ui::surface::UiResolvedTextRun {
                    kind: crate::ui::surface::UiTextRunKind::Plain,
                    text: "hello".to_string(),
                    source_range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
                    visual_range: crate::ui::surface::UiTextRange { start: 0, end: 5 },
                    direction: crate::ui::surface::UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            }],
            overflow_clipped: false,
            editable: Some(editable),
        }),
        text: Some("hello".to_string()),
        image: None,
        opacity: 1.0,
    };
    let extract = UiRenderExtract {
        tree_id: tree_id.clone(),
        list: UiRenderList {
            commands: vec![command],
        },
    };
    let event = UiPointerEvent {
        kind: UiPointerEventKind::Move,
        button: None,
        point: UiPoint { x: 1.0, y: 2.0 },
        scroll_delta: 0.0,
    };
    let context = UiPointerDispatchContext {
        node_id,
        route: crate::ui::surface::UiPointerRoute {
            kind: event.kind,
            button: event.button,
            activation_phase: crate::ui::surface::UiPointerActivationPhase::Hover,
            point: event.point,
            scroll_delta: event.scroll_delta,
            target: Some(node_id),
            hit_path: crate::ui::surface::UiHitPath::default(),
            bubbled: Vec::new(),
            stacked: vec![node_id],
            entered: Vec::new(),
            left: Vec::new(),
            captured: None,
            pressed: None,
            click_target: None,
            release_inside_pressed: false,
            focused: None,
            fallback_to_root: false,
            root_targets: vec![node_id],
        },
    };
    let node = UiTreeNode::new(node_id, crate::ui::event_ui::UiNodePath::new("root"))
        .with_frame(frame)
        .with_input_policy(UiInputPolicy::Receive)
        .with_constraints(BoxConstraints::default())
        .with_z_index(1);
    let mut tree = UiTree {
        tree_id: tree_id.clone(),
        roots: vec![node_id],
        nodes: Default::default(),
    };
    let _ = tree.nodes.insert(node_id, node);

    assert_eq!(extract.list.commands.len(), 1);
    assert_eq!(context.route.point.x, 1.0);
    let pointer_result = crate::ui::dispatch::UiPointerDispatchResult::new(context.route.clone());
    assert!(pointer_result.diagnostics.pointer_routed);
    assert!(pointer_result.diagnostics.ignored_same_target_hover);
    assert_eq!(tree.roots, vec![node_id]);
    assert!(serde_json::to_string(&extract)
        .unwrap()
        .contains("commands"));
}

#[test]
fn ui_component_state_with_value_clears_reference_source_metadata() {
    let source = UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    );
    let mut state = UiComponentState::new();
    state.reference_sources.insert("value".to_string(), source);

    let state = state.with_value(
        "value",
        UiValue::AssetRef("res://textures/overridden.png".to_string()),
    );

    assert_eq!(state.reference_source("value"), None);
    assert_eq!(
        state.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/overridden.png".to_string()
        ))
    );
}

#[test]
fn ui_component_template_policy_localization_and_package_contracts_construct() {
    let descriptor =
        UiComponentDescriptor::new("button", "Button", UiComponentCategory::Input, "control")
            .with_prop(UiPropSchema {
                name: "label".to_string(),
                value_kind: UiValueKind::String,
                required: true,
                default_value: Some(UiValue::String("OK".to_string())),
                options: Vec::new(),
                min: None,
                max: None,
                step: None,
            })
            .slot(UiSlotSchema {
                name: "content".to_string(),
                required: false,
                multiple: true,
            })
            .event(UiComponentEventKind::Press)
            .drop_policy(UiDropPolicy::default())
            .requires_host_capability(UiHostCapability::PointerInput)
            .requires_render_capability(UiRenderCapability::Text);
    let expression = UiBindingExpression::parse("\"label\"").unwrap();
    let template = UiTemplateDocument {
        version: 1,
        components: Default::default(),
        root: UiTemplateNode {
            component: Some("button".to_string()),
            ..UiTemplateNode::default()
        },
    };
    let asset = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "ui/main".to_string(),
            version: 1,
            display_name: "Main".to_string(),
        },
        imports: UiAssetImports {
            resources: vec![UiResourceRef {
                uri: "res://fonts/body.ttf".to_string(),
                kind: UiResourceKind::Font,
                fallback: Default::default(),
            }],
            ..UiAssetImports::default()
        },
        tokens: Default::default(),
        root: Some(UiNodeDefinition {
            node_id: "root".to_string(),
            ..UiNodeDefinition::default()
        }),
        components: Default::default(),
        stylesheets: Default::default(),
    };
    let policy = UiActionHostPolicy::runtime_default();
    let text_ref = UiLocalizedTextRef {
        key: "menu.file".to_string(),
        table: None,
        fallback: Some("File".to_string()),
    };
    let package_report = UiCompiledAssetPackageValidationReport {
        profile: UiCompiledAssetPackageProfile::Runtime,
        header: UiCompiledAssetHeader {
            asset: asset.asset.clone(),
            source_schema_version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            compiler_schema_version: UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
            package_schema_version: UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
            descriptor_registry_revision: 1,
            component_contract_revision: UiAssetFingerprint::from_bytes(b"component"),
            root_document_fingerprint: UiAssetFingerprint::from_bytes(b"root"),
            compile_cache_key: UiCompileCacheKey {
                root_document: UiAssetFingerprint::from_bytes(b"root"),
                widget_imports: Default::default(),
                style_imports: Default::default(),
                descriptor_registry_revision: 1,
                component_contract_revision: UiAssetFingerprint::from_bytes(b"component"),
                resource_dependencies_revision: UiAssetFingerprint::from_bytes(b"resource"),
            },
        },
        dependencies: UiCompiledAssetDependencyManifest::default(),
        retained_sections: vec![UiCompiledAssetPackageSection::RuntimeTemplateTree],
        stripped_sections: vec![UiCompiledAssetPackageSection::AuthoringDiagnostics],
        invalidation_report: Default::default(),
        action_policy_report: UiActionPolicyReport::default(),
        localization_report: UiLocalizationReport::default(),
    };

    assert_eq!(descriptor.id, "button");
    assert_eq!(
        expression,
        UiBindingExpression::Literal(UiValue::String("label".to_string()))
    );
    assert_eq!(template.root.node_kind_count(), 1);
    assert_eq!(asset.root_node_id(), Some("root"));
    assert!(policy.allows(UiActionSideEffectClass::LocalUi));
    assert_eq!(
        UiActionSideEffectClass::infer(None, Some("save_asset")),
        UiActionSideEffectClass::AssetIo
    );
    assert!(text_ref.validate("root.label").is_none());
    assert_eq!(UiTextDirection::Auto, UiTextDirection::Auto);
    assert_eq!(package_report.retained_sections.len(), 1);
}

#[test]
fn ui_selector_contracts_parse_reject_trailing_child_and_serialize() {
    let selector = UiSelector::parse("Button.primary > Label:part(text)").unwrap();
    let serialized = serde_json::to_string(&selector).unwrap();
    let round_trip: UiSelector = serde_json::from_str(&serialized).unwrap();

    assert_eq!(round_trip, selector);
    assert_eq!(selector.segments.len(), 2);
    assert_eq!(
        selector.segments[0],
        UiSelectorSegment {
            combinator: None,
            tokens: vec![
                UiSelectorToken::Type("Button".to_string()),
                UiSelectorToken::Class("primary".to_string())
            ],
        }
    );
    assert_eq!(
        selector.segments[1].combinator,
        Some(UiSelectorCombinator::Child)
    );
    assert!(selector.segments[0]
        .tokens
        .contains(&UiSelectorToken::Class("primary".to_string())));
    assert!(matches!(
        UiSelector::parse("Button >"),
        Err(crate::ui::template::UiAssetError::InvalidSelector(_))
    ));
}

#[test]
fn ui_schema_report_contracts_serialize() {
    let mut report = UiAssetMigrationReport::new(UiAssetSchemaSourceKind::OlderTree, Some(1));
    report.push_step(UiAssetMigrationStep::SourceVersionBumped { from: 1, to: 2 });
    report.push_diagnostic(UiAssetSchemaDiagnostic {
        severity: UiAssetSchemaDiagnosticSeverity::Warning,
        code: "schema.bump".to_string(),
        message: "source schema version was upgraded".to_string(),
    });

    let serialized = serde_json::to_string(&report).unwrap();
    let round_trip: UiAssetMigrationReport = serde_json::from_str(&serialized).unwrap();

    assert_eq!(round_trip, report);
    assert_eq!(round_trip.source_kind, UiAssetSchemaSourceKind::OlderTree);
    assert_eq!(
        round_trip.diagnostics[0].severity,
        UiAssetSchemaDiagnosticSeverity::Warning
    );
}
