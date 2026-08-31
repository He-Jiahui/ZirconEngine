#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadBackend {
    Gilrs,
    BrowserGamepadApi,
}

impl GamepadBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gilrs => "gilrs",
            Self::BrowserGamepadApi => "browser_gamepad_api",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadEventBackend {
    GilrsEventPolling,
    BrowserGamepadApiPolling,
}

impl GamepadEventBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GilrsEventPolling => "gilrs_event_polling",
            Self::BrowserGamepadApiPolling => "browser_gamepad_api_polling",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadRumbleBackend {
    GilrsForceFeedback,
    BrowserGamepadHaptics,
}

impl GamepadRumbleBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GilrsForceFeedback => "gilrs_force_feedback",
            Self::BrowserGamepadHaptics => "browser_gamepad_haptics",
        }
    }
}
