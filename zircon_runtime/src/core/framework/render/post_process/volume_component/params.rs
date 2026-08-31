use crate::core::math::{Real, Vec3};

use super::{type_mismatch, VolumeComponentApplyError};

pub type VolumeParamInterpFn =
    fn(from: VolumeParamValue, to: VolumeParamValue, weight: Real) -> VolumeParamValue;

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

    pub(crate) fn float(
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

    pub(crate) fn vec3(
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

    pub(crate) fn uint(
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

    pub(crate) fn enum_id(
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

    pub(crate) fn bool(
        self,
        component_id: &'static str,
        param_name: &'static str,
    ) -> Result<bool, VolumeComponentApplyError> {
        match self {
            Self::Bool(value) => Ok(value),
            other => Err(type_mismatch(
                component_id,
                param_name,
                VolumeParamType::Bool,
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

pub(super) const fn float_param(name: &'static str, default: Real) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Float(default), interp_float_lerp)
}

pub(super) const fn vec3_param(name: &'static str, default: Vec3) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Vec3(default), interp_vec3_lerp)
}

pub(super) const fn uint_param(name: &'static str, default: u32) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Uint(default), interp_discrete)
}

pub(super) const fn enum_param(name: &'static str, default: u32) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Enum(default), interp_discrete)
}

pub(super) const fn bool_param(name: &'static str, default: bool) -> VolumeParamSchema {
    VolumeParamSchema::new(name, VolumeParamValue::Bool(default), interp_bool)
}

const fn lerp(from: Real, to: Real, weight: Real) -> Real {
    from + (to - from) * weight
}
