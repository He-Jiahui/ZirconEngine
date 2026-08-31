use std::collections::HashMap;

use crate::graphics::shader::{
    ShaderBindingResourceType, ShaderBindingStage, ShaderBindingVisibility,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderResourceLayoutBinding {
    group: u32,
    binding: u32,
    resource_type: ShaderBindingResourceType,
    min_binding_size: Option<u64>,
    texture_filterable: Option<bool>,
    sampler_binding_type: Option<MeshShaderSamplerBindingType>,
    visibility: ShaderBindingVisibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MeshShaderSamplerBindingType {
    Filtering,
    NonFiltering,
    Comparison,
}

impl MeshShaderResourceLayoutBinding {
    pub(super) const fn new(
        group: u32,
        binding: u32,
        resource_type: ShaderBindingResourceType,
        visibility: ShaderBindingVisibility,
    ) -> Self {
        Self {
            group,
            binding,
            resource_type,
            min_binding_size: None,
            texture_filterable: None,
            sampler_binding_type: None,
            visibility,
        }
    }

    pub(super) const fn with_min_binding_size(mut self, min_binding_size: Option<u64>) -> Self {
        self.min_binding_size = min_binding_size;
        self
    }

    pub(super) const fn with_texture_filterability(mut self, filterable: Option<bool>) -> Self {
        self.texture_filterable = filterable;
        self
    }

    pub(super) const fn with_sampler_binding_type(
        mut self,
        binding_type: Option<MeshShaderSamplerBindingType>,
    ) -> Self {
        self.sampler_binding_type = binding_type;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderSamplingPairRequirement {
    texture_group: u32,
    texture_binding: u32,
    sampler_group: u32,
    sampler_binding: u32,
}

impl MeshShaderSamplingPairRequirement {
    pub(super) const fn new(
        texture_group: u32,
        texture_binding: u32,
        sampler_group: u32,
        sampler_binding: u32,
    ) -> Self {
        Self {
            texture_group,
            texture_binding,
            sampler_group,
            sampler_binding,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderResourceRequirement {
    group: u32,
    binding: u32,
    resource_type: ShaderBindingResourceType,
    min_binding_size: Option<u64>,
    stage: ShaderBindingStage,
}

impl MeshShaderResourceRequirement {
    pub(super) const fn new(
        group: u32,
        binding: u32,
        resource_type: ShaderBindingResourceType,
        stage: ShaderBindingStage,
    ) -> Self {
        Self {
            group,
            binding,
            resource_type,
            min_binding_size: None,
            stage,
        }
    }

    pub(super) const fn with_min_binding_size(mut self, min_binding_size: Option<u64>) -> Self {
        self.min_binding_size = min_binding_size;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct MeshShaderPipelineLayoutContract {
    bindings: HashMap<(u32, u32), MeshShaderResourceLayoutBinding>,
}

impl MeshShaderPipelineLayoutContract {
    pub(super) fn try_new(
        bindings: impl IntoIterator<Item = MeshShaderResourceLayoutBinding>,
    ) -> Result<Self, String> {
        let mut by_location = HashMap::new();
        for binding in bindings {
            let location = (binding.group, binding.binding);
            if by_location.insert(location, binding).is_some() {
                return Err(format!(
                    "pipeline layout declares group {} binding {} more than once",
                    binding.group, binding.binding
                ));
            }
        }
        Ok(Self {
            bindings: by_location,
        })
    }

    pub(super) fn validate_requirements(
        &self,
        requirements: &[MeshShaderResourceRequirement],
    ) -> Result<(), String> {
        for requirement in requirements {
            self.validate_requirement(*requirement)?;
        }
        Ok(())
    }

    pub(super) fn validate_requirement(
        &self,
        requirement: MeshShaderResourceRequirement,
    ) -> Result<(), String> {
        let Some(layout_binding) = self.bindings.get(&(requirement.group, requirement.binding))
        else {
            return Err(format!(
                "shader requires group {} binding {}, but pipeline layout omits it",
                requirement.group, requirement.binding
            ));
        };
        if requirement.resource_type == ShaderBindingResourceType::Unsupported
            || layout_binding.resource_type == ShaderBindingResourceType::Unsupported
        {
            return Err(format!(
                "group {} binding {} uses a resource type outside the Mesh MVP ABI",
                requirement.group, requirement.binding
            ));
        }
        if layout_binding.resource_type != requirement.resource_type {
            return Err(format!(
                "group {} binding {} resource type mismatch: shader requires {:?}, pipeline layout declares {:?}",
                requirement.group,
                requirement.binding,
                requirement.resource_type,
                layout_binding.resource_type
            ));
        }
        if let (Some(shader_minimum), Some(layout_minimum)) = (
            requirement.min_binding_size,
            layout_binding.min_binding_size,
        ) {
            if layout_minimum < shader_minimum {
                return Err(format!(
                    "group {} binding {} minimum buffer binding size is too small: shader requires {} bytes, pipeline layout declares {} bytes",
                    requirement.group, requirement.binding, shader_minimum, layout_minimum
                ));
            }
        }
        if !layout_binding.visibility.contains(requirement.stage) {
            return Err(format!(
                "group {} binding {} visibility does not include {:?}",
                requirement.group, requirement.binding, requirement.stage
            ));
        }
        Ok(())
    }

    pub(super) fn validate_sampling_pair(
        &self,
        requirement: MeshShaderSamplingPairRequirement,
    ) -> Result<(), String> {
        let Some(texture) = self
            .bindings
            .get(&(requirement.texture_group, requirement.texture_binding))
        else {
            return Err(format!(
                "shader samples missing texture group {} binding {}",
                requirement.texture_group, requirement.texture_binding
            ));
        };
        let Some(sampler) = self
            .bindings
            .get(&(requirement.sampler_group, requirement.sampler_binding))
        else {
            return Err(format!(
                "shader samples with missing sampler group {} binding {}",
                requirement.sampler_group, requirement.sampler_binding
            ));
        };
        let ShaderBindingResourceType::SampledTexture { sample_type, .. } = texture.resource_type
        else {
            return Err(format!(
                "shader sampling texture group {} binding {} is not a sampled texture",
                requirement.texture_group, requirement.texture_binding
            ));
        };
        let ShaderBindingResourceType::Sampler { comparison } = sampler.resource_type else {
            return Err(format!(
                "shader sampling sampler group {} binding {} is not a sampler",
                requirement.sampler_group, requirement.sampler_binding
            ));
        };
        let Some(sampler_binding_type) = sampler.sampler_binding_type else {
            return Err(format!(
                "sampler group {} binding {} omits its filtering operation class",
                requirement.sampler_group, requirement.sampler_binding
            ));
        };
        let binding_is_comparison =
            sampler_binding_type == MeshShaderSamplerBindingType::Comparison;
        if binding_is_comparison != comparison {
            return Err(format!(
                "sampler group {} binding {} has inconsistent comparison metadata",
                requirement.sampler_group, requirement.sampler_binding
            ));
        }
        if sampler_binding_type != MeshShaderSamplerBindingType::Filtering {
            return Ok(());
        }

        match sample_type {
            crate::graphics::shader::ShaderTextureSampleType::Float => {
                match texture.texture_filterable {
                    Some(true) => Ok(()),
                    Some(false) => Err(format!(
                        "filtering sampler group {} binding {} samples non-filterable float texture group {} binding {}",
                        requirement.sampler_group,
                        requirement.sampler_binding,
                        requirement.texture_group,
                        requirement.texture_binding
                    )),
                    None => Err(format!(
                        "float texture group {} binding {} omits its filterability class",
                        requirement.texture_group, requirement.texture_binding
                    )),
                }
            }
            crate::graphics::shader::ShaderTextureSampleType::Sint
            | crate::graphics::shader::ShaderTextureSampleType::Uint => Err(format!(
                "filtering sampler group {} binding {} samples integer texture group {} binding {}",
                requirement.sampler_group,
                requirement.sampler_binding,
                requirement.texture_group,
                requirement.texture_binding
            )),
            crate::graphics::shader::ShaderTextureSampleType::Depth => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::shader::{ShaderTextureSampleType, ShaderTextureViewDimension};

    #[test]
    fn required_resources_are_a_subset_of_the_pipeline_layout() {
        let contract = MeshShaderPipelineLayoutContract::try_new([
            layout_binding(
                0,
                0,
                ShaderBindingResourceType::UniformBuffer,
                &[ShaderBindingStage::Vertex, ShaderBindingStage::Fragment],
            ),
            layout_binding(
                0,
                1,
                sampled_texture(ShaderTextureViewDimension::D2),
                &[ShaderBindingStage::Fragment],
            ),
        ])
        .unwrap();

        assert!(
            contract
                .validate_requirements(&[requirement(
                    0,
                    0,
                    ShaderBindingResourceType::UniformBuffer,
                    ShaderBindingStage::Vertex,
                )])
                .is_ok()
        );
    }

    #[test]
    fn duplicate_and_missing_bindings_are_rejected() {
        let duplicate = layout_binding(
            2,
            4,
            ShaderBindingResourceType::Sampler { comparison: false },
            &[ShaderBindingStage::Fragment],
        );
        assert!(MeshShaderPipelineLayoutContract::try_new([duplicate, duplicate]).is_err());

        let contract = MeshShaderPipelineLayoutContract::try_new([]).unwrap();
        assert!(
            contract
                .validate_requirements(&[requirement(
                    3,
                    9,
                    ShaderBindingResourceType::StorageBuffer { read_only: true },
                    ShaderBindingStage::Vertex,
                )])
                .unwrap_err()
                .contains("group 3 binding 9")
        );
    }

    #[test]
    fn resource_type_and_visibility_mismatches_are_rejected() {
        let contract = MeshShaderPipelineLayoutContract::try_new([
            layout_binding(
                1,
                2,
                ShaderBindingResourceType::Sampler { comparison: false },
                &[ShaderBindingStage::Fragment],
            ),
            layout_binding(
                2,
                0,
                ShaderBindingResourceType::UniformBuffer,
                &[ShaderBindingStage::Fragment],
            ),
        ])
        .unwrap();

        assert!(
            contract
                .validate_requirements(&[requirement(
                    1,
                    2,
                    ShaderBindingResourceType::Sampler { comparison: true },
                    ShaderBindingStage::Fragment,
                )])
                .unwrap_err()
                .contains("resource type mismatch")
        );
        assert!(
            contract
                .validate_requirements(&[requirement(
                    2,
                    0,
                    ShaderBindingResourceType::UniformBuffer,
                    ShaderBindingStage::Vertex,
                )])
                .unwrap_err()
                .contains("visibility")
        );
    }

    #[test]
    fn texture_dimension_storage_access_and_unsupported_resources_are_exact() {
        let contract = MeshShaderPipelineLayoutContract::try_new([
            layout_binding(
                1,
                5,
                sampled_texture(ShaderTextureViewDimension::Cube),
                &[ShaderBindingStage::Fragment],
            ),
            layout_binding(
                3,
                1,
                ShaderBindingResourceType::StorageBuffer { read_only: true },
                &[ShaderBindingStage::Vertex],
            ),
            layout_binding(
                4,
                7,
                ShaderBindingResourceType::Unsupported,
                &[ShaderBindingStage::Fragment],
            ),
        ])
        .unwrap();

        assert!(
            contract
                .validate_requirements(&[requirement(
                    1,
                    5,
                    sampled_texture(ShaderTextureViewDimension::D2),
                    ShaderBindingStage::Fragment,
                )])
                .is_err()
        );
        assert!(
            contract
                .validate_requirements(&[requirement(
                    3,
                    1,
                    ShaderBindingResourceType::StorageBuffer { read_only: false },
                    ShaderBindingStage::Vertex,
                )])
                .is_err()
        );
        assert!(
            contract
                .validate_requirements(&[requirement(
                    4,
                    7,
                    ShaderBindingResourceType::Unsupported,
                    ShaderBindingStage::Fragment,
                )])
                .is_err()
        );
    }

    #[test]
    fn explicit_buffer_minimum_must_cover_the_shader_while_none_stays_late_bound() {
        let fixed = MeshShaderPipelineLayoutContract::try_new([layout_binding(
            0,
            0,
            ShaderBindingResourceType::UniformBuffer,
            &[ShaderBindingStage::Vertex],
        )
        .with_min_binding_size(Some(64))])
        .unwrap();
        let late_bound = MeshShaderPipelineLayoutContract::try_new([layout_binding(
            0,
            0,
            ShaderBindingResourceType::UniformBuffer,
            &[ShaderBindingStage::Vertex],
        )])
        .unwrap();
        let exact = requirement(
            0,
            0,
            ShaderBindingResourceType::UniformBuffer,
            ShaderBindingStage::Vertex,
        )
        .with_min_binding_size(Some(64));
        let too_large = exact.with_min_binding_size(Some(80));

        assert!(fixed.validate_requirement(exact).is_ok());
        assert!(
            fixed
                .validate_requirement(too_large)
                .unwrap_err()
                .contains("requires 80 bytes")
        );
        assert!(late_bound.validate_requirement(too_large).is_ok());
    }

    #[test]
    fn sampling_pairs_reject_filtering_of_nonfilterable_float_textures() {
        let filtering = MeshShaderPipelineLayoutContract::try_new([
            layout_binding(
                2,
                3,
                sampled_texture(ShaderTextureViewDimension::D2),
                &[ShaderBindingStage::Fragment],
            )
            .with_texture_filterability(Some(false)),
            layout_binding(
                2,
                4,
                ShaderBindingResourceType::Sampler { comparison: false },
                &[ShaderBindingStage::Fragment],
            )
            .with_sampler_binding_type(Some(MeshShaderSamplerBindingType::Filtering)),
        ])
        .unwrap();
        let non_filtering = MeshShaderPipelineLayoutContract::try_new([
            layout_binding(
                2,
                3,
                sampled_texture(ShaderTextureViewDimension::D2),
                &[ShaderBindingStage::Fragment],
            )
            .with_texture_filterability(Some(false)),
            layout_binding(
                2,
                4,
                ShaderBindingResourceType::Sampler { comparison: false },
                &[ShaderBindingStage::Fragment],
            )
            .with_sampler_binding_type(Some(MeshShaderSamplerBindingType::NonFiltering)),
        ])
        .unwrap();
        let pair = MeshShaderSamplingPairRequirement::new(2, 3, 2, 4);

        assert!(
            filtering
                .validate_sampling_pair(pair)
                .unwrap_err()
                .contains("non-filterable float texture")
        );
        assert!(non_filtering.validate_sampling_pair(pair).is_ok());
    }

    fn sampled_texture(view_dimension: ShaderTextureViewDimension) -> ShaderBindingResourceType {
        ShaderBindingResourceType::SampledTexture {
            view_dimension,
            sample_type: ShaderTextureSampleType::Float,
            multisampled: false,
        }
    }

    fn layout_binding(
        group: u32,
        binding: u32,
        resource_type: ShaderBindingResourceType,
        stages: &[ShaderBindingStage],
    ) -> MeshShaderResourceLayoutBinding {
        MeshShaderResourceLayoutBinding::new(
            group,
            binding,
            resource_type,
            ShaderBindingVisibility::from_stages(stages.iter().copied()),
        )
    }

    fn requirement(
        group: u32,
        binding: u32,
        resource_type: ShaderBindingResourceType,
        stage: ShaderBindingStage,
    ) -> MeshShaderResourceRequirement {
        MeshShaderResourceRequirement::new(group, binding, resource_type, stage)
    }
}
