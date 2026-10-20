use std::collections::BTreeMap;

use zircon_runtime_interface::reflect::ReflectedValue;

use crate::scene::dynamic_scene::{DynamicSceneError, EntityRemap};

// 实体引用在提交前随源实体映射迁移；映射外的 ID 保持原值，以允许引用已有目标实体。
pub(in crate::scene::dynamic_scene) fn remap_reflected_value(
    value: &ReflectedValue,
    remap: &EntityRemap,
) -> Result<ReflectedValue, DynamicSceneError> {
    Ok(match value {
        ReflectedValue::Entity(Some(entity)) => {
            ReflectedValue::Entity(Some(remap.get(*entity).unwrap_or(*entity)))
        }
        ReflectedValue::List(values) => ReflectedValue::List(
            values
                .iter()
                .map(|value| remap_reflected_value(value, remap))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ReflectedValue::Map(values) => ReflectedValue::Map(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), remap_reflected_value(value, remap)?)))
                .collect::<Result<BTreeMap<_, _>, DynamicSceneError>>()?,
        ),
        value => value.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn typed_entity_remap_preserves_json_objects() {
        let mut remap = EntityRemap::new();
        remap.insert(11, 22);
        let payload = json!({
            "entity": 11,
            "nested": [{"entity": 11}, {"entity": null}],
        });
        let value = ReflectedValue::List(vec![
            ReflectedValue::Entity(Some(11)),
            ReflectedValue::Entity(None),
            ReflectedValue::Entity(Some(99)),
            ReflectedValue::Json(json!({"entity": 11})),
            ReflectedValue::Map(BTreeMap::from([
                ("reference".to_string(), ReflectedValue::Entity(Some(11))),
                ("payload".to_string(), ReflectedValue::Json(payload.clone())),
            ])),
        ]);
        let expected = ReflectedValue::List(vec![
            ReflectedValue::Entity(Some(22)),
            ReflectedValue::Entity(None),
            ReflectedValue::Entity(Some(99)),
            ReflectedValue::Json(json!({"entity": 11})),
            ReflectedValue::Map(BTreeMap::from([
                ("reference".to_string(), ReflectedValue::Entity(Some(22))),
                ("payload".to_string(), ReflectedValue::Json(payload)),
            ])),
        ]);

        assert_eq!(remap_reflected_value(&value, &remap).unwrap(), expected);
    }
}
