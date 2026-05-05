use serde::{Deserialize, Serialize};

use crate::ui::layout::UiMargin;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderResourceKind {
    Image,
    Icon,
    Vector,
    Font,
    Material,
    Texture,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderResourceKey {
    pub kind: UiRenderResourceKind,
    pub id: String,
    pub revision: Option<u64>,
    pub atlas_page: Option<u32>,
}

impl UiRenderResourceKey {
    pub fn new(kind: UiRenderResourceKind, id: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            revision: None,
            atlas_page: None,
        }
    }

    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = Some(revision);
        self
    }

    pub fn with_atlas_page(mut self, atlas_page: u32) -> Self {
        self.atlas_page = Some(atlas_page);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiBrushSet {
    pub fill: Option<UiBrushPayload>,
    pub border: Option<UiBrushPayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBrushPayload {
    Solid(UiSolidBrushPayload),
    Image(UiImageBrushPayload),
    Box(UiImageBrushPayload),
    Border(UiBorderBrushPayload),
    Rounded(UiRoundedBrushPayload),
    Gradient(UiGradientBrushPayload),
    Vector(UiVectorBrushPayload),
    Material(UiMaterialBrushPayload),
}

impl UiBrushPayload {
    pub fn solid(color: impl Into<String>) -> Self {
        Self::Solid(UiSolidBrushPayload {
            color: color.into(),
        })
    }

    pub fn rounded(color: impl Into<String>, radius: f32) -> Self {
        Self::Rounded(UiRoundedBrushPayload {
            color: color.into(),
            radius,
        })
    }

    pub fn border(color: impl Into<String>, width: f32) -> Self {
        Self::Border(UiBorderBrushPayload {
            color: color.into(),
            width,
            margin: UiMargin::new(width, width, width, width),
            radius: 0.0,
        })
    }

    pub fn image(resource: UiRenderResourceKey) -> Self {
        Self::Image(UiImageBrushPayload {
            resource,
            tint: None,
            margin: UiMargin::default(),
        })
    }

    pub fn material(material_id: impl Into<String>) -> Self {
        Self::Material(UiMaterialBrushPayload {
            material_id: material_id.into(),
            variant: None,
            revision: None,
            fallback_color: None,
        })
    }

    pub fn resource_key(&self) -> Option<&UiRenderResourceKey> {
        match self {
            Self::Image(payload) | Self::Box(payload) => Some(&payload.resource),
            Self::Vector(payload) => Some(&payload.resource),
            _ => None,
        }
    }

    pub fn material_key(&self) -> Option<UiRenderResourceKey> {
        match self {
            Self::Material(payload) => Some(payload.resource_key()),
            _ => None,
        }
    }

    pub fn with_tint(mut self, tint: impl Into<String>) -> Self {
        let tint = Some(tint.into());
        match &mut self {
            Self::Image(payload) | Self::Box(payload) => payload.tint = tint,
            _ => {}
        }
        self
    }

    pub fn with_material_variant(mut self, variant: impl Into<String>) -> Self {
        if let Self::Material(payload) = &mut self {
            payload.variant = Some(variant.into());
        }
        self
    }

    pub fn with_material_revision(mut self, revision: u64) -> Self {
        if let Self::Material(payload) = &mut self {
            payload.revision = Some(revision);
        }
        self
    }

    pub fn with_fallback_color(mut self, color: impl Into<String>) -> Self {
        if let Self::Material(payload) = &mut self {
            payload.fallback_color = Some(color.into());
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSolidBrushPayload {
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRoundedBrushPayload {
    pub color: String,
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBorderBrushPayload {
    pub color: String,
    pub width: f32,
    pub margin: UiMargin,
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiImageBrushPayload {
    pub resource: UiRenderResourceKey,
    pub tint: Option<String>,
    pub margin: UiMargin,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGradientBrushPayload {
    pub stops: Vec<UiGradientStop>,
    pub angle_degrees: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGradientStop {
    pub offset: f32,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiVectorBrushPayload {
    pub resource: UiRenderResourceKey,
    pub tint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMaterialBrushPayload {
    pub material_id: String,
    pub variant: Option<String>,
    pub revision: Option<u64>,
    pub fallback_color: Option<String>,
}

impl UiMaterialBrushPayload {
    pub fn resource_key(&self) -> UiRenderResourceKey {
        let id = self
            .variant
            .as_ref()
            .map(|variant| format!("{}#{variant}", self.material_id))
            .unwrap_or_else(|| self.material_id.clone());
        let mut key = UiRenderResourceKey::new(UiRenderResourceKind::Material, id);
        key.revision = self.revision;
        key
    }
}
