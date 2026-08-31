use crate::graphics::shader::template::{
    ShaderFragmentOutputNumericType, ShaderTemplateReflection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderFragmentTargetRequirement {
    location: u32,
    numeric_type: ShaderFragmentOutputNumericType,
}

impl MeshShaderFragmentTargetRequirement {
    pub(super) const fn new(location: u32, numeric_type: ShaderFragmentOutputNumericType) -> Self {
        Self {
            location,
            numeric_type,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MeshShaderFragmentOutputContract {
    targets: Vec<MeshShaderFragmentTargetRequirement>,
}

impl MeshShaderFragmentOutputContract {
    pub(super) fn new(
        targets: impl IntoIterator<Item = MeshShaderFragmentTargetRequirement>,
    ) -> Result<Self, String> {
        let mut targets = targets.into_iter().collect::<Vec<_>>();
        targets.sort_unstable_by_key(|target| target.location);
        if let Some(duplicate) = targets
            .windows(2)
            .find(|targets| targets[0].location == targets[1].location)
        {
            return Err(format!(
                "duplicate Mesh fragment target @location({})",
                duplicate[0].location
            ));
        }
        Ok(Self { targets })
    }

    pub(super) fn validate(
        &self,
        reflection: &ShaderTemplateReflection,
        fragment_entry_name: &str,
    ) -> Result<(), String> {
        reflection.validate_fragment_output_stage_interface(fragment_entry_name, |location| {
            self.numeric_type(location)
        })
    }

    pub(super) fn numeric_type(&self, location: u32) -> Option<ShaderFragmentOutputNumericType> {
        self.targets
            .binary_search_by_key(&location, |target| target.location)
            .ok()
            .map(|index| self.targets[index].numeric_type)
    }
}
