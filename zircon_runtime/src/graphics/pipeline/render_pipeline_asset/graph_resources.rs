use std::collections::BTreeMap;

use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use crate::render_graph::{RenderGraphExternalResourceBinding, RenderGraphExternalResourceType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PipelineGraphResourcePlan {
    pub(super) kind: RenderFeatureResourceKind,
    pub(super) external_binding: RenderGraphExternalResourceBinding,
    pub(super) minimum_size_bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PipelineGraphResourceUsage {
    kind: RenderFeatureResourceKind,
    has_read: bool,
    has_write: bool,
    explicit_external: bool,
    external_binding: RenderGraphExternalResourceBinding,
    minimum_size_bytes: Option<u64>,
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
                            resource.minimum_size_bytes,
                            &descriptor.name,
                            &pass.pass_name,
                        )
                    })
                    .or_insert_with(|| {
                        PipelineGraphResourceUsage::new(
                            resource.kind,
                            resource.access,
                            resource.external_binding,
                            resource.minimum_size_bytes,
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

    Ok(resources
        .into_iter()
        .map(|(name, usage)| (name, usage.into_plan()))
        .collect())
}

impl PipelineGraphResourceUsage {
    fn new(
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        external_binding: RenderGraphExternalResourceBinding,
        minimum_size_bytes: Option<u64>,
    ) -> Self {
        let mut usage = Self {
            kind,
            has_read: false,
            has_write: false,
            explicit_external: kind == RenderFeatureResourceKind::External,
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            minimum_size_bytes,
            error: None,
        };
        if kind == RenderFeatureResourceKind::External {
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
        minimum_size_bytes: Option<u64>,
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
            if !self.merge_external_binding(external_binding) {
                self.error = Some(format!(
                    "resource `{resource_name}` has conflicting external resource binding in feature descriptor `{descriptor_name}` pass `{pass_name}`"
                ));
                return;
            }
        }
        self.minimum_size_bytes = match (self.minimum_size_bytes, minimum_size_bytes) {
            (Some(current), Some(incoming)) => Some(current.max(incoming)),
            (current @ Some(_), None) => current,
            (None, incoming) => incoming,
        };
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

    fn into_plan(self) -> PipelineGraphResourcePlan {
        let kind = if self.kind == RenderFeatureResourceKind::External || !self.has_write {
            RenderFeatureResourceKind::External
        } else {
            self.kind
        };
        PipelineGraphResourcePlan {
            kind,
            external_binding: if kind == RenderFeatureResourceKind::External {
                self.external_binding
            } else {
                RenderGraphExternalResourceBinding::report_only()
            },
            minimum_size_bytes: self.minimum_size_bytes,
        }
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
}
