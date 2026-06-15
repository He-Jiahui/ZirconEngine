use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureMode, RenderExposureSettings,
};
use crate::core::math::{Real, Vec3};

use super::effect_stack_settings::{
    RenderBlurSettings, RenderChromaticAberrationSettings, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderDepthOfFieldSettings, RenderDitherSettings,
    RenderFilmGrainSettings, RenderFogSettings, RenderMotionBlurSettings,
    RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
    RenderVignetteSettings,
};
use super::resolved_stack::RenderResolvedPostProcessSettings;

pub type VolumeParamInterpFn =
    fn(from: VolumeParamValue, to: VolumeParamValue, weight: Real) -> VolumeParamValue;
pub type VolumeComponentReadFn =
    fn(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue>;
pub type VolumeComponentApplyFn = fn(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeParamType {
    Float,
    Vec3,
    Bool,
    Uint,
    Enum,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VolumeParamValue {
    Float(Real),
    Vec3(Vec3),
    Bool(bool),
    Uint(u32),
    Enum(u32),
}

impl VolumeParamValue {
    pub const fn param_type(self) -> VolumeParamType {
        match self {
            Self::Float(_) => VolumeParamType::Float,
            Self::Vec3(_) => VolumeParamType::Vec3,
            Self::Bool(_) => VolumeParamType::Bool,
            Self::Uint(_) => VolumeParamType::Uint,
            Self::Enum(_) => VolumeParamType::Enum,
        }
    }

    fn float(
        self,
        component_id: &'static str,
        param_name: &'static str,
    ) -> Result<Real, VolumeComponentApplyError> {
        match self {
            Self::Float(value) => Ok(value),
            other => Err(type_mismatch(
                component_id,
                param_name,
                VolumeParamType::Float,
                other,
            )),
        }
    }

    fn vec3(
        self,
        component_id: &'static str,
        param_name: &'static str,
    ) -> Result<Vec3, VolumeComponentApplyError> {
        match self {
            Self::Vec3(value) => Ok(value),
            other => Err(type_mismatch(
                component_id,
                param_name,
                VolumeParamType::Vec3,
                other,
            )),
        }
    }

    fn uint(
        self,
        component_id: &'static str,
        param_name: &'static str,
    ) -> Result<u32, VolumeComponentApplyError> {
        match self {
            Self::Uint(value) => Ok(value),
            other => Err(type_mismatch(
                component_id,
                param_name,
                VolumeParamType::Uint,
                other,
            )),
        }
    }

    fn enum_id(
        self,
        component_id: &'static str,
        param_name: &'static str,
    ) -> Result<u32, VolumeComponentApplyError> {
        match self {
            Self::Enum(value) => Ok(value),
            other => Err(type_mismatch(
                component_id,
                param_name,
                VolumeParamType::Enum,
                other,
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VolumeParamSchema {
    pub name: &'static str,
    pub default: VolumeParamValue,
    pub interp: VolumeParamInterpFn,
}

impl VolumeParamSchema {
    pub const fn new(
        name: &'static str,
        default: VolumeParamValue,
        interp: VolumeParamInterpFn,
    ) -> Self {
        Self {
            name,
            default,
            interp,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VolumeComponentDescriptor {
    pub component_id: &'static str,
    pub params: &'static [VolumeParamSchema],
    pub read: VolumeComponentReadFn,
    pub apply: VolumeComponentApplyFn,
}

impl VolumeComponentDescriptor {
    pub const fn new(
        component_id: &'static str,
        params: &'static [VolumeParamSchema],
        read: VolumeComponentReadFn,
        apply: VolumeComponentApplyFn,
    ) -> Self {
        Self {
            component_id,
            params,
            read,
            apply,
        }
    }

    pub fn default_values(self) -> Vec<VolumeParamValue> {
        self.params.iter().map(|param| param.default).collect()
    }

    pub fn read_values(
        self,
        settings: &RenderResolvedPostProcessSettings,
    ) -> Vec<VolumeParamValue> {
        (self.read)(settings)
    }

    pub fn apply_defaults(
        self,
        settings: &mut RenderResolvedPostProcessSettings,
    ) -> Result<(), VolumeComponentApplyError> {
        let values = self.default_values();
        self.apply_values(settings, &values)
    }

    pub fn apply_values(
        self,
        settings: &mut RenderResolvedPostProcessSettings,
        values: &[VolumeParamValue],
    ) -> Result<(), VolumeComponentApplyError> {
        if values.len() != self.params.len() {
            return Err(VolumeComponentApplyError::ParamCountMismatch {
                component_id: self.component_id,
                expected: self.params.len(),
                actual: values.len(),
            });
        }

        (self.apply)(settings, self.component_id, values)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VolumeComponentApplyError {
    ParamCountMismatch {
        component_id: &'static str,
        expected: usize,
        actual: usize,
    },
    ParamTypeMismatch {
        component_id: &'static str,
        param_name: &'static str,
        expected: VolumeParamType,
        actual: VolumeParamType,
    },
}

pub const BUILTIN_POST_PROCESS_VOLUME_COMPONENTS: &[VolumeComponentDescriptor] =
    &BUILTIN_POST_PROCESS_VOLUME_COMPONENTS_ARRAY;

const BLOOM_PARAMS: [VolumeParamSchema; 3] = [
    float_param("threshold", 1.0),
    float_param("intensity", 0.0),
    float_param("radius", 0.0),
];

const DEPTH_OF_FIELD_PARAMS: [VolumeParamSchema; 7] = [
    float_param("focus_distance", 10.0),
    float_param("focus_range", 3.0),
    float_param("aperture", 0.0),
    float_param("focal_length_mm", 50.0),
    float_param("max_blur_radius", 0.0),
    uint_param("bokeh_blade_count", 6),
    float_param("bokeh_rotation_radians", 0.0),
];

const MOTION_BLUR_PARAMS: [VolumeParamSchema; 2] =
    [float_param("shutter_angle", 0.0), uint_param("samples", 0)];

const EXPOSURE_PARAMS: [VolumeParamSchema; 9] = [
    enum_param("mode", 0),
    float_param(
        "manual_ev100",
        crate::core::framework::render::DEFAULT_CAMERA_EXPOSURE_EV100,
    ),
    float_param("compensation_ev", 0.0),
    float_param("min_ev100", -8.0),
    float_param("max_ev100", 8.0),
    float_param("low_percent", 0.10),
    float_param("high_percent", 0.90),
    float_param("speed_brighten", 3.0),
    float_param("speed_darken", 1.0),
];

const SCREEN_SPACE_REFLECTION_PARAMS: [VolumeParamSchema; 6] = [
    float_param("intensity", 0.0),
    float_param("thickness", 0.1),
    float_param("max_ray_distance", 50.0),
    uint_param("max_steps", 0),
    float_param("temporal_blend_factor", 0.9),
    float_param("roughness_mip_bias", 0.0),
];

const SCREEN_SPACE_FOG_PARAMS: [VolumeParamSchema; 3] = [
    float_param("density", 0.0),
    float_param("height_falloff", 0.0),
    vec3_param("color", Vec3::ONE),
];

const COLOR_GRADING_PARAMS: [VolumeParamSchema; 5] = [
    float_param("exposure", 1.0),
    float_param("contrast", 1.0),
    float_param("saturation", 1.0),
    float_param("gamma", 1.0),
    vec3_param("tint", Vec3::ONE),
];

const TONEMAP_PARAMS: [VolumeParamSchema; 3] = [
    enum_param("operator", 0),
    float_param("exposure_bias", 0.0),
    float_param("white_point", 1.0),
];

const VIGNETTE_PARAMS: [VolumeParamSchema; 3] = [
    float_param("intensity", 0.0),
    float_param("smoothness", 0.5),
    float_param("roundness", 1.0),
];

const GRAIN_PARAMS: [VolumeParamSchema; 2] =
    [float_param("intensity", 0.0), float_param("response", 1.0)];

const DITHER_PARAMS: [VolumeParamSchema; 2] =
    [float_param("intensity", 0.0), float_param("scale", 1.0)];

const CHROMATIC_ABERRATION_PARAMS: [VolumeParamSchema; 2] = [
    float_param("intensity", 0.0),
    float_param("sample_spread", 1.0),
];

const COLOR_LOOKUP_PARAMS: [VolumeParamSchema; 3] = [
    enum_param("texture_layout", 0),
    uint_param("texture_size", 0),
    float_param("intensity", 0.0),
];

const BLUR_PARAMS: [VolumeParamSchema; 1] = [float_param("radius", 0.0)];

const BUILTIN_POST_PROCESS_VOLUME_COMPONENTS_ARRAY: [VolumeComponentDescriptor; 14] = [
    VolumeComponentDescriptor::new(
        "post.depth-of-field",
        &DEPTH_OF_FIELD_PARAMS,
        read_depth_of_field,
        apply_depth_of_field,
    ),
    VolumeComponentDescriptor::new(
        "post.motion-blur",
        &MOTION_BLUR_PARAMS,
        read_motion_blur,
        apply_motion_blur,
    ),
    VolumeComponentDescriptor::new("post.bloom", &BLOOM_PARAMS, read_bloom, apply_bloom),
    VolumeComponentDescriptor::new(
        "post.exposure",
        &EXPOSURE_PARAMS,
        read_exposure,
        apply_exposure,
    ),
    VolumeComponentDescriptor::new(
        "post.screen-space-reflection",
        &SCREEN_SPACE_REFLECTION_PARAMS,
        read_screen_space_reflection,
        apply_screen_space_reflection,
    ),
    VolumeComponentDescriptor::new(
        "post.screen-space-fog",
        &SCREEN_SPACE_FOG_PARAMS,
        read_fog,
        apply_fog,
    ),
    VolumeComponentDescriptor::new(
        "post.color-grading",
        &COLOR_GRADING_PARAMS,
        read_color_grading,
        apply_color_grading,
    ),
    VolumeComponentDescriptor::new("post.tonemap", &TONEMAP_PARAMS, read_tonemap, apply_tonemap),
    VolumeComponentDescriptor::new(
        "post.vignette",
        &VIGNETTE_PARAMS,
        read_vignette,
        apply_vignette,
    ),
    VolumeComponentDescriptor::new("post.grain", &GRAIN_PARAMS, read_grain, apply_grain),
    VolumeComponentDescriptor::new("post.dither", &DITHER_PARAMS, read_dither, apply_dither),
    VolumeComponentDescriptor::new(
        "post.chromatic-aberration",
        &CHROMATIC_ABERRATION_PARAMS,
        read_chromatic_aberration,
        apply_chromatic_aberration,
    ),
    VolumeComponentDescriptor::new(
        "post.color-lookup",
        &COLOR_LOOKUP_PARAMS,
        read_color_lookup,
        apply_color_lookup,
    ),
    VolumeComponentDescriptor::new("post.blur", &BLUR_PARAMS, read_blur, apply_blur),
];

pub fn interp_float_lerp(
    from: VolumeParamValue,
    to: VolumeParamValue,
    weight: Real,
) -> VolumeParamValue {
    match (from, to) {
        (VolumeParamValue::Float(from), VolumeParamValue::Float(to)) => {
            VolumeParamValue::Float(lerp(from, to, weight))
        }
        _ => interp_discrete(from, to, weight),
    }
}

pub fn interp_vec3_lerp(
    from: VolumeParamValue,
    to: VolumeParamValue,
    weight: Real,
) -> VolumeParamValue {
    match (from, to) {
        (VolumeParamValue::Vec3(from), VolumeParamValue::Vec3(to)) => {
            VolumeParamValue::Vec3(from + (to - from) * weight)
        }
        _ => interp_discrete(from, to, weight),
    }
}

pub fn interp_discrete(
    from: VolumeParamValue,
    to: VolumeParamValue,
    weight: Real,
) -> VolumeParamValue {
    if weight >= 0.5 {
        to
    } else {
        from
    }
}

pub fn interp_bool(from: VolumeParamValue, to: VolumeParamValue, weight: Real) -> VolumeParamValue {
    interp_discrete(from, to, weight)
}

const fn float_param(name: &'static str, default: Real) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Float(default), interp_float_lerp)
}

const fn vec3_param(name: &'static str, default: Vec3) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Vec3(default), interp_vec3_lerp)
}

const fn uint_param(name: &'static str, default: u32) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Uint(default), interp_discrete)
}

const fn enum_param(name: &'static str, default: u32) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Enum(default), interp_discrete)
}

fn read_bloom(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    vec![
        VolumeParamValue::Float(settings.bloom.threshold),
        VolumeParamValue::Float(settings.bloom.intensity),
        VolumeParamValue::Float(settings.bloom.radius),
    ]
}

fn read_depth_of_field(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.depth_of_field;
    vec![
        VolumeParamValue::Float(value.focus_distance),
        VolumeParamValue::Float(value.focus_range),
        VolumeParamValue::Float(value.aperture),
        VolumeParamValue::Float(value.focal_length_mm),
        VolumeParamValue::Float(value.max_blur_radius),
        VolumeParamValue::Uint(value.bokeh_blade_count),
        VolumeParamValue::Float(value.bokeh_rotation_radians),
    ]
}

fn read_motion_blur(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.motion_blur;
    vec![
        VolumeParamValue::Float(value.shutter_angle),
        VolumeParamValue::Uint(value.samples),
    ]
}

fn read_exposure(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.exposure;
    vec![
        VolumeParamValue::Enum(value.mode.volume_id()),
        VolumeParamValue::Float(value.manual_ev100),
        VolumeParamValue::Float(value.compensation_ev),
        VolumeParamValue::Float(value.min_ev100),
        VolumeParamValue::Float(value.max_ev100),
        VolumeParamValue::Float(value.low_percent),
        VolumeParamValue::Float(value.high_percent),
        VolumeParamValue::Float(value.speed_brighten),
        VolumeParamValue::Float(value.speed_darken),
    ]
}

fn read_screen_space_reflection(
    settings: &RenderResolvedPostProcessSettings,
) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.screen_space_reflection;
    vec![
        VolumeParamValue::Float(value.intensity),
        VolumeParamValue::Float(value.thickness),
        VolumeParamValue::Float(value.max_ray_distance),
        VolumeParamValue::Uint(value.max_steps),
        VolumeParamValue::Float(value.temporal_blend_factor),
        VolumeParamValue::Float(value.roughness_mip_bias),
    ]
}

fn read_fog(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.fog;
    vec![
        VolumeParamValue::Float(value.density),
        VolumeParamValue::Float(value.height_falloff),
        VolumeParamValue::Vec3(value.color),
    ]
}

fn read_color_grading(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    vec![
        VolumeParamValue::Float(settings.color_grading.exposure),
        VolumeParamValue::Float(settings.color_grading.contrast),
        VolumeParamValue::Float(settings.color_grading.saturation),
        VolumeParamValue::Float(settings.color_grading.gamma),
        VolumeParamValue::Vec3(settings.color_grading.tint),
    ]
}

fn read_tonemap(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.tonemap;
    vec![
        VolumeParamValue::Enum(tonemap_operator_id(value.operator)),
        VolumeParamValue::Float(value.exposure_bias),
        VolumeParamValue::Float(value.white_point),
    ]
}

fn read_vignette(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.vignette;
    vec![
        VolumeParamValue::Float(value.intensity),
        VolumeParamValue::Float(value.smoothness),
        VolumeParamValue::Float(value.roundness),
    ]
}

fn read_grain(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.grain;
    vec![
        VolumeParamValue::Float(value.intensity),
        VolumeParamValue::Float(value.response),
    ]
}

fn read_dither(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.dither;
    vec![
        VolumeParamValue::Float(value.intensity),
        VolumeParamValue::Float(value.scale),
    ]
}

fn read_chromatic_aberration(
    settings: &RenderResolvedPostProcessSettings,
) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.chromatic_aberration;
    vec![
        VolumeParamValue::Float(value.intensity),
        VolumeParamValue::Float(value.sample_spread),
    ]
}

fn read_color_lookup(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    let value = settings.effect_stack.color_lookup;
    let (layout_id, size) = color_lookup_layout_ids(value.texture_layout);
    vec![
        VolumeParamValue::Enum(layout_id),
        VolumeParamValue::Uint(size),
        VolumeParamValue::Float(value.intensity),
    ]
}

fn read_blur(settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
    vec![VolumeParamValue::Float(settings.effect_stack.blur.radius)]
}

fn apply_bloom(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.bloom = RenderBloomSettings {
        threshold: values[0].float(component_id, "threshold")?,
        intensity: values[1].float(component_id, "intensity")?,
        radius: values[2].float(component_id, "radius")?,
    };
    Ok(())
}

fn apply_depth_of_field(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.depth_of_field = RenderDepthOfFieldSettings {
        focus_distance: values[0].float(component_id, "focus_distance")?,
        focus_range: values[1].float(component_id, "focus_range")?,
        aperture: values[2].float(component_id, "aperture")?,
        focal_length_mm: values[3].float(component_id, "focal_length_mm")?,
        max_blur_radius: values[4].float(component_id, "max_blur_radius")?,
        bokeh_blade_count: values[5].uint(component_id, "bokeh_blade_count")?,
        bokeh_rotation_radians: values[6].float(component_id, "bokeh_rotation_radians")?,
    };
    Ok(())
}

fn apply_motion_blur(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.motion_blur = RenderMotionBlurSettings {
        shutter_angle: values[0].float(component_id, "shutter_angle")?,
        samples: values[1].uint(component_id, "samples")?,
    };
    Ok(())
}

fn apply_exposure(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.exposure = RenderExposureSettings {
        mode: exposure_mode_from_id(values[0].enum_id(component_id, "mode")?),
        manual_ev100: values[1].float(component_id, "manual_ev100")?,
        compensation_ev: values[2].float(component_id, "compensation_ev")?,
        min_ev100: values[3].float(component_id, "min_ev100")?,
        max_ev100: values[4].float(component_id, "max_ev100")?,
        low_percent: values[5].float(component_id, "low_percent")?,
        high_percent: values[6].float(component_id, "high_percent")?,
        speed_brighten: values[7].float(component_id, "speed_brighten")?,
        speed_darken: values[8].float(component_id, "speed_darken")?,
    };
    Ok(())
}

fn apply_screen_space_reflection(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.screen_space_reflection = RenderScreenSpaceReflectionSettings {
        intensity: values[0].float(component_id, "intensity")?,
        thickness: values[1].float(component_id, "thickness")?,
        max_ray_distance: values[2].float(component_id, "max_ray_distance")?,
        max_steps: values[3].uint(component_id, "max_steps")?,
        temporal_blend_factor: values[4].float(component_id, "temporal_blend_factor")?,
        roughness_mip_bias: values[5].float(component_id, "roughness_mip_bias")?,
    };
    Ok(())
}

fn apply_fog(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.fog = RenderFogSettings {
        density: values[0].float(component_id, "density")?,
        height_falloff: values[1].float(component_id, "height_falloff")?,
        color: values[2].vec3(component_id, "color")?,
    };
    Ok(())
}

fn apply_color_grading(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.color_grading = RenderColorGradingSettings {
        exposure: values[0].float(component_id, "exposure")?,
        contrast: values[1].float(component_id, "contrast")?,
        saturation: values[2].float(component_id, "saturation")?,
        gamma: values[3].float(component_id, "gamma")?,
        tint: values[4].vec3(component_id, "tint")?,
    };
    Ok(())
}

fn apply_tonemap(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.tonemap = RenderTonemapSettings {
        operator: tonemap_operator_from_id(values[0].enum_id(component_id, "operator")?),
        exposure_bias: values[1].float(component_id, "exposure_bias")?,
        white_point: values[2].float(component_id, "white_point")?,
    };
    Ok(())
}

fn apply_vignette(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.vignette = RenderVignetteSettings {
        intensity: values[0].float(component_id, "intensity")?,
        smoothness: values[1].float(component_id, "smoothness")?,
        roundness: values[2].float(component_id, "roundness")?,
    };
    Ok(())
}

fn apply_grain(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.grain = RenderFilmGrainSettings {
        intensity: values[0].float(component_id, "intensity")?,
        response: values[1].float(component_id, "response")?,
    };
    Ok(())
}

fn apply_dither(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.dither = RenderDitherSettings {
        intensity: values[0].float(component_id, "intensity")?,
        scale: values[1].float(component_id, "scale")?,
    };
    Ok(())
}

fn apply_chromatic_aberration(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.chromatic_aberration = RenderChromaticAberrationSettings {
        intensity: values[0].float(component_id, "intensity")?,
        sample_spread: values[1].float(component_id, "sample_spread")?,
    };
    Ok(())
}

fn apply_color_lookup(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    let layout = color_lookup_layout_from_ids(
        values[0].enum_id(component_id, "texture_layout")?,
        values[1].uint(component_id, "texture_size")?,
    );
    settings.effect_stack.color_lookup = RenderColorLookupSettings {
        texture: settings.effect_stack.color_lookup.texture,
        texture_layout: layout,
        intensity: values[2].float(component_id, "intensity")?,
    };
    Ok(())
}

fn apply_blur(
    settings: &mut RenderResolvedPostProcessSettings,
    component_id: &'static str,
    values: &[VolumeParamValue],
) -> Result<(), VolumeComponentApplyError> {
    settings.effect_stack.blur = RenderBlurSettings {
        radius: values[0].float(component_id, "radius")?,
    };
    Ok(())
}

fn type_mismatch(
    component_id: &'static str,
    param_name: &'static str,
    expected: VolumeParamType,
    value: VolumeParamValue,
) -> VolumeComponentApplyError {
    VolumeComponentApplyError::ParamTypeMismatch {
        component_id,
        param_name,
        expected,
        actual: value.param_type(),
    }
}

fn tonemap_operator_from_id(id: u32) -> RenderTonemapOperator {
    match id {
        1 => RenderTonemapOperator::Reinhard,
        2 => RenderTonemapOperator::Aces,
        3 => RenderTonemapOperator::Filmic,
        _ => RenderTonemapOperator::None,
    }
}

fn tonemap_operator_id(operator: RenderTonemapOperator) -> u32 {
    match operator {
        RenderTonemapOperator::None => 0,
        RenderTonemapOperator::Reinhard => 1,
        RenderTonemapOperator::Aces => 2,
        RenderTonemapOperator::Filmic => 3,
    }
}

fn exposure_mode_from_id(id: u32) -> RenderExposureMode {
    RenderExposureMode::from_volume_id(id)
}

fn color_lookup_layout_from_ids(layout_id: u32, size: u32) -> RenderColorLookupTextureLayout {
    match layout_id {
        1 => RenderColorLookupTextureLayout::Texture2dStrip { size },
        2 => RenderColorLookupTextureLayout::Texture3d { size },
        _ => RenderColorLookupTextureLayout::Auto,
    }
}

fn color_lookup_layout_ids(layout: RenderColorLookupTextureLayout) -> (u32, u32) {
    match layout {
        RenderColorLookupTextureLayout::Auto => (0, 0),
        RenderColorLookupTextureLayout::Texture2dStrip { size } => (1, size),
        RenderColorLookupTextureLayout::Texture3d { size } => (2, size),
    }
}

fn lerp(from: Real, to: Real, weight: Real) -> Real {
    from + (to - from) * weight
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderBloomSettings, RenderColorGradingSettings, RenderExposureMode,
        RenderExposureSettings, RenderPostProcessEffectStackSettings,
    };
    use crate::core::math::Vec3;

    use super::super::resolved_stack::RenderResolvedPostProcessSettings;
    use super::{
        interp_discrete, interp_float_lerp, interp_vec3_lerp, VolumeComponentApplyError,
        VolumeComponentDescriptor, VolumeParamSchema, VolumeParamType, VolumeParamValue,
        BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
    };

    const TEST_PARAMS: [VolumeParamSchema; 1] = [VolumeParamSchema::new(
        "value",
        VolumeParamValue::Float(0.0),
        interp_float_lerp,
    )];

    fn read_test_value(_settings: &RenderResolvedPostProcessSettings) -> Vec<VolumeParamValue> {
        vec![VolumeParamValue::Float(0.0)]
    }

    fn apply_test_value(
        _settings: &mut RenderResolvedPostProcessSettings,
        component_id: &'static str,
        values: &[VolumeParamValue],
    ) -> Result<(), VolumeComponentApplyError> {
        values[0].float(component_id, "value")?;
        Ok(())
    }

    #[test]
    fn render_volume_param_interp_blends_float_vec3_and_discrete_values() {
        assert_eq!(
            interp_float_lerp(
                VolumeParamValue::Float(1.0),
                VolumeParamValue::Float(5.0),
                0.25
            ),
            VolumeParamValue::Float(2.0)
        );
        assert_eq!(
            interp_vec3_lerp(
                VolumeParamValue::Vec3(Vec3::ZERO),
                VolumeParamValue::Vec3(Vec3::new(2.0, 4.0, 6.0)),
                0.5
            ),
            VolumeParamValue::Vec3(Vec3::new(1.0, 2.0, 3.0))
        );
        assert_eq!(
            interp_discrete(VolumeParamValue::Enum(1), VolumeParamValue::Enum(2), 0.49),
            VolumeParamValue::Enum(1)
        );
        assert_eq!(
            interp_discrete(VolumeParamValue::Enum(1), VolumeParamValue::Enum(2), 0.5),
            VolumeParamValue::Enum(2)
        );
    }

    #[test]
    fn render_volume_component_descriptor_applies_defaults_to_resolved_stack() {
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings {
                intensity: 9.0,
                ..Default::default()
            },
            RenderExposureSettings {
                mode: RenderExposureMode::Histogram,
                compensation_ev: 2.0,
                ..Default::default()
            },
            RenderColorGradingSettings {
                exposure: 2.0,
                ..Default::default()
            },
            RenderPostProcessEffectStackSettings::default(),
        );

        for descriptor in BUILTIN_POST_PROCESS_VOLUME_COMPONENTS {
            descriptor.apply_defaults(&mut settings).unwrap();
        }

        assert_eq!(settings.bloom, RenderBloomSettings::default());
        assert_eq!(settings.exposure, RenderExposureSettings::default());
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
    fn render_volume_component_descriptor_applies_authored_values() {
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings::default(),
            RenderExposureSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );
        let descriptor = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
            .iter()
            .find(|descriptor| descriptor.component_id == "post.vignette")
            .copied()
            .unwrap();

        descriptor
            .apply_values(
                &mut settings,
                &[
                    VolumeParamValue::Float(0.25),
                    VolumeParamValue::Float(0.75),
                    VolumeParamValue::Float(0.9),
                ],
            )
            .unwrap();

        assert_eq!(settings.effect_stack.vignette.intensity, 0.25);
        assert_eq!(settings.effect_stack.vignette.smoothness, 0.75);
        assert_eq!(settings.effect_stack.vignette.roundness, 0.9);
    }

    #[test]
    fn render_volume_component_descriptor_applies_exposure_values() {
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings::default(),
            RenderExposureSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );
        let descriptor = BUILTIN_POST_PROCESS_VOLUME_COMPONENTS
            .iter()
            .find(|descriptor| descriptor.component_id == "post.exposure")
            .copied()
            .unwrap();

        descriptor
            .apply_values(
                &mut settings,
                &[
                    VolumeParamValue::Enum(1),
                    VolumeParamValue::Float(7.0),
                    VolumeParamValue::Float(1.5),
                    VolumeParamValue::Float(-6.0),
                    VolumeParamValue::Float(10.0),
                    VolumeParamValue::Float(0.2),
                    VolumeParamValue::Float(0.8),
                    VolumeParamValue::Float(2.0),
                    VolumeParamValue::Float(0.5),
                ],
            )
            .unwrap();

        assert_eq!(settings.exposure.mode, RenderExposureMode::Histogram);
        assert_eq!(settings.exposure.manual_ev100, 7.0);
        assert_eq!(settings.exposure.compensation_ev, 1.5);
        assert_eq!(settings.exposure.render_histogram_range(), (-6.0, 10.0));
        assert_eq!(settings.exposure.render_filter_range(), (0.2, 0.8));
        assert_eq!(settings.exposure.render_speed_brighten(), 2.0);
        assert_eq!(settings.exposure.render_speed_darken(), 0.5);
        assert_eq!(
            descriptor.read_values(&settings),
            vec![
                VolumeParamValue::Enum(1),
                VolumeParamValue::Float(7.0),
                VolumeParamValue::Float(1.5),
                VolumeParamValue::Float(-6.0),
                VolumeParamValue::Float(10.0),
                VolumeParamValue::Float(0.2),
                VolumeParamValue::Float(0.8),
                VolumeParamValue::Float(2.0),
                VolumeParamValue::Float(0.5),
            ]
        );
    }

    #[test]
    fn render_volume_component_descriptor_rejects_bad_value_shape() {
        let descriptor = VolumeComponentDescriptor::new(
            "post.test",
            &TEST_PARAMS,
            read_test_value,
            apply_test_value,
        );
        let mut settings = RenderResolvedPostProcessSettings::new(
            RenderBloomSettings::default(),
            RenderExposureSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );

        assert_eq!(
            descriptor.apply_values(&mut settings, &[]),
            Err(VolumeComponentApplyError::ParamCountMismatch {
                component_id: "post.test",
                expected: 1,
                actual: 0,
            })
        );
        assert_eq!(
            descriptor.apply_values(&mut settings, &[VolumeParamValue::Uint(1)]),
            Err(VolumeComponentApplyError::ParamTypeMismatch {
                component_id: "post.test",
                param_name: "value",
                expected: VolumeParamType::Float,
                actual: VolumeParamType::Uint,
            })
        );
    }
}
