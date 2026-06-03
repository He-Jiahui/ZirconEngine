use super::globals::PaneSurfaceHostContext;
use super::surface_hit_test::TemplateNodePointerHit;
use super::template_input_semantics::hit_is_text_input;
use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TemplatePrimaryActivationRoute {
    TextInputFocusOnly,
    Inspector,
    Asset,
    Welcome,
    Showcase,
    WorkbenchOption,
    WorkbenchMenuItem,
    SurfaceBinding,
    SurfaceAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssetPrimaryActivationKind {
    Click,
    Change,
}

struct AssetPrimaryActivation {
    source: SharedString,
    control_id: SharedString,
    kind: AssetPrimaryActivationKind,
}

pub(super) fn dispatch_template_node_primary_press(
    pane_host: &PaneSurfaceHostContext<'_>,
    hit: TemplateNodePointerHit,
) {
    match primary_activation_route(&hit) {
        TemplatePrimaryActivationRoute::TextInputFocusOnly => {}
        TemplatePrimaryActivationRoute::Inspector => {
            pane_host.invoke_inspector_control_clicked(hit.control_id)
        }
        TemplatePrimaryActivationRoute::Asset => {
            dispatch_asset_template_node_primary_press(pane_host, hit)
        }
        TemplatePrimaryActivationRoute::Welcome => {
            pane_host.invoke_welcome_control_clicked(action_or_control_id(&hit))
        }
        TemplatePrimaryActivationRoute::Showcase => {
            pane_host.invoke_component_showcase_control_activated(hit.control_id, hit.action_id)
        }
        TemplatePrimaryActivationRoute::WorkbenchOption => pane_host
            .invoke_component_showcase_option_selected(
                hit.control_id,
                hit.action_id,
                hit.value_text,
            ),
        TemplatePrimaryActivationRoute::WorkbenchMenuItem
        | TemplatePrimaryActivationRoute::SurfaceAction => {
            pane_host.invoke_surface_control_clicked(hit.control_id, hit.action_id)
        }
        TemplatePrimaryActivationRoute::SurfaceBinding => {
            pane_host.invoke_surface_control_clicked(hit.control_id, hit.binding_id)
        }
    }
}

fn primary_activation_route(hit: &TemplateNodePointerHit) -> TemplatePrimaryActivationRoute {
    if hit_is_text_input(hit) {
        return TemplatePrimaryActivationRoute::TextInputFocusOnly;
    }
    match hit.dispatch_kind.as_str() {
        "inspector" => TemplatePrimaryActivationRoute::Inspector,
        kind if asset_dispatch_source(kind).is_some() => TemplatePrimaryActivationRoute::Asset,
        "welcome" => TemplatePrimaryActivationRoute::Welcome,
        "showcase" => TemplatePrimaryActivationRoute::Showcase,
        "workbench_option" => TemplatePrimaryActivationRoute::WorkbenchOption,
        "workbench_menu_item" => TemplatePrimaryActivationRoute::WorkbenchMenuItem,
        _ if !hit.binding_id.is_empty() => TemplatePrimaryActivationRoute::SurfaceBinding,
        _ => TemplatePrimaryActivationRoute::SurfaceAction,
    }
}

fn dispatch_asset_template_node_primary_press(
    pane_host: &PaneSurfaceHostContext<'_>,
    hit: TemplateNodePointerHit,
) {
    let Some(activation) = asset_primary_activation(&hit) else {
        return;
    };
    match activation.kind {
        AssetPrimaryActivationKind::Click => {
            pane_host.invoke_asset_control_clicked(activation.source, activation.control_id)
        }
        AssetPrimaryActivationKind::Change => pane_host.invoke_asset_control_changed(
            activation.source,
            activation.control_id,
            hit.value_text,
        ),
    }
}

fn asset_primary_activation(hit: &TemplateNodePointerHit) -> Option<AssetPrimaryActivation> {
    let source = asset_dispatch_source(hit.dispatch_kind.as_str())?;
    let control_id = action_or_control_id(hit);
    let kind = if is_asset_change_control(control_id.as_str()) {
        AssetPrimaryActivationKind::Change
    } else {
        AssetPrimaryActivationKind::Click
    };
    Some(AssetPrimaryActivation {
        source: source.into(),
        control_id,
        kind,
    })
}

fn asset_dispatch_source(dispatch_kind: &str) -> Option<&str> {
    if dispatch_kind == "asset" {
        return Some("activity");
    }
    dispatch_kind.strip_prefix("asset:")
}

fn is_asset_change_control(control_id: &str) -> bool {
    matches!(
        control_id,
        "SearchEdited" | "SetKindFilter" | "SetViewMode" | "SetUtilityTab"
    )
}

fn action_or_control_id(hit: &TemplateNodePointerHit) -> SharedString {
    if hit.action_id.is_empty() {
        hit.control_id.clone()
    } else {
        hit.action_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::super::data::FrameRect;
    use super::super::template_component_family::TemplateComponentFamily;
    use super::*;

    #[test]
    fn text_input_family_routes_to_focus_only_activation() {
        let mut hit = hit_with_kind("");
        hit.component_family = Some(TemplateComponentFamily::TextInput);
        hit.binding_id = "TextField.Binding".into();

        assert_eq!(
            primary_activation_route(&hit),
            TemplatePrimaryActivationRoute::TextInputFocusOnly
        );
    }

    #[test]
    fn workbench_option_route_does_not_fall_back_to_binding() {
        let mut hit = hit_with_kind("workbench_option");
        hit.binding_id = "Dropdown.Binding".into();
        hit.value_text = "selected-option".into();

        assert_eq!(
            primary_activation_route(&hit),
            TemplatePrimaryActivationRoute::WorkbenchOption
        );
    }

    #[test]
    fn workbench_menu_item_routes_as_surface_action() {
        let mut hit = hit_with_kind("workbench_menu_item");
        hit.action_id = "Menu/Open".into();
        hit.binding_id = "MenuBindingShouldNotWin".into();

        assert_eq!(
            primary_activation_route(&hit),
            TemplatePrimaryActivationRoute::WorkbenchMenuItem
        );
    }

    #[test]
    fn asset_dispatch_source_and_change_controls_are_classified() {
        let mut hit = hit_with_kind("asset:browser");
        hit.action_id = "SearchEdited".into();

        let activation = asset_primary_activation(&hit).expect("asset dispatch should route");

        assert_eq!(activation.source.as_str(), "browser");
        assert_eq!(activation.control_id.as_str(), "SearchEdited");
        assert_eq!(activation.kind, AssetPrimaryActivationKind::Change);
    }

    #[test]
    fn asset_dispatch_uses_control_id_when_action_is_empty() {
        let hit = hit_with_kind("asset");

        let activation = asset_primary_activation(&hit).expect("asset dispatch should route");

        assert_eq!(activation.source.as_str(), "activity");
        assert_eq!(activation.control_id.as_str(), "Control");
        assert_eq!(activation.kind, AssetPrimaryActivationKind::Click);
    }

    fn hit_with_kind(dispatch_kind: &str) -> TemplateNodePointerHit {
        TemplateNodePointerHit {
            control_id: "Control".into(),
            action_id: SharedString::new(),
            binding_id: SharedString::new(),
            dispatch_kind: dispatch_kind.into(),
            component_role: SharedString::new(),
            component_family: None,
            value_text: SharedString::new(),
            edit_action_id: SharedString::new(),
            commit_action_id: SharedString::new(),
            frame: FrameRect::default(),
        }
    }
}
