use std::fs;
use std::path::Path;

use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::dispatch::UiPointerEvent;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::ui::v2::UiV2AssetError;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/editor/product_binding_fixture.zui")
}

#[test]
fn product_binding_fixture_executes_param_targets_for_repeated_instances() {
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let outcome = cache
        .load_store([fixture_path()])
        .expect("product binding fixture should compile from disk");
    let primary = outcome
        .compiled
        .arena
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("ProductBindingFixturePrimary"))
        .expect("primary product binding instance");
    let secondary = outcome
        .compiled
        .arena
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("ProductBindingFixtureSecondary"))
        .expect("secondary product binding instance");

    assert_eq!(
        primary.props.get("text").and_then(toml::Value::as_str),
        Some("Primary ready")
    );
    assert_eq!(
        secondary.props.get("text").and_then(toml::Value::as_str),
        Some("Secondary ready")
    );
    assert_eq!(
        primary.events[0].targets[0].expression,
        r#""Primary applied""#
    );
    assert_eq!(
        secondary.events[0].targets[0].expression,
        r#""Secondary applied""#
    );

    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("editor.product_binding_fixture"),
        outcome.root_document.as_ref(),
        outcome.compiled.as_ref(),
    )
    .expect("product binding fixture surface");
    surface
        .compute_layout(UiSize::new(360.0, 96.0))
        .expect("product binding fixture layout");
    let primary_id = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ProductBindingFixturePrimary")
        })
        .expect("primary product binding surface node")
        .node_id;
    let primary_frame = surface
        .arranged_tree
        .get(primary_id)
        .expect("primary product binding frame")
        .frame;
    let point = UiPoint::new(primary_frame.x + 2.0, primary_frame.y + 2.0);
    let dispatcher = zircon_runtime::ui::dispatch::UiPointerDispatcher::default();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("primary product binding pointer down");
    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("primary product binding pointer up");

    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].rejected_count, 0);
    assert_eq!(
        result.component_events[0].binding_id,
        "ProductBindingFixture/Commit"
    );
    assert_eq!(
        result.component_events[0]
            .template_action
            .as_ref()
            .and_then(|action| action.payload.get("label")),
        Some(&UiValue::String("Primary applied".to_string()))
    );
    assert_eq!(
        surface
            .tree
            .node(primary_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("text"))
            .and_then(toml::Value::as_str),
        Some("Primary applied")
    );
}

#[test]
fn product_binding_fixture_reload_and_negative_schema_keep_last_known_good() {
    let temp_root = std::env::temp_dir().join(format!(
        "zircon_editor_product_binding_fixture_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_root);
    let temp_fixture_path = temp_root.join("assets/ui/editor/product_binding_fixture.zui");
    fs::create_dir_all(temp_fixture_path.parent().expect("fixture parent"))
        .expect("create product binding fixture temp tree");
    let source_asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    for relative in ["ui/theme/editor_base.zui", "ui/theme/editor_material.zui"] {
        let destination = temp_root.join("assets").join(relative);
        fs::create_dir_all(destination.parent().expect("theme dependency parent"))
            .expect("create product binding fixture theme tree");
        fs::copy(source_asset_root.join(relative), destination)
            .expect("copy product binding fixture theme dependency");
    }
    let original = fs::read_to_string(fixture_path()).expect("read product binding fixture source");
    fs::write(&temp_fixture_path, &original).expect("write product binding fixture temp source");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let first = cache
        .load_store([temp_fixture_path.clone()])
        .expect("initial product binding fixture compile");

    let reloaded_source = original.replace(
        "applied_label = \"Primary applied\"",
        "applied_label = \"Primary applied after reload\"",
    );
    assert_ne!(
        reloaded_source, original,
        "reload mutation marker must exist"
    );
    fs::write(&temp_fixture_path, &reloaded_source)
        .expect("write reloaded product binding fixture source");
    let reloaded = cache
        .load_store([temp_fixture_path.clone()])
        .expect("reloaded product binding fixture should compile");
    assert!(!reloaded.cache_hit);
    assert!(!std::sync::Arc::ptr_eq(&first.compiled, &reloaded.compiled));
    let reloaded_primary = reloaded
        .compiled
        .arena
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("ProductBindingFixturePrimary"))
        .expect("reloaded primary product binding instance");
    assert_eq!(
        reloaded_primary.events[0].targets[0].expression,
        r#""Primary applied after reload""#
    );

    let invalid_source = reloaded_source.replace("enabled = true }", "enabled = \"not-a-bool\" }");
    assert_ne!(
        invalid_source, reloaded_source,
        "negative schema mutation marker must exist"
    );
    fs::write(&temp_fixture_path, invalid_source)
        .expect("write invalid product binding fixture source");
    let error = cache
        .load_store([temp_fixture_path.clone()])
        .expect_err("invalid product binding fixture reload must fail closed");
    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. }
            if detail.contains("cannot be represented as Bool")
    ));

    let last_known_good = cache
        .load_store_cached([temp_fixture_path])
        .expect("failed reload must preserve the last-known-good product fixture");
    assert!(std::sync::Arc::ptr_eq(
        &reloaded.compiled,
        &last_known_good.compiled
    ));
    let _ = fs::remove_dir_all(temp_root);
}
