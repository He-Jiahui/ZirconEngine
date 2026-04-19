use std::collections::{BTreeMap, HashMap};

use crate::layout::WorkbenchLayout;
use crate::snapshot::ViewContentKind;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::view::{ViewDescriptor, ViewDescriptorId};

use super::super::active_tab::active_document_tab;
use super::super::constraints::{
    default_constraints_for_content, default_region_constraints, merge_constraints,
};
use super::super::region_state::RegionState;
use super::super::{PaneConstraints, ShellRegionId};

pub(crate) fn build_document_region_state(
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, &ViewDescriptor>,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> RegionState {
    let tab = active_document_tab(model);
    let descriptor_constraints = tab
        .and_then(|tab| descriptors.get(&tab.descriptor_id).copied())
        .map(|descriptor| {
            if descriptor.default_constraints == PaneConstraints::default() {
                default_constraints_for_content(
                    tab.map(|tab| tab.content_kind)
                        .unwrap_or(ViewContentKind::Placeholder),
                )
            } else {
                descriptor.default_constraints
            }
        })
        .unwrap_or_else(|| default_region_constraints(ShellRegionId::Document));
    let layout_override = layout
        .region_overrides
        .get(&ShellRegionId::Document)
        .copied();
    let view_override = tab.and_then(|tab| layout.view_overrides.get(&tab.instance_id).copied());
    let mut constraints = merge_constraints(
        default_region_constraints(ShellRegionId::Document),
        layout_override,
        descriptor_constraints,
        view_override,
    );
    if let Some(preferred) =
        transient_region_preferred.and_then(|map| map.get(&ShellRegionId::Document).copied())
    {
        constraints.width.preferred = preferred;
    }
    RegionState {
        visible: true,
        expanded: true,
        constraints,
    }
}
