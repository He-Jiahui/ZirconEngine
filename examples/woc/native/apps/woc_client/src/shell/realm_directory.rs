use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealmType {
    Normal,
    Pvp,
    Rp,
    RpPvp,
}

impl RealmType {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Normal => "realmTypes.normal",
            Self::Pvp => "realmTypes.pvp",
            Self::Rp => "realmTypes.rp",
            Self::RpPvp => "realmTypes.rpPvp",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealmDefinition {
    pub name: String,
    pub base_url: String,
    pub realm_type: RealmType,
    pub character_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealmStatus {
    pub online: bool,
    pub players: u32,
    pub cap: u32,
}

impl RealmStatus {
    pub const fn online(players: u32, cap: u32) -> Self {
        Self {
            online: true,
            players,
            cap,
        }
    }

    pub const fn offline() -> Self {
        Self {
            online: false,
            players: 0,
            cap: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealmPopulationBand {
    Offline,
    Full,
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealmPopulationView {
    pub band: RealmPopulationBand,
    pub label_key: &'static str,
    pub tip_key: &'static str,
    pub style_class: &'static str,
}

const fn population(
    band: RealmPopulationBand,
    label_key: &'static str,
    tip_key: &'static str,
    style_class: &'static str,
) -> RealmPopulationView {
    RealmPopulationView {
        band,
        label_key,
        tip_key,
        style_class,
    }
}

pub const fn realm_population(online: bool, players: u32, cap: u32) -> RealmPopulationView {
    if !online {
        return population(
            RealmPopulationBand::Offline,
            "realm.offline",
            "realm.popTipOffline",
            "offline",
        );
    }
    if cap > 0 && players >= cap {
        return population(
            RealmPopulationBand::Full,
            "realm.full",
            "realm.popTipFull",
            "full",
        );
    }
    if players >= 80 {
        return population(
            RealmPopulationBand::High,
            "realm.high",
            "realm.popTipHigh",
            "high",
        );
    }
    if players >= 15 {
        return population(
            RealmPopulationBand::Medium,
            "realm.medium",
            "realm.popTipMedium",
            "med",
        );
    }
    population(
        RealmPopulationBand::Low,
        "realm.low",
        "realm.popTipLow",
        "low",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealmStatusState {
    Checking,
    Resolved {
        status: RealmStatus,
        population: RealmPopulationView,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealmDirectoryRow {
    pub definition: RealmDefinition,
    pub status: RealmStatusState,
    pub recommended: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RealmDirectoryEffect {
    ShowList,
    SelectRealm {
        realm_name: String,
        base_url: String,
    },
    NavigateToModeSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RealmDirectoryError {
    EmptyRealmName { index: usize },
    DuplicateRealmName { realm_name: String },
    RealmNotFound { realm_name: String },
    StaleGeneration { expected: u64, actual: u64 },
}

pub struct RealmDirectoryModel {
    entries: Vec<RealmDirectoryRow>,
    generation: u64,
}

impl Default for RealmDirectoryModel {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            generation: 0,
        }
    }
}

impl RealmDirectoryModel {
    pub fn entries(&self) -> &[RealmDirectoryRow] {
        &self.entries
    }

    /// Identifies the directory snapshot that status probes belong to.
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn replace_directory(
        &mut self,
        definitions: Vec<RealmDefinition>,
        remembered_realm: Option<&str>,
    ) -> Result<RealmDirectoryEffect, RealmDirectoryError> {
        validate_definitions(&definitions)?;
        let entries = definitions
            .into_iter()
            .map(|definition| RealmDirectoryRow {
                definition,
                status: RealmStatusState::Checking,
                recommended: false,
            })
            .collect::<Vec<_>>();
        self.generation = self.generation.saturating_add(1);
        let remembered = remembered_realm.and_then(|remembered| {
            entries
                .iter()
                .find(|entry| entry.definition.name == remembered)
                .map(|entry| RealmDirectoryEffect::SelectRealm {
                    realm_name: entry.definition.name.clone(),
                    base_url: entry.definition.base_url.clone(),
                })
        });
        self.entries = entries;
        Ok(remembered.unwrap_or(RealmDirectoryEffect::ShowList))
    }

    pub fn set_status(
        &mut self,
        realm_name: &str,
        status: RealmStatus,
    ) -> Result<(), RealmDirectoryError> {
        self.set_status_for_generation(self.generation, realm_name, status)
    }

    /// Applies a probe only when it belongs to the currently displayed
    /// directory. Late responses from a replaced directory are rejected.
    pub fn set_status_for_generation(
        &mut self,
        generation: u64,
        realm_name: &str,
        status: RealmStatus,
    ) -> Result<(), RealmDirectoryError> {
        self.require_generation(generation)?;
        let row = self.entry_mut(realm_name)?;
        row.status = RealmStatusState::Resolved {
            status,
            population: realm_population(status.online, status.players, status.cap),
        };
        row.recommended = false;
        Ok(())
    }

    pub fn finish_status_refresh(&mut self) {
        self.finish_status_refresh_for_generation(self.generation);
    }

    /// Completes recommendation calculation for one directory generation.
    /// Returns `false` when the refresh was stale and no state changed.
    pub fn finish_status_refresh_for_generation(&mut self, generation: u64) -> bool {
        if self.require_generation(generation).is_err() {
            return false;
        }
        let mut best: Option<(usize, u32)> = None;
        for (index, entry) in self.entries.iter_mut().enumerate() {
            entry.recommended = false;
            let RealmStatusState::Resolved { status, .. } = entry.status else {
                continue;
            };
            if status.online && best.is_none_or(|(_, players)| status.players < players) {
                best = Some((index, status.players));
            }
        }
        if let Some((index, _)) = best {
            self.entries[index].recommended = true;
        }
        true
    }

    pub fn recommended_realm_name(&self) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.recommended)
            .map(|entry| entry.definition.name.as_str())
    }

    pub fn select(&self, realm_name: &str) -> Result<RealmDirectoryEffect, RealmDirectoryError> {
        let entry = self.entry(realm_name)?;
        Ok(RealmDirectoryEffect::SelectRealm {
            realm_name: entry.definition.name.clone(),
            base_url: entry.definition.base_url.clone(),
        })
    }

    pub const fn back(&self) -> RealmDirectoryEffect {
        RealmDirectoryEffect::NavigateToModeSelection
    }

    fn entry(&self, realm_name: &str) -> Result<&RealmDirectoryRow, RealmDirectoryError> {
        self.entries
            .iter()
            .find(|entry| entry.definition.name == realm_name)
            .ok_or_else(|| RealmDirectoryError::RealmNotFound {
                realm_name: realm_name.to_string(),
            })
    }

    fn entry_mut(
        &mut self,
        realm_name: &str,
    ) -> Result<&mut RealmDirectoryRow, RealmDirectoryError> {
        self.entries
            .iter_mut()
            .find(|entry| entry.definition.name == realm_name)
            .ok_or_else(|| RealmDirectoryError::RealmNotFound {
                realm_name: realm_name.to_string(),
            })
    }

    fn require_generation(&self, generation: u64) -> Result<(), RealmDirectoryError> {
        if generation == self.generation {
            Ok(())
        } else {
            Err(RealmDirectoryError::StaleGeneration {
                expected: self.generation,
                actual: generation,
            })
        }
    }
}

fn validate_definitions(definitions: &[RealmDefinition]) -> Result<(), RealmDirectoryError> {
    let mut names = BTreeSet::new();
    for (index, definition) in definitions.iter().enumerate() {
        if definition.name.trim().is_empty() {
            return Err(RealmDirectoryError::EmptyRealmName { index });
        }
        if !names.insert(definition.name.as_str()) {
            return Err(RealmDirectoryError::DuplicateRealmName {
                realm_name: definition.name.clone(),
            });
        }
    }
    Ok(())
}
