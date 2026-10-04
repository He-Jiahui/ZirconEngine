use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderMaterialDependencySet, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{
    AssetReference, ResourceId, ResourceKind, ResourceReadinessGeneration,
    ResourceReadinessRowIdentity,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use crate::graphics::types::GraphicsError;
use crate::plugin::ShaderModuleSourceBinding;

use super::super::fallback_shader_uri;
use super::super::prepared::PreparedShader;
use super::super::runtime::ShaderRuntime;
use super::ResourceStreamer;

const MAX_SHADER_DEPENDENCY_NODES: usize = 1024;
const MAX_SHADER_DEPENDENCY_DEPTH: usize = 128;
const MAX_SHADER_SOURCE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ShaderSourcePreparationFailure {
    Cycle { path: Vec<ResourceId> },
    MissingDependency { locator: String },
    NodeBudget { limit: usize },
    DepthBudget { limit: usize },
    SourceBudget { limit: usize },
}

impl fmt::Display for ShaderSourcePreparationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle { path } => write!(
                formatter,
                "shader dependency cycle rejected: {}",
                path.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            Self::NodeBudget { limit } => {
                write!(
                    formatter,
                    "shader dependency node budget exceeded (limit {limit})"
                )
            }
            Self::MissingDependency { locator } => {
                write!(formatter, "shader dependency is not ready: {locator}")
            }
            Self::DepthBudget { limit } => {
                write!(
                    formatter,
                    "shader dependency depth budget exceeded (limit {limit})"
                )
            }
            Self::SourceBudget { limit } => {
                write!(
                    formatter,
                    "shader dependency source budget exceeded (limit {limit} bytes)"
                )
            }
        }
    }
}

#[derive(Default)]
struct ShaderSourcePreparationTraversal {
    visiting: HashMap<ResourceId, usize>,
    stack: Vec<ResourceId>,
    completed: HashSet<ResourceId>,
    staged: HashMap<ResourceId, PreparedShader>,
    visited_nodes: usize,
    source_bytes: usize,
    cache_hits: usize,
    rebuilds: usize,
    dependency_generation_invalidations: usize,
}

impl ShaderSourcePreparationTraversal {
    fn active(&self, shader_id: ResourceId) -> bool {
        self.visiting.contains_key(&shader_id)
    }

    fn enter(&mut self, shader_id: ResourceId) -> bool {
        self.try_enter(shader_id, 0).is_ok_and(|is_new| is_new)
    }

    fn try_enter(
        &mut self,
        shader_id: ResourceId,
        depth: usize,
    ) -> Result<bool, ShaderSourcePreparationFailure> {
        if self.completed.contains(&shader_id) {
            return Ok(false);
        }
        if let Some(index) = self.visiting.get(&shader_id).copied() {
            let mut path = self.stack[index..].to_vec();
            path.push(shader_id);
            return Err(ShaderSourcePreparationFailure::Cycle { path });
        }
        if depth > MAX_SHADER_DEPENDENCY_DEPTH {
            return Err(ShaderSourcePreparationFailure::DepthBudget {
                limit: MAX_SHADER_DEPENDENCY_DEPTH,
            });
        }
        if self.visited_nodes >= MAX_SHADER_DEPENDENCY_NODES {
            return Err(ShaderSourcePreparationFailure::NodeBudget {
                limit: MAX_SHADER_DEPENDENCY_NODES,
            });
        }
        self.visiting.insert(shader_id, self.stack.len());
        self.stack.push(shader_id);
        self.visited_nodes = self.visited_nodes.saturating_add(1);
        Ok(true)
    }

    fn finish(&mut self, shader_id: ResourceId, completed: bool) {
        self.visiting.remove(&shader_id);
        if self.stack.last().copied() == Some(shader_id) {
            self.stack.pop();
        }
        if completed {
            self.completed.insert(shader_id);
        }
    }

    fn stage(&mut self, shader_id: ResourceId, shader: PreparedShader) {
        self.staged.insert(shader_id, shader);
    }

    fn add_source_bytes(&mut self, bytes: usize) -> Result<(), ShaderSourcePreparationFailure> {
        self.source_bytes = self.source_bytes.saturating_add(bytes);
        if self.source_bytes > MAX_SHADER_SOURCE_BYTES {
            return Err(ShaderSourcePreparationFailure::SourceBudget {
                limit: MAX_SHADER_SOURCE_BYTES,
            });
        }
        Ok(())
    }

    fn publish(&mut self, shaders: &mut HashMap<ResourceId, PreparedShader>) {
        for (shader_id, shader) in self.staged.drain() {
            shaders.insert(shader_id, shader);
        }
    }

    fn discard(&mut self) {
        self.staged.clear();
    }
}

impl ResourceStreamer {
    pub(crate) fn ensure_shader_source(
        &mut self,
        reference: &AssetReference,
    ) -> Result<
        (
            ResourceId,
            u64,
            ResourceReadinessRowIdentity,
            Option<RenderMaterialReadinessReport>,
        ),
        GraphicsError,
    > {
        let readiness_generation = self
            .asset_manager()?
            .resource_manager()
            .readiness_generation();
        let mut traversal = ShaderSourcePreparationTraversal::default();
        let result = self.ensure_shader_source_recursive(
            reference,
            readiness_generation.as_ref(),
            &mut traversal,
            0,
        );
        if result.is_ok() {
            traversal.publish(&mut self.shaders);
        } else {
            traversal.discard();
        }
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
        depth: usize,
    ) -> Result<
        (
            ResourceId,
            u64,
            ResourceReadinessRowIdentity,
            Option<RenderMaterialReadinessReport>,
        ),
        GraphicsError,
    > {
        let uri = &reference.locator;
        let mut fallback_report = None;
        let asset_manager = self.asset_manager()?;
        let resolved_shader_id = asset_manager.resolve_asset_id(uri);
        if let Some(shader_id) = resolved_shader_id {
            if traversal.active(shader_id) {
                return Err(GraphicsError::Asset(
                    ShaderSourcePreparationFailure::Cycle {
                        path: traversal
                            .stack
                            .iter()
                            .copied()
                            .chain(std::iter::once(shader_id))
                            .collect(),
                    }
                    .to_string(),
                ));
            }
            let requested_revision = self.resource_revision(shader_id)?;
            // A cached root is admissible only after the same bounded import
            // and registry dependency closure used by rebuilds. The previous
            // fast path returned before reading imports, allowing a newly
            // missing child or cycle below an otherwise current root to pass.
            if let Some(cache_shader) = asset_manager.load_shader_asset_snapshot(shader_id).ok() {
                let import_dependencies = cache_shader
                    .imports
                    .iter()
                    .filter_map(|import| import.redirect.clone())
                    .collect::<Vec<_>>();
                if let Some(dependency_identity) = readiness_generation.row_identity(shader_id) {
                    if self.shaders.get(&shader_id).is_some_and(|prepared| {
                        shader_artifact_identity_is_current(
                            prepared.revision,
                            &prepared.dependency_identity,
                            requested_revision,
                            &dependency_identity,
                        )
                    }) {
                        let is_new = traversal
                            .try_enter(shader_id, depth)
                            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
                        if is_new {
                            let result = (|| {
                                for dependency in &import_dependencies {
                                    let _ = self.ensure_shader_source_recursive(
                                        dependency,
                                        readiness_generation,
                                        traversal,
                                        depth.saturating_add(1),
                                    )?;
                                }
                                self.ensure_shader_dependency_sources(
                                    asset_manager.as_ref(),
                                    shader_id,
                                    readiness_generation,
                                    traversal,
                                    depth,
                                )?;
                                traversal.cache_hits = traversal.cache_hits.saturating_add(1);
                                Ok((shader_id, requested_revision, dependency_identity, None))
                            })();
                            traversal.finish(shader_id, result.is_ok());
                            return result;
                        }
                        traversal.cache_hits = traversal.cache_hits.saturating_add(1);
                        return Ok((shader_id, requested_revision, dependency_identity, None));
                    }
                }
            }
        }
        let (shader_id, revision, shader) = match resolved_shader_id {
            Some(shader_id) => match asset_manager.load_shader_asset_snapshot(shader_id) {
                Ok(shader) => {
                    let revision = shader.revision();
                    (shader_id, revision, (*shader).clone())
                }
                Err(error) if depth > 0 => {
                    return Err(GraphicsError::Asset(
                        ShaderSourcePreparationFailure::MissingDependency {
                            locator: format!("{uri}: {error}"),
                        }
                        .to_string(),
                    ));
                }
                Err(_) => {
                    fallback_report = Some(missing_shader_readiness_report(reference));
                    self.load_fallback_shader()?
                }
            },
            None if depth > 0 => {
                return Err(GraphicsError::Asset(
                    ShaderSourcePreparationFailure::MissingDependency {
                        locator: uri.to_string(),
                    }
                    .to_string(),
                ));
            }
            None => {
                fallback_report = Some(missing_shader_readiness_report(reference));
                self.load_fallback_shader()?
            }
        };
        let (shader_id, revision, shader) = if shader.runtime_wgsl_source().is_some() {
            (shader_id, revision, shader)
        } else if depth > 0 {
            return Err(GraphicsError::Asset(
                ShaderSourcePreparationFailure::MissingDependency {
                    locator: format!("{uri}: runtime WGSL source is unavailable"),
                }
                .to_string(),
            ));
        } else {
            fallback_report = Some(missing_runtime_shader_readiness_report(reference));
            self.load_fallback_shader()?
        };
        let dependency_identity =
            readiness_generation
                .row_identity(shader_id)
                .ok_or_else(|| {
                    GraphicsError::Asset(
                        ShaderSourcePreparationFailure::MissingDependency {
                            locator: shader_id.to_string(),
                        }
                        .to_string(),
                    )
                })?;

        let is_new = traversal
            .try_enter(shader_id, depth)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        if !is_new {
            return Ok((shader_id, revision, dependency_identity, fallback_report));
        }
        let result = (|| {
            let import_dependencies = shader
                .imports
                .iter()
                .filter_map(|import| import.redirect.clone())
                .collect::<Vec<_>>();
            if self.shaders.get(&shader_id).is_some_and(|prepared| {
                shader_artifact_identity_is_current(
                    prepared.revision,
                    &prepared.dependency_identity,
                    revision,
                    &dependency_identity,
                )
            }) {
                for dependency in &import_dependencies {
                    let _ = self.ensure_shader_source_recursive(
                        dependency,
                        readiness_generation,
                        traversal,
                        depth.saturating_add(1),
                    )?;
                }
                self.ensure_shader_dependency_sources(
                    asset_manager.as_ref(),
                    shader_id,
                    readiness_generation,
                    traversal,
                    depth,
                )?;
                traversal.cache_hits = traversal.cache_hits.saturating_add(1);
                return Ok((shader_id, revision, dependency_identity, fallback_report));
            }
            if self.shaders.get(&shader_id).is_some_and(|prepared| {
                prepared.revision == revision && prepared.dependency_identity != dependency_identity
            }) {
                traversal.dependency_generation_invalidations = traversal
                    .dependency_generation_invalidations
                    .saturating_add(1);
            }
            traversal.rebuilds = traversal.rebuilds.saturating_add(1);
            let source_text = shader
                .runtime_wgsl_source()
                .ok_or_else(|| {
                    GraphicsError::Asset(format!(
                        "shader {} has no runtime WGSL source",
                        shader.uri
                    ))
                })?
                .to_string();
            traversal
                .add_source_bytes(source_text.len())
                .map_err(|error| GraphicsError::Asset(error.to_string()))?;
            let source = Arc::<str>::from(source_text);
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
                dependency_identity: dependency_identity.clone(),
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
                    depth.saturating_add(1),
                )?;
            }
            self.ensure_shader_dependency_sources(
                asset_manager.as_ref(),
                shader_id,
                readiness_generation,
                traversal,
                depth,
            )?;
            traversal.stage(shader_id, prepared_shader);
            crate::profile_counter!("render", "shader_artifact_publish", 1);
            Ok((shader_id, revision, dependency_identity, fallback_report))
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
        depth: usize,
    ) -> Result<(), GraphicsError> {
        let resource_manager = asset_manager.resource_manager();
        let registry = resource_manager.registry();
        let dependencies = shader_dependency_ids(asset_manager, shader_id)
            .into_iter()
            .map(|dependency_id| {
                let record = registry.get(dependency_id).ok_or_else(|| {
                    GraphicsError::Asset(
                        ShaderSourcePreparationFailure::MissingDependency {
                            locator: dependency_id.to_string(),
                        }
                        .to_string(),
                    )
                })?;
                if record.kind != ResourceKind::Shader {
                    return Err(GraphicsError::Asset(
                        ShaderSourcePreparationFailure::MissingDependency {
                            locator: record.primary_locator.to_string(),
                        }
                        .to_string(),
                    ));
                }
                Ok(AssetReference::from_locator(record.primary_locator.clone()))
            })
            .collect::<Result<Vec<_>, GraphicsError>>()?;
        for dependency in dependencies {
            self.ensure_shader_source_recursive(
                &dependency,
                readiness_generation,
                traversal,
                depth.saturating_add(1),
            )?;
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
    prepared_dependency_identity: &ResourceReadinessRowIdentity,
    requested_revision: u64,
    requested_dependency_identity: &ResourceReadinessRowIdentity,
) -> bool {
    prepared_revision == requested_revision
        && prepared_dependency_identity == requested_dependency_identity
}

#[cfg(test)]
#[path = "tests/resource_streamer_ensure_shader_source.rs"]
mod tests;

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
