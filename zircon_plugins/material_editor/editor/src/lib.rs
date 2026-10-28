#[path = "capability.rs"]
mod capability;
mod compilation;
mod connections;
mod details_selection;
mod diagnostics;
mod evaluation;
#[path = "extension_ids.rs"]
mod extension_ids;
mod graph_contract;
mod graph_index;
mod graph_presentation;
mod instance_projection;
mod node_creation;
mod node_creation_autowire;
mod node_deletion;
mod node_insertion;
mod node_pin_autowire;
mod outputs;
mod overview;
mod palette_search;
mod parameters;
#[path = "plugin.rs"]
mod plugin;
mod search_creation;
mod shader_compilation;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/output_contract.rs"]
mod output_contract_tests;

#[cfg(test)]
#[path = "tests/connections.rs"]
mod connection_tests;

#[cfg(test)]
#[path = "tests/evaluation.rs"]
mod evaluation_tests;

#[cfg(test)]
#[path = "tests/details_selection.rs"]
mod details_selection_tests;

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod diagnostic_tests;

#[cfg(test)]
#[path = "tests/palette_search.rs"]
mod palette_search_tests;

#[cfg(test)]
#[path = "tests/node_creation.rs"]
mod node_creation_tests;

#[cfg(test)]
#[path = "tests/node_creation_autowire.rs"]
mod node_creation_autowire_tests;

#[cfg(test)]
#[path = "tests/node_deletion.rs"]
mod node_deletion_tests;

#[cfg(test)]
#[path = "tests/node_insertion.rs"]
mod node_insertion_tests;

#[cfg(test)]
#[path = "tests/node_pin_autowire.rs"]
mod node_pin_autowire_tests;

#[cfg(test)]
#[path = "tests/graph_presentation.rs"]
mod graph_presentation_tests;

#[cfg(test)]
#[path = "tests/graph_spline.rs"]
mod graph_spline_tests;

#[cfg(test)]
#[path = "tests/graph_bounds.rs"]
mod graph_bounds_tests;

#[cfg(test)]
#[path = "tests/overview.rs"]
mod overview_tests;

#[cfg(test)]
#[path = "tests/parameters.rs"]
mod parameter_tests;

#[cfg(test)]
#[path = "tests/search_creation.rs"]
mod search_creation_tests;

#[cfg(test)]
#[path = "tests/shader_compilation.rs"]
mod shader_compilation_tests;

#[cfg(test)]
#[path = "tests/instance_projection.rs"]
mod instance_projection_tests;

pub use capability::{
    CAPABILITY, EDITOR_CAPABILITIES, EDITOR_CRATE_NAME, MATERIAL_EDITOR_DECLARATION,
    NATIVE_EDITOR_ENTRY, NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, PLUGIN_ID,
};
pub use compilation::{
    compile_material_graph, compile_material_graph_operation,
    compile_material_graph_with_diagnostics, MaterialGraphCompileReport,
};
pub use connections::{
    can_connect_material_graph, MaterialGraphConnectionAction, MaterialGraphConnectionPlan,
    MaterialGraphValueType,
};
pub use details_selection::{
    plan_material_graph_details_edit, plan_material_graph_expression_details_batch_edit,
    plan_material_graph_expression_details_edit, project_material_graph_details_selection,
    MaterialGraphDetailsEdit, MaterialGraphDetailsEditPlan, MaterialGraphDetailsSelection,
    MaterialGraphDetailsTarget, MaterialGraphDetailsValue, MaterialGraphExpressionDetailsBatchEdit,
    MaterialGraphExpressionDetailsBatchPlan, MaterialGraphExpressionDetailsEdit,
    MaterialGraphExpressionDetailsEditPlan, MaterialGraphExpressionDetailsValues,
};
pub use diagnostics::{
    validate_material_graph, validate_material_graph_with_diagnostics, MaterialGraphDiagnostic,
    MaterialGraphDiagnosticTarget,
};
use evaluation::{MaterialColorInput, MaterialGraphEvaluator};
pub use extension_ids::{
    MATERIAL_EDITOR_DRAWER_ID, MATERIAL_EDITOR_TEMPLATE_ID, MATERIAL_EDITOR_VIEW_ID,
};
use graph_index::MaterialGraphIndex;
pub use graph_presentation::{
    MaterialGraphEdgeProjection, MaterialGraphEdgeSplineProjection, MaterialGraphPresentation,
};
pub use instance_projection::{
    check_material_document_asset_references, plan_material_document_property_edit,
    plan_material_document_texture_edit, project_authored_material_document,
    MaterialDocumentAssetReferenceError, MaterialDocumentProjectionError,
    MaterialDocumentPropertyEdit, MaterialDocumentPropertyEditError,
    MaterialDocumentPropertyEditPlan, MaterialDocumentTextureEdit,
    MaterialDocumentTextureEditError, MaterialDocumentTextureEditPlan,
};
pub use node_creation::validate_material_graph_node_creation;
pub use node_creation_autowire::{
    plan_material_graph_node_creation, MaterialGraphNodeCreationPlan,
};
pub use node_deletion::{plan_material_graph_node_deletion, MaterialGraphNodeDeletionPlan};
pub use node_insertion::{plan_material_graph_node_insertion, MaterialGraphNodeInsertionPlan};
pub use node_pin_autowire::suggest_material_graph_autowire;
pub use overview::MaterialGraphTextureUsage;
pub use palette_search::{
    search_material_graph_node_palette, MaterialGraphNodeSearch, MaterialGraphNodeSearchNavigation,
};
pub use parameters::{
    plan_material_graph_parameter_edit, plan_material_graph_parameter_vector_edit,
    MaterialGraphParameterEdit, MaterialGraphParameterEditPlan, MaterialGraphParameterView,
};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor,
    material_editor_dist_module_manifest, package_manifest, plugin_registration,
    MaterialEditorPlugin, MATERIAL_EDITOR_DIST_CRATE_NAME, MATERIAL_EDITOR_DIST_EDITOR_ENTRY,
};
pub use search_creation::plan_material_graph_node_search_creation;
pub use shader_compilation::{
    compile_material_graph_with_shader_contract, MaterialGraphShaderCompileError,
};
use zircon_runtime::asset::{
    AlphaMode, AssetReference, MaterialAsset, MaterialGraphAsset, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};
