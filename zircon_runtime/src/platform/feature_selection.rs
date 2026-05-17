use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformFeatureSelection {
    pub platform_window: bool,
    pub platform_winit: bool,
    pub platform_headless: bool,
    pub platform_x11: bool,
    pub platform_wayland: bool,
    pub platform_android_game_activity: bool,
    pub platform_android_native_activity: bool,
    pub platform_web: bool,
    pub input_mouse: bool,
    pub input_keyboard: bool,
    pub input_touch: bool,
    pub input_gestures: bool,
    pub input_gamepad: bool,
    pub gamepad_gilrs: bool,
    #[serde(default)]
    pub gamepad_browser: bool,
}

impl PlatformFeatureSelection {
    pub const fn none() -> Self {
        Self {
            platform_window: false,
            platform_winit: false,
            platform_headless: false,
            platform_x11: false,
            platform_wayland: false,
            platform_android_game_activity: false,
            platform_android_native_activity: false,
            platform_web: false,
            input_mouse: false,
            input_keyboard: false,
            input_touch: false,
            input_gestures: false,
            input_gamepad: false,
            gamepad_gilrs: false,
            gamepad_browser: false,
        }
    }

    pub const fn bevy_default_platform() -> Self {
        Self {
            platform_window: true,
            platform_winit: true,
            platform_headless: false,
            platform_x11: true,
            platform_wayland: true,
            platform_android_game_activity: true,
            platform_android_native_activity: false,
            platform_web: true,
            input_mouse: true,
            input_keyboard: true,
            input_touch: true,
            input_gestures: false,
            input_gamepad: true,
            gamepad_gilrs: true,
            gamepad_browser: false,
        }
    }

    pub const fn headless() -> Self {
        Self {
            platform_headless: true,
            ..Self::none()
        }
    }

    pub fn from_compiled_features() -> Self {
        Self {
            platform_window: cfg!(feature = "platform-window"),
            platform_winit: cfg!(feature = "platform-winit"),
            platform_headless: cfg!(feature = "platform-headless"),
            platform_x11: cfg!(feature = "platform-x11"),
            platform_wayland: cfg!(feature = "platform-wayland"),
            platform_android_game_activity: cfg!(feature = "platform-android-game-activity"),
            platform_android_native_activity: cfg!(feature = "platform-android-native-activity"),
            platform_web: cfg!(feature = "platform-web"),
            input_mouse: cfg!(feature = "input-mouse"),
            input_keyboard: cfg!(feature = "input-keyboard"),
            input_touch: cfg!(feature = "input-touch"),
            input_gestures: cfg!(feature = "input-gestures"),
            input_gamepad: cfg!(feature = "input-gamepad"),
            gamepad_gilrs: cfg!(feature = "gamepad-gilrs"),
            gamepad_browser: cfg!(feature = "gamepad-browser"),
        }
    }
}

impl Default for PlatformFeatureSelection {
    fn default() -> Self {
        Self::from_compiled_features()
    }
}
