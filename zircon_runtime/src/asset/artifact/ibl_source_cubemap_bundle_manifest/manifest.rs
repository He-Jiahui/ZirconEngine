use crate::core::framework::render::{
    IblBakeArtifactRequest, IblBakeKey, IBL_BAKE_ALGORITHM_VERSION,
};

use super::error::IblSourceCubemapBundleManifestError;
use super::payload_stamp::IblSourceCubemapBundlePayloadStamp;
use super::wire::{read_bytes, read_u32, read_u64, write_bytes, write_u32, write_u64};
use crate::asset::artifact::ibl_source_cubemap_staging::IBL_SOURCE_CUBEMAP_STAGING_VERSION;

pub(crate) const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_DIRECTORY: &str = "render/ibl-source-bundle";
pub(crate) const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME: &str = "bundle.zriblmeta";
pub(crate) const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub(crate) const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE: usize = 252;

const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_MAGIC: [u8; 8] = *b"ZRIBLBND";
const IBL_SOURCE_CUBEMAP_BUNDLE_WIRE_PLATFORM: u32 = 1;
const IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_BODY_SIZE: usize =
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE - blake3::OUT_LEN;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct IblSourceImageIdentity {
    width: u32,
    height: u32,
    format_identity: u32,
}

impl IblSourceImageIdentity {
    pub(crate) const fn new(width: u32, height: u32, format_identity: u32) -> Self {
        Self {
            width,
            height,
            format_identity,
        }
    }

    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) const fn format_identity(self) -> u32 {
        self.format_identity
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IblSourceCubemapBundleManifest {
    source_image: IblSourceImageIdentity,
    bake_key: IblBakeKey,
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    required_contents: u32,
    source: IblSourceCubemapBundlePayloadStamp,
    derived: IblSourceCubemapBundlePayloadStamp,
}

impl IblSourceCubemapBundleManifest {
    pub(crate) fn new(
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
        source_bytes: &[u8],
        derived_bytes: &[u8],
    ) -> Self {
        Self {
            source_image,
            bake_key: request.bake_key(),
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            pmrem_face_size: request.pmrem_face_size(),
            pmrem_mip_count: request.pmrem_mip_count(),
            required_contents: request.required_contents().bits(),
            source: IblSourceCubemapBundlePayloadStamp::from_bytes(source_bytes),
            derived: IblSourceCubemapBundlePayloadStamp::from_bytes(derived_bytes),
        }
    }

    pub(crate) fn matches(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
    ) -> bool {
        self.source_image == source_image
            && self.bake_key == request.bake_key()
            && self.source_face_size == request.source_face_size()
            && self.source_mip_count == request.source_mip_count()
            && self.pmrem_face_size == request.pmrem_face_size()
            && self.pmrem_mip_count == request.pmrem_mip_count()
            && self.required_contents == request.required_contents().bits()
    }

    pub(crate) fn matches_request(&self, request: &IblBakeArtifactRequest) -> bool {
        self.bake_key == request.bake_key()
            && self.source_face_size == request.source_face_size()
            && self.source_mip_count == request.source_mip_count()
            && self.pmrem_face_size == request.pmrem_face_size()
            && self.pmrem_mip_count == request.pmrem_mip_count()
            && self.required_contents == request.required_contents().bits()
    }

    pub(crate) const fn source_image(&self) -> IblSourceImageIdentity {
        self.source_image
    }

    pub(crate) const fn source(&self) -> IblSourceCubemapBundlePayloadStamp {
        self.source
    }

    pub(crate) const fn derived(&self) -> IblSourceCubemapBundlePayloadStamp {
        self.derived
    }

    pub(crate) fn encode(&self) -> [u8; IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE] {
        let mut output = [0; IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE];
        let mut cursor = 0;
        write_bytes(
            &mut output,
            &mut cursor,
            &IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_MAGIC,
        );
        write_u32(
            &mut output,
            &mut cursor,
            IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION,
        );
        write_u64(&mut output, &mut cursor, IBL_SOURCE_CUBEMAP_STAGING_VERSION);
        write_u64(&mut output, &mut cursor, IBL_BAKE_ALGORITHM_VERSION);
        write_u32(
            &mut output,
            &mut cursor,
            IBL_SOURCE_CUBEMAP_BUNDLE_WIRE_PLATFORM,
        );
        write_u32(&mut output, &mut cursor, self.source_image.width());
        write_u32(&mut output, &mut cursor, self.source_image.height());
        write_u32(
            &mut output,
            &mut cursor,
            self.source_image.format_identity(),
        );
        write_bake_key(&mut output, &mut cursor, self.bake_key);
        write_u32(&mut output, &mut cursor, self.source_face_size);
        write_u32(&mut output, &mut cursor, self.source_mip_count);
        write_u32(&mut output, &mut cursor, self.pmrem_face_size);
        write_u32(&mut output, &mut cursor, self.pmrem_mip_count);
        write_u32(&mut output, &mut cursor, self.required_contents);
        write_stamp(&mut output, &mut cursor, self.source);
        write_stamp(&mut output, &mut cursor, self.derived);
        debug_assert_eq!(cursor, IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_BODY_SIZE);
        let checksum = *blake3::hash(&output[..cursor]).as_bytes();
        write_bytes(&mut output, &mut cursor, &checksum);
        debug_assert_eq!(cursor, output.len());
        output
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, IblSourceCubemapBundleManifestError> {
        if bytes.len() != IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE {
            return Err(IblSourceCubemapBundleManifestError::InvalidSize {
                actual: bytes.len(),
                expected: IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE,
            });
        }
        let expected_checksum =
            blake3::hash(&bytes[..IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_BODY_SIZE]);
        if expected_checksum.as_bytes() != &bytes[IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_BODY_SIZE..] {
            return Err(IblSourceCubemapBundleManifestError::ChecksumMismatch);
        }

        let mut cursor = 0;
        if read_bytes(bytes, &mut cursor) != IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_MAGIC {
            return Err(IblSourceCubemapBundleManifestError::InvalidMagic);
        }
        let schema = read_u32(bytes, &mut cursor);
        if schema != IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION {
            return Err(IblSourceCubemapBundleManifestError::StaleSchema {
                actual: schema,
                expected: IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION,
            });
        }
        let staging_version = read_u64(bytes, &mut cursor);
        if staging_version != IBL_SOURCE_CUBEMAP_STAGING_VERSION {
            return Err(IblSourceCubemapBundleManifestError::StaleStagingVersion {
                actual: staging_version,
                expected: IBL_SOURCE_CUBEMAP_STAGING_VERSION,
            });
        }
        let bake_version = read_u64(bytes, &mut cursor);
        if bake_version != IBL_BAKE_ALGORITHM_VERSION {
            return Err(IblSourceCubemapBundleManifestError::StaleBakeVersion {
                actual: bake_version,
                expected: IBL_BAKE_ALGORITHM_VERSION,
            });
        }
        let wire_platform = read_u32(bytes, &mut cursor);
        if wire_platform != IBL_SOURCE_CUBEMAP_BUNDLE_WIRE_PLATFORM {
            return Err(
                IblSourceCubemapBundleManifestError::UnsupportedWirePlatform {
                    actual: wire_platform,
                    expected: IBL_SOURCE_CUBEMAP_BUNDLE_WIRE_PLATFORM,
                },
            );
        }
        let source_image = IblSourceImageIdentity::new(
            read_u32(bytes, &mut cursor),
            read_u32(bytes, &mut cursor),
            read_u32(bytes, &mut cursor),
        );
        let bake_key = read_bake_key(bytes, &mut cursor);
        let source_face_size = read_u32(bytes, &mut cursor);
        let source_mip_count = read_u32(bytes, &mut cursor);
        let pmrem_face_size = read_u32(bytes, &mut cursor);
        let pmrem_mip_count = read_u32(bytes, &mut cursor);
        let required_contents = read_u32(bytes, &mut cursor);
        let source = read_stamp(bytes, &mut cursor);
        let derived = read_stamp(bytes, &mut cursor);
        debug_assert_eq!(cursor, IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_BODY_SIZE);
        Ok(Self {
            source_image,
            bake_key,
            source_face_size,
            source_mip_count,
            pmrem_face_size,
            pmrem_mip_count,
            required_contents,
            source,
            derived,
        })
    }
}

fn write_bake_key(output: &mut [u8], cursor: &mut usize, key: IblBakeKey) {
    write_u32(output, cursor, key.source_kind);
    write_u64(output, cursor, key.source_revision);
    for values in [
        key.horizon_color,
        key.zenith_color,
        key.ground_color,
        key.source_hash,
    ] {
        for value in values {
            write_u32(output, cursor, value);
        }
    }
}

fn read_bake_key(input: &[u8], cursor: &mut usize) -> IblBakeKey {
    IblBakeKey {
        source_kind: read_u32(input, cursor),
        source_revision: read_u64(input, cursor),
        horizon_color: read_u32_array(input, cursor),
        zenith_color: read_u32_array(input, cursor),
        ground_color: read_u32_array(input, cursor),
        source_hash: read_u32_array(input, cursor),
    }
}

fn read_u32_array(input: &[u8], cursor: &mut usize) -> [u32; 4] {
    [
        read_u32(input, cursor),
        read_u32(input, cursor),
        read_u32(input, cursor),
        read_u32(input, cursor),
    ]
}

fn write_stamp(output: &mut [u8], cursor: &mut usize, stamp: IblSourceCubemapBundlePayloadStamp) {
    write_u64(output, cursor, stamp.encoded_len());
    write_bytes(output, cursor, &stamp.digest());
}

fn read_stamp(input: &[u8], cursor: &mut usize) -> IblSourceCubemapBundlePayloadStamp {
    IblSourceCubemapBundlePayloadStamp::from_parts(
        read_u64(input, cursor),
        read_bytes(input, cursor),
    )
}
