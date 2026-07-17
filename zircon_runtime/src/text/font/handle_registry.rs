use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::core::framework::text::TextFontFaceHandle;
use crate::text::{FontFaceId, InstancedFaceId};

use super::shared_font_database_generation;

#[derive(Default)]
struct FontHandleRegistry {
    generation: u64,
    faces: Vec<FontFaceId>,
    face_slots: HashMap<FontFaceId, u32>,
    instances: Vec<InstancedFaceId>,
    instance_slots: HashMap<InstancedFaceId, u32>,
}

impl FontHandleRegistry {
    fn reset_for_generation(&mut self, generation: u64) -> bool {
        if self.generation == generation {
            return true;
        }
        if generation < self.generation {
            return false;
        }
        self.generation = generation;
        self.faces.clear();
        self.face_slots.clear();
        self.instances.clear();
        self.instance_slots.clear();
        true
    }

    fn register_face(&mut self, face: FontFaceId, generation: u64) -> Option<TextFontFaceHandle> {
        self.reset_for_generation(generation).then_some(())?;
        let index = match self.face_slots.get(&face) {
            Some(index) => *index,
            None => {
                let index = u32::try_from(self.faces.len()).ok()?;
                self.faces.push(face);
                self.face_slots.insert(face, index);
                index
            }
        };
        Some(TextFontFaceHandle::new(index, generation))
    }

    fn register_instance(
        &mut self,
        instance: InstancedFaceId,
        generation: u64,
    ) -> Option<TextFontFaceHandle> {
        self.reset_for_generation(generation).then_some(())?;
        let index = match self.instance_slots.get(&instance) {
            Some(index) => *index,
            None => {
                let index = u32::try_from(self.instances.len()).ok()?;
                self.instances.push(instance);
                self.instance_slots.insert(instance, index);
                index
            }
        };
        Some(TextFontFaceHandle::new(index, generation))
    }

    fn resolve_face(&self, handle: TextFontFaceHandle) -> Option<FontFaceId> {
        (self.generation == handle.generation)
            .then(|| self.faces.get(handle.index as usize).copied())
            .flatten()
    }

    fn resolve_instance(&self, handle: TextFontFaceHandle) -> Option<InstancedFaceId> {
        (self.generation == handle.generation)
            .then(|| self.instances.get(handle.index as usize).copied())
            .flatten()
    }
}

fn registry() -> &'static Mutex<FontHandleRegistry> {
    static REGISTRY: OnceLock<Mutex<FontHandleRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(FontHandleRegistry::default()))
}

pub(crate) fn register_font_face_handle(
    face: FontFaceId,
    generation: u64,
) -> Option<TextFontFaceHandle> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .register_face(face, generation)
}

pub(crate) fn register_font_instance_handle(
    instance: InstancedFaceId,
    generation: u64,
) -> Option<TextFontFaceHandle> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .register_instance(instance, generation)
}

pub(crate) fn resolve_font_face_handle(handle: TextFontFaceHandle) -> Option<FontFaceId> {
    if handle.generation != shared_font_database_generation() {
        return None;
    }
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .resolve_face(handle)
}

pub(crate) fn resolve_font_instance_handle(handle: TextFontFaceHandle) -> Option<InstancedFaceId> {
    if handle.generation != shared_font_database_generation() {
        return None;
    }
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .resolve_instance(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::font::shared::force_publish_shared_font_database;
    use crate::text::font::shared_font_database_snapshot;

    #[test]
    fn generation_change_invalidates_old_slots_without_reinterpreting_backend_ids() {
        let mut registry = FontHandleRegistry::default();
        let backend_face = FontFaceId(u64::from(u32::MAX) + 41);
        let first = registry
            .register_face(backend_face, 9)
            .expect("first handle");

        assert_eq!(registry.resolve_face(first), Some(backend_face));

        let reloaded = registry
            .register_face(backend_face, 10)
            .expect("reloaded handle");
        assert_eq!(registry.resolve_face(first), None);
        assert_eq!(registry.resolve_face(reloaded), Some(backend_face));
        assert_eq!(reloaded.generation, 10);
    }

    #[test]
    fn shared_database_reload_rejects_pre_reload_handle() {
        let (generation, database) = shared_font_database_snapshot();
        let backend_face = FontFaceId(1);
        let before_reload = register_font_face_handle(backend_face, generation)
            .expect("pre-reload face should receive a slot");
        assert_eq!(resolve_font_face_handle(before_reload), Some(backend_face));

        let reloaded_generation = force_publish_shared_font_database(&database);

        assert!(reloaded_generation > generation);
        assert_eq!(resolve_font_face_handle(before_reload), None);
        let after_reload = register_font_face_handle(backend_face, reloaded_generation)
            .expect("reloaded face should receive a new-generation slot");
        assert_eq!(resolve_font_face_handle(after_reload), Some(backend_face));
        assert_ne!(before_reload, after_reload);
    }

    #[test]
    fn stale_projection_cannot_roll_registry_generation_back() {
        let mut registry = FontHandleRegistry::default();
        let current = registry
            .register_face(FontFaceId(7), 12)
            .expect("current generation handle");

        assert_eq!(registry.register_face(FontFaceId(9), 11), None);
        assert_eq!(registry.generation, 12);
        assert_eq!(registry.resolve_face(current), Some(FontFaceId(7)));
    }
}
