use super::{
    IblBakeArtifactDescriptor, IblBakeArtifactHeader, IblBakeArtifactHeaderError,
    IblBakeArtifactPayload, IblBakeArtifactPayloadError, IblBakeArtifactRequest,
    IBL_BAKE_ARTIFACT_HEADER_SIZE,
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
        let payload =
            IblBakeArtifactPayload::decode(descriptor, &bytes[IBL_BAKE_ARTIFACT_HEADER_SIZE..])
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
        IBL_BAKE_ARTIFACT_HEADER_SIZE + self.payload.bytes().len()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        bytes.extend_from_slice(&self.header.encode());
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
    Header(IblBakeArtifactHeaderError),
    Payload(IblBakeArtifactPayloadError),
    DescriptorNotCurrent {
        descriptor: IblBakeArtifactDescriptor,
    },
}
