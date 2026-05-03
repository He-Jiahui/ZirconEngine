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
        event_ui::{UiBindingCodec, UiControlRequest, UiInvocationContext},
        layout::{BoxConstraints, UiFrame, UiPoint},
        surface::{
            UiPointerEventKind, UiRenderCommand, UiRenderCommandKind, UiRenderExtract,
            UiRenderList, UiResolvedStyle, UiTextAlign, UiTextWrap,
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
        tree::{UiInputPolicy, UiTree, UiTreeNode},
    },
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV1, ZrRuntimeHostFetchRequestV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_FETCH_FLAG_STREAMING_V1,
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
    let command = UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: Some(frame),
        z_index: 1,
        style: UiResolvedStyle {
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::Word,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
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
            point: event.point,
            scroll_delta: event.scroll_delta,
            target: Some(node_id),
            bubbled: Vec::new(),
            stacked: vec![node_id],
            entered: Vec::new(),
            left: Vec::new(),
            captured: None,
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
