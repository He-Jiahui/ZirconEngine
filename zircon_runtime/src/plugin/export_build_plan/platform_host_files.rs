use crate::core::framework::project::ExportProfile;

use self::{browser::browser_host_files, mobile::mobile_host_files};
use super::ExportGeneratedFile;

mod browser;
mod mobile;

pub(super) fn platform_host_files(
    profile: &ExportProfile,
    has_native_dynamic_plugins: bool,
) -> Vec<ExportGeneratedFile> {
    let policy = profile.target_platform.policy();
    match policy.host_kind {
        crate::core::framework::project::ExportPlatformHostKind::Desktop => {
            vec![ExportGeneratedFile {
                path: "src/main.rs".to_string(),
                purpose: "generated desktop runtime entry point".to_string(),
                contents: super::main_template::main_template(profile, has_native_dynamic_plugins),
            }]
        }
        crate::core::framework::project::ExportPlatformHostKind::Headless => {
            vec![ExportGeneratedFile {
                path: "src/main.rs".to_string(),
                purpose: "generated headless runtime entry point".to_string(),
                contents: super::main_template::main_template(profile, has_native_dynamic_plugins),
            }]
        }
        crate::core::framework::project::ExportPlatformHostKind::MobileApp => {
            mobile_host_files(profile)
        }
        crate::core::framework::project::ExportPlatformHostKind::Browser => {
            browser_host_files(profile)
        }
    }
}

fn runtime_library_file(profile: &ExportProfile, host_label: &str) -> ExportGeneratedFile {
    ExportGeneratedFile {
        path: "src/lib.rs".to_string(),
        purpose: format!("generated {host_label} runtime library entry point"),
        contents: runtime_library_template(profile, host_label),
    }
}

fn runtime_library_template(profile: &ExportProfile, host_label: &str) -> String {
    let target_platform = profile.target_platform.as_str();
    format!(
        r#"mod zircon_plugins;

enum ZirconProductCompositionState {{
    Vacant,
    Starting,
    Running(zircon_app::ProductComposition),
    Stopping,
}}

static ZIRCON_PRODUCT_COMPOSITION: std::sync::Mutex<ZirconProductCompositionState> =
    std::sync::Mutex::new(ZirconProductCompositionState::Vacant);

struct ZirconProductStartGuard {{
    active: bool,
}}

impl Drop for ZirconProductStartGuard {{
    fn drop(&mut self) {{
        if !self.active {{
            return;
        }}
        if let Ok(mut composition_state) = ZIRCON_PRODUCT_COMPOSITION.lock() {{
            if let ZirconProductCompositionState::Starting = &*composition_state {{
                *composition_state = ZirconProductCompositionState::Vacant;
            }}
        }}
    }}
}}

/// Starts the Zircon runtime from a generated {host_label} scaffold.
pub fn zircon_export_bootstrap() -> Result<(), Box<dyn std::error::Error>> {{
    {{
        let mut composition_state = ZIRCON_PRODUCT_COMPOSITION
            .lock()
            .map_err(|_| std::io::Error::other("Zircon product composition owner is poisoned"))?;
        match &*composition_state {{
            ZirconProductCompositionState::Vacant => {{
                *composition_state = ZirconProductCompositionState::Starting;
            }}
            ZirconProductCompositionState::Running(_) => return Ok(()),
            ZirconProductCompositionState::Starting => {{
                return Err(std::io::Error::other("Zircon product composition is already starting").into());
            }}
            ZirconProductCompositionState::Stopping => {{
                return Err(std::io::Error::other("Zircon product composition is stopping").into());
            }}
        }}
    }}
    let mut start_guard = ZirconProductStartGuard {{ active: true }};
    let composition = zircon_app::bootstrap_export_runtime(
        zircon_plugins::export_runtime_bootstrap_config(),
    )?;
    let mut composition_state = ZIRCON_PRODUCT_COMPOSITION
        .lock()
        .map_err(|_| std::io::Error::other("Zircon product composition owner is poisoned"))?;
    if !matches!(&*composition_state, ZirconProductCompositionState::Starting) {{
        return Err(std::io::Error::other("Zircon product composition start state changed unexpectedly").into());
    }}
    *composition_state = ZirconProductCompositionState::Running(composition);
    start_guard.active = false;
    Ok(())
}}

fn zircon_export_is_running() -> bool {{
    match ZIRCON_PRODUCT_COMPOSITION.lock() {{
        Ok(composition_state) => matches!(
            &*composition_state,
            ZirconProductCompositionState::Running(_)
        ),
        Err(_) => false,
    }}
}}

fn zircon_export_ffi_guard(operation: impl FnOnce() -> bool) -> bool {{
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or(false)
}}

#[no_mangle]
pub extern "C" fn zircon_export_start() -> bool {{
    zircon_export_ffi_guard(|| zircon_export_bootstrap().is_ok())
}}

#[no_mangle]
pub extern "C" fn zircon_export_shutdown() -> bool {{
    zircon_export_ffi_guard(|| {{
        let composition = {{
            let mut composition_state = match ZIRCON_PRODUCT_COMPOSITION.lock() {{
                Ok(composition_state) => composition_state,
                Err(_) => return false,
            }};
            match &*composition_state {{
                ZirconProductCompositionState::Vacant => return true,
                ZirconProductCompositionState::Starting
                | ZirconProductCompositionState::Stopping => return false,
                ZirconProductCompositionState::Running(_) => {{}}
            }}
            match std::mem::replace(
                &mut *composition_state,
                ZirconProductCompositionState::Stopping,
            ) {{
                ZirconProductCompositionState::Running(composition) => composition,
                _ => unreachable!("running state was checked while holding the owner lock"),
            }}
        }};
        drop(composition);
        let mut composition_state = match ZIRCON_PRODUCT_COMPOSITION.lock() {{
            Ok(composition_state) => composition_state,
            Err(_) => return false,
        }};
        if !matches!(&*composition_state, ZirconProductCompositionState::Stopping) {{
            return false;
        }}
        *composition_state = ZirconProductCompositionState::Vacant;
        true
    }})
}}

#[no_mangle]
pub extern "C" fn zircon_export_handle_lifecycle(_state: u32) -> bool {{
    zircon_export_ffi_guard(zircon_export_is_running)
}}

#[no_mangle]
pub extern "C" fn zircon_export_handle_touch(
    _pointer_id: u64,
    _phase: u32,
    _x: f32,
    _y: f32,
) -> bool {{
    zircon_export_ffi_guard(zircon_export_is_running)
}}

#[no_mangle]
pub extern "C" fn zircon_export_handle_keyboard(
    _action: u32,
    _key_code: u32,
    _scan_code: u32,
    _text: *const u8,
    _text_len: usize,
) -> bool {{
    zircon_export_ffi_guard(zircon_export_is_running)
}}

#[no_mangle]
pub extern "C" fn zircon_export_handle_viewport_metrics(
    _logical_width: u32,
    _logical_height: u32,
    _scale: f32,
) -> bool {{
    zircon_export_ffi_guard(zircon_export_is_running)
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_start(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
) -> bool {{
    zircon_export_ffi_guard(|| zircon_export_start())
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_shutdown(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
) -> bool {{
    zircon_export_ffi_guard(|| zircon_export_shutdown())
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_dispatchLifecycle(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    state: i32,
) -> bool {{
    zircon_export_ffi_guard(|| zircon_export_handle_lifecycle(state as u32))
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_dispatchTouch(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    pointer_id: i64,
    phase: i32,
    x: f32,
    y: f32,
) -> bool {{
    zircon_export_ffi_guard(|| {{
        zircon_export_handle_touch(pointer_id as u64, phase as u32, x, y)
    }})
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_dispatchKeyboard(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    action: i32,
    key_code: i32,
    scan_code: i32,
    _text: *mut core::ffi::c_void,
) -> bool {{
    zircon_export_ffi_guard(|| {{
        zircon_export_handle_keyboard(
            action as u32,
            key_code as u32,
            scan_code as u32,
            core::ptr::null(),
            0,
        )
    }})
}}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_dev_zircon_export_ZirconRuntime_dispatchViewportMetrics(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    width: i32,
    height: i32,
    scale: f32,
) -> bool {{
    zircon_export_ffi_guard(|| {{
        zircon_export_handle_viewport_metrics(width.max(0) as u32, height.max(0) as u32, scale)
    }})
}}

#[no_mangle]
pub extern "C" fn zircon_export_fetch_resource(
    _uri: *const core::ffi::c_char,
    _flags: u32,
) -> bool {{
    zircon_export_ffi_guard(zircon_export_is_running)
}}

pub const ZIRCON_EXPORT_TARGET_PLATFORM: &str = "{target_platform}";
"#
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_library_template_does_not_rescan_the_completed_source() {
        let source = include_str!("platform_host_files.rs");
        let replacement_pass = ["    .rep", "lace("].concat();
        let template_body = source
            .split("fn runtime_library_template")
            .nth(1)
            .and_then(|body| body.split("fn native_library_stem").next())
            .expect("runtime library template body should remain available");

        assert!(!template_body.contains(&replacement_pass));
    }
}

fn native_library_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' | '_' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' => '_',
            _ => '_',
        })
        .collect()
}

fn bundle_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '-',
        })
        .collect()
}

fn android_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '.',
        })
        .collect::<String>()
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_escape(value: &str) -> String {
    xml_escape(value)
}

fn swift_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn javascript_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn json_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn gradle_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn powershell_string_escape(value: &str) -> String {
    value.replace('`', "``").replace('\'', "''")
}

fn properties_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn toml_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
