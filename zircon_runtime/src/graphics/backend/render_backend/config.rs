#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderBackendConfig {
    pub(crate) backends: wgpu::Backends,
    pub(crate) instance_flags: wgpu::InstanceFlags,
}

impl RenderBackendConfig {
    pub(crate) fn from_environment() -> Self {
        Self::from_env_values(
            std::env::var("WGPU_BACKEND").ok().as_deref(),
            std::env::var("WGPU_DEBUG").ok().as_deref(),
            std::env::var("WGPU_VALIDATION").ok().as_deref(),
        )
    }

    pub(crate) fn from_env_values(
        backend: Option<&str>,
        debug: Option<&str>,
        validation: Option<&str>,
    ) -> Self {
        let mut instance_flags = wgpu::InstanceFlags::from_build_config();
        apply_optional_flag(&mut instance_flags, wgpu::InstanceFlags::DEBUG, debug);
        apply_optional_flag(
            &mut instance_flags,
            wgpu::InstanceFlags::VALIDATION,
            validation,
        );

        Self {
            backends: backend
                .map(wgpu::Backends::from_comma_list)
                .unwrap_or_default(),
            instance_flags,
        }
    }

    pub(crate) fn instance_descriptor(self) -> wgpu::InstanceDescriptor {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = self.backends;
        descriptor.flags = self.instance_flags;
        descriptor.backend_options = wgpu::BackendOptions::from_env_or_default();
        descriptor
    }
}

fn apply_optional_flag(
    flags: &mut wgpu::InstanceFlags,
    flag: wgpu::InstanceFlags,
    value: Option<&str>,
) {
    if let Some(value) = value {
        flags.set(flag, value != "0");
    }
}
