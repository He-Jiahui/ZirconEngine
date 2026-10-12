use super::*;

#[test]
fn normal_srgb_metadata_is_an_error() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Normal,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/normal.png",
        "rgba8unorm_srgb",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
}

#[test]
fn albedo_linear_metadata_is_a_warning() {
    let metadata = TextureMetadata {
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/albedo.png",
        "rgba8unorm",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning
            && diagnostic.message
                == "albedo texture 'textures/albedo.png' declares linear; expected srgb unless intentional"
    }));
}

#[test]
fn hdr_metadata_requires_a_float_format() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/sky.png",
        "rgba8unorm",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            && diagnostic.message
                == "hdr texture 'textures/sky.png' requires a float format, got 'rgba8unorm'"
    }));
}

#[test]
fn hdr_metadata_accepts_float_format() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/sky.hdr",
        "rgba16float",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(!diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
}

#[test]
fn hdr_metadata_accepts_bc6h_compressed_container_formats() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    for format in [
        "dds/dxgi-95",
        "dds/dxgi-96",
        "ktx/gl-internal-0x8e8e",
        "ktx/gl-internal-0x8e8f",
        "ktx2/vk-143",
        "ktx2/vk-144",
    ] {
        let diagnostics = validate_texture_metadata(
            "textures/environment.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(
            !diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            }),
            "{format} should support HDR metadata: {diagnostics:?}"
        );
    }
}

#[test]
fn hdr_metadata_rejects_non_float_compressed_container_formats() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    for format in ["dds/dxt1", "ktx/gl-internal-0x83f0", "ktx2/vk-131"] {
        let diagnostics = validate_texture_metadata(
            "textures/invalid-environment.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("requires a float format")
        }));
    }
}

#[test]
fn hdr_metadata_rejects_container_tokens_with_spurious_float_suffixes() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    for format in [
        "dds/dxgi-71-float",
        "ktx/gl-internal-0x83f0-float",
        "ktx2/vk-131-float",
        "astc/4x4x1-float",
    ] {
        let diagnostics = validate_texture_metadata(
            "textures/invalid-hdr-container.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("requires a float format")
        }));
    }
}

#[test]
fn hdr_metadata_rejects_unrecognized_float_format_tokens() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        ..TextureMetadata::default()
    };
    for format in [
        "rgba16float-srgb",
        "unrecognized-float-format",
        "bc6h-untrusted",
    ] {
        let diagnostics = validate_texture_metadata(
            "textures/invalid-hdr-format.texture",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("requires a float format")
        }));
    }
}

#[test]
fn normal_non_bc5_compression_is_a_warning() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Normal,
        color_space: RenderImageColorSpace::Linear,
        compression: TextureCompressionTarget::Bc7,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/normal.png",
        "rgba8unorm",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning
            && diagnostic.message
                == "normal map 'textures/normal.png' should compress as bc5, got 'bc7'"
    }));
}

#[test]
fn ui_mip_policy_is_a_warning() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Ui,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "ui/icon.png",
        "rgba8unorm_srgb",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning));
}

#[test]
fn srgb_metadata_accepts_supported_compressed_container_formats() {
    let metadata = TextureMetadata::default();
    for format in [
        "dds/dxt1",
        "dds/dxt3",
        "dds/dxt5",
        "astc/4x4x1",
        "ktx/gl-internal-0x8c4c",
        "ktx/gl-internal-0x8e8d",
        "ktx2/vk-132",
    ] {
        let diagnostics = validate_texture_metadata(
            "textures/albedo.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(
            !diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            }),
            "{format} should support sRGB metadata: {diagnostics:?}"
        );
    }
}

#[test]
fn srgb_metadata_rejects_linear_compressed_container_formats() {
    let metadata = TextureMetadata::default();
    for format in ["dds/dxgi-71", "ktx/gl-internal-0x83f0", "ktx2/vk-131"] {
        let diagnostics = validate_texture_metadata(
            "textures/linear.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("has no srgb variant")
        }));
    }
}

#[test]
fn srgb_metadata_rejects_container_tokens_with_spurious_srgb_suffixes() {
    let metadata = TextureMetadata::default();
    for format in [
        "dds/dxgi-71-srgb",
        "ktx/gl-internal-0x83f0-srgb",
        "ktx2/vk-131-srgb",
    ] {
        let diagnostics = validate_texture_metadata(
            "textures/invalid-srgb-container.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("has no srgb variant")
        }));
    }
}

#[test]
fn srgb_metadata_rejects_unsupported_astc_container_formats() {
    let metadata = TextureMetadata::default();
    for format in ["astc/4x4", "astc/4x4x2", "astc/7x7x1"] {
        let diagnostics = validate_texture_metadata(
            "textures/unsupported-astc.container",
            format,
            &metadata,
            &RenderSamplerDescriptor::default(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("has no srgb variant")
        }));
    }
}

#[test]
fn runtime_mip_policy_requires_box_filter() {
    let metadata = TextureMetadata {
        mip_policy: TextureMipPolicy::GenerateRuntime,
        mip_filter: TextureMipFilter::Kaiser,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/runtime-mips.png",
        "rgba8unorm_srgb",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            && diagnostic.message.contains("mip_filter=box")
    }));
}

#[test]
fn runtime_mip_policy_rejects_non_rgba8_storage_formats() {
    let metadata = TextureMetadata {
        usage_hint: TextureUsageHint::Hdr,
        color_space: RenderImageColorSpace::Linear,
        mip_policy: TextureMipPolicy::GenerateRuntime,
        mip_filter: TextureMipFilter::Box,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/runtime-hdr-mips.exr",
        "rgba16float",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            && diagnostic
                .message
                .contains("runtime mip generation supports only rgba8unorm storage")
    }));
}

#[test]
fn runtime_mip_policy_accepts_rgba8_storage_formats() {
    let metadata = TextureMetadata {
        mip_policy: TextureMipPolicy::GenerateRuntime,
        mip_filter: TextureMipFilter::Box,
        ..TextureMetadata::default()
    };
    let diagnostics = validate_texture_metadata(
        "textures/runtime-mips.png",
        "rgba8unorm_srgb",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(!diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.severity == TextureMetadataDiagnosticSeverity::Error }));
}

#[test]
fn invalid_anisotropy_is_an_error() {
    let metadata = TextureMetadata {
        max_anisotropy: 3,
        ..TextureMetadata::default()
    };

    let diagnostics = validate_texture_metadata(
        "textures/invalid-anisotropy.png",
        "rgba8unorm_srgb",
        &metadata,
        &RenderSamplerDescriptor::default(),
    );

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
            && diagnostic.message.contains("max_anisotropy")
    }));
}
