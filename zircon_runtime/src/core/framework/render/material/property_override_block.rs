use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RenderMaterialPropertyValue;

#[derive(Clone, Debug, Default, PartialEq)]
/// 单次绘制可叠加的具名属性覆盖；序列化为透明映射，材质资产与已发布 uniform 保持原样。
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
#[path = "../../../render/material/tests/property_override_block.rs"]
mod tests;

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for MaterialPropertyOverrideBlock {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        zircon_runtime_interface::retained_memory::RetainedOwnerV1::retained_bound_v1(
            &self.values,
            m,
        )
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1
    for MaterialPropertyOverrideBlock
{
}
