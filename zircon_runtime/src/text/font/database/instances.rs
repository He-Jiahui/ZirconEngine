use std::sync::Arc;

use crate::text::{FontFaceId, InstancedFaceId, VariationCoords};

use super::{FontDatabase, FontDatabaseError};
use crate::text::font::instance::{
    font_instance_identity, EffectiveInstanceCacheKey, EffectiveInstanceCacheReport,
    EffectiveInstanceCacheValue, FontInstance,
};

impl FontDatabase {
    pub(crate) fn instance(
        &mut self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        if self.face(face).is_none() {
            return Err(FontDatabaseError::UnknownFace(face));
        }
        let variations = self
            .face_metadata(face)?
            .effective_variations(variations, None);
        self.instances
            .resolve_or_insert(face, &variations)
            .map_err(FontDatabaseError::from)
    }

    pub(crate) fn default_instance_id(
        &self,
        face: FontFaceId,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        self.default_instances
            .get(&face)
            .copied()
            .ok_or(FontDatabaseError::UnknownFace(face))
    }

    pub(crate) fn font_instance(&self, id: InstancedFaceId) -> Option<&FontInstance> {
        self.instances.get(id)
    }

    pub(crate) fn default_font_instance(
        &self,
        face: FontFaceId,
    ) -> Result<&FontInstance, FontDatabaseError> {
        let instance = self.default_instance_id(face)?;
        self.font_instance(instance)
            .ok_or(FontDatabaseError::UnknownFace(face))
    }

    pub(crate) fn effective_variations(
        &self,
        face: FontFaceId,
        font_weight: u16,
    ) -> Result<VariationCoords, FontDatabaseError> {
        self.effective_instance_variations(face, None, font_weight)
    }

    pub(crate) fn effective_instance_id(
        &self,
        face: FontFaceId,
        font_weight: u16,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        Ok(self.effective_instance_value(face, None, font_weight)?.id)
    }

    pub(crate) fn effective_instance_variations(
        &self,
        face: FontFaceId,
        instance: Option<InstancedFaceId>,
        font_weight: u16,
    ) -> Result<VariationCoords, FontDatabaseError> {
        Ok((*self.effective_instance_variations_shared(face, instance, font_weight)?).clone())
    }

    pub(crate) fn effective_instance_variations_shared(
        &self,
        face: FontFaceId,
        instance: Option<InstancedFaceId>,
        font_weight: u16,
    ) -> Result<Arc<VariationCoords>, FontDatabaseError> {
        Ok(self
            .effective_instance_value(face, instance, font_weight)?
            .variations)
    }

    fn effective_instance_value(
        &self,
        face: FontFaceId,
        instance: Option<InstancedFaceId>,
        font_weight: u16,
    ) -> Result<EffectiveInstanceCacheValue, FontDatabaseError> {
        let default_instance = self.default_instance_id(face)?;
        let instance = instance
            .filter(|instance| {
                self.font_instance(*instance)
                    .is_some_and(|stored| stored.face == face)
            })
            .unwrap_or(default_instance);
        let key = EffectiveInstanceCacheKey {
            face,
            instance,
            font_weight,
        };
        if let Some(value) = self.effective_instances.get(key) {
            return Ok(value);
        }
        let base = self
            .font_instance(instance)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        let variations = self
            .face_metadata(face)?
            .effective_variations(&base.variations, Some(font_weight));
        let id = font_instance_identity(face, &variations).map_err(FontDatabaseError::from)?;
        let value = EffectiveInstanceCacheValue {
            id,
            variations: Arc::new(variations),
        };
        self.effective_instances.insert(key, value.clone());
        Ok(value)
    }

    #[cfg(test)]
    pub(super) fn effective_instance_cache_len(&self) -> usize {
        self.effective_instances.report().entry_count
    }

    pub(crate) fn effective_instance_cache_report(&self) -> EffectiveInstanceCacheReport {
        self.effective_instances.report()
    }
}
