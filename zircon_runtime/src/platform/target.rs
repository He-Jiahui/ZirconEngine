use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformTarget {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    WebGpu,
    Wasm,
    Headless,
}

impl PlatformTarget {
    pub const ALL: [Self; 8] = [
        Self::Windows,
        Self::Linux,
        Self::Macos,
        Self::Android,
        Self::Ios,
        Self::WebGpu,
        Self::Wasm,
        Self::Headless,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::WebGpu => "web_gpu",
            Self::Wasm => "wasm",
            Self::Headless => "headless",
        }
    }

    pub const fn is_desktop(self) -> bool {
        matches!(self, Self::Windows | Self::Linux | Self::Macos)
    }

    pub const fn is_mobile(self) -> bool {
        matches!(self, Self::Android | Self::Ios)
    }

    pub const fn is_browser(self) -> bool {
        matches!(self, Self::WebGpu | Self::Wasm)
    }

    pub fn current() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::Wasm
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "ios") {
            Self::Ios
        } else {
            Self::Headless
        }
    }
}
