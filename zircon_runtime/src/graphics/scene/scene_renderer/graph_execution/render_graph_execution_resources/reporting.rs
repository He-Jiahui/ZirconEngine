use crate::core::framework::render::{
    RenderGraphExecutionAliasRecord, RenderGraphExecutionAliasReport,
    RenderGraphExecutionResourceReport, RenderGraphMaterializationReport,
};
use crate::render_graph::CompiledRenderGraph;

use super::RenderGraphExecutionResources;

impl RenderGraphExecutionResources {
    pub fn has_texture_view(&self, name: &str) -> bool {
        self.imported_texture_views.contains_key(name)
    }

    pub fn has_buffer(&self, name: &str) -> bool {
        self.buffer(name).is_some()
    }

    pub fn has_bound_resource(&self, name: &str) -> bool {
        self.has_texture_view(name) || self.has_buffer(name)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn bound_texture_view_names(
        &self,
    ) -> impl Iterator<Item = &str> {
        self.imported_texture_views.keys().map(String::as_str)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn bound_buffer_names(
        &self,
    ) -> impl Iterator<Item = &str> {
        self.buffer_backings.keys().map(String::as_str)
    }

    pub fn resource_report(&self) -> RenderGraphExecutionResourceReport {
        let texture_view_count = self.imported_texture_views.len();
        let owned_texture_count = self.owned_textures.len();
        let owned_backed_texture_view_count = self
            .imported_texture_views
            .keys()
            .filter(|name| self.is_owned_backed_texture_view(name))
            .count();
        RenderGraphExecutionResourceReport::new(
            texture_view_count,
            texture_view_count.saturating_sub(owned_backed_texture_view_count),
            owned_texture_count,
            self.buffers.len().saturating_add(self.owned_buffers.len()),
        )
        .with_access_binding_report(self.access_binding_report())
    }

    pub fn validate_materialized_graph_resources(
        &self,
        graph: &CompiledRenderGraph,
    ) -> Result<RenderGraphMaterializationReport, String> {
        super::super::materialization_validation::validate_materialized_graph_resources(self, graph)
    }

    pub fn resource_alias_report(&self) -> RenderGraphExecutionAliasReport {
        let mut texture_aliases = self
            .owned_texture_backings
            .iter()
            .filter(|(logical_name, _)| !self.texture_view_aliases.contains_key(*logical_name))
            .map(|(logical_name, backing_name)| {
                RenderGraphExecutionAliasRecord::new(logical_name.clone(), backing_name.clone())
            })
            .collect::<Vec<_>>();
        for (logical_name, (parent_name, range)) in &self.texture_view_aliases {
            texture_aliases.push(RenderGraphExecutionAliasRecord::new(
                logical_name.clone(),
                texture_subresource_alias_label(parent_name, *range),
            ));
        }
        texture_aliases.sort_by(|left, right| {
            left.logical_name
                .cmp(&right.logical_name)
                .then_with(|| left.backing_name.cmp(&right.backing_name))
        });

        let mut buffer_aliases = self
            .buffer_backings
            .iter()
            .filter(|(logical_name, backing_name)| {
                self.owned_buffers.contains_key(*backing_name)
                    || logical_name.as_str() != backing_name.as_str()
            })
            .map(|(logical_name, backing_name)| {
                RenderGraphExecutionAliasRecord::new(logical_name.clone(), backing_name.clone())
            })
            .collect::<Vec<_>>();
        buffer_aliases.sort_by(|left, right| {
            left.logical_name
                .cmp(&right.logical_name)
                .then_with(|| left.backing_name.cmp(&right.backing_name))
        });

        RenderGraphExecutionAliasReport::new(texture_aliases, buffer_aliases)
    }

    fn is_owned_backed_texture_view(&self, name: &str) -> bool {
        self.owned_texture_backings.contains_key(name)
    }
}

fn texture_subresource_alias_label(
    parent_name: &str,
    range: crate::render_graph::RenderGraphTextureSubresourceRange,
) -> String {
    let mip_count = range
        .mip_level_count
        .map_or_else(|| "all".to_string(), |count| count.to_string());
    let layer_count = range
        .array_layer_count
        .map_or_else(|| "all".to_string(), |count| count.to_string());
    format!(
        "{parent_name}:mip{}+{mip_count}:layer{}+{layer_count}:{:?}",
        range.base_mip_level, range.base_array_layer, range.aspect
    )
}
