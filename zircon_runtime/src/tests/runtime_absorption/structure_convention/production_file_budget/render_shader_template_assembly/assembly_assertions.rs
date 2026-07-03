#[path = "assembly_assertions/mesh_cache_contracts.rs"]
mod mesh_cache_contracts;
#[path = "assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs"]
mod mesh_pipeline_shadow_graph_contracts;
#[path = "assembly_assertions/owner_budget.rs"]
mod owner_budget;
#[path = "assembly_assertions/template_contracts.rs"]
mod template_contracts;

use super::sources::RenderShaderTemplateAssemblySources;

pub(super) fn assert_render_shader_template_assembly_is_folder_backed(
    sources: &RenderShaderTemplateAssemblySources,
) {
    template_contracts::assert_shader_template_contracts(sources);
    mesh_cache_contracts::assert_mesh_cache_contracts(sources);
    mesh_pipeline_shadow_graph_contracts::assert_mesh_pipeline_shadow_graph_contracts(sources);
    owner_budget::assert_render_shader_template_assembly_production_owners_stay_below_budget(
        sources,
    );
}
