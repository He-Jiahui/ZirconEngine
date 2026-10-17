use super::*;
use crate::asset::artifact::RenderArtifactBlockCodec;
use crate::asset::runtime::fetch::fetch_bytes;
use crate::core::runtime::TaskCancellationToken;
impl RenderArtifactStore {
    pub fn from_url(root: impl Into<Arc<str>>) -> Self {
        Self { root: root.into() }
    }
    pub async fn read_block_async(
        &self,
        descriptor: &RenderArtifactBlockDescriptor,
        limits: RenderArtifactStoreLimits,
        cancellation: &TaskCancellationToken,
    ) -> Result<Arc<[u8]>, RenderArtifactStoreError> {
        validate_limit(
            "encoded block",
            descriptor.encoded_bytes(),
            limits.max_encoded_block_bytes(),
        )?;
        if descriptor.codec() != RenderArtifactBlockCodec::Raw {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Web export requires Raw blocks; transfer compression belongs to HTTP",
            )
            .into());
        }
        let hash = content_id_hex(descriptor.content_id());
        let url = format!(
            "{}/render-artifacts/blocks/{}/{}.zr-render-block",
            self.root.trim_end_matches('/'),
            &hash[..2],
            hash
        );
        let bytes = fetch_bytes(
            &url,
            Some(descriptor.encoded_bytes()),
            limits.max_encoded_block_bytes(),
            cancellation,
        )
        .await
        .map_err(fetch_error)?;
        if content_id_for(&bytes) != descriptor.content_id() {
            return Err(RenderArtifactStoreError::BlockContentHashMismatch {
                content_id: descriptor.content_id(),
            });
        }
        Ok(bytes)
    }
    pub async fn read_manifest_async(
        &self,
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: &str,
        limits: RenderArtifactStoreLimits,
        cancellation: &TaskCancellationToken,
    ) -> Result<RenderArtifactManifest, RenderArtifactStoreError> {
        validate_limit("manifest", 1, limits.max_manifest_bytes())?;
        let target_hash = blake3::hash(target_platform.as_bytes()).to_hex();
        let url = format!(
            "{}/render-artifacts/manifests/{:02}/{}/{}/{}.zr-render-manifest",
            self.root.trim_end_matches('/'),
            super::super::validation::resource_kind_tag(resource.kind()),
            resource.id(),
            asset_revision,
            target_hash
        );
        let bytes = fetch_bytes(&url, None, limits.max_manifest_bytes(), cancellation)
            .await
            .map_err(fetch_error)?;
        let bytes = bytes
            .strip_prefix(RENDER_ARTIFACT_MANIFEST_MAGIC)
            .ok_or(RenderArtifactStoreError::ManifestMagicMismatch)?;
        let manifest: RenderArtifactManifest = PositionalEncoding::FixedLittleEndian
            .decode_bounded(bytes, limits.max_manifest_bytes())
            .map_err(RenderArtifactStoreError::ManifestDeserialize)?;
        manifest.validate()?;
        if manifest.resource() != resource
            || manifest.asset_revision() != asset_revision
            || manifest.target_platform() != target_platform
        {
            return Err(RenderArtifactStoreError::ManifestIdentityMismatch);
        }
        Ok(manifest)
    }
}
fn fetch_error(message: String) -> RenderArtifactStoreError {
    io::Error::new(io::ErrorKind::Other, message).into()
}
