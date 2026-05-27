use super::super::*;

#[test]
fn platform_target_all_stays_complete_and_ordered_for_matrix_iteration() {
    assert_eq!(
        PlatformTarget::ALL,
        [
            PlatformTarget::Windows,
            PlatformTarget::Linux,
            PlatformTarget::Macos,
            PlatformTarget::Android,
            PlatformTarget::Ios,
            PlatformTarget::WebGpu,
            PlatformTarget::Wasm,
            PlatformTarget::Headless,
        ]
    );
}

#[test]
fn platform_target_strings_match_diagnostic_tokens() {
    let tokens: Vec<_> = PlatformTarget::ALL
        .iter()
        .map(|target| target.as_str())
        .collect();

    assert_eq!(
        tokens,
        vec!["windows", "linux", "macos", "android", "ios", "web_gpu", "wasm", "headless",]
    );
}

#[test]
fn platform_target_topology_helpers_partition_host_families() {
    for target in PlatformTarget::ALL {
        let category_count = usize::from(target.is_desktop())
            + usize::from(target.is_mobile())
            + usize::from(target.is_browser());

        assert!(
            category_count <= 1,
            "platform target {:?} should belong to at most one host family",
            target
        );
    }

    for target in [
        PlatformTarget::Windows,
        PlatformTarget::Linux,
        PlatformTarget::Macos,
    ] {
        assert!(target.is_desktop(), "{target:?} should be a desktop target");
        assert!(
            !target.is_mobile(),
            "{target:?} should not be a mobile target"
        );
        assert!(
            !target.is_browser(),
            "{target:?} should not be a browser target"
        );
    }

    for target in [PlatformTarget::Android, PlatformTarget::Ios] {
        assert!(target.is_mobile(), "{target:?} should be a mobile target");
        assert!(
            !target.is_desktop(),
            "{target:?} should not be a desktop target"
        );
        assert!(
            !target.is_browser(),
            "{target:?} should not be a browser target"
        );
    }

    for target in [PlatformTarget::WebGpu, PlatformTarget::Wasm] {
        assert!(target.is_browser(), "{target:?} should be a browser target");
        assert!(
            !target.is_desktop(),
            "{target:?} should not be a desktop target"
        );
        assert!(
            !target.is_mobile(),
            "{target:?} should not be a mobile target"
        );
    }

    assert!(!PlatformTarget::Headless.is_desktop());
    assert!(!PlatformTarget::Headless.is_mobile());
    assert!(!PlatformTarget::Headless.is_browser());
}

#[test]
fn platform_target_current_matches_compile_time_cfg_mapping() {
    let expected = if cfg!(target_arch = "wasm32") {
        PlatformTarget::Wasm
    } else if cfg!(target_os = "windows") {
        PlatformTarget::Windows
    } else if cfg!(target_os = "linux") {
        PlatformTarget::Linux
    } else if cfg!(target_os = "macos") {
        PlatformTarget::Macos
    } else if cfg!(target_os = "android") {
        PlatformTarget::Android
    } else if cfg!(target_os = "ios") {
        PlatformTarget::Ios
    } else {
        PlatformTarget::Headless
    };

    assert_eq!(PlatformTarget::current(), expected);
}
