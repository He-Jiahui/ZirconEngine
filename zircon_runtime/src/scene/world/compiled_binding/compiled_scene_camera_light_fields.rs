#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledCameraProperty {
    FovYRadians,
    ZNear,
    ZFar,
}

impl CompiledCameraProperty {
    pub(super) fn from_canonical_key(key: &str) -> Option<Self> {
        match key {
            "camera.fovyradians" => Some(Self::FovYRadians),
            "camera.znear" => Some(Self::ZNear),
            "camera.zfar" => Some(Self::ZFar),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledLightProperty {
    AmbientColor,
    AmbientIntensity,
    AmbientAffectsLightmappedMeshes,
    DirectionalDirection,
    DirectionalColor,
    DirectionalIntensity,
    PointColor,
    PointIntensity,
    PointRange,
    RectColor,
    RectIntensity,
    RectRange,
    RectSize,
    SpotDirection,
    SpotColor,
    SpotIntensity,
    SpotRange,
    SpotInnerAngleRadians,
    SpotOuterAngleRadians,
}

impl CompiledLightProperty {
    pub(super) fn from_canonical_key(key: &str) -> Option<Self> {
        match key {
            "ambientlight.color" => Some(Self::AmbientColor),
            "ambientlight.intensity" => Some(Self::AmbientIntensity),
            "ambientlight.affectslightmappedmeshes" => Some(Self::AmbientAffectsLightmappedMeshes),
            "directionallight.direction" => Some(Self::DirectionalDirection),
            "directionallight.color" => Some(Self::DirectionalColor),
            "directionallight.intensity" => Some(Self::DirectionalIntensity),
            "pointlight.color" => Some(Self::PointColor),
            "pointlight.intensity" => Some(Self::PointIntensity),
            "pointlight.range" => Some(Self::PointRange),
            "rectlight.color" => Some(Self::RectColor),
            "rectlight.intensity" => Some(Self::RectIntensity),
            "rectlight.range" => Some(Self::RectRange),
            "rectlight.size" => Some(Self::RectSize),
            "spotlight.direction" => Some(Self::SpotDirection),
            "spotlight.color" => Some(Self::SpotColor),
            "spotlight.intensity" => Some(Self::SpotIntensity),
            "spotlight.range" => Some(Self::SpotRange),
            "spotlight.innerangleradians" => Some(Self::SpotInnerAngleRadians),
            "spotlight.outerangleradians" => Some(Self::SpotOuterAngleRadians),
            _ => None,
        }
    }
}
