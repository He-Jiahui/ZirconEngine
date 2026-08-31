use super::super::globals::PaneSurfaceHostContext;
use super::super::surface_hit_test::TemplateNodePointerHit;
use super::helpers::action_or_control_id;
use crate::ui::retained_host::asset_control_ids::{
    asset_dispatch_source, asset_surface_binding_control_id,
};
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum AssetPrimaryActivationKind {
    Click,
    Change,
}

pub(in crate::ui::retained_host::host_contract) struct AssetPrimaryActivation {
    pub(in crate::ui::retained_host::host_contract) source: SharedString,
    pub(in crate::ui::retained_host::host_contract) control_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) kind: AssetPrimaryActivationKind,
}

pub(in crate::ui::retained_host::host_contract) fn dispatch_asset_template_node_primary_press(
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

pub(in crate::ui::retained_host::host_contract) fn asset_primary_activation(
    hit: &TemplateNodePointerHit,
) -> Option<AssetPrimaryActivation> {
    let source = asset_dispatch_source(hit.dispatch_kind.as_str())?;
    if hit.component_family == Some(TemplateComponentFamily::Dropdown) && hit.action_id.is_empty() {
        return None;
    }
    let action_or_control_id = action_or_control_id(hit);
    let control_id = asset_surface_binding_control_id(action_or_control_id.as_str())
        .unwrap_or(action_or_control_id.as_str());
    let kind = if is_asset_change_control(control_id) {
        AssetPrimaryActivationKind::Change
    } else {
        AssetPrimaryActivationKind::Click
    };
    Some(AssetPrimaryActivation {
        source: source.into(),
        control_id: control_id.into(),
        kind,
    })
}

fn is_asset_change_control(control_id: &str) -> bool {
    matches!(
        control_id,
        "SearchEdited"
            | "SetKindFilter"
            | "SetViewMode"
            | "SetUtilityTab"
            | "workbench.asset.search.edit"
            | "workbench.asset.kind_filter.set"
            | "workbench.asset.view_mode.set"
            | "workbench.asset.utility_tab.set"
    )
}
