use zircon_editor::EditorPluginRegistrationReport;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    resolve_plugin_selections, PluginSelectionResolutionReport, ProjectPluginManifest,
};

type EditorPluginRegistrationProvider = fn() -> EditorPluginRegistrationReport;

pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<EditorPluginRegistrationReport> {
    if target_mode != RuntimeTargetMode::EditorHost {
        return resolve_plugin_selections(target_mode, manifest, |_| None);
    }
    resolve_plugin_selections(target_mode, manifest, |plugin_id| {
        first_party_editor_registration_provider(plugin_id).map(|provider| provider())
    })
}

pub fn first_party_registration_for_editor_plugin(
    plugin_id: RuntimePluginId,
) -> Option<EditorPluginRegistrationReport> {
    first_party_editor_registration_provider(&plugin_id).map(|provider| provider())
}

fn first_party_editor_registration_provider(
    _plugin_id: &RuntimePluginId,
) -> Option<EditorPluginRegistrationProvider> {
    // @cargo-zircon:editor-registration-begin
    #[cfg(feature = "ai-editor-plugin")]
    if *_plugin_id == RuntimePluginId::Ai {
        return Some(ai_registration_with_root);
    }
    #[cfg(feature = "navigation-editor-plugin")]
    if *_plugin_id == RuntimePluginId::Navigation {
        return Some(zircon_plugin_navigation_editor::plugin_registration);
    }
    #[cfg(feature = "neural-editor-plugin")]
    if _plugin_id.key() == "neural" {
        return Some(zircon_plugin_neural_editor::plugin_registration);
    }
    #[cfg(feature = "particles-editor-plugin")]
    if *_plugin_id == RuntimePluginId::Particles {
        return Some(particles_registration_with_root);
    }
    #[cfg(feature = "sound-editor-plugin")]
    if *_plugin_id == RuntimePluginId::Sound {
        return Some(sound_registration_with_root);
    }
    // @cargo-zircon:editor-registration-end
    None
}

#[cfg(feature = "ai-editor-plugin")]
fn ai_registration_with_root() -> EditorPluginRegistrationReport {
    zircon_plugin_ai_editor::plugin_registration().with_ui_template_root(ai_ui_template_root())
}

#[cfg(feature = "ai-editor-plugin")]
pub(super) fn ai_ui_template_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("ai")
}

#[cfg(feature = "particles-editor-plugin")]
fn particles_registration_with_root() -> EditorPluginRegistrationReport {
    static REGISTRATION: std::sync::OnceLock<EditorPluginRegistrationReport> =
        std::sync::OnceLock::new();

    // Particles discovery uses only linked descriptors and a compile-time root.
    // Keep selection resolution live and return an owned report to each caller.
    REGISTRATION
        .get_or_init(|| {
            zircon_plugin_particles_editor::plugin_registration()
                .with_ui_template_root(particles_ui_template_root())
        })
        .clone()
}

#[cfg(feature = "particles-editor-plugin")]
pub(super) fn particles_ui_template_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("particles")
}

#[cfg(feature = "sound-editor-plugin")]
fn sound_registration_with_root() -> EditorPluginRegistrationReport {
    static REGISTRATION: std::sync::OnceLock<EditorPluginRegistrationReport> =
        std::sync::OnceLock::new();

    // Sound discovery uses only linked descriptors and a compile-time root. Build
    // it once; each caller owns a clone, including its later lifecycle state.
    REGISTRATION
        .get_or_init(|| {
            zircon_plugin_sound_editor::plugin_registration()
                .with_ui_template_root(sound_ui_template_root())
        })
        .clone()
}

#[cfg(feature = "sound-editor-plugin")]
pub(super) fn sound_ui_template_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("sound")
}
