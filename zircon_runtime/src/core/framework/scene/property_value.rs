use serde::{Deserialize, Serialize};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;

use super::{ComponentPropertyPath, EntityId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenePropertyValue {
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Scalar(Real),
    Vec2([Real; 2]),
    Vec3([Real; 3]),
    Vec4([Real; 4]),
    Quaternion([Real; 4]),
    String(String),
    Enum(String),
    Entity(Option<EntityId>),
    Resource(String),
    AnimationParameter(AnimationParameterValue),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ScenePropertyEntry {
    pub property_path: ComponentPropertyPath,
    pub value: ScenePropertyValue,
    pub animatable: bool,
}

impl ScenePropertyEntry {
    pub(crate) fn new(
        property_path: ComponentPropertyPath,
        value: ScenePropertyValue,
        animatable: bool,
    ) -> Self {
        Self {
            property_path,
            value,
            animatable,
        }
    }
}
