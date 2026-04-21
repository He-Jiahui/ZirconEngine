use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::AnimationTrackPathError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationTrackPath {
    raw: String,
}

impl AnimationTrackPath {
    pub fn new(entity_path: EntityPath, property_path: ComponentPropertyPath) -> Self {
        Self {
            raw: format!("{entity_path}:{property_path}"),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, AnimationTrackPathError> {
        let (entity_path, property_path) = raw.split_once(':').ok_or(AnimationTrackPathError)?;
        let entity_path = EntityPath::parse(entity_path).map_err(|_| AnimationTrackPathError)?;
        let property_path =
            ComponentPropertyPath::parse(property_path).map_err(|_| AnimationTrackPathError)?;
        Ok(Self {
            raw: format!("{entity_path}:{property_path}"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn split(&self) -> Result<(EntityPath, ComponentPropertyPath), AnimationTrackPathError> {
        let (entity_path, property_path) =
            self.raw.split_once(':').ok_or(AnimationTrackPathError)?;
        Ok((
            EntityPath::parse(entity_path).map_err(|_| AnimationTrackPathError)?,
            ComponentPropertyPath::parse(property_path).map_err(|_| AnimationTrackPathError)?,
        ))
    }

    pub fn entity_path(&self) -> Result<EntityPath, AnimationTrackPathError> {
        self.split().map(|(entity_path, _)| entity_path)
    }

    pub fn property_path(&self) -> Result<ComponentPropertyPath, AnimationTrackPathError> {
        self.split().map(|(_, property_path)| property_path)
    }
}

impl std::fmt::Display for AnimationTrackPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}
