#[path = "error.rs"]
mod error;
#[path = "manifest.rs"]
mod manifest;
#[path = "../../ibl_source_cubemap_bundle_manifest/payload_stamp.rs"]
mod payload_stamp;
#[path = "wire.rs"]
mod wire;

pub(crate) use error::IblSourceCubemapBundleManifestError;
pub(crate) use manifest::{
    IblSourceCubemapBundleManifest, IblSourceImageIdentity,
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_DIRECTORY, IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME,
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION, IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE,
};
pub(crate) use payload_stamp::IblSourceCubemapBundlePayloadStamp;

#[cfg(test)]
#[path = "../tests/cases.rs"]
mod tests;
