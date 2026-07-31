use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RenderMaterialPropertyValue;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialPropertyOverrideBlock {
    values: BTreeMap<String, RenderMaterialPropertyValue>,
}

impl Serialize for MaterialPropertyOverrideBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MaterialPropertyOverrideBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BTreeMap::<String, RenderMaterialPropertyValue>::deserialize(deserializer)
            .map(Self::from_values)
    }
}

impl MaterialPropertyOverrideBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_values(values: BTreeMap<String, RenderMaterialPropertyValue>) -> Self {
        Self { values }
    }

    pub fn with_value(
        mut self,
        name: impl Into<String>,
        value: RenderMaterialPropertyValue,
    ) -> Self {
        self.values.insert(name.into(), value);
        self
    }

    pub fn insert(&mut self, name: impl Into<String>, value: RenderMaterialPropertyValue) {
        self.values.insert(name.into(), value);
    }

    pub fn values(&self) -> &BTreeMap<String, RenderMaterialPropertyValue> {
        &self.values
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_property_override_block_keeps_transparent_value_map_shape() {
        let block = MaterialPropertyOverrideBlock::new()
            .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.5 });

        let encoded = serde_json::to_string(&block).expect("override block should serialize");
        let decoded: MaterialPropertyOverrideBlock =
            serde_json::from_str(&encoded).expect("override block should deserialize");

        assert!(encoded.contains("gain"));
        assert!(!encoded.contains("values"));
        assert_eq!(decoded, block);
        assert!(
            serde_json::from_str::<MaterialPropertyOverrideBlock>("{}")
                .expect("empty override map should deserialize")
                .is_empty()
        );
    }
}
