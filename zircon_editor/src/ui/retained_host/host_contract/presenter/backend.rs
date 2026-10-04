#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum HostPresenterBackend {
    Gpu,
    Softbuffer,
}

impl HostPresenterBackend {
    pub(in crate::ui::retained_host::host_contract) fn default_native() -> Self {
        if profile_force_softbuffer() {
            return Self::Softbuffer;
        }
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

fn profile_force_softbuffer() -> bool {
    if !cfg!(feature = "profiling") {
        return false;
    }
    std::env::var("ZIRCON_PROFILE_FORCE_SOFTBUFFER")
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "tests/backend.rs"]
mod tests;
