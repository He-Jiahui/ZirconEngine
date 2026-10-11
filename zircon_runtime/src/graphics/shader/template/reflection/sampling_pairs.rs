use super::ShaderSamplingPairIdentity;

// 只保留该入口实际使用的纹理/采样器对；布局哈希依赖这组关系，不能仅凭声明列表推断。
// module 与 entry_info 必须来自同一次 Naga 验证；sampling_set 的 image/sampler global 在该前提下才可按 binding 解析。
// 排序和去重使等价配对的枚举顺序不影响资源 hash。
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
