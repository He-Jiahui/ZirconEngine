use super::{
    IblBakeArtifactDescriptor, IblBakeArtifactHeader, IblBakeArtifactHeaderError,
    IblBakeArtifactPayload, IblBakeArtifactPayloadError, IblBakeArtifactRequest,
    IBL_BAKE_ARTIFACT_HEADER_SIZE, IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactBlob {
    header: IblBakeArtifactHeader,
    payload: IblBakeArtifactPayload,
}

impl IblBakeArtifactBlob {
    pub fn from_payload(payload: IblBakeArtifactPayload) -> Self {
        Self {
            header: IblBakeArtifactHeader::from_descriptor(payload.descriptor()),
            payload,
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, IblBakeArtifactBlobError> {
        if bytes.len() < IBL_BAKE_ARTIFACT_HEADER_SIZE {
            return Err(IblBakeArtifactBlobError::TruncatedHeader {
                expected: IBL_BAKE_ARTIFACT_HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let header = IblBakeArtifactHeader::decode(&bytes[..IBL_BAKE_ARTIFACT_HEADER_SIZE])
            .map_err(IblBakeArtifactBlobError::Header)?;
        let descriptor = header.descriptor();
        let payload_offset = IBL_BAKE_ARTIFACT_HEADER_SIZE
            .checked_add(IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE)
            .expect("IBL artifact header and checksum sizes must fit usize");
        if bytes.len() < payload_offset {
            return Err(IblBakeArtifactBlobError::TruncatedPayloadChecksum {
                expected: IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE,
                actual: bytes.len() - IBL_BAKE_ARTIFACT_HEADER_SIZE,
            });
        }
        let expected_payload_checksum = &bytes[IBL_BAKE_ARTIFACT_HEADER_SIZE..payload_offset];
        let payload_bytes = &bytes[payload_offset..];
        let expected_payload_len = descriptor.expected_payload_size_bytes();
        if payload_bytes.len() != expected_payload_len {
            return Err(IblBakeArtifactBlobError::Payload(
                IblBakeArtifactPayloadError::InvalidPayloadLength {
                    expected: expected_payload_len,
                    actual: payload_bytes.len(),
                },
            ));
        }

        // Verify bytes before payload construction so corrupt blobs avoid a full copy.
        let actual_payload_checksum = payload_checksum(payload_bytes);
        if expected_payload_checksum != actual_payload_checksum.as_slice() {
            return Err(IblBakeArtifactBlobError::PayloadChecksumMismatch {
                expected: payload_checksum_array(expected_payload_checksum),
                actual: actual_payload_checksum,
            });
        }
        let payload = IblBakeArtifactPayload::decode(descriptor, payload_bytes)
            .map_err(IblBakeArtifactBlobError::Payload)?;

        Ok(Self { header, payload })
    }

    pub fn decode_current_for_request(
        request: &IblBakeArtifactRequest,
        bytes: &[u8],
    ) -> Result<Self, IblBakeArtifactBlobError> {
        let blob = Self::decode(bytes)?;
        let descriptor = blob.descriptor();
        if !descriptor.is_current_for(request) {
            return Err(IblBakeArtifactBlobError::DescriptorNotCurrent { descriptor });
        }
        Ok(blob)
    }

    pub fn decode_current_runtime_cache_for_request(
        request: &IblBakeArtifactRequest,
        bytes: &[u8],
    ) -> Result<Self, IblBakeArtifactBlobError> {
        let blob = Self::decode(bytes)?;
        let descriptor = blob.descriptor();
        if !descriptor.is_current_runtime_cache_for(request) {
            return Err(IblBakeArtifactBlobError::DescriptorNotCurrent { descriptor });
        }
        Ok(blob)
    }

    pub const fn header(&self) -> IblBakeArtifactHeader {
        self.header
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.header.descriptor()
    }

    pub fn payload(&self) -> &IblBakeArtifactPayload {
        &self.payload
    }

    pub fn encoded_len(&self) -> usize {
        IBL_BAKE_ARTIFACT_HEADER_SIZE
            + IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE
            + self.payload.bytes().len()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        bytes.extend_from_slice(&self.header.encode());
        bytes.extend_from_slice(&payload_checksum(self.payload.bytes()));
        bytes.extend_from_slice(self.payload.bytes());
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactBlobError {
    TruncatedHeader {
        expected: usize,
        actual: usize,
    },
    TruncatedPayloadChecksum {
        expected: usize,
        actual: usize,
    },
    Header(IblBakeArtifactHeaderError),
    Payload(IblBakeArtifactPayloadError),
    PayloadChecksumMismatch {
        expected: [u8; IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE],
        actual: [u8; IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE],
    },
    DescriptorNotCurrent {
        descriptor: IblBakeArtifactDescriptor,
    },
}

fn payload_checksum(bytes: &[u8]) -> [u8; IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE] {
    *blake3::hash(bytes).as_bytes()
}

fn payload_checksum_array(bytes: &[u8]) -> [u8; IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE] {
    let mut checksum = [0; IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE];
    checksum.copy_from_slice(bytes);
    checksum
}
