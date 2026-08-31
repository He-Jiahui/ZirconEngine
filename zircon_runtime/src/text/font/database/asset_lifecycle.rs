use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use glyphon::fontdb;

use crate::asset::{FontAsset, FontBlobArtifact};
use crate::text::{CompositeFontDescriptor, FontFaceDescriptor, FontFaceId, FontFamilyName};

use super::{
    FontAssetOwnerState, FontAssetUpdateReport, FontDatabase, FontDatabaseError,
    read_decoded_font_source,
};
use crate::text::font::asset_registration::{FontAssetSourceKey, font_asset_faces};
use crate::text::font::descriptors::descriptor_from_font_metadata;
use crate::text::font::face_metadata::FontFaceMetadata;
use crate::text::font::matching::font_family_identity;

impl FontDatabase {
    pub(crate) fn register_font_family_alias(
        &mut self,
        face: FontFaceId,
        alias: FontFamilyName,
    ) -> bool {
        if alias.is_empty() || self.face(face).is_none() {
            return false;
        }
        let alias_identity = font_family_identity(alias.as_str());
        if self
            .family_alias_index
            .get(&alias_identity)
            .is_some_and(|aliases| aliases.contains(&face))
        {
            return false;
        }
        let Some(primary_backend) = self.backend_faces.backend_face_id(face) else {
            return false;
        };
        let Some(mut backend_alias) = self.backend_database.face(primary_backend).cloned() else {
            return false;
        };
        backend_alias.id = fontdb::ID::dummy();
        backend_alias.families = vec![(
            (alias.as_str()).to_string(),
            fontdb::Language::English_UnitedStates,
        )];
        backend_alias.post_script_name = alias.as_str().to_string();
        let alias_backend = self.backend_database.push_face_info(backend_alias);
        self.backend_faces.insert_alias(alias_backend, face);
        let aliases = self.family_alias_index.entry(alias_identity).or_default();
        aliases.push(face);
        self.detach_face_dependent_caches();
        true
    }

    pub(crate) fn font_asset_primary_face(&self, owner: &str) -> Option<FontFaceId> {
        self.asset_owners.get(owner)?.faces.first().copied()
    }

    pub(in crate::text::font) fn font_asset_fallback_families(
        &self,
        owner: &str,
    ) -> Option<&[FontFamilyName]> {
        self.asset_owners
            .get(owner)
            .map(|state| state.fallback_families.as_slice())
    }

    pub(in crate::text::font) fn font_asset_base_fallback_families(&self) -> &[FontFamilyName] {
        &self.fallback_base_families
    }

    pub(in crate::text::font) fn has_font_asset_owner(&self, owner: &str) -> bool {
        self.asset_owners.contains_key(owner)
    }

    pub(crate) fn replace_font_asset(
        &mut self,
        owner: &str,
        asset: &FontAsset,
        source_path: impl AsRef<Path>,
    ) -> Result<FontAssetUpdateReport, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let bytes = read_decoded_font_source(source_path)?;
        let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
        self.replace_font_asset_bytes(owner, asset, source_path, bytes)
    }

    pub(crate) fn replace_font_asset_blob(
        &mut self,
        owner: &str,
        asset: &FontAsset,
        source_path: impl AsRef<Path>,
        blob: &FontBlobArtifact,
    ) -> Result<FontAssetUpdateReport, FontDatabaseError> {
        if !blob.is_valid_for_runtime() {
            return Err(FontDatabaseError::InvalidCookedArtifact);
        }
        self.replace_font_asset_bytes(owner, asset, source_path.as_ref(), blob.shared_bytes())
    }

    fn replace_font_asset_bytes(
        &mut self,
        owner: &str,
        asset: &FontAsset,
        source_path: &Path,
        bytes: Arc<[u8]>,
    ) -> Result<FontAssetUpdateReport, FontDatabaseError> {
        let registrations = font_asset_faces(asset, bytes.as_ref(), source_path)
            .into_iter()
            .map(|registration| (registration.descriptor, registration.metadata))
            .collect();
        let fallback_families = normalized_fallback_families(&asset.fallback_families);
        self.replace_asset_registrations(
            owner,
            source_path,
            bytes,
            registrations,
            fallback_families,
            asset.composite_font.clone(),
        )
    }

    pub(crate) fn replace_font_source(
        &mut self,
        owner: &str,
        source_path: impl AsRef<Path>,
        family: Option<&str>,
        face_index: u32,
    ) -> Result<FontAssetUpdateReport, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let bytes = read_decoded_font_source(source_path)?;
        let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
        let metadata = FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), face_index);
        let descriptor = descriptor_from_font_metadata(&metadata, family, source_path, face_index);
        self.replace_asset_registrations(
            owner,
            source_path,
            bytes,
            vec![(descriptor, metadata)],
            Vec::new(),
            None,
        )
    }

    pub(crate) fn remove_font_asset(&mut self, owner: &str) -> FontAssetUpdateReport {
        if !self.asset_owners.contains_key(owner) {
            return FontAssetUpdateReport {
                faces: Vec::new(),
                retired_faces: Vec::new(),
                database_changed: false,
                asset_mapping_changed: false,
            };
        }

        let retired_faces = self.remove_asset_owner(owner);
        FontAssetUpdateReport {
            faces: Vec::new(),
            database_changed: true,
            retired_faces,
            asset_mapping_changed: true,
        }
    }

    fn replace_asset_registrations(
        &mut self,
        owner: &str,
        source_path: &Path,
        bytes: Arc<[u8]>,
        registrations: Vec<(FontFaceDescriptor, FontFaceMetadata)>,
        fallback_families: Vec<FontFamilyName>,
        composite_font: Option<CompositeFontDescriptor>,
    ) -> Result<FontAssetUpdateReport, FontDatabaseError> {
        let mut next = {
            crate::profile_scope!(
                "runtime",
                "text.font_database",
                "owner_registration_staging_clone"
            );
            let next = self.clone();
            crate::profile_counter!(
                "runtime",
                "text.font_database.owner_registration_staging_clone_face_count",
                self.face_count()
            );
            next
        };
        let previous = next.asset_owners.get(owner).cloned().unwrap_or_default();
        let mut source_keys = Vec::new();
        let mut faces = Vec::new();

        for (descriptor, metadata) in registrations {
            let (source_key, face) = next.register_asset_registration(
                descriptor,
                metadata,
                Arc::clone(&bytes),
                source_path,
            )?;
            if !source_keys.contains(&source_key) {
                source_keys.push(source_key.clone());
            }
            if !faces.contains(&face) {
                faces.push(face);
            }
            next.asset_source_owners
                .entry(source_key)
                .or_default()
                .insert(owner.to_string());
        }

        let retained = source_keys.iter().cloned().collect::<HashSet<_>>();
        let mut retired_faces = Vec::new();
        for source_key in previous.sources {
            if !retained.contains(&source_key) {
                next.detach_asset_source_owner(owner, &source_key, &mut retired_faces);
            }
        }
        next.asset_owners.insert(
            owner.to_string(),
            FontAssetOwnerState {
                sources: source_keys,
                faces: Arc::from(faces.clone().into_boxed_slice()),
                fallback_families,
                composite_font,
            },
        );
        next.rebuild_asset_fallback_families();

        let asset_mapping_changed = self.asset_owners.get(owner) != next.asset_owners.get(owner);
        if asset_mapping_changed {
            next.detach_matching_and_fallback_caches();
        }
        let database_changed = !self.has_same_render_inputs(&next);
        *self = next;
        Ok(FontAssetUpdateReport {
            faces,
            retired_faces,
            database_changed,
            asset_mapping_changed,
        })
    }

    fn remove_asset_owner(&mut self, owner: &str) -> Vec<FontFaceId> {
        let Some(previous) = self.asset_owners.remove(owner) else {
            return Vec::new();
        };
        let mut retired_faces = Vec::new();
        for source_key in previous.sources {
            self.detach_asset_source_owner(owner, &source_key, &mut retired_faces);
        }
        self.rebuild_asset_fallback_families();
        self.detach_matching_and_fallback_caches();
        retired_faces
    }

    fn detach_asset_source_owner(
        &mut self,
        owner: &str,
        source_key: &FontAssetSourceKey,
        retired_faces: &mut Vec<FontFaceId>,
    ) {
        let should_retire = self.asset_source_owners.get_mut(source_key).is_some_and(
            |owners: &mut HashSet<String>| {
                owners.remove(owner);
                owners.is_empty()
            },
        );
        if !should_retire {
            return;
        }
        self.asset_source_owners.remove(source_key);
        if let Some(face) = self.asset_source_index.remove(source_key) {
            self.retire_face(face);
            if !retired_faces.contains(&face) {
                retired_faces.push(face);
            }
        }
    }

    fn retire_face(&mut self, face: FontFaceId) {
        let Some(index) = face.0.checked_sub(1).map(|index| index as usize) else {
            return;
        };
        let Some(stored) = self.faces.get_mut(index) else {
            return;
        };
        if !stored.active {
            return;
        }
        stored.active = false;
        let family = font_family_identity(stored.descriptor.family.as_str());
        stored.source = super::StoredFontSource::SharedBytes(Arc::from(Vec::<u8>::new()));
        stored.source_bytes = Arc::new(std::sync::OnceLock::new());
        stored.standalone_bytes = Arc::new(std::sync::OnceLock::new());
        stored.metadata = Arc::new(std::sync::OnceLock::new());

        let remove_family = if let Some(faces) = self.family_index.get_mut(&family) {
            faces.retain(|candidate| *candidate != face);
            faces.is_empty()
        } else {
            false
        };
        if remove_family {
            self.family_index.remove(&family);
        }
        self.family_alias_index.retain(|_, aliases| {
            aliases.retain(|candidate| *candidate != face);
            !aliases.is_empty()
        });
        self.source_face_index
            .retain(|_, candidate| *candidate != face);
        self.asset_source_index
            .retain(|_, candidate| *candidate != face);
        self.default_instances.remove(&face);
        self.instances.remove_face(face);
        for backend in self.backend_faces.remove_face(face) {
            self.backend_database.remove_face(backend);
        }
        self.active_face_count = self.active_face_count.saturating_sub(1);
        self.detach_face_dependent_caches();
    }

    fn rebuild_asset_fallback_families(&mut self) {
        let mut fallback_families = self.fallback_base_families.clone();
        let mut identities = fallback_families
            .iter()
            .map(|family| font_family_identity(family.as_str()))
            .collect::<HashSet<_>>();
        let mut owners = self.asset_owners.iter().collect::<Vec<_>>();
        owners.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        for (_, state) in owners {
            for family in &state.fallback_families {
                let identity = font_family_identity(family.as_str());
                if identities.insert(identity) {
                    fallback_families.push(family.clone());
                }
            }
        }
        if self.fallback_families != fallback_families {
            self.fallback_families = fallback_families;
        }
    }
}

fn normalized_fallback_families(families: &[String]) -> Vec<FontFamilyName> {
    let mut identities = HashSet::new();
    families
        .iter()
        .map(|family| FontFamilyName::from(family.as_str()))
        .filter(|family| !family.is_empty())
        .filter(|family| identities.insert(font_family_identity(family.as_str())))
        .collect()
}
