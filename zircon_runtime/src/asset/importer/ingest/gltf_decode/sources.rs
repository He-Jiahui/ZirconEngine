use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use crate::asset::project::{LexicalProjectPathIdentity, ProjectPaths};
use crate::asset::{AssetImportContext, AssetImportError};

use super::super::auxiliary_source::AuxiliarySourceResolver;
use super::{gltf_parse_error, MAX_GLTF_AUXILIARY_BYTES};

pub(super) struct ExternalSources<'a> {
    context: &'a AssetImportContext,
    resolver: Option<AuxiliarySourceResolver>,
    snapshots: BTreeMap<LexicalProjectPathIdentity, Cow<'a, [u8]>>,
    remaining: u64,
}

impl<'a> ExternalSources<'a> {
    pub(super) fn new(context: &'a AssetImportContext) -> Self {
        Self {
            context,
            resolver: None,
            snapshots: BTreeMap::new(),
            remaining: MAX_GLTF_AUXILIARY_BYTES,
        }
    }

    pub(super) fn read(
        &mut self,
        uri: &str,
        limit: u64,
    ) -> Result<(&Path, &[u8]), AssetImportError> {
        if self.resolver.is_none() {
            let base = self
                .context
                .source_path
                .parent()
                .unwrap_or_else(|| Path::new(""));
            self.resolver = Some(AuxiliarySourceResolver::new(self.context, base)?);
        }
        let resolver = self.resolver.as_ref().expect("resolver initialized above");
        let path = resolver.resolve_gltf_uri_lexical(uri)?;
        let key = ProjectPaths::lexical_identity(&path)
            .map_err(|error| gltf_parse_error(format!("gltf source identity: {error}")))?;
        if !self.snapshots.contains_key(&key) {
            if self.snapshots.len() >= AuxiliarySourceResolver::MAX_SNAPSHOT_FILES {
                return Err(gltf_parse_error(
                    "gltf source snapshot exceeds its file cumulative limit",
                ));
            }
            let bytes = if let Some(bytes) = self.context.source_file_snapshot(&path) {
                Cow::Borrowed(bytes)
            } else if self.context.has_source_file_snapshots() {
                return Err(gltf_parse_error(format!(
                    "gltf auxiliary snapshot does not contain `{uri}` at {}",
                    path.display()
                )));
            } else {
                let (_, bytes, _) =
                    resolver.read_gltf_uri_snapshot(uri, self.remaining.min(limit))?;
                Cow::Owned(bytes)
            };
            if bytes.len() as u64 > limit {
                return Err(gltf_parse_error(format!(
                    "gltf source exceeds the {limit}-byte read budget"
                )));
            }
            self.remaining = self.remaining.checked_sub(bytes.len() as u64).ok_or_else(|| {
                gltf_parse_error(format!(
                    "gltf auxiliary sources exceed the {MAX_GLTF_AUXILIARY_BYTES}-byte cumulative limit"
                ))
            })?;
            self.snapshots.insert(key.clone(), bytes);
        }
        let (path, bytes) = self
            .snapshots
            .get_key_value(&key)
            .expect("snapshot inserted above");
        if bytes.len() as u64 > limit {
            return Err(gltf_parse_error(format!(
                "gltf source exceeds the {limit}-byte read budget"
            )));
        }
        Ok((path.path(), bytes))
    }
}
