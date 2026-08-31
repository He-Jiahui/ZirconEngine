use std::collections::BTreeMap;

use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderFeatureTextureViewAlias, RenderResourceSchema,
};
use crate::render_graph::{
    RenderGraphExternalResourceBinding, RenderGraphExternalResourceType,
    RenderGraphResourceUsageFlags,
};
use crate::rhi::TextureUsage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PipelineGraphResourcePlan {
    pub(super) kind: RenderFeatureResourceKind,
    pub(super) external_binding: RenderGraphExternalResourceBinding,
    pub(super) usage: RenderGraphResourceUsageFlags,
    pub(super) minimum_size_bytes: Option<u64>,
    pub(super) schema: Option<RenderResourceSchema>,
    pub(super) requires_storage_texture: bool,
    pub(super) texture_view_alias: Option<RenderFeatureTextureViewAlias>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PipelineGraphResourceUsage {
    kind: RenderFeatureResourceKind,
    has_read: bool,
    has_write: bool,
    explicit_external: bool,
    external_binding: RenderGraphExternalResourceBinding,
    usage: RenderGraphResourceUsageFlags,
    minimum_size_bytes: Option<u64>,
    schema: Option<RenderResourceSchema>,
    requires_storage_texture: bool,
    texture_view_alias: Option<RenderFeatureTextureViewAlias>,
    error: Option<String>,
}

pub(super) fn pipeline_graph_resources(
    descriptors: &[RenderFeatureDescriptor],
) -> Result<BTreeMap<String, PipelineGraphResourcePlan>, String> {
    let mut resources = BTreeMap::<String, PipelineGraphResourceUsage>::new();
    for descriptor in descriptors {
        for pass in &descriptor.stage_passes {
            for resource in &pass.resources {
                resources
                    .entry(resource.name.clone())
                    .and_modify(|usage| {
                        usage.add_access(
                            &resource.name,
                            resource.kind,
                            resource.access,
                            resource.external_binding,
                            resource.usage,
                            resource.minimum_size_bytes,
                            resource.schema,
                            resource.write_mode,
                            resource.texture_view_alias.clone(),
                            &descriptor.name,
                            &pass.pass_name,
                        )
                    })
                    .or_insert_with(|| {
                        PipelineGraphResourceUsage::new(
                            resource.kind,
                            resource.access,
                            resource.external_binding,
                            resource.usage,
                            resource.minimum_size_bytes,
                            resource.schema,
                            resource.write_mode,
                            resource.texture_view_alias.clone(),
                        )
                    });
                if let Some(error) = resources
                    .get(&resource.name)
                    .and_then(PipelineGraphResourceUsage::take_error)
                {
                    return Err(error);
                }
            }
        }
    }

    let mut plans = BTreeMap::new();
    for (name, usage) in resources {
        plans.insert(name.clone(), usage.into_plan(&name)?);
    }
    Ok(plans)
}

impl PipelineGraphResourceUsage {
    fn new(
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        external_binding: RenderGraphExternalResourceBinding,
        usage: RenderGraphResourceUsageFlags,
        minimum_size_bytes: Option<u64>,
        schema: Option<RenderResourceSchema>,
        write_mode: RenderFeatureResourceWriteMode,
        texture_view_alias: Option<RenderFeatureTextureViewAlias>,
    ) -> Self {
        let mut usage = Self {
            kind,
            has_read: false,
            has_write: false,
            explicit_external: kind == RenderFeatureResourceKind::External,
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            usage,
            minimum_size_bytes,
            schema,
            requires_storage_texture: requires_storage_texture(kind, external_binding, write_mode),
            texture_view_alias,
            error: None,
        };
        if kind == RenderFeatureResourceKind::External
            || !matches!(
                external_binding.resource_type,
                RenderGraphExternalResourceType::Unknown
            )
        {
            usage.merge_external_binding(external_binding);
        }
        usage.record_access(access);
        usage
    }

    fn add_access(
        &mut self,
        resource_name: &str,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        external_binding: RenderGraphExternalResourceBinding,
        usage: RenderGraphResourceUsageFlags,
        minimum_size_bytes: Option<u64>,
        schema: Option<RenderResourceSchema>,
        write_mode: RenderFeatureResourceWriteMode,
        texture_view_alias: Option<RenderFeatureTextureViewAlias>,
        descriptor_name: &str,
        pass_name: &str,
    ) {
        if self.conflicts_with(kind) {
            self.error = Some(format!(
                "resource `{resource_name}` has conflicting resource kind or explicit external resource usage in feature descriptor `{descriptor_name}` pass `{pass_name}`"
            ));
            return;
        }
        if kind == RenderFeatureResourceKind::External {
            self.kind = RenderFeatureResourceKind::External;
            self.explicit_external = true;
        }
        if kind == RenderFeatureResourceKind::External
            || !matches!(
                external_binding.resource_type,
                RenderGraphExternalResourceType::Unknown
            )
        {
            if !self.merge_external_binding(external_binding) {
                self.error = Some(format!(
                    "resource `{resource_name}` has conflicting external resource binding in feature descriptor `{descriptor_name}` pass `{pass_name}`"
                ));
                return;
            }
        }
        self.merge_schema(resource_name, schema, descriptor_name, pass_name);
        self.merge_usage(usage);
        self.merge_texture_view_alias(
            resource_name,
            texture_view_alias,
            descriptor_name,
            pass_name,
        );
        self.minimum_size_bytes = match (self.minimum_size_bytes, minimum_size_bytes) {
            (Some(current), Some(incoming)) => Some(current.max(incoming)),
            (current @ Some(_), None) => current,
            (None, incoming) => incoming,
        };
        self.requires_storage_texture |=
            requires_storage_texture(kind, external_binding, write_mode);
        self.record_access(access);
    }

    fn conflicts_with(&self, kind: RenderFeatureResourceKind) -> bool {
        if self.kind == kind {
            return false;
        }

        if self.explicit_external || kind == RenderFeatureResourceKind::External {
            return true;
        }

        self.kind != RenderFeatureResourceKind::External
            && kind != RenderFeatureResourceKind::External
    }

    fn record_access(&mut self, access: RenderFeatureResourceAccess) {
        match access {
            RenderFeatureResourceAccess::Read => self.has_read = true,
            RenderFeatureResourceAccess::Write => self.has_write = true,
        }
    }

    fn take_error(&self) -> Option<String> {
        self.error.clone()
    }

    fn into_plan(self, resource_name: &str) -> Result<PipelineGraphResourcePlan, String> {
        let kind = if self.kind == RenderFeatureResourceKind::External || !self.has_write {
            RenderFeatureResourceKind::External
        } else {
            self.kind
        };
        if self.requires_storage_texture {
            if let Some(schema) = self.schema {
                let texture = schema.texture_schema().ok_or_else(|| {
                    format!(
                        "storage texture resource `{resource_name}` requires RenderResourceSchema::Texture"
                    )
                })?;
                if !texture.usage.contains(TextureUsage::STORAGE) {
                    return Err(format!(
                        "storage texture resource `{resource_name}` schema must declare STORAGE usage"
                    ));
                }
                if !texture.format.supports_write_only_storage() {
                    return Err(format!(
                        "storage texture resource `{resource_name}` schema format {:?} is unsupported for write-only storage",
                        texture.format
                    ));
                }
            }
        }
        if self.texture_view_alias.is_some() {
            if kind != RenderFeatureResourceKind::Texture {
                return Err(format!(
                    "texture view alias `{resource_name}` must be a written transient texture resource"
                ));
            }
            if self.schema.is_some() {
                return Err(format!(
                    "texture view alias `{resource_name}` derives its physical descriptor from its parent and cannot declare RenderResourceSchema"
                ));
            }
        }
        Ok(PipelineGraphResourcePlan {
            kind,
            external_binding: if kind == RenderFeatureResourceKind::External {
                self.external_binding
            } else {
                RenderGraphExternalResourceBinding::report_only()
            },
            usage: self.usage,
            minimum_size_bytes: self.minimum_size_bytes,
            schema: self.schema,
            requires_storage_texture: self.requires_storage_texture,
            texture_view_alias: self.texture_view_alias,
        })
    }

    fn merge_external_binding(&mut self, binding: RenderGraphExternalResourceBinding) -> bool {
        if matches!(
            binding.resource_type,
            RenderGraphExternalResourceType::Unknown
        ) {
            return true;
        }
        if matches!(
            self.external_binding.resource_type,
            RenderGraphExternalResourceType::Unknown
        ) || self.external_binding.resource_type == binding.resource_type
        {
            self.external_binding.resource_type = binding.resource_type;
            if binding.is_required() {
                self.external_binding.requirement = binding.requirement;
            }
            return true;
        }
        false
    }

    fn merge_schema(
        &mut self,
        resource_name: &str,
        schema: Option<RenderResourceSchema>,
        descriptor_name: &str,
        pass_name: &str,
    ) {
        let Some(schema) = schema else {
            return;
        };
        if let Some(existing) = self.schema {
            if existing != schema {
                self.error = Some(format!(
                    "resource `{resource_name}` has conflicting RenderResourceSchema in feature descriptor `{descriptor_name}` pass `{pass_name}`"
                ));
            }
            return;
        }
        self.schema = Some(schema);
    }

    fn merge_usage(&mut self, usage: RenderGraphResourceUsageFlags) {
        self.usage.present |= usage.present;
        self.usage.readback |= usage.readback;
        self.usage.persistent |= usage.persistent;
    }

    fn merge_texture_view_alias(
        &mut self,
        resource_name: &str,
        alias: Option<RenderFeatureTextureViewAlias>,
        descriptor_name: &str,
        pass_name: &str,
    ) {
        let Some(alias) = alias else {
            return;
        };
        if self.kind != RenderFeatureResourceKind::Texture {
            self.error = Some(format!(
                "resource `{resource_name}` declares a texture view alias in non-texture feature descriptor `{descriptor_name}` pass `{pass_name}`"
            ));
            return;
        }
        if let Some(existing) = &self.texture_view_alias {
            if existing != &alias {
                self.error = Some(format!(
                    "resource `{resource_name}` has conflicting texture view alias declarations in feature descriptor `{descriptor_name}` pass `{pass_name}`"
                ));
            }
            return;
        }
        self.texture_view_alias = Some(alias);
    }
}

fn requires_storage_texture(
    kind: RenderFeatureResourceKind,
    external_binding: RenderGraphExternalResourceBinding,
    write_mode: RenderFeatureResourceWriteMode,
) -> bool {
    write_mode == RenderFeatureResourceWriteMode::Storage
        && (kind == RenderFeatureResourceKind::Texture
            || (kind == RenderFeatureResourceKind::External
                && external_binding.resource_type == RenderGraphExternalResourceType::Texture))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::feature::RenderFeaturePassDescriptor;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

    #[test]
    fn resource_plan_unions_explicit_cull_root_usage_without_inferring_from_name() {
        let descriptor = RenderFeatureDescriptor::new(
            "typed-cull-root",
            Vec::new(),
            Vec::new(),
            vec![
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "terminal-write",
                    QueueLane::Graphics,
                )
                .write_present_external_texture_with_ops(
                    "test.terminal-output",
                    RenderGraphAttachmentOps::clear_store(),
                ),
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "terminal-read",
                    QueueLane::Graphics,
                )
                .read_external_texture("test.terminal-output"),
            ],
        );

        let plan =
            pipeline_graph_resources(&[descriptor]).expect("typed resource usage should aggregate");
        let terminal = plan
            .get("test.terminal-output")
            .expect("terminal external resource plan");

        assert!(terminal.usage.present);
        assert!(!terminal.usage.readback);
        assert!(!terminal.usage.persistent);
        assert_eq!(terminal.kind, RenderFeatureResourceKind::External);
        assert_eq!(
            terminal.external_binding,
            RenderGraphExternalResourceBinding::report_only_texture()
        );
    }
}
