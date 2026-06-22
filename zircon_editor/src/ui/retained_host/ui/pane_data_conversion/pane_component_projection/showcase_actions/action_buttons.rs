use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::binding_ids::{showcase_action_id_for_binding_id, showcase_binding_with_suffix};

pub(in super::super) fn preferred_showcase_action_buttons(
    control_id: &str,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneActionData> {
    action_button_specs(control_id)
        .iter()
        .filter_map(|(label, suffix)| {
            showcase_binding_with_suffix(bindings, suffix).map(|binding| {
                host_contract::TemplatePaneActionData {
                    label: (*label).into(),
                    action_id: showcase_action_id_for_binding_id(&binding.binding_id).into(),
                }
            })
        })
        .collect()
}

fn action_button_specs(control_id: &str) -> &'static [(&'static str, &'static str)] {
    match control_id {
        "AssetFieldDemo" => &[
            ("Find", "AssetFieldLocate"),
            ("Open", "AssetFieldOpen"),
            ("Clear", "AssetFieldClear"),
        ],
        "InstanceFieldDemo" => &[
            ("Find", "InstanceFieldLocate"),
            ("Open", "InstanceFieldOpen"),
            ("Clear", "InstanceFieldClear"),
        ],
        "ObjectFieldDemo" => &[
            ("Find", "ObjectFieldLocate"),
            ("Open", "ObjectFieldOpen"),
            ("Clear", "ObjectFieldClear"),
        ],
        "ArrayFieldDemo" => &[
            ("Add", "ArrayFieldAddElement"),
            ("Set", "ArrayFieldSetElement"),
            ("Remove", "ArrayFieldRemoveElement"),
            ("Move", "ArrayFieldMoveElement"),
        ],
        "MapFieldDemo" => &[
            ("Add", "MapFieldAddEntry"),
            ("Set", "MapFieldSetEntry"),
            ("Remove", "MapFieldRemoveEntry"),
        ],
        _ => &[],
    }
}
