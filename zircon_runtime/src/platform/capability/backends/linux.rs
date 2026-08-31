#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxWindowProtocol {
    X11,
    Wayland,
}

impl LinuxWindowProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}
