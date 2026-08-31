use super::*;

mod contracts;
mod runtime_projection;
mod state_projection;

fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn component_showcase_contract_source() -> String {
    [
        "assets/ui/editor/component_showcase.zui",
        "assets/ui/editor/components/showcase/showcase_command_toolbar.zui",
        "assets/ui/editor/components/showcase/showcase_bottom_log.zui",
        "assets/ui/editor/components/showcase/showcase_category_nav.zui",
        "assets/ui/editor/components/showcase/showcase_state_panel.zui",
        "assets/ui/editor/components/showcase/showcase_visual_section.zui",
        "assets/ui/editor/components/showcase/showcase_input_section.zui",
        "assets/ui/editor/components/showcase/showcase_selection_section.zui",
        "assets/ui/editor/components/showcase/showcase_collections_section.zui",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n")
}

fn expected_showcase_action_id(suffix: &str, action_suffix: &str) -> String {
    format!(
        "ui_component_showcase.{}",
        camel_to_snake(&format!("{suffix}{action_suffix}"))
    )
}

fn camel_to_snake(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn default_component_showcase_nodes_for_size(
    width: f32,
    height: f32,
) -> Vec<crate::ui::retained_host::TemplatePaneNodeData> {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "res://ui/editor/component_showcase.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let mut pane = host_pane("component-showcase", "UI Component Showcase");
    pane.kind = "UiComponentShowcase".into();
    pane.pane_presentation = Some(host_window::PanePresentation::new(
        host_window::PaneShellPresentation::new(
            "UI Component Showcase",
            "ui-components",
            "Runtime components",
            "",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    ));
    let host_contract_pane =
        super::pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane(
            &pane,
            host_window::PaneContentSize::new(width, height),
        );

    (0..host_contract_pane.nodes.row_count())
        .filter_map(|row| host_contract_pane.nodes.row_data(row))
        .collect()
}

fn template_node<'a>(
    nodes: &'a [crate::ui::retained_host::TemplatePaneNodeData],
    control_id: &str,
) -> &'a crate::ui::retained_host::TemplatePaneNodeData {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("component showcase should expose `{control_id}`"))
}
