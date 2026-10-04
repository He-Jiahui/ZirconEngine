use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::AnimationTrackPathError;

/// 连接场景实体与组件属性的规范化标识，供编辑器绑定、序列轨道和自动化共用。
/// 构造时使用已解析路径；接收外部字符串时应通过 `parse` 归一化。
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

    /// 接受可归一化的旧式输入，并用共享场景路径解析器决定最终等价关系。
    pub fn parse(raw: &str) -> Result<Self, AnimationTrackPathError> {
        let (entity_path, property_path) = raw.split_once(':').ok_or(AnimationTrackPathError)?;
        if canonical_entity_path(entity_path) && canonical_component_property_path(property_path) {
            return Ok(Self {
                raw: raw.to_owned(),
            });
        }
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

fn canonical_entity_path(path: &str) -> bool {
    path.split('/').all(canonical_path_segment)
}

fn canonical_component_property_path(path: &str) -> bool {
    let mut segments = path.split('.');
    let Some(component) = segments.next() else {
        return false;
    };
    if !canonical_path_segment(component) {
        return false;
    }
    let Some(first_property) = segments.next() else {
        return false;
    };
    canonical_path_segment(first_property) && segments.all(canonical_path_segment)
}

fn canonical_path_segment(segment: &str) -> bool {
    !segment.is_empty() && segment.trim() == segment
}

impl std::fmt::Display for AnimationTrackPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}

#[cfg(test)]
#[path = "track_path/tests/canonical_parse_tests.rs"]
mod canonical_parse_tests;
