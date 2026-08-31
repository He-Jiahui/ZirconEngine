#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ViewerMaterialFixture {
    #[default]
    MetalMirror,
    DielectricIor,
}

impl ViewerMaterialFixture {
    pub(crate) const fn cli_value(self) -> &'static str {
        match self {
            Self::MetalMirror => "metal-mirror",
            Self::DielectricIor => "dielectric-ior",
        }
    }

    pub(crate) const fn project_root_component(self) -> Option<&'static str> {
        match self {
            // Keep the original project path stable for the established mirror baseline.
            Self::MetalMirror => None,
            Self::DielectricIor => Some("dielectric-ior"),
        }
    }

    pub(crate) fn from_cli_value(value: &str) -> Result<Self, String> {
        match value {
            "metal-mirror" => Ok(Self::MetalMirror),
            "dielectric-ior" => Ok(Self::DielectricIor),
            _ => Err(format!(
                "--material-fixture must be metal-mirror or dielectric-ior, got {value}"
            )),
        }
    }

    pub(crate) const fn requires_generic_forward_pipeline(self) -> bool {
        matches!(self, Self::DielectricIor)
    }

    pub(crate) const fn project_asset_identity_prefix(self) -> &'static str {
        match self {
            Self::MetalMirror => "viewer-project-v4",
            Self::DielectricIor => "viewer-project-v4/dielectric-ior",
        }
    }

    pub(crate) const fn material_name(self) -> &'static str {
        match self {
            Self::MetalMirror => "Interactive Perfect Mirror Sphere",
            Self::DielectricIor => "Interactive Dielectric IOR 2.0 Sphere",
        }
    }

    pub(crate) const fn base_color(self) -> [f32; 4] {
        match self {
            Self::MetalMirror => [1.0, 1.0, 1.0, 1.0],
            Self::DielectricIor => [0.86, 0.9, 1.0, 1.0],
        }
    }

    pub(crate) const fn metallic(self) -> f32 {
        match self {
            Self::MetalMirror => 1.0,
            Self::DielectricIor => 0.0,
        }
    }

    pub(crate) const fn roughness(self) -> f32 {
        match self {
            Self::MetalMirror => 0.0,
            Self::DielectricIor => 0.08,
        }
    }

    pub(crate) const fn dielectric_ior(self) -> Option<f64> {
        match self {
            Self::MetalMirror => None,
            Self::DielectricIor => Some(2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ViewerMaterialFixture;

    #[test]
    fn fixture_identities_are_stable() {
        assert_eq!(
            ViewerMaterialFixture::MetalMirror.project_asset_identity_prefix(),
            "viewer-project-v4"
        );
        assert_eq!(
            ViewerMaterialFixture::MetalMirror.cli_value(),
            "metal-mirror"
        );
        assert_eq!(
            ViewerMaterialFixture::DielectricIor.cli_value(),
            "dielectric-ior"
        );
        assert_eq!(
            ViewerMaterialFixture::DielectricIor.project_root_component(),
            Some("dielectric-ior")
        );
        assert_eq!(
            ViewerMaterialFixture::DielectricIor.project_asset_identity_prefix(),
            "viewer-project-v4/dielectric-ior"
        );
    }

    #[test]
    fn dielectric_fixture_has_a_non_default_f0_input_and_separate_project_identity() {
        assert_eq!(ViewerMaterialFixture::MetalMirror.dielectric_ior(), None);
        assert_eq!(
            ViewerMaterialFixture::DielectricIor.dielectric_ior(),
            Some(2.0)
        );
        assert!(ViewerMaterialFixture::DielectricIor.requires_generic_forward_pipeline());
        assert!(!ViewerMaterialFixture::MetalMirror.requires_generic_forward_pipeline());
    }

    #[test]
    fn fixture_cli_values_are_closed() {
        assert_eq!(
            ViewerMaterialFixture::from_cli_value("metal-mirror"),
            Ok(ViewerMaterialFixture::MetalMirror)
        );
        assert_eq!(
            ViewerMaterialFixture::from_cli_value("dielectric-ior"),
            Ok(ViewerMaterialFixture::DielectricIor)
        );
        assert!(ViewerMaterialFixture::from_cli_value("glass").is_err());
    }
}
