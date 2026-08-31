#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopPolicy {
    Game,
    DesktopApp,
    Mobile,
    Continuous,
    Headless,
}

impl EventLoopPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::DesktopApp => "desktop_app",
            Self::Mobile => "mobile",
            Self::Continuous => "continuous",
            Self::Headless => "headless",
        }
    }
}
