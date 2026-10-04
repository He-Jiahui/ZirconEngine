//! 槽位场景使用版本化封装；头必须先于载荷，使解码器在读取载荷前确定支持范围与迁移路径。
use std::fmt;

use serde::{
    de::{self, DeserializeSeed, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::value::RawValue;
use zircon_runtime_interface::serialization::{PayloadHeader, VersionedSchema};

use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct SceneDocumentRef<'a> {
    #[serde(rename = "$zircon")]
    envelope: SceneEnvelopeRef<'a>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct SceneEnvelopeRef<'a> {
    header: &'a PayloadHeader,
    payload: &'a DynamicScene,
}

#[derive(Deserialize)]
#[serde(field_identifier)]
enum SceneDocumentField {
    #[serde(rename = "$zircon")]
    Envelope,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SceneEnvelopeField {
    Header,
    Payload,
}

pub(super) fn serialize<S>(scene: &DynamicScene, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    scene
        .ensure_supported()
        .map_err(serde::ser::Error::custom)?;
    SceneDocumentRef {
        envelope: SceneEnvelopeRef {
            header: &scene.payload_header,
            payload: scene,
        },
    }
    .serialize(serializer)
}

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<DynamicScene, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(SceneDocumentVisitor)
}

struct SceneDocumentVisitor;

impl<'de> Visitor<'de> for SceneDocumentVisitor {
    type Value = DynamicScene;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a zircon dynamic scene document")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut scene = None;
        while let Some(field) = map.next_key()? {
            match field {
                SceneDocumentField::Envelope => {
                    if scene.is_some() {
                        return Err(de::Error::duplicate_field("$zircon"));
                    }
                    scene = Some(map.next_value_seed(SceneEnvelopeSeed)?);
                }
            }
        }
        scene.ok_or_else(|| de::Error::missing_field("$zircon"))
    }
}

struct SceneEnvelopeSeed;

impl<'de> DeserializeSeed<'de> for SceneEnvelopeSeed {
    type Value = DynamicScene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SceneEnvelopeVisitor)
    }
}

struct SceneEnvelopeVisitor;

impl<'de> Visitor<'de> for SceneEnvelopeVisitor {
    type Value = DynamicScene;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a versioned zircon dynamic scene envelope")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut header = None;
        let mut typed_payload: Option<DynamicScene> = None;
        let mut raw_payload: Option<Box<RawValue>> = None;

        while let Some(field) = map.next_key()? {
            match field {
                SceneEnvelopeField::Header => {
                    if header.is_some() {
                        return Err(de::Error::duplicate_field("header"));
                    }
                    header = Some(map.next_value()?);
                }
                SceneEnvelopeField::Payload => {
                    if typed_payload.is_some() || raw_payload.is_some() {
                        return Err(de::Error::duplicate_field("payload"));
                    }
                    let Some(header) = header.as_ref() else {
                        return Err(de::Error::custom(
                            "zircon scene envelope header must precede payload",
                        ));
                    };
                    // 当前版本直接解码强类型场景；仅旧版本暂存原始 JSON，交给场景迁移入口处理。
                    if is_current_header(header) {
                        typed_payload = Some(map.next_value()?);
                    } else if is_supported_legacy_header(header).map_err(de::Error::custom)? {
                        raw_payload = Some(map.next_value()?);
                    }
                }
            }
        }

        let header = header.ok_or_else(|| de::Error::missing_field("header"))?;
        if is_current_header(&header) {
            if let Some(scene) = typed_payload {
                scene.ensure_supported().map_err(de::Error::custom)?;
                return Ok(scene);
            }
        }
        let payload = raw_payload.ok_or_else(|| de::Error::missing_field("payload"))?;
        DynamicScene::from_versioned_json_payload(header, &payload).map_err(de::Error::custom)
    }
}

fn is_current_header(header: &PayloadHeader) -> bool {
    header.schema_id == DynamicScene::SCHEMA && header.schema_version == DynamicScene::VERSION
}

fn is_supported_legacy_header(header: &PayloadHeader) -> Result<bool, DynamicSceneError> {
    if header.schema_id != DynamicScene::SCHEMA {
        return Err(DynamicSceneError::UnsupportedSchema {
            expected: DynamicScene::SCHEMA.as_str().to_string(),
            actual: header.schema_id.as_str().to_string(),
        });
    }
    if header.schema_version > DynamicScene::VERSION {
        return Err(DynamicSceneError::UnsupportedFormatVersion {
            expected: DynamicScene::VERSION,
            actual: header.schema_version,
        });
    }
    Ok(header.schema_version < DynamicScene::VERSION)
}

#[cfg(test)]
#[path = "tests/scene_document.rs"]
mod tests;
