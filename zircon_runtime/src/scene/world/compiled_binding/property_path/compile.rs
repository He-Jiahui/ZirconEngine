use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::scene::{EntityId, SceneError, SceneResult, World};

use super::super::compiled_scene_dynamic_field::CompiledDynamicProperty;
use super::model::{
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, CompiledScenePropertyWriterKind,
};

impl World {
    /// Compiles a scene-path target at an import or edit boundary.
    ///
    /// A retained target remains valid until the resolved entity's hierarchy or
    /// a name in its hierarchy changes. The typed field dispatch is added by the
    /// property-access cutover; this boundary intentionally does not fall back to
    /// per-frame path resolution.
    pub fn compile_scene_property_target(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> Option<CompiledScenePropertyTarget> {
        let entity = self.get_entity_by_path(entity_path)?;
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        Some(
            self.compile_scene_property_target_for_entity_with_canonical_field(
                entity,
                entity_path.as_str(),
                &component_field_key,
            ),
        )
    }

    fn compile_scene_property_target_for_entity_with_canonical_field(
        &mut self,
        entity: EntityId,
        entity_identity: &str,
        component_field_key: &str,
    ) -> CompiledScenePropertyTarget {
        let root = self.scene_binding_root(entity);
        let generation = self.scene_binding_generation(root);
        let path_id = self.scene_binding_generations.intern_path(entity_identity);
        let component_field_id = self
            .scene_binding_generations
            .intern_component_field(component_field_key);

        self.record_scene_property_target_compilation();
        CompiledScenePropertyTarget::new(entity, root, path_id, component_field_id, generation)
    }

    /// Compiles a typed scene-property writer at an import or edit boundary.
    ///
    /// Missing entities remain an optional lookup. Unsupported fields fail at
    /// the compile boundary and never fall back to the generic string visitor
    /// during steady-state application.
    pub fn compile_scene_property_writer(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        let property =
            self.compile_scene_property_writer_kind(property_path, &component_field_key)?;
        let Some(entity) = self.get_entity_by_path(entity_path) else {
            return Ok(None);
        };
        let target = self.compile_scene_property_target_for_entity_with_canonical_field(
            entity,
            entity_path.as_str(),
            &component_field_key,
        );
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    /// Compiles a writer after an import boundary has already resolved its
    /// target entity and canonicalized its path. The path is interned once for
    /// stable runtime identity and never parsed during writer application.
    pub(crate) fn compile_scene_property_writer_for_entity(
        &mut self,
        entity: EntityId,
        canonical_entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        let property =
            self.compile_scene_property_writer_kind(property_path, &component_field_key)?;
        if !self.contains_entity(entity) {
            return Ok(None);
        }
        let target = self.compile_scene_property_target_for_entity_with_canonical_field(
            entity,
            canonical_entity_path.as_str(),
            &component_field_key,
        );
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    fn compile_scene_property_writer_kind(
        &self,
        property_path: &ComponentPropertyPath,
        component_field_key: &str,
    ) -> SceneResult<CompiledScenePropertyWriterKind> {
        self.record_scene_property_field_dispatch_compilation();
        CompiledScenePropertyWriterKind::from_canonical_key(component_field_key)
            .or_else(|| {
                CompiledDynamicProperty::compile(self, property_path)
                    .map(CompiledScenePropertyWriterKind::Dynamic)
            })
            .ok_or_else(|| SceneError::UnknownProperty {
                property_path: property_path.to_string(),
            })
    }

    fn scene_binding_root(&self, entity: EntityId) -> EntityId {
        let mut root = entity;
        let mut remaining = self.entities.len().saturating_add(1);
        while remaining > 0 {
            let Some(parent) = self.parent_of(root) else {
                break;
            };
            root = parent;
            remaining -= 1;
        }
        root
    }
}
