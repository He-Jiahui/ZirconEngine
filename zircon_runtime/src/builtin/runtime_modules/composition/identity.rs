use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::RuntimeProfileId;
use crate::core::{ModuleDescriptor, StartupMode};
use crate::plugin::{CompiledProjectPluginPlan, PluginCatalogGeneration};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeModuleCompositionIdentity {
    catalog_generation: Option<PluginCatalogGeneration>,
    source_manifest_fingerprint: Option<u64>,
    target_mode: RuntimeTargetMode,
    runtime_profile: Option<RuntimeProfileId>,
    composition_hash: [u8; 32],
}

impl RuntimeModuleCompositionIdentity {
    pub fn catalog_generation(&self) -> Option<PluginCatalogGeneration> {
        self.catalog_generation
    }

    pub fn source_manifest_fingerprint(&self) -> Option<u64> {
        self.source_manifest_fingerprint
    }

    pub fn target_mode(&self) -> RuntimeTargetMode {
        self.target_mode
    }

    pub fn runtime_profile(&self) -> Option<RuntimeProfileId> {
        self.runtime_profile
    }

    pub fn composition_hash(&self) -> [u8; 32] {
        self.composition_hash
    }

    pub fn composition_hash_hex(&self) -> String {
        blake3::Hash::from(self.composition_hash)
            .to_hex()
            .to_string()
    }
}

#[derive(Clone, Copy)]
pub(in crate::builtin::runtime_modules) struct RuntimeModuleCompositionIdentitySeed {
    catalog_generation: Option<PluginCatalogGeneration>,
    source_manifest_fingerprint: Option<u64>,
    target_mode: RuntimeTargetMode,
    runtime_profile: Option<RuntimeProfileId>,
}

impl RuntimeModuleCompositionIdentitySeed {
    pub(super) fn compiled(
        plan: &CompiledProjectPluginPlan,
        runtime_profile: Option<RuntimeProfileId>,
    ) -> Self {
        Self {
            catalog_generation: Some(plan.catalog_generation()),
            source_manifest_fingerprint: Some(plan.source_manifest_fingerprint()),
            target_mode: plan.target_mode(),
            runtime_profile,
        }
    }

    pub(in crate::builtin::runtime_modules) fn legacy(
        target_mode: RuntimeTargetMode,
        runtime_profile: Option<RuntimeProfileId>,
    ) -> Self {
        Self {
            catalog_generation: None,
            source_manifest_fingerprint: None,
            target_mode,
            runtime_profile,
        }
    }

    pub(super) fn finish(
        self,
        descriptors: &[ModuleDescriptor],
    ) -> RuntimeModuleCompositionIdentity {
        let mut hasher = blake3::Hasher::new();
        write_bytes(&mut hasher, b"zircon.runtime.module-composition.v1");
        write_optional_generation(&mut hasher, self.catalog_generation);
        write_optional_u64(&mut hasher, self.source_manifest_fingerprint);
        write_bytes(&mut hasher, runtime_target_key(self.target_mode));
        write_optional_profile(&mut hasher, self.runtime_profile);
        write_len(&mut hasher, descriptors.len());
        for descriptor in descriptors {
            write_descriptor(&mut hasher, descriptor);
        }
        RuntimeModuleCompositionIdentity {
            catalog_generation: self.catalog_generation,
            source_manifest_fingerprint: self.source_manifest_fingerprint,
            target_mode: self.target_mode,
            runtime_profile: self.runtime_profile,
            composition_hash: *hasher.finalize().as_bytes(),
        }
    }
}

fn write_descriptor(hasher: &mut blake3::Hasher, descriptor: &ModuleDescriptor) {
    write_str(hasher, &descriptor.name);
    write_str(hasher, &descriptor.description);
    write_str(hasher, descriptor.init_level.as_str());
    write_len(hasher, descriptor.module_dependencies.len());
    for dependency in &descriptor.module_dependencies {
        write_str(hasher, &dependency.module_name);
    }
    write_services(hasher, b"driver", &descriptor.drivers);
    write_services(hasher, b"manager", &descriptor.managers);
    write_services(hasher, b"plugin", &descriptor.plugins);
}

fn write_services<T>(hasher: &mut blake3::Hasher, kind: &[u8], services: &[T])
where
    T: CompositionServiceDescriptor,
{
    write_bytes(hasher, kind);
    write_len(hasher, services.len());
    for service in services {
        write_str(hasher, service.name());
        write_bytes(
            hasher,
            match service.startup_mode() {
                StartupMode::Immediate => b"immediate",
                StartupMode::Lazy => b"lazy",
            },
        );
        write_len(hasher, service.dependencies().len());
        for dependency in service.dependencies() {
            write_str(hasher, dependency.name.as_str());
        }
    }
}

trait CompositionServiceDescriptor {
    fn name(&self) -> &str;
    fn startup_mode(&self) -> StartupMode;
    fn dependencies(&self) -> &[crate::core::DependencySpec];
}

macro_rules! impl_composition_service_descriptor {
    ($descriptor:ty) => {
        impl CompositionServiceDescriptor for $descriptor {
            fn name(&self) -> &str {
                self.name.as_str()
            }

            fn startup_mode(&self) -> StartupMode {
                self.startup_mode
            }

            fn dependencies(&self) -> &[crate::core::DependencySpec] {
                &self.dependencies
            }
        }
    };
}

impl_composition_service_descriptor!(crate::core::DriverDescriptor);
impl_composition_service_descriptor!(crate::core::ManagerDescriptor);
impl_composition_service_descriptor!(crate::core::PluginDescriptor);

fn write_optional_profile(hasher: &mut blake3::Hasher, profile: Option<RuntimeProfileId>) {
    match profile {
        Some(profile) => {
            write_bytes(hasher, b"profile");
            write_bytes(hasher, runtime_profile_key(profile));
        }
        None => write_bytes(hasher, b"no-profile"),
    }
}

fn write_optional_u64(hasher: &mut blake3::Hasher, value: Option<u64>) {
    match value {
        Some(value) => {
            write_bytes(hasher, b"some");
            write_bytes(hasher, &value.to_le_bytes());
        }
        None => write_bytes(hasher, b"none"),
    }
}

fn write_optional_generation(
    hasher: &mut blake3::Hasher,
    generation: Option<PluginCatalogGeneration>,
) {
    write_optional_u64(hasher, generation.map(PluginCatalogGeneration::get));
}

fn runtime_target_key(target: RuntimeTargetMode) -> &'static [u8] {
    match target {
        RuntimeTargetMode::ClientRuntime => b"client-runtime",
        RuntimeTargetMode::ServerRuntime => b"server-runtime",
        RuntimeTargetMode::EditorHost => b"editor-host",
    }
}

fn runtime_profile_key(profile: RuntimeProfileId) -> &'static [u8] {
    match profile {
        RuntimeProfileId::Minimal => b"minimal",
        RuntimeProfileId::Client2d => b"client-2d",
        RuntimeProfileId::Client3d => b"client-3d",
        RuntimeProfileId::Editor => b"editor",
        RuntimeProfileId::Dev => b"dev",
        RuntimeProfileId::Server => b"server",
    }
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    write_bytes(hasher, value.as_bytes());
}

fn write_bytes(hasher: &mut blake3::Hasher, value: &[u8]) {
    write_len(hasher, value.len());
    hasher.update(value);
}

fn write_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}
