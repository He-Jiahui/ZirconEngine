use super::ShaderSamplingPairIdentity;

pub(super) fn reflect_entry_sampling_pairs(
    module: &naga::Module,
    entry_info: &naga::valid::FunctionInfo,
) -> Vec<ShaderSamplingPairIdentity> {
    let mut pairs = entry_info
        .sampling_set
        .iter()
        .map(|pair| {
            let texture = module.global_variables[pair.image]
                .binding
                .expect("validated sampling texture must have a resource binding");
            let sampler = module.global_variables[pair.sampler]
                .binding
                .expect("validated sampling sampler must have a resource binding");
            ShaderSamplingPairIdentity {
                texture_group: texture.group,
                texture_binding: texture.binding,
                sampler_group: sampler.group,
                sampler_binding: sampler.binding,
            }
        })
        .collect::<Vec<_>>();
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}
