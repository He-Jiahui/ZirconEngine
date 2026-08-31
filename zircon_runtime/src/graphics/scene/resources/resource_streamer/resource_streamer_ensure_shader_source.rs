use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderMaterialDependencySet, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{
    AssetReference, ResourceId, ResourceKind, ResourceReadinessGeneration,
};
use std::{collections::HashSet, sync::Arc};

use crate::graphics::types::GraphicsError;
use crate::plugin::ShaderModuleSourceBinding;

use super::super::fallback_shader_uri;
use super::super::prepared::PreparedShader;
use super::super::runtime::ShaderRuntime;
use super::ResourceStreamer;

#[derive(Default)]
struct ShaderSourcePreparationTraversal {
    visiting: HashSet<ResourceId>,
    completed: HashSet<ResourceId>,
    cache_hits: usize,
    rebuilds: usize,
    dependency_generation_invalidations: usize,
}

impl ShaderSourcePreparationTraversal {
    fn enter(&mut self, shader_id: ResourceId) -> bool {
        !self.completed.contains(&shader_id) && self.visiting.insert(shader_id)
    }

    fn finish(&mut self, shader_id: ResourceId, completed: bool) {
        self.visiting.remove(&shader_id);
        if completed {
            self.completed.insert(shader_id);
        }
    }
}

impl ResourceStreamer {
    pub(crate) fn ensure_shader_source(
        &mut self,
        reference: &AssetReference,
    ) -> Result<(ResourceId, u64, u64, Option<RenderMaterialReadinessReport>), GraphicsError> {
        let readiness_generation = self
            .asset_manager()?
            .resource_manager()
            .readiness_generation();
        let mut traversal = ShaderSourcePreparationTraversal::default();
        let result = self.ensure_shader_source_recursive(
            reference,
            readiness_generation.as_ref(),
            &mut traversal,
        );
        crate::profile_counter!(
            "render",
            "shader_artifact_cache_hit",
            traversal.cache_hits as u64
        );
        crate::profile_counter!(
            "render",
            "shader_artifact_rebuild",
            traversal.rebuilds as u64
        );
        crate::profile_counter!(
            "render",
            "shader_dependency_generation_invalidation",
            traversal.dependency_generation_invalidations as u64
        );
        result
    }

    fn ensure_shader_source_recursive(
        &mut self,
        reference: &AssetReference,
        readiness_generation: &ResourceReadinessGeneration,
        traversal: &mut ShaderSourcePreparationTraversal,
    ) -> Result<(ResourceId, u64, u64, Option<RenderMaterialReadinessReport>), GraphicsError> {
        let uri = &reference.locator;
        let mut fallback_report = None;
        let asset_manager = self.asset_manager()?;
        let resolved_shader_id = asset_manager.resolve_asset_id(uri);
        if let Some(shader_id) = resolved_shader_id {
            let requested_revision = self.resource_revision(shader_id)?;
            let dependency_revision = readiness_generation
                .dependency_revision(shader_id)
                .unwrap_or(0);
            if self.shaders.get(&shader_id).is_some_and(|prepared| {
                shader_artifact_identity_is_current(
                    prepared.revision,
                    prepared.dependency_revision,
                    requested_revision,
                    dependency_revision,
                )
            }) {
                traversal.cache_hits = traversal.cache_hits.saturating_add(1);
                return Ok((shader_id, requested_revision, dependency_revision, None));
            }
        }
        let (shader_id, revision, shader) = match resolved_shader_id {
            Some(shader_id) => match asset_manager.load_shader_asset_snapshot(shader_id) {
                Ok(shader) => {
                    let revision = shader.revision();
                    (shader_id, revision, (*shader).clone())
                }
                Err(_) => {
                    fallback_report = Some(missing_shader_readiness_report(reference));
                    self.load_fallback_shader()?
                }
            },
            None => {
                fallback_report = Some(missing_shader_readiness_report(reference));
                self.load_fallback_shader()?
            }
        };
        let (shader_id, revision, shader) = if shader.runtime_wgsl_source().is_some() {
            (shader_id, revision, shader)
        } else {
            fallback_report = Some(missing_runtime_shader_readiness_report(reference));
            self.load_fallback_shader()?
        };
        let dependency_revision = readiness_generation
            .dependency_revision(shader_id)
            .unwrap_or(0);

        if !traversal.enter(shader_id) {
            return Ok((shader_id, revision, dependency_revision, fallback_report));
        }
        let result = (|| {
            if self.shaders.get(&shader_id).is_some_and(|prepared| {
                shader_artifact_identity_is_current(
                    prepared.revision,
                    prepared.dependency_revision,
                    revision,
                    dependency_revision,
                )
            }) {
                traversal.cache_hits = traversal.cache_hits.saturating_add(1);
                return Ok((shader_id, revision, dependency_revision, fallback_report));
            }
            if self.shaders.get(&shader_id).is_some_and(|prepared| {
                prepared.revision == revision && prepared.dependency_revision != dependency_revision
            }) {
                traversal.dependency_generation_invalidations = traversal
                    .dependency_generation_invalidations
                    .saturating_add(1);
            }
            traversal.rebuilds = traversal.rebuilds.saturating_add(1);
            let import_dependencies = shader
                .imports
                .iter()
                .filter_map(|import| import.redirect.clone())
                .collect::<Vec<_>>();
            let source = Arc::<str>::from(
                shader
                    .runtime_wgsl_source()
                    .ok_or_else(|| {
                        GraphicsError::Asset(format!(
                            "shader {} has no runtime WGSL source",
                            shader.uri
                        ))
                    })?
                    .to_string(),
            );
            let surface_source_contract = shader.surface_source_contract().map_err(|error| {
                GraphicsError::Asset(format!(
                    "shader {} has an invalid surface source contract: {error}",
                    shader.uri
                ))
            })?;
            let module_source_binding = shader
                .kind
                .is_include()
                .then(|| {
                    shader.import_path.clone().map(|import_path| {
                        let locator = asset_manager
                            .resource_manager()
                            .registry()
                            .get(shader_id)
                            .map(|record| record.primary_locator.to_string())
                            .unwrap_or_else(|| shader.uri.to_string());
                        ShaderModuleSourceBinding::new(
                            format!("project:{shader_id}"),
                            import_path,
                            source.clone(),
                            format!("project shader asset {locator}"),
                        )
                    })
                })
                .flatten();
            let prepared_shader = PreparedShader {
                revision,
                dependency_revision,
                runtime: ShaderRuntime {
                    source,
                    kind: shader.kind,
                    surface_source_contract,
                    import_path: shader.import_path.clone(),
                    imports: shader.imports.clone(),
                    material_option_table: shader.material_option_table,
                    generated_material_wgsl: shader.generated_material_wgsl,
                },
                module_source_binding,
            };
            for dependency in import_dependencies {
                let _ = self.ensure_shader_source_recursive(
                    &dependency,
                    readiness_generation,
                    traversal,
                )?;
            }
            self.ensure_shader_dependency_sources(
                asset_manager.as_ref(),
                shader_id,
                readiness_generation,
                traversal,
            )?;
            self.shaders.insert(shader_id, prepared_shader);
            crate::profile_counter!("render", "shader_artifact_publish", 1);
            Ok((shader_id, revision, dependency_revision, fallback_report))
        })();
        traversal.finish(shader_id, result.is_ok());
        result
    }

    fn ensure_shader_dependency_sources(
        &mut self,
        asset_manager: &ProjectAssetManager,
        shader_id: ResourceId,
        readiness_generation: &ResourceReadinessGeneration,
        traversal: &mut ShaderSourcePreparationTraversal,
    ) -> Result<(), GraphicsError> {
        let resource_manager = asset_manager.resource_manager();
        let registry = resource_manager.registry();
        let dependencies = shader_dependency_ids(asset_manager, shader_id)
            .into_iter()
            .filter_map(|dependency_id| registry.get(dependency_id).cloned())
            .map(|record| AssetReference::from_locator(record.primary_locator.clone()))
            .collect::<Vec<_>>();
        for dependency in dependencies {
            let _ =
                self.ensure_shader_source_recursive(&dependency, readiness_generation, traversal)?;
        }
        Ok(())
    }

    fn load_fallback_shader(
        &self,
    ) -> Result<(ResourceId, u64, crate::asset::ShaderAsset), GraphicsError> {
        let fallback_uri = fallback_shader_uri();
        let asset_manager = self.asset_manager()?;
        let shader_id = asset_manager
            .resolve_asset_id(&fallback_uri)
            .ok_or_else(|| {
                GraphicsError::Asset(format!("missing shader resource for {fallback_uri}"))
            })?;
        let shader = asset_manager
            .load_shader_asset_snapshot(shader_id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let revision = shader.revision();
        Ok((shader_id, revision, (*shader).clone()))
    }
}

fn shader_artifact_identity_is_current(
    prepared_revision: u64,
    prepared_dependency_revision: u64,
    requested_revision: u64,
    requested_dependency_revision: u64,
) -> bool {
    prepared_revision == requested_revision
        && prepared_dependency_revision == requested_dependency_revision
}

#[cfg(test)]
mod tests {
    use super::{ShaderSourcePreparationTraversal, shader_artifact_identity_is_current};
    use crate::core::resource::ResourceId;

    #[test]
    fn shader_source_preparation_traversal_deduplicates_completed_shared_descendants() {
        let root = ResourceId::from_stable_label("shader-root");
        let left = ResourceId::from_stable_label("shader-left");
        let right = ResourceId::from_stable_label("shader-right");
        let shared = ResourceId::from_stable_label("shader-shared");
        let mut traversal = ShaderSourcePreparationTraversal::default();

        assert!(traversal.enter(root));
        assert!(traversal.enter(left));
        assert!(traversal.enter(shared));
        traversal.finish(shared, true);
        traversal.finish(left, true);
        assert!(traversal.enter(right));
        assert!(
            !traversal.enter(shared),
            "a shared completed descendant must not be revisited through a second edge"
        );
        traversal.finish(right, true);
        traversal.finish(root, true);
    }

    #[test]
    fn shader_artifact_identity_requires_root_and_transitive_revisions() {
        assert!(shader_artifact_identity_is_current(7, 11, 7, 11));
        assert!(!shader_artifact_identity_is_current(7, 11, 8, 11));
        assert!(!shader_artifact_identity_is_current(7, 11, 7, 12));
    }

    #[test]
    fn shader_rebuild_pairs_source_with_its_atomic_resource_revision() {
        let production = include_str!("resource_streamer_ensure_shader_source.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("shader preparation test boundary");

        assert!(production.contains("load_shader_asset_snapshot(shader_id)"));
        assert!(production.contains("let revision = shader.revision();"));
        assert!(!production.contains(
            "let revision = self.resource_revision(shader_id)?;\n        let dependency_revision"
        ));
    }
}

pub(super) fn shader_dependency_ids(
    asset_manager: &ProjectAssetManager,
    shader_id: ResourceId,
) -> Vec<ResourceId> {
    let resource_manager = asset_manager.resource_manager();
    let registry = resource_manager.registry();
    registry
        .get(shader_id)
        .into_iter()
        .flat_map(|record| record.dependency_ids.iter().copied())
        .filter(|dependency_id| {
            registry
                .get(*dependency_id)
                .is_some_and(|record| record.kind == ResourceKind::Shader)
        })
        .collect()
}

fn missing_shader_readiness_report(reference: &AssetReference) -> RenderMaterialReadinessReport {
    RenderMaterialReadinessReport {
        material_name: None,
        dependencies: RenderMaterialDependencySet::new(reference.clone()),
        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        validation_errors: vec![RenderMaterialValidationError::UnresolvedShaderReference {
            reference: reference.clone(),
        }],
        fallback_usages: vec![RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Shader {
                reference: reference.clone(),
            },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }],
        property_value_summary: None,
        property_value_states: Vec::new(),
        uniform_summary: None,
        uniform_fields: Vec::new(),
        uniform_unsupported: Vec::new(),
        standard_texture_slot_summary: None,
        standard_texture_slot_states: Vec::new(),
        texture_slot_summary: None,
        non_standard_texture_slot_states: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn missing_runtime_shader_readiness_report(
    reference: &AssetReference,
) -> RenderMaterialReadinessReport {
    RenderMaterialReadinessReport {
        material_name: None,
        dependencies: RenderMaterialDependencySet::new(reference.clone()),
        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        validation_errors: vec![RenderMaterialValidationError::MissingRuntimeShaderSource],
        fallback_usages: vec![RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Shader {
                reference: reference.clone(),
            },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }],
        property_value_summary: None,
        property_value_states: Vec::new(),
        uniform_summary: None,
        uniform_fields: Vec::new(),
        uniform_unsupported: Vec::new(),
        standard_texture_slot_summary: None,
        standard_texture_slot_states: Vec::new(),
        texture_slot_summary: None,
        non_standard_texture_slot_states: Vec::new(),
        diagnostics: Vec::new(),
    }
}
