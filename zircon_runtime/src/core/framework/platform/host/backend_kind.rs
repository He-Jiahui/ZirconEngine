#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformHostBackendKind {
    Headless,
    Winit,
    Browser,
    AndroidNative,
    IosNative,
}

impl PlatformHostBackendKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Headless => "headless",
            Self::Winit => "winit",
            Self::Browser => "browser",
            Self::AndroidNative => "android_native",
            Self::IosNative => "ios_native",
        }
    }
}
