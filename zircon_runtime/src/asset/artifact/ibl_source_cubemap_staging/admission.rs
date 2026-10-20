//! Digest admission is separate from the metadata-only hot probe.
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use super::super::ibl_source_cubemap_bundle_manifest::IblSourceCubemapBundlePayloadStamp;
use super::{
    decode_source_cubemap_for_request, ibl_bake_artifact_request_identity_hash,
    IblBakeArtifactBlob, IblBakeArtifactRequest, IblSourceCubemapBundleManifest,
    IblSourceCubemapStagingError, IblSourceCubemapStagingStore, IblSourceImageIdentity,
    PreparedFileWrite, ZcubeSourceCubemap, IBL_SOURCE_CUBEMAP_BUNDLE_READ_ATTEMPTS,
};

#[path = "admission/platform.rs"]
mod platform;

const MAX_CREDENTIALS: usize = 64;
static CREDENTIALS: OnceLock<Mutex<VecDeque<Credential>>> = OnceLock::new();

struct VerifiedFile {
    path: PathBuf,
    object: File,
    stamp: platform::Stamp,
}

impl VerifiedFile {
    fn current(&self) -> io::Result<bool> {
        let current = match platform::open(&self.path, false) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };
        Ok(
            platform::stamp(&current)? == self.stamp
                && platform::stamp(&self.object)? == self.stamp,
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Key {
    root: PathBuf,
    request: String,
    manifest: Vec<u8>,
}

#[derive(Clone)]
struct Credential {
    key: Key,
    source: Arc<VerifiedFile>,
    derived: Arc<VerifiedFile>,
}

pub(crate) enum BundleAdmission {
    Current,
    VerifiedSourceOnly(ZcubeSourceCubemap, VerifiedSourceProof),
    Missing,
}

impl BundleAdmission {
    pub(crate) fn is_current(&self) -> bool {
        matches!(self, Self::Current)
    }
}

/// Owned original digest and physical identity; preparation must consume this exact observation.
pub(crate) struct VerifiedSourceProof {
    key: Key,
    source: Arc<VerifiedFile>,
    expected: IblSourceCubemapBundlePayloadStamp,
}

fn cache() -> io::Result<std::sync::MutexGuard<'static, VecDeque<Credential>>> {
    CREDENTIALS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .map_err(|_| io::Error::other("IBL admission credentials are poisoned"))
}

fn inspect_error(path: &Path, source: io::Error) -> IblSourceCubemapStagingError {
    IblSourceCubemapStagingError::InspectBundlePayload {
        path: path.to_path_buf(),
        source,
    }
}

fn key(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    manifest: &[u8],
) -> Result<Key, IblSourceCubemapStagingError> {
    Ok(Key {
        root: std::fs::canonicalize(&store.cache_root)
            .map_err(|error| inspect_error(&store.cache_root, error))?,
        request: ibl_bake_artifact_request_identity_hash(request),
        manifest: manifest.to_vec(),
    })
}

pub(super) fn invalidate_root(
    store: &IblSourceCubemapStagingStore,
) -> Result<(), IblSourceCubemapStagingError> {
    let root = match std::fs::canonicalize(&store.cache_root) {
        Ok(root) => root,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(inspect_error(&store.cache_root, error)),
    };
    cache()
        .map_err(|error| inspect_error(&root, error))?
        .retain(|entry| entry.key.root != root);
    Ok(())
}

/// Called under the publication barrier. This function opens metadata objects, never payloads.
pub(super) fn cached_bundle_matches(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    manifest: &[u8],
) -> Result<bool, IblSourceCubemapStagingError> {
    let key = key(store, request, manifest)?;
    let entry = {
        let mut entries = cache().map_err(|error| inspect_error(&key.root, error))?;
        entries
            .iter()
            .position(|entry| entry.key == key)
            .map(|index| {
                let entry = entries
                    .remove(index)
                    .expect("located credential must exist");
                entries.push_back(entry.clone());
                entry
            })
    };
    let Some(entry) = entry else {
        return Ok(false);
    };
    let current = entry
        .source
        .current()
        .and_then(|source| entry.derived.current().map(|derived| source && derived))
        .map_err(|error| inspect_error(&key.root, error))?;
    if !current {
        cache()
            .map_err(|error| inspect_error(&key.root, error))?
            .retain(|entry| entry.key != key);
    }
    Ok(current)
}

fn remember(credential: Credential) -> Result<(), IblSourceCubemapStagingError> {
    let mut entries = cache().map_err(|error| inspect_error(&credential.key.root, error))?;
    entries.retain(|entry| {
        entry.key.root != credential.key.root || entry.key.request != credential.key.request
    });
    while entries.len() >= MAX_CREDENTIALS {
        entries.pop_front();
    }
    entries.push_back(credential);
    Ok(())
}

fn read_payload(
    path: &Path,
    expected: IblSourceCubemapBundlePayloadStamp,
) -> Result<Option<(Vec<u8>, Arc<VerifiedFile>)>, IblSourceCubemapStagingError> {
    let mut file = match platform::open(path, true) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(inspect_error(path, error)),
    };
    let before = platform::stamp(&file).map_err(|error| inspect_error(path, error))?;
    if before.len != expected.encoded_len() {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(expected.encoded_len().saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| inspect_error(path, error))?;
    #[cfg(test)]
    tests::observe_payload_read(path);
    let after = platform::stamp(&file).map_err(|error| inspect_error(path, error))?;
    let object = platform::open(path, false).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            IblSourceCubemapStagingError::BundleReadChangedDuringPublication
        } else {
            inspect_error(path, error)
        }
    })?;
    if before != after
        || platform::stamp(&object).map_err(|error| inspect_error(path, error))? != before
    {
        return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
    }
    if !expected.matches_bytes(&bytes) {
        return Ok(None);
    }
    Ok(Some((
        bytes,
        Arc::new(VerifiedFile {
            path: path.to_path_buf(),
            object,
            stamp: before,
        }),
    )))
}

impl IblSourceCubemapStagingStore {
    pub(crate) fn admit_bundle_for_staging(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
    ) -> Result<BundleAdmission, IblSourceCubemapStagingError> {
        self.admit_bundle(request, Some(source_image))
    }

    pub(crate) fn seed_committed_bundle_admission(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<(), IblSourceCubemapStagingError> {
        if self.admit_bundle(request, None)?.is_current() {
            Ok(())
        } else {
            Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
        }
    }

    fn admit_bundle(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: Option<IblSourceImageIdentity>,
    ) -> Result<BundleAdmission, IblSourceCubemapStagingError> {
        for _ in 0..IBL_SOURCE_CUBEMAP_BUNDLE_READ_ATTEMPTS {
            // Recovery takes the same owner lock: never recurse into it while holding the barrier.
            self.recover_pending_bundle_writes()?;
            if self.read_bundle_manifest_bytes(request)?.is_none() {
                return Ok(BundleAdmission::Missing);
            }
            let owner = self
                .asset_derived_store()
                .acquire_bundle_owner_lock()
                .map_err(IblSourceCubemapStagingError::BundleObservation)?;
            let result = self.admit_bundle_locked(request, source_image);
            drop(owner);
            match result {
                Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication) => continue,
                result => return result,
            }
        }
        Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
    }

    fn admit_bundle_locked(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: Option<IblSourceImageIdentity>,
    ) -> Result<BundleAdmission, IblSourceCubemapStagingError> {
        if self.bundle_publication_is_pending()? {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        }
        let Some((_, manifest_bytes)) = self.read_bundle_manifest_bytes(request)? else {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        };
        let check_snapshot = || -> Result<(), IblSourceCubemapStagingError> {
            if !matches!(self.read_bundle_manifest_bytes(request)?,
                Some((_, ref bytes)) if bytes == &manifest_bytes)
                || self.bundle_publication_is_pending()?
            {
                return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
            }
            Ok(())
        };
        let Ok(manifest) = IblSourceCubemapBundleManifest::decode(&manifest_bytes) else {
            check_snapshot()?;
            return Ok(BundleAdmission::Missing);
        };
        if !manifest.matches_request(request)
            || source_image.is_some_and(|image| !manifest.matches(request, image))
        {
            check_snapshot()?;
            return Ok(BundleAdmission::Missing);
        }
        let key = key(self, request, &manifest_bytes)?;
        cache()
            .map_err(|error| inspect_error(&key.root, error))?
            .retain(|entry| entry.key.root != key.root || entry.key.request != key.request);
        let source_path = self.source_cubemap_path(request);
        let Some((source_bytes, source)) = read_payload(&source_path, manifest.source())? else {
            check_snapshot()?;
            return Ok(BundleAdmission::Missing);
        };
        let Ok(decoded_source) =
            decode_source_cubemap_for_request(request, &source_path, &source_bytes)
        else {
            check_snapshot()?;
            return Ok(BundleAdmission::Missing);
        };
        let derived_path = self.asset_derived_store().asset_derived_path(request);
        let derived = read_payload(&derived_path, manifest.derived())?;
        let derived = derived.and_then(|(bytes, file)| {
            IblBakeArtifactBlob::decode_current_for_request(request, &bytes)
                .ok()
                .map(|_| file)
        });
        let current_manifest = matches!(self.read_bundle_manifest_bytes(request)?,
            Some((_, bytes)) if bytes == manifest_bytes);
        let derived_current = match &derived {
            Some(file) => file
                .current()
                .map_err(|error| inspect_error(&derived_path, error))?,
            None => true,
        };
        if !current_manifest
            || self.bundle_publication_is_pending()?
            || !source
                .current()
                .map_err(|error| inspect_error(&source_path, error))?
            || !derived_current
        {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        }
        match derived {
            Some(derived) => {
                remember(Credential {
                    key,
                    source,
                    derived,
                })?;
                Ok(BundleAdmission::Current)
            }
            None => Ok(BundleAdmission::VerifiedSourceOnly(
                decoded_source,
                VerifiedSourceProof {
                    key,
                    source,
                    expected: manifest.source(),
                },
            )),
        }
    }
}

impl VerifiedSourceProof {
    pub(super) fn prepare_manifest(
        self,
        store: &IblSourceCubemapStagingStore,
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
        derived_bytes: &[u8],
    ) -> Result<PreparedFileWrite, IblSourceCubemapStagingError> {
        store.recover_pending_bundle_writes()?;
        let _owner = store
            .asset_derived_store()
            .acquire_bundle_owner_lock()
            .map_err(IblSourceCubemapStagingError::BundleObservation)?;
        let observed = store.read_bundle_manifest_bytes(request)?;
        let expected_key = key(store, request, &self.key.manifest)?;
        let original_manifest = IblSourceCubemapBundleManifest::decode(&self.key.manifest)
            .map_err(IblSourceCubemapStagingError::RejectedBundleManifest)?;
        if expected_key != self.key
            || !original_manifest.matches(request, source_image)
            || !matches!(observed, Some((_, ref bytes)) if bytes == &self.key.manifest)
            || store.bundle_publication_is_pending()?
            || !self
                .source
                .current()
                .map_err(|error| inspect_error(&self.source.path, error))?
        {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        }
        let Some((bytes, observed_source)) = read_payload(&self.source.path, self.expected)? else {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        };
        if observed_source.stamp != self.source.stamp
            || !self
                .source
                .current()
                .map_err(|error| inspect_error(&self.source.path, error))?
            || !matches!(store.read_bundle_manifest_bytes(request)?,
                Some((_, ref bytes)) if bytes == &self.key.manifest)
            || store.bundle_publication_is_pending()?
        {
            return Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication);
        }
        Ok(PreparedFileWrite::new(
            store.bundle_manifest_path(request),
            IblSourceCubemapBundleManifest::new(request, source_image, &bytes, derived_bytes)
                .encode()
                .to_vec(),
        ))
    }
}

#[cfg(test)]
#[path = "admission/tests.rs"]
mod tests;
