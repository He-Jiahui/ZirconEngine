use super::*;
use crate::core::i18n::EditorLocale;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostValue};
use crate::ui::workbench::reference::{
    build_editor_workbench_template_surface, EditorWorkbenchReferenceMetrics,
};
use zircon_runtime::ui::surface::UiPropertyMutationRequest;
use zircon_runtime_interface::ui::component::UiValue;

#[test]
fn actual_authored_headers_share_one_localized_property_through_measure_host_and_native_projection()
{
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let i18n = Arc::new(EditorI18nService::default());
    let mut surface = build_editor_workbench_template_surface(
        &runtime,
        EditorWorkbenchReferenceMetrics::default(),
    )
    .unwrap();
    surface.install_localization_context(Arc::clone(&i18n));
    for locale in [
        EditorLocale::english(),
        EditorLocale::parse("zh-CN").unwrap(),
    ] {
        i18n.set_active_locale(locale.clone()).unwrap();
        surface.refresh_after_state_change(&runtime).unwrap();
        let native = crate::ui::retained_host::to_host_contract_workbench_window_nodes(Some(
            &surface.host_projection,
        ));
        for (control, key) in [
            ("WorkbenchSceneTitle", "editor.workbench.outliner.title"),
            ("WorkbenchInspectorTitle", "editor.workbench.details.title"),
        ] {
            let expected = i18n.translate_for_locale(&locale, key);
            let node_id = surface.control_node_id(control).unwrap();
            let metadata = surface
                .surface
                .tree
                .node(node_id)
                .unwrap()
                .template_metadata
                .as_ref()
                .unwrap();
            assert_eq!(
                metadata.attributes["text"].as_str(),
                Some(expected.as_ref())
            );
            assert_eq!(metadata.localized_text_references["text"].key, key);
            let host = surface
                .host_projection
                .nodes
                .iter()
                .find(|n| n.control_id.as_deref() == Some(control))
                .unwrap();
            assert_eq!(
                host.properties.get("text"),
                Some(&RetainedUiHostValue::String(expected.to_string()))
            );
            let native_node = native
                .iter()
                .find(|n| n.control_id.as_str() == control)
                .unwrap();
            assert_eq!(native_node.text.as_str(), expected.as_ref());
            let command = surface
                .surface
                .render_extract
                .list
                .commands
                .iter()
                .find(|c| c.node_id == node_id && c.text.as_deref() == Some(expected.as_ref()))
                .unwrap();
            assert!(
                command.text_layout.is_some(),
                "shared font/text pipeline must shape the same resolved property before paint"
            );
        }
        let transform = surface
            .host_projection
            .nodes
            .iter()
            .find(|n| n.control_id.as_deref() == Some("WorkbenchTransformLabel"))
            .unwrap();
        assert_eq!(
            transform.properties.get("text"),
            Some(&RetainedUiHostValue::String("Transform".to_owned()))
        );
    }
}

#[test]
fn live_actor_title_and_explicit_empty_reset_survive_locale_changes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let i18n = Arc::new(EditorI18nService::default());
    let mut surface = build_editor_workbench_template_surface(
        &runtime,
        EditorWorkbenchReferenceMetrics::default(),
    )
    .unwrap();
    surface.install_localization_context(Arc::clone(&i18n));
    surface.refresh_after_state_change(&runtime).unwrap();
    let id = surface.control_node_id("WorkbenchInspectorTitle").unwrap();
    for (text, locale) in [("Camera", "zh-CN"), ("", "en")] {
        surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                id,
                "text",
                UiValue::String(text.to_owned()),
            ))
            .unwrap();
        i18n.set_active_locale(EditorLocale::parse(locale).unwrap())
            .unwrap();
        surface.refresh_after_state_change(&runtime).unwrap();
        let host = surface
            .host_projection
            .nodes
            .iter()
            .find(|n| n.control_id.as_deref() == Some("WorkbenchInspectorTitle"))
            .unwrap();
        assert_eq!(
            host.properties.get("text"),
            Some(&RetainedUiHostValue::String(text.to_owned()))
        );
        assert!(surface
            .surface
            .tree
            .node(id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .localized_text_references
            .is_empty());
    }
}
