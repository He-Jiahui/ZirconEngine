use super::character_roster::{normalize_character_name, CharacterNameError};
use super::{offline_class_presentation, offline_class_preview, OfflineClassPreview};
use woc_protocol::{
    OfflineSessionBootstrap, OfflineWeaponSkinAccount, OFFLINE_SESSION_BOOTSTRAP_VERSION,
    STANDARD_OFFLINE_WORLD_SEED,
};

pub const OFFLINE_SESSION_LAUNCH_VERSION: u16 = OFFLINE_SESSION_BOOTSTRAP_VERSION;
pub const OFFLINE_WORLD_SEED: u32 = STANDARD_OFFLINE_WORLD_SEED;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflinePlayerClass {
    Warrior,
    Paladin,
    Hunter,
    Rogue,
    Priest,
    Shaman,
    Mage,
    Warlock,
    Druid,
}

impl OfflinePlayerClass {
    pub const ALL: [Self; 9] = [
        Self::Warrior,
        Self::Paladin,
        Self::Hunter,
        Self::Rogue,
        Self::Priest,
        Self::Shaman,
        Self::Mage,
        Self::Warlock,
        Self::Druid,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warrior => "warrior",
            Self::Paladin => "paladin",
            Self::Hunter => "hunter",
            Self::Rogue => "rogue",
            Self::Priest => "priest",
            Self::Shaman => "shaman",
            Self::Mage => "mage",
            Self::Warlock => "warlock",
            Self::Druid => "druid",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|player_class| player_class.as_str() == value)
    }

    /// Target content uses this stable order for fresh `Sim.addPlayer` facts;
    /// it differs from the picker's retained presentation order.
    pub const fn bootstrap_index(self) -> u8 {
        match self {
            Self::Warrior => 0,
            Self::Mage => 1,
            Self::Rogue => 2,
            Self::Paladin => 3,
            Self::Hunter => 4,
            Self::Priest => 5,
            Self::Shaman => 6,
            Self::Warlock => 7,
            Self::Druid => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineSessionError {
    CharacterName(CharacterNameError),
    MissingPlayerClass,
    InvalidSkinVariant {
        player_class: OfflinePlayerClass,
        skin_variant: u16,
        skin_count: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflineSessionLaunch {
    pub schema_version: u16,
    pub player_class: OfflinePlayerClass,
    pub player_name: String,
    pub skin_variant: u16,
    pub world_seed: u32,
    pub weapon_skin_account: OfflineWeaponSkinAccount,
}

impl OfflineSessionLaunch {
    /// Hosts load this account-owned cosmetic state before the first authoritative
    /// tick; the simulation owns all later admission and loadout mutations.
    pub fn with_weapon_skin_account(mut self, account: OfflineWeaponSkinAccount) -> Self {
        self.weapon_skin_account = account;
        self
    }

    pub fn preference_scope(&self) -> String {
        format!(
            "offline:{}:{}",
            self.player_class.as_str(),
            self.player_name
        )
    }

    pub fn bootstrap(&self) -> OfflineSessionBootstrap {
        OfflineSessionBootstrap {
            launch_version: self.schema_version,
            world_seed: self.world_seed,
            player_class: self.player_class.bootstrap_index(),
            player_name: self.player_name.clone(),
            skin_variant: self.skin_variant,
            weapon_skin_account: self.weapon_skin_account.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflineSessionDraft {
    player_class: Option<OfflinePlayerClass>,
    raw_name: String,
    skin_variant: u16,
    weapon_skin_account: OfflineWeaponSkinAccount,
}

impl Default for OfflineSessionDraft {
    fn default() -> Self {
        Self {
            player_class: None,
            raw_name: String::new(),
            skin_variant: 0,
            weapon_skin_account: OfflineWeaponSkinAccount::default(),
        }
    }
}

impl OfflineSessionDraft {
    pub fn set_player_class(&mut self, player_class: OfflinePlayerClass) {
        self.player_class = Some(player_class);
        self.skin_variant = 0;
    }

    pub fn set_raw_name(&mut self, raw_name: impl Into<String>) {
        self.raw_name = raw_name.into();
    }

    pub fn set_skin_variant(&mut self, skin_variant: u16) {
        self.skin_variant = skin_variant;
    }

    pub fn set_weapon_skin_account(&mut self, account: OfflineWeaponSkinAccount) {
        self.weapon_skin_account = account;
    }

    pub fn player_class(&self) -> Option<OfflinePlayerClass> {
        self.player_class
    }

    pub fn raw_name(&self) -> &str {
        &self.raw_name
    }

    pub fn skin_variant(&self) -> u16 {
        self.skin_variant
    }

    pub fn preview(&self) -> Result<OfflineClassPreview, OfflineSessionError> {
        let player_class = self
            .player_class
            .ok_or(OfflineSessionError::MissingPlayerClass)?;
        offline_class_preview(player_class, self.skin_variant)
            .ok_or_else(|| self.invalid_skin_variant(player_class))
    }

    /// Builds a fresh standard-game session. The target fixes its normal offline world seed;
    /// editor playtests use a separate world-content handoff rather than this player flow.
    pub fn launch(&self) -> Result<OfflineSessionLaunch, OfflineSessionError> {
        let player_name =
            normalize_character_name(&self.raw_name).map_err(OfflineSessionError::CharacterName)?;
        let player_class = self
            .player_class
            .ok_or(OfflineSessionError::MissingPlayerClass)?;
        self.preview()?;
        Ok(OfflineSessionLaunch {
            schema_version: OFFLINE_SESSION_LAUNCH_VERSION,
            player_class,
            player_name,
            skin_variant: self.skin_variant,
            world_seed: OFFLINE_WORLD_SEED,
            weapon_skin_account: self.weapon_skin_account.clone(),
        })
    }

    fn invalid_skin_variant(&self, player_class: OfflinePlayerClass) -> OfflineSessionError {
        let skin_count = offline_class_presentation(player_class).skin_count;
        OfflineSessionError::InvalidSkinVariant {
            player_class,
            skin_variant: self.skin_variant,
            skin_count,
        }
    }
}
