#[test]
fn runtime_library_template_does_not_rescan_the_completed_source() {
    let source = include_str!("../platform_host_files.rs");
    let replacement_pass = ["    .rep", "lace("].concat();
    let template_body = source
        .split("fn runtime_library_template")
        .nth(1)
        .and_then(|body| body.split("fn native_library_stem").next())
        .expect("runtime library template body should remain available");

    assert!(!template_body.contains(&replacement_pass));
}

#[test]
fn webgpu_host_script_preserves_profile_line_terminators_as_escapes() {
    use crate::core::framework::platform::RuntimeTargetMode;
    use crate::core::framework::project::{ExportProfile, ExportTargetPlatform, RuntimeProfileId};

    // Public profile validation rejects these names. Exercise template robustness directly.
    // These quote-free fixtures use the common JSON/JavaScript string-escape subset.
    for name in [
        "profile\rCR",
        "profile\nLF",
        "profile\u{2028}LS",
        "profile\u{2029}PS",
        "literal\\n\r\n\u{2028}\u{2029}中文",
    ] {
        let mut profile = ExportProfile::new(
            name,
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::WebGpu,
            RuntimeProfileId::Client2d,
        );
        profile.output_name = "game".to_string();

        let files = super::platform_host_files(&profile, false);
        let script = &files
            .iter()
            .find(|file| file.path == "platform/webgpu/src/zircon_webgpu_host.js")
            .expect("WebGPU template must generate its actual launcher")
            .contents;

        for prefix in [
            "WebGPU is unavailable for Zircon export profile ",
            "WebGPU adapter is unavailable for Zircon export profile ",
        ] {
            let marker = format!("throw new Error('{prefix}");
            assert_eq!(script.matches(&marker).count(), 1);
            let encoded_name = script
                .split_once(&marker)
                .expect("launcher must contain the profile error message")
                .1
                .split_once("');")
                .expect("profile error literal must terminate")
                .0;

            assert!(
                !encoded_name
                    .chars()
                    .any(|character| matches!(character, '\r' | '\n' | '\u{2028}' | '\u{2029}')),
                "profile literal contains a raw line terminator: {name:?}"
            );
            let decoded: String = serde_json::from_str(&format!("\"{encoded_name}\""))
                .expect("profile escapes must preserve a decodable string value");
            assert_eq!(decoded, name);
        }
    }
}
