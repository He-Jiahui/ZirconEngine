#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum HostPresenterBackend {
    Gpu,
    Softbuffer,
}

impl HostPresenterBackend {
    pub(in crate::ui::retained_host::host_contract) const fn default_native() -> Self {
        Self::Gpu
    }

    pub(in crate::ui::retained_host::host_contract) const fn fallback() -> Self {
        Self::Softbuffer
    }

    pub(in crate::ui::retained_host::host_contract) const fn label(self) -> &'static str {
        match self {
            Self::Gpu => "gpu",
            Self::Softbuffer => "softbuffer",
        }
    }

    pub(in crate::ui::retained_host::host_contract) const fn is_gpu(self) -> bool {
        matches!(self, Self::Gpu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_host_defaults_to_gpu_backend() {
        assert_eq!(
            HostPresenterBackend::default_native(),
            HostPresenterBackend::Gpu
        );
        assert_eq!(
            HostPresenterBackend::fallback(),
            HostPresenterBackend::Softbuffer
        );
    }
}
