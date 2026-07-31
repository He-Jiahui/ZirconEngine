use woc_client::{
    classify_gpu_renderer, first_run_graphics_preset, native_safe_graphics_preset,
    resolve_default_graphics_preset, GpuClass, GraphicsPreset, GraphicsRuntimeHints,
};

fn desktop() -> GraphicsRuntimeHints<'static> {
    GraphicsRuntimeHints::default()
}

fn phone() -> GraphicsRuntimeHints<'static> {
    GraphicsRuntimeHints {
        max_touch_points: 5,
        coarse_pointer: true,
        ..GraphicsRuntimeHints::default()
    }
}

#[test]
fn gpu_classifier_matches_the_target_family_ladders() {
    for (renderer, class) in [
        (None, GpuClass::Unknown),
        (Some(""), GpuClass::Unknown),
        (Some("Apple GPU"), GpuClass::Unknown),
        (Some("Google SwiftShader"), GpuClass::Software),
        (
            Some("Mesa llvmpipe (LLVM 15.0.7, 256 bits)"),
            GpuClass::Software,
        ),
        (Some("Apple Software Renderer"), GpuClass::Software),
        (
            Some("ANGLE (Intel, Intel(R) Iris(TM) Plus Graphics 655)"),
            GpuClass::Weak,
        ),
        (Some("Adreno (TM) 330"), GpuClass::Weak),
        (Some("PowerVR SGX 544"), GpuClass::Weak),
        (Some("Mali-G51"), GpuClass::Weak),
        (Some("Mali-G52"), GpuClass::Weak),
        (
            Some("ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11)"),
            GpuClass::Weak,
        ),
        (
            Some("ANGLE (Intel, Intel(R) UHD Graphics 770)"),
            GpuClass::MidIntegrated,
        ),
        (
            Some("ANGLE (Intel, Intel(R) Iris(R) Xe Graphics)"),
            GpuClass::MidIntegrated,
        ),
        (
            Some("ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11)"),
            GpuClass::MidIntegrated,
        ),
        (
            Some("ANGLE (AMD, AMD Radeon(TM) Vega 8 Graphics)"),
            GpuClass::MidIntegrated,
        ),
        (Some("AMD Radeon Vega 8 Graphics"), GpuClass::MidIntegrated),
        (Some("Mali-G57"), GpuClass::MidMobile),
        (Some("Apple A100 GPU"), GpuClass::Unknown),
        (
            Some("ANGLE (NVIDIA, NVIDIA GeForce RTX 4080)"),
            GpuClass::StrongDesktop,
        ),
        (Some("Adreno (TM) 740"), GpuClass::FlagshipMobile),
    ] {
        assert_eq!(classify_gpu_renderer(renderer), class, "{renderer:?}");
    }
}

#[test]
fn unknown_and_mid_devices_use_medium_without_low_signal_demotion() {
    assert_eq!(
        resolve_default_graphics_preset(&desktop()),
        GraphicsPreset::Medium
    );
    for hints in [
        GraphicsRuntimeHints {
            gpu_renderer: Some("Apple GPU"),
            ..desktop()
        },
        GraphicsRuntimeHints {
            gpu_renderer: Some("Intel Iris Xe"),
            ..desktop()
        },
        GraphicsRuntimeHints {
            device_memory_gib: Some(2.0),
            hardware_concurrency: Some(2),
            ..desktop()
        },
        GraphicsRuntimeHints {
            gpu_renderer: Some("ANGLE (AMD, AMD Radeon(TM) Graphics Direct3D11)"),
            device_memory_gib: Some(8.0),
            hardware_concurrency: Some(16),
            ..desktop()
        },
    ] {
        assert_eq!(
            resolve_default_graphics_preset(&hints),
            GraphicsPreset::Medium
        );
    }
}

#[test]
fn software_and_weak_gpu_classes_are_the_only_automatic_low_path() {
    for renderer in [
        "Google SwiftShader",
        "Apple Software Renderer",
        "Adreno (TM) 330",
        "Mali-G52",
    ] {
        let hints = GraphicsRuntimeHints {
            gpu_renderer: Some(renderer),
            device_memory_gib: Some(8.0),
            hardware_concurrency: Some(16),
            ..phone()
        };
        assert_eq!(
            resolve_default_graphics_preset(&hints),
            GraphicsPreset::Low,
            "{renderer}"
        );
    }
}

#[test]
fn touch_devices_cap_strong_and_flagship_hardware_at_high() {
    for renderer in ["Adreno (TM) 740", "Apple M2"] {
        let hints = GraphicsRuntimeHints {
            gpu_renderer: Some(renderer),
            device_memory_gib: Some(8.0),
            hardware_concurrency: Some(16),
            ..phone()
        };
        assert_eq!(
            resolve_default_graphics_preset(&hints),
            GraphicsPreset::High
        );
    }
    assert_eq!(
        resolve_default_graphics_preset(&phone()),
        GraphicsPreset::Medium
    );
}

#[test]
fn strong_desktop_and_unknown_desktop_promotion_require_target_evidence() {
    let rtx = Some("ANGLE (NVIDIA, NVIDIA GeForce RTX 4080)");
    assert_eq!(
        resolve_default_graphics_preset(&GraphicsRuntimeHints {
            gpu_renderer: rtx,
            ..desktop()
        }),
        GraphicsPreset::Ultra
    );
    assert_eq!(
        resolve_default_graphics_preset(&GraphicsRuntimeHints {
            gpu_renderer: rtx,
            device_memory_gib: Some(4.0),
            hardware_concurrency: Some(4),
            ..desktop()
        }),
        GraphicsPreset::High
    );
    assert_eq!(
        resolve_default_graphics_preset(&GraphicsRuntimeHints {
            device_memory_gib: Some(8.0),
            hardware_concurrency: Some(12),
            ..desktop()
        }),
        GraphicsPreset::High
    );
    assert_eq!(
        resolve_default_graphics_preset(&GraphicsRuntimeHints {
            device_memory_gib: Some(8.0),
            hardware_concurrency: Some(4),
            ..desktop()
        }),
        GraphicsPreset::Medium
    );
}

#[test]
fn first_run_marking_and_native_startup_clamp_preserve_player_control() {
    let weak = GraphicsRuntimeHints {
        gpu_renderer: Some("Google SwiftShader"),
        ..desktop()
    };
    assert_eq!(
        first_run_graphics_preset(false, &weak),
        Some(GraphicsPreset::Low)
    );
    assert_eq!(first_run_graphics_preset(true, &weak), None);
    assert_eq!(first_run_graphics_preset(false, &desktop()), None);

    assert_eq!(
        native_safe_graphics_preset(GraphicsPreset::Advanced),
        GraphicsPreset::High
    );
    assert_eq!(
        native_safe_graphics_preset(GraphicsPreset::Ultra),
        GraphicsPreset::High
    );
    assert_eq!(
        native_safe_graphics_preset(GraphicsPreset::Low),
        GraphicsPreset::Low
    );
}
