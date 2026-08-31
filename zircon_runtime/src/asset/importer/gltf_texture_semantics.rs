use crate::core::framework::render::TextureUsageHint;

/// glTF defines texture transfer semantics from each material slot, not from an image source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GltfTextureColorSpace {
    Srgb,
    Linear,
}

impl GltfTextureColorSpace {
    fn label_suffix(self) -> &'static str {
        match self {
            Self::Srgb => "Srgb",
            Self::Linear => "Linear",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct GltfTextureUsage {
    srgb: bool,
    normal: bool,
    data: bool,
}

impl GltfTextureUsage {
    fn register(&mut self, color_space: GltfTextureColorSpace, usage_hint: TextureUsageHint) {
        match color_space {
            GltfTextureColorSpace::Srgb => self.srgb = true,
            GltfTextureColorSpace::Linear => match usage_hint {
                TextureUsageHint::Normal => self.normal = true,
                TextureUsageHint::Mask | TextureUsageHint::Data => self.data = true,
                TextureUsageHint::Albedo | TextureUsageHint::Hdr | TextureUsageHint::Ui => {
                    self.data = true;
                }
            },
        }
    }

    fn has_linear_usage(self) -> bool {
        self.normal || self.data
    }

    fn has_conflicting_color_spaces(self) -> bool {
        self.srgb && self.has_linear_usage()
    }

    fn has_normal_data_conflict(self) -> bool {
        self.normal && self.data
    }

    pub fn texture_variants(self) -> Vec<GltfTextureVariant> {
        let mut variants = Vec::with_capacity(3);
        if self.srgb || !self.has_linear_usage() {
            variants.push(gltf_texture_variant(
                GltfTextureColorSpace::Srgb,
                TextureUsageHint::Albedo,
            ));
        }
        if self.normal {
            variants.push(gltf_texture_variant(
                GltfTextureColorSpace::Linear,
                TextureUsageHint::Normal,
            ));
        }
        if self.data {
            variants.push(gltf_texture_variant(
                GltfTextureColorSpace::Linear,
                TextureUsageHint::Data,
            ));
        }
        variants
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GltfTextureVariant {
    color_space: GltfTextureColorSpace,
    usage_hint: TextureUsageHint,
}

impl GltfTextureVariant {
    pub const fn color_space(self) -> GltfTextureColorSpace {
        self.color_space
    }

    pub const fn usage_hint(self) -> TextureUsageHint {
        self.usage_hint
    }
}

pub fn gltf_texture_variant(
    color_space: GltfTextureColorSpace,
    usage_hint: TextureUsageHint,
) -> GltfTextureVariant {
    GltfTextureVariant {
        color_space,
        usage_hint: if color_space == GltfTextureColorSpace::Srgb {
            TextureUsageHint::Albedo
        } else {
            usage_hint
        },
    }
}

pub fn gltf_texture_color_space_usages(document: &gltf::Document) -> Vec<GltfTextureUsage> {
    let mut usages = vec![GltfTextureUsage::default(); document.textures().count()];
    for material in document.materials() {
        let pbr = material.pbr_metallic_roughness();
        register_gltf_texture_info_usage(
            &mut usages,
            pbr.base_color_texture()
                .as_ref()
                .map(|info| info.texture().index()),
            GltfTextureColorSpace::Srgb,
            TextureUsageHint::Albedo,
        );
        register_gltf_texture_info_usage(
            &mut usages,
            pbr.metallic_roughness_texture()
                .as_ref()
                .map(|info| info.texture().index()),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Data,
        );
        register_gltf_texture_info_usage(
            &mut usages,
            material
                .normal_texture()
                .as_ref()
                .map(|info| info.texture().index()),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Normal,
        );
        register_gltf_texture_info_usage(
            &mut usages,
            material
                .occlusion_texture()
                .as_ref()
                .map(|info| info.texture().index()),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Data,
        );
        register_gltf_texture_info_usage(
            &mut usages,
            material
                .emissive_texture()
                .as_ref()
                .map(|info| info.texture().index()),
            GltfTextureColorSpace::Srgb,
            TextureUsageHint::Albedo,
        );
        if let Some(clearcoat) = material.extension_value("KHR_materials_clearcoat") {
            register_gltf_texture_info_usage(
                &mut usages,
                gltf_extension_texture_index(clearcoat, "clearcoatNormalTexture"),
                GltfTextureColorSpace::Linear,
                TextureUsageHint::Normal,
            );
        }
    }
    usages
}

fn gltf_extension_texture_index(extension: &serde_json::Value, field: &str) -> Option<usize> {
    extension
        .get(field)?
        .get("index")?
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
}

pub fn gltf_texture_label(
    texture_index: usize,
    variant: GltfTextureVariant,
    usages: &[GltfTextureUsage],
) -> String {
    let label = format!("Texture{texture_index}");
    let Some(usage) = usages.get(texture_index).copied() else {
        return label;
    };
    if usage.has_normal_data_conflict() {
        let suffix = match variant.usage_hint {
            TextureUsageHint::Normal => "Normal",
            TextureUsageHint::Data => "Data",
            _ => variant.color_space.label_suffix(),
        };
        return format!("{label}/{suffix}");
    }
    if usage.has_conflicting_color_spaces() {
        return format!("{label}/{}", variant.color_space.label_suffix());
    }
    label
}

fn register_gltf_texture_info_usage(
    usages: &mut [GltfTextureUsage],
    texture_index: Option<usize>,
    color_space: GltfTextureColorSpace,
    usage_hint: TextureUsageHint,
) {
    if let Some(usage) = texture_index.and_then(|index| usages.get_mut(index)) {
        usage.register(color_space, usage_hint);
    }
}

#[cfg(test)]
mod tests {
    use super::{GltfTextureColorSpace, gltf_texture_color_space_usages, gltf_texture_label};
    use crate::core::framework::render::TextureUsageHint;

    #[test]
    fn clearcoat_normal_texture_registers_the_normal_variant_owner() {
        let gltf = gltf::Gltf::from_slice(
            br#"{
                "asset": { "version": "2.0" },
                "extensionsUsed": ["KHR_materials_clearcoat"],
                "images": [{ "uri": "coat.png" }],
                "textures": [{ "source": 0 }],
                "materials": [{
                    "extensions": {
                        "KHR_materials_clearcoat": {
                            "clearcoatNormalTexture": { "index": 0 }
                        }
                    }
                }]
            }"#,
        )
        .expect("clearcoat normal usage fixture");

        let usages = gltf_texture_color_space_usages(&gltf.document);
        let variants = usages[0].texture_variants();

        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].color_space(), GltfTextureColorSpace::Linear);
        assert_eq!(variants[0].usage_hint(), TextureUsageHint::Normal);
        assert_eq!(gltf_texture_label(0, variants[0], &usages), "Texture0");
    }
}
