use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::core::framework::project::ExportProfile;

pub(super) fn deserialize_export_profiles<'de, D>(
    deserializer: D,
) -> Result<Vec<ExportProfile>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ExportProfilesInput {
        List(Vec<ExportProfile>),
        Map(BTreeMap<String, ExportProfile>),
    }

    Ok(match ExportProfilesInput::deserialize(deserializer)? {
        ExportProfilesInput::List(profiles) => profiles,
        ExportProfilesInput::Map(profiles) => profiles
            .into_iter()
            .map(|(name, mut profile)| {
                if profile.name.is_empty() {
                    profile.name = name;
                }
                profile
            })
            .collect(),
    })
}
