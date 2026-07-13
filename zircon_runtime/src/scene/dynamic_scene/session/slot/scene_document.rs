use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::scene::dynamic_scene::DynamicScene;

pub(super) fn serialize<S>(scene: &DynamicScene, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let text = scene
        .to_versioned_json_pretty()
        .map_err(serde::ser::Error::custom)?;
    let document: serde_json::Value =
        serde_json::from_str(&text).map_err(serde::ser::Error::custom)?;
    document.serialize(serializer)
}

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<DynamicScene, D::Error>
where
    D: Deserializer<'de>,
{
    let document = serde_json::Value::deserialize(deserializer)?;
    let text = serde_json::to_string(&document).map_err(serde::de::Error::custom)?;
    DynamicScene::from_versioned_json(&text).map_err(serde::de::Error::custom)
}
