use zircon_runtime::asset::{TextureAsset, TextureUploadReadiness, TextureUploadSupport};
use zircon_runtime::core::framework::render::{RenderImageDescriptor, TextureMetadataDiagnostic};

/// An inspection view, including malformed authoring values that must remain visible.
/// Reading this value does not establish source validity, catalog identity or GPU readiness.
#[derive(Debug)]
pub struct TextureSource<'a> {
    pub asset: &'a TextureAsset,
    /// Derived by the existing Runtime conversion. Never substitute this for authored fields.
    pub effective_descriptor: RenderImageDescriptor,
    /// None means the original asset has no authored descriptor. Some(empty) means the
    /// existing metadata validator emitted no diagnostics; it is not full asset validation.
    pub authored_metadata_diagnostics: Option<Vec<TextureMetadataDiagnostic>>,
    /// None preserves an absent descriptor; true exposes normalization without writing it back.
    pub descriptor_normalization_changed: Option<bool>,
}

impl<'a> TextureSource<'a> {
    /// Borrows the active CPU representation, which may still be an encoded container.
    /// These are neither source-file bytes nor a pixel/thumbnail/GPU-upload result.
    pub fn active_payload_bytes(&self) -> &'a [u8] {
        self.asset.as_ref()
    }

    /// Counts initialized retained payload bytes, including any inactive rgba vector beside
    /// a container. Allocation capacity, source-file size and GPU memory are separate facts.
    pub fn retained_payload_byte_len(&self) -> Option<usize> {
        match &self.asset.payload {
            zircon_runtime::asset::TexturePayload::Rgba8 => Some(self.asset.rgba.len()),
            zircon_runtime::asset::TexturePayload::Container { bytes, .. } => {
                self.asset.rgba.len().checked_add(bytes.len())
            }
        }
    }

    /// Runs the original Runtime planning contract with the caller's explicit support.
    /// It deliberately retains Runtime normalization/validation and its exact diagnostics;
    /// Ready is a CPU plan, not authoring validity, an actual device, upload or rendered frame.
    /// The host must separately inspect raw diagnostics and prove its current device/provider.
    pub fn upload_preflight(&self, support: TextureUploadSupport) -> TextureUploadReadiness {
        self.asset.upload_readiness(support)
    }
}

pub struct TextureSourceAdapter;

impl TextureSourceAdapter {
    /// Borrows one actual loaded TextureAsset. No URI lookup, import, decoder, repair,
    /// source mutation, asset registration, upload, cache, callback or second store is made.
    /// An unavailable provider must stay unavailable instead of constructing an empty asset.
    pub fn read(asset: &TextureAsset) -> TextureSource<'_> {
        let authored_metadata_diagnostics = asset
            .descriptor
            .as_ref()
            .map(|descriptor| descriptor.validate_metadata(&asset.uri.to_string()));
        let descriptor_normalization_changed = asset
            .descriptor
            .as_ref()
            .map(|descriptor| descriptor.clone().normalized() != *descriptor);
        TextureSource {
            asset,
            effective_descriptor: asset.render_image_descriptor(),
            authored_metadata_diagnostics,
            descriptor_normalization_changed,
        }
    }
}
