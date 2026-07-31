pub const MAX_WELCOME_RELEASES_SHOWN: usize = 5;
const ARMORY_INTENT_KEY: &str = "woc.welcome.openArmoryIntent";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WelcomePlatformInput {
    pub native_app: bool,
    pub desktop_app: bool,
    pub mobile_touch: bool,
    pub offline: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WelcomeDiscordInput {
    pub enabled: Option<bool>,
    pub linked: Option<bool>,
    pub guild_member: Option<bool>,
    pub fetch_failed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WelcomeChestInput {
    pub ready: bool,
    pub unknown: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WelcomeConnectionInput {
    pub ready: bool,
    pub offline: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeNewsState {
    Loading,
    Loaded,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinueButtonState {
    Connecting,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeContinueHint {
    Keyboard,
    Touch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WelcomeReleaseSummary {
    pub id: u64,
    pub tag: String,
    pub name: String,
    pub published_at: String,
}

pub trait WelcomeRelease {
    fn release_id(&self) -> u64;
}

impl WelcomeRelease for WelcomeReleaseSummary {
    fn release_id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkedWelcomeRelease<T> {
    pub release: T,
    pub is_new: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WelcomeNewsInput<T> {
    pub state: WelcomeNewsState,
    pub releases: Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WelcomeScreenView<T> {
    pub news_state: WelcomeNewsState,
    pub releases: Vec<MarkedWelcomeRelease<T>>,
    pub show_armory_card: bool,
    pub show_chest_tile: bool,
    pub show_discord_strip: bool,
    pub continue_state: ContinueButtonState,
}

pub trait WelcomeSessionStorage {
    fn get_item(&self, key: &str) -> Option<String>;
    fn set_item(&mut self, key: &str, value: &str);
    fn remove_item(&mut self, key: &str);
}

fn should_show_store_promo(platform: WelcomePlatformInput) -> bool {
    !platform.native_app && !platform.desktop_app && !platform.mobile_touch
}

pub fn armory_card_visible(
    platform: WelcomePlatformInput,
    armory_promo_enabled_on_server: bool,
) -> bool {
    !platform.offline && should_show_store_promo(platform) && armory_promo_enabled_on_server
}

pub fn chest_tile_visible(platform: WelcomePlatformInput, chest: WelcomeChestInput) -> bool {
    !platform.offline && should_show_store_promo(platform) && chest.ready && !chest.unknown
}

pub fn discord_strip_visible(discord: WelcomeDiscordInput, offline: bool) -> bool {
    if offline || discord.fetch_failed {
        return true;
    }
    if discord.enabled == Some(false) {
        return false;
    }
    !(discord.linked == Some(true) && discord.guild_member == Some(true))
}

pub fn continue_button_state(connection: WelcomeConnectionInput) -> ContinueButtonState {
    if connection.offline || connection.ready {
        ContinueButtonState::Ready
    } else {
        ContinueButtonState::Connecting
    }
}

pub fn welcome_continue_hint(platform: WelcomePlatformInput) -> WelcomeContinueHint {
    if platform.mobile_touch || platform.native_app {
        WelcomeContinueHint::Touch
    } else {
        WelcomeContinueHint::Keyboard
    }
}

pub fn mark_new_releases<T: WelcomeRelease + Clone>(
    releases: &[T],
    last_seen_release_id: Option<u64>,
) -> Vec<MarkedWelcomeRelease<T>> {
    releases
        .iter()
        .take(MAX_WELCOME_RELEASES_SHOWN)
        .cloned()
        .map(|release| MarkedWelcomeRelease {
            is_new: last_seen_release_id.is_none_or(|last_seen| release.release_id() > last_seen),
            release,
        })
        .collect()
}

pub fn next_last_seen_release_id<T: WelcomeRelease>(
    releases: &[T],
    previous: Option<u64>,
) -> Option<u64> {
    releases
        .iter()
        .map(WelcomeRelease::release_id)
        .max()
        .map_or(previous, |maximum| {
            Some(previous.map_or(maximum, |old| old.max(maximum)))
        })
}

#[allow(clippy::too_many_arguments)]
pub fn build_welcome_screen_view<T: WelcomeRelease + Clone>(
    platform: WelcomePlatformInput,
    news: &WelcomeNewsInput<T>,
    discord: WelcomeDiscordInput,
    chest: WelcomeChestInput,
    connection: WelcomeConnectionInput,
    armory_promo_enabled_on_server: bool,
    last_seen_release_id: Option<u64>,
) -> WelcomeScreenView<T> {
    WelcomeScreenView {
        news_state: news.state,
        releases: mark_new_releases(&news.releases, last_seen_release_id),
        show_armory_card: armory_card_visible(platform, armory_promo_enabled_on_server),
        show_chest_tile: chest_tile_visible(platform, chest),
        show_discord_strip: discord_strip_visible(discord, platform.offline),
        continue_state: continue_button_state(connection),
    }
}

pub fn set_armory_open_intent(storage: &mut impl WelcomeSessionStorage) {
    storage.set_item(ARMORY_INTENT_KEY, "1");
}

pub fn consume_armory_open_intent(storage: &mut impl WelcomeSessionStorage) -> bool {
    let had_intent = storage.get_item(ARMORY_INTENT_KEY).as_deref() == Some("1");
    if had_intent {
        storage.remove_item(ARMORY_INTENT_KEY);
    }
    had_intent
}
