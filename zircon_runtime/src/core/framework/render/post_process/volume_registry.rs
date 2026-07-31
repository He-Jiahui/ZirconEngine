use std::collections::BTreeSet;

use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings,
    RenderPostProcessEffectStackSettings,
};

use super::resolved_stack::RenderResolvedPostProcessSettings;
use super::volume_component::{
    BUILTIN_POST_PROCESS_VOLUME_COMPONENTS, VolumeComponentApplyError, VolumeComponentDescriptor,
};

#[derive(Clone, Debug, Default)]
pub struct VolumeComponentRegistry {
    descriptors: Vec<VolumeComponentDescriptor>,
}

impl VolumeComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtin_post_process_components() -> Self {
        let mut registry = Self::new();
        registry
            .register_builtin_post_process_components()
            .expect("built-in post-process volume component ids and params must be unique");
        registry
    }

    pub fn register_builtin_post_process_components(&mut self) -> Result<(), VolumeRegistryError> {
        for descriptor in BUILTIN_POST_PROCESS_VOLUME_COMPONENTS {
            self.register(*descriptor)?;
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        descriptor: VolumeComponentDescriptor,
    ) -> Result<(), VolumeRegistryError> {
        validate_descriptor(descriptor)?;
        if self
            .descriptors
            .iter()
            .any(|registered| registered.component_id == descriptor.component_id)
        {
            return Err(VolumeRegistryError::DuplicateComponentId {
                component_id: descriptor.component_id.to_string(),
            });
        }

        self.descriptors.push(descriptor);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &VolumeComponentDescriptor> {
        self.descriptors.iter()
    }

    pub fn contains(&self, component_id: &str) -> bool {
        self.get(component_id).is_some()
    }

    pub fn get(&self, component_id: &str) -> Option<&VolumeComponentDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == component_id)
    }

    pub fn default_resolved_post_process_settings(
        &self,
    ) -> Result<RenderResolvedPostProcessSettings, VolumeComponentApplyError> {
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings::default(),
            RenderExposureSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );
        for descriptor in &self.descriptors {
            descriptor.apply_defaults(&mut settings)?;
        }
        Ok(settings)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VolumeRegistryError {
    EmptyComponentId,
    DuplicateComponentId {
        component_id: String,
    },
    EmptyParamName {
        component_id: String,
    },
    DuplicateParamName {
        component_id: String,
        param_name: String,
    },
}

fn validate_descriptor(descriptor: VolumeComponentDescriptor) -> Result<(), VolumeRegistryError> {
    if descriptor.component_id.trim().is_empty() {
        return Err(VolumeRegistryError::EmptyComponentId);
    }

    let mut param_names = BTreeSet::new();
    for param in descriptor.params {
        if param.name.trim().is_empty() {
            return Err(VolumeRegistryError::EmptyParamName {
                component_id: descriptor.component_id.to_string(),
            });
        }
        if !param_names.insert(param.name) {
            return Err(VolumeRegistryError::DuplicateParamName {
                component_id: descriptor.component_id.to_string(),
                param_name: param.name.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderBloomSettings, RenderColorGradingSettings, RenderPostProcessEffectStackSettings,
    };

    use super::super::resolved_stack::RenderResolvedPostProcessSettings;
    use super::super::volume_component::{
        BUILTIN_POST_PROCESS_VOLUME_COMPONENTS, VolumeComponentDescriptor, VolumeParamSchema,
        VolumeParamValue, interp_float_lerp,
    };
    use super::{VolumeComponentRegistry, VolumeRegistryError};

    const INVALID_PARAM_NAME_PARAMS: [VolumeParamSchema; 1] = [VolumeParamSchema::new(
        "",
        VolumeParamValue::Float(0.0),
        interp_float_lerp,
    )];
    const DUPLICATE_PARAM_NAME_PARAMS: [VolumeParamSchema; 2] = [
        VolumeParamSchema::new("value", VolumeParamValue::Float(0.0), interp_float_lerp),
        VolumeParamSchema::new("value", VolumeParamValue::Float(1.0), interp_float_lerp),
    ];

    fn read_empty(_settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
        Vec::new()
    }

    fn apply_ok(
        _settings: &mut RenderResolvedPostProcessSettings,
        _component_id: &'static str,
        _values: &[VolumeParamValue],
    ) -> Result<(), super::super::volume_component::VolumeComponentApplyError> {
        Ok(())
    }

    #[test]
    fn render_volume_registry_exposes_builtin_post_process_components() {
        let registry = VolumeComponentRegistry::with_builtin_post_process_components();
        let expected_ids = [
            "lighting.volumetric-fog",
            "post.depth-of-field",
            "post.motion-blur",
            "post.bloom",
            "post.exposure",
            "post.screen-space-reflection",
            "post.screen-space-fog",
            "post.color-grading",
            "post.tonemap",
            "post.vignette",
            "post.grain",
            "post.dither",
            "post.chromatic-aberration",
            "post.color-lookup",
            "post.blur",
        ];

        assert_eq!(registry.len(), expected_ids.len());
        for component_id in expected_ids {
            let descriptor = registry
                .get(component_id)
                .unwrap_or_else(|| panic!("missing built-in volume component `{component_id}`"));
            assert!(
                !descriptor.params.is_empty(),
                "component `{component_id}` should expose at least one parameter"
            );
        }
    }

    #[test]
    fn render_volume_registry_default_stack_matches_existing_defaults() {
        let registry = VolumeComponentRegistry::with_builtin_post_process_components();

        let settings = registry.default_resolved_post_process_settings().unwrap();

        assert_eq!(settings.bloom, RenderBloomSettings::default());
        assert_eq!(
            settings.color_grading,
            RenderColorGradingSettings::default()
        );
        assert_eq!(
            settings.effect_stack,
            RenderPostProcessEffectStackSettings::default()
        );
    }

    #[test]
    fn render_volume_registry_rejects_duplicate_component_id() {
        let depth_of_field = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
            .iter()
            .find(|descriptor| descriptor.component_id == "post.depth-of-field")
            .copied()
            .expect("depth-of-field must remain a built-in volume component");
        let mut registry = VolumeComponentRegistry::new();
        registry.register(depth_of_field).unwrap();

        assert_eq!(
            registry.register(depth_of_field),
            Err(VolumeRegistryError::DuplicateComponentId {
                component_id: "post.depth-of-field".to_string(),
            })
        );
    }

    #[test]
    fn render_volume_registry_rejects_invalid_component_or_param_names() {
        let mut registry = VolumeComponentRegistry::new();

        assert_eq!(
            registry.register(VolumeComponentDescriptor::new(
                "",
                &[],
                read_empty,
                apply_ok
            )),
            Err(VolumeRegistryError::EmptyComponentId)
        );

        assert_eq!(
            registry.register(VolumeComponentDescriptor::new(
                "post.invalid",
                &INVALID_PARAM_NAME_PARAMS,
                read_empty,
                apply_ok,
            )),
            Err(VolumeRegistryError::EmptyParamName {
                component_id: "post.invalid".to_string(),
            })
        );
    }

    #[test]
    fn render_volume_registry_rejects_duplicate_param_names() {
        let mut registry = VolumeComponentRegistry::new();

        assert_eq!(
            registry.register(VolumeComponentDescriptor::new(
                "post.duplicate-param",
                &DUPLICATE_PARAM_NAME_PARAMS,
                read_empty,
                apply_ok,
            )),
            Err(VolumeRegistryError::DuplicateParamName {
                component_id: "post.duplicate-param".to_string(),
                param_name: "value".to_string(),
            })
        );
    }
}
