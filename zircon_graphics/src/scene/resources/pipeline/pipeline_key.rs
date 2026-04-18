use zircon_resource::ResourceId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) shader_id: ResourceId,
    pub(crate) shader_revision: u64,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
}
