use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportTargetPlatform, ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::{
    RenderProductFeature, RenderProductProfile, RenderProfileBundle,
};
use zircon_runtime::core::framework::window::WindowDescriptor;
use zircon_runtime::platform::PlatformTarget;
use zircon_runtime::plugin::RuntimeProfileDescriptor;

use crate::entry::EntryProfile;

use super::{
    EntryConfig, ProductCapabilityRequirement, ProductConfigSource, ProductConfigSourceSet,
    ProductHostConfigError, ProductHostConfigProvenance, ProductPlatformClass,
    ProductRoleDescriptor, ProductRoleRequest, ResolvedProductHostConfig,
};

impl EntryConfig {
    pub fn resolve(&self) -> Result<ResolvedProductHostConfig, ProductHostConfigError> {
        let role_descriptor = *self.role.descriptor();
        if !role_descriptor
            .artifact_manifest()
            .has_configuration_owner()
        {
            return Err(ProductHostConfigError::UnsupportedProductRole(self.role));
        }
        let profile = role_descriptor.entry_profile();
        let role_target = role_descriptor.target_mode();
        let (platform_target, platform_target_source) =
            resolve_platform_target(role_descriptor, self.export_profile.as_ref())?;
        let export_runtime_profile = match self.export_profile.as_ref() {
            Some(profile) => Some(
                profile
                    .runtime_profile_id
                    .ok_or(ProductHostConfigError::ExportRuntimeProfileMissing)?,
            ),
            None => None,
        };
        if let (Some(requested), Some(export)) = (self.runtime_profile, export_runtime_profile) {
            if requested != export {
                return Err(ProductHostConfigError::RuntimeProfileConflict { requested, export });
            }
        }
        let runtime_profile = export_runtime_profile.or(self.runtime_profile);
        if let Some(runtime_profile) = runtime_profile {
            let runtime_role = ProductRoleRequest::from_runtime_profile(runtime_profile);
            if runtime_role != self.role {
                return Err(ProductHostConfigError::RuntimeProfileRoleConflict {
                    role: self.role,
                    runtime_profile,
                });
            }
        }

        let runtime_descriptor = runtime_profile.map(RuntimeProfileDescriptor::for_id);
        if let Some(descriptor) = runtime_descriptor.as_ref() {
            ensure_target_mode(
                self.role,
                role_target,
                descriptor.target_mode,
                ProductConfigSource::RuntimeProfile,
            )?;
        }
        if let Some(target_mode) = self.target_mode {
            ensure_target_mode(
                self.role,
                role_target,
                target_mode,
                ProductConfigSource::EntryRequest,
            )?;
        }
        if let Some(export_profile) = self.export_profile.as_ref() {
            ensure_target_mode(
                self.role,
                role_target,
                export_profile.target_mode,
                ProductConfigSource::ExportProfile,
            )?;
        }

        if profile != EntryProfile::Editor
            && (self.editor_enabled_subsystems.is_some()
                || self.editor_runtime_sandbox_enabled.is_some())
        {
            return Err(ProductHostConfigError::EditorSettingsRequireEditorHost);
        }

        let (project_plugins, project_plugins_source) = resolve_project_plugins(
            self.project_plugins.as_ref(),
            self.project_plugins_source,
            runtime_descriptor.as_ref(),
            &self.required_runtime_plugins,
            &self.optional_runtime_plugins,
        )?;
        let (render_profile, render_profile_source) = self
            .render_profile
            .clone()
            .map(|profile| (profile, ProductConfigSource::EntryRequest))
            .unwrap_or_else(|| {
                (
                    default_render_profile_for_profile(profile),
                    ProductConfigSource::ProductRole,
                )
            });
        ensure_render_capability(
            self.role,
            role_descriptor.capabilities().render(),
            render_profile.profile(),
        )?;
        let (window_descriptor, window_descriptor_source) = self
            .window_descriptor
            .clone()
            .map(|descriptor| (descriptor, ProductConfigSource::EntryRequest))
            .unwrap_or_else(|| {
                if matches!(
                    runtime_profile,
                    Some(RuntimeProfileId::Minimal | RuntimeProfileId::Server)
                ) {
                    (
                        WindowDescriptor::default().without_primary_window(),
                        ProductConfigSource::RuntimeProfile,
                    )
                } else {
                    (
                        default_window_descriptor_for_profile(profile),
                        ProductConfigSource::ProductRole,
                    )
                }
            });
        ensure_window_capability(
            self.role,
            role_descriptor.capabilities().window(),
            window_descriptor.primary_window.is_some(),
        )?;

        let target_mode_source = if self.export_profile.is_some() {
            ProductConfigSource::ExportProfile
        } else if self.target_mode.is_some() {
            ProductConfigSource::EntryRequest
        } else if runtime_profile.is_some() {
            ProductConfigSource::RuntimeProfile
        } else {
            ProductConfigSource::ProductRole
        };
        let runtime_profile_source = if self.export_profile.is_some() {
            ProductConfigSource::ExportProfile
        } else if runtime_profile.is_some() {
            ProductConfigSource::RuntimeProfile
        } else {
            ProductConfigSource::ProductRole
        };
        let editor_enabled_subsystems_source = if self.editor_enabled_subsystems.is_some() {
            ProductConfigSource::EntryRequest
        } else {
            ProductConfigSource::ProductRole
        };
        let editor_runtime_sandbox_source = if self.editor_runtime_sandbox_enabled.is_some() {
            ProductConfigSource::EntryRequest
        } else {
            ProductConfigSource::ProductRole
        };

        Ok(ResolvedProductHostConfig {
            role: self.role,
            role_descriptor,
            profile,
            runtime_profile,
            target_mode: role_target,
            platform_target,
            project_plugins,
            export_profile: self.export_profile.clone(),
            render_profile,
            window_descriptor,
            editor_enabled_subsystems: self.editor_enabled_subsystems.clone(),
            editor_runtime_sandbox_enabled: self.editor_runtime_sandbox_enabled.unwrap_or(true),
            provenance: ProductHostConfigProvenance {
                profile: ProductConfigSource::ProductRole,
                runtime_profile: runtime_profile_source,
                target_mode: target_mode_source,
                platform_target: platform_target_source,
                project_plugins: project_plugins_source,
                export_profile: if self.export_profile.is_some() {
                    ProductConfigSource::ExportProfile
                } else {
                    ProductConfigSource::ProductRole
                },
                render_profile: render_profile_source,
                window_descriptor: window_descriptor_source,
                editor_enabled_subsystems: editor_enabled_subsystems_source,
                editor_runtime_sandbox: editor_runtime_sandbox_source,
            },
        })
    }
}

fn resolve_platform_target(
    descriptor: ProductRoleDescriptor,
    export_profile: Option<&zircon_runtime::core::framework::project::ExportProfile>,
) -> Result<(PlatformTarget, ProductConfigSource), ProductHostConfigError> {
    let platform_class = descriptor.capabilities().platform();
    let (target, source) = match export_profile {
        Some(profile) => (
            platform_target_from_export_target(profile.target_platform),
            ProductConfigSource::ExportProfile,
        ),
        None => (
            default_platform_target(platform_class),
            ProductConfigSource::ProductRole,
        ),
    };
    if platform_class.accepts(target) {
        Ok((target, source))
    } else {
        Err(ProductHostConfigError::PlatformTargetConflict {
            role: descriptor.role(),
            expected: platform_class,
            actual: target,
        })
    }
}

fn ensure_render_capability(
    role: ProductRoleRequest,
    requirement: ProductCapabilityRequirement,
    profile: RenderProductProfile,
) -> Result<(), ProductHostConfigError> {
    let enabled = profile != RenderProductProfile::Headless;
    match (requirement, enabled) {
        (ProductCapabilityRequirement::Required, false) => {
            Err(ProductHostConfigError::RenderCapabilityRequired(role))
        }
        (ProductCapabilityRequirement::Forbidden, true) => {
            Err(ProductHostConfigError::RenderCapabilityForbidden(role))
        }
        _ => Ok(()),
    }
}

fn ensure_window_capability(
    role: ProductRoleRequest,
    requirement: ProductCapabilityRequirement,
    has_primary_window: bool,
) -> Result<(), ProductHostConfigError> {
    match (requirement, has_primary_window) {
        (ProductCapabilityRequirement::Required, false) => {
            Err(ProductHostConfigError::WindowCapabilityRequired(role))
        }
        (ProductCapabilityRequirement::Forbidden, true) => {
            Err(ProductHostConfigError::WindowCapabilityForbidden(role))
        }
        _ => Ok(()),
    }
}

const fn platform_target_from_export_target(target: ExportTargetPlatform) -> PlatformTarget {
    match target {
        ExportTargetPlatform::Windows => PlatformTarget::Windows,
        ExportTargetPlatform::Linux => PlatformTarget::Linux,
        ExportTargetPlatform::Macos => PlatformTarget::Macos,
        ExportTargetPlatform::Android => PlatformTarget::Android,
        ExportTargetPlatform::Ios => PlatformTarget::Ios,
        ExportTargetPlatform::WebGpu => PlatformTarget::WebGpu,
        ExportTargetPlatform::Wasm => PlatformTarget::Wasm,
        ExportTargetPlatform::Headless => PlatformTarget::Headless,
    }
}

fn default_platform_target(platform_class: ProductPlatformClass) -> PlatformTarget {
    match platform_class {
        ProductPlatformClass::Desktop | ProductPlatformClass::HostProvided => {
            PlatformTarget::current()
        }
        ProductPlatformClass::DesktopOrHeadless => PlatformTarget::Headless,
        ProductPlatformClass::Browser => PlatformTarget::Wasm,
        ProductPlatformClass::Mobile => PlatformTarget::Android,
    }
}

fn ensure_target_mode(
    role: ProductRoleRequest,
    expected: RuntimeTargetMode,
    actual: RuntimeTargetMode,
    source: ProductConfigSource,
) -> Result<(), ProductHostConfigError> {
    if expected == actual {
        return Ok(());
    }
    Err(ProductHostConfigError::TargetModeConflict {
        source,
        role,
        expected,
        actual,
    })
}

fn resolve_project_plugins(
    requested: Option<&ProjectPluginManifest>,
    requested_source: Option<ProductConfigSource>,
    runtime_profile: Option<&RuntimeProfileDescriptor>,
    required: &[zircon_runtime::builtin::RuntimePluginId],
    optional: &[zircon_runtime::builtin::RuntimePluginId],
) -> Result<(Option<ProjectPluginManifest>, ProductConfigSourceSet), ProductHostConfigError> {
    if let Some(conflict) = required
        .iter()
        .find(|required_id| optional.iter().any(|id| id == *required_id))
    {
        return Err(ProductHostConfigError::RuntimePluginRequirementConflict(
            conflict.clone(),
        ));
    }

    if requested.is_none()
        && runtime_profile.is_none()
        && required.is_empty()
        && optional.is_empty()
    {
        return Ok((
            None,
            ProductConfigSourceSet::single(ProductConfigSource::ProductRole),
        ));
    }

    let mut manifest = runtime_profile
        .map(RuntimeProfileDescriptor::project_manifest)
        .unwrap_or_default();
    let mut sources = ProductConfigSourceSet::empty();
    if runtime_profile.is_some() {
        sources = sources.with(ProductConfigSource::RuntimeProfile);
    }
    if let Some(requested) = requested {
        for selection in &requested.selections {
            manifest.set_enabled(selection.clone());
        }
        sources = sources.with(requested_source.unwrap_or(ProductConfigSource::EntryRequest));
    }
    if !required.is_empty() || !optional.is_empty() {
        sources = sources.with(ProductConfigSource::EntryRequest);
    }
    for id in required {
        apply_runtime_plugin_requirement(&mut manifest, id, true);
    }
    for id in optional {
        apply_runtime_plugin_requirement(&mut manifest, id, false);
    }

    Ok((Some(manifest), sources))
}

fn apply_runtime_plugin_requirement(
    manifest: &mut ProjectPluginManifest,
    id: &zircon_runtime::builtin::RuntimePluginId,
    required: bool,
) {
    if let Some(existing) = manifest
        .selections
        .iter_mut()
        .find(|selection| selection.id == id.key())
    {
        existing.enabled = true;
        existing.required |= required;
        return;
    }
    manifest.set_enabled(ProjectPluginSelection::runtime_plugin(
        id.clone(),
        true,
        required,
    ));
}

fn default_render_profile_for_profile(profile: EntryProfile) -> RenderProfileBundle {
    match profile {
        EntryProfile::Editor => RenderProfileBundle::default_render()
            .with_features([RenderProductFeature::HybridGlobalIllumination]),
        EntryProfile::Runtime => RenderProfileBundle::default_render(),
        EntryProfile::Headless => RenderProfileBundle::headless(),
    }
}

fn default_window_descriptor_for_profile(profile: EntryProfile) -> WindowDescriptor {
    match profile {
        EntryProfile::Editor | EntryProfile::Runtime => WindowDescriptor::default(),
        EntryProfile::Headless => WindowDescriptor::default().without_primary_window(),
    }
}
