use super::{IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeArtifactPayloadError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactReadbackSectionKind {
    Pmrem,
    IrradianceSh9,
    IrradianceCube,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactReadbackSections {
    descriptor: IblBakeArtifactDescriptor,
    pmrem_rgba16f_bytes: Option<Vec<u8>>,
    irradiance_sh9_bytes: Option<Vec<u8>>,
    irradiance_cube_rgba16f_bytes: Option<Vec<u8>>,
}

impl IblBakeArtifactReadbackSections {
    pub const fn new(descriptor: IblBakeArtifactDescriptor) -> Self {
        Self {
            descriptor,
            pmrem_rgba16f_bytes: None,
            irradiance_sh9_bytes: None,
            irradiance_cube_rgba16f_bytes: None,
        }
    }

    pub fn with_pmrem_rgba16f_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.pmrem_rgba16f_bytes = Some(bytes.into());
        self
    }

    pub fn with_irradiance_sh9_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.irradiance_sh9_bytes = Some(bytes.into());
        self
    }

    pub fn with_irradiance_cube_rgba16f_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.irradiance_cube_rgba16f_bytes = Some(bytes.into());
        self
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub fn pmrem_rgba16f_bytes(&self) -> Option<&[u8]> {
        self.pmrem_rgba16f_bytes.as_deref()
    }

    pub fn irradiance_sh9_bytes(&self) -> Option<&[u8]> {
        self.irradiance_sh9_bytes.as_deref()
    }

    pub fn irradiance_cube_rgba16f_bytes(&self) -> Option<&[u8]> {
        self.irradiance_cube_rgba16f_bytes.as_deref()
    }

    pub fn into_payload(self) -> Result<IblBakeArtifactPayload, IblBakeArtifactReadbackError> {
        let descriptor = self.descriptor;
        let mut bytes = Vec::with_capacity(descriptor.expected_payload_size_bytes());
        append_section_bytes(
            &mut bytes,
            IblBakeArtifactReadbackSectionKind::Pmrem,
            descriptor.expected_pmrem_rgba16f_size_bytes(),
            self.pmrem_rgba16f_bytes,
        )?;
        append_section_bytes(
            &mut bytes,
            IblBakeArtifactReadbackSectionKind::IrradianceSh9,
            descriptor.expected_irradiance_sh9_size_bytes(),
            self.irradiance_sh9_bytes,
        )?;
        append_section_bytes(
            &mut bytes,
            IblBakeArtifactReadbackSectionKind::IrradianceCube,
            descriptor.expected_irradiance_cube_rgba16f_size_bytes(),
            self.irradiance_cube_rgba16f_bytes,
        )?;
        IblBakeArtifactPayload::decode(descriptor, &bytes)
            .map_err(IblBakeArtifactReadbackError::Payload)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactReadbackError {
    MissingSection {
        section: IblBakeArtifactReadbackSectionKind,
    },
    UnexpectedSection {
        section: IblBakeArtifactReadbackSectionKind,
        actual: usize,
    },
    InvalidSectionLength {
        section: IblBakeArtifactReadbackSectionKind,
        expected: usize,
        actual: usize,
    },
    Payload(IblBakeArtifactPayloadError),
}

fn append_section_bytes(
    payload: &mut Vec<u8>,
    section: IblBakeArtifactReadbackSectionKind,
    expected: Option<usize>,
    bytes: Option<Vec<u8>>,
) -> Result<(), IblBakeArtifactReadbackError> {
    match (expected, bytes) {
        (Some(expected), Some(bytes)) if bytes.len() == expected => {
            payload.extend_from_slice(&bytes);
            Ok(())
        }
        (Some(expected), Some(bytes)) => Err(IblBakeArtifactReadbackError::InvalidSectionLength {
            section,
            expected,
            actual: bytes.len(),
        }),
        (Some(_), None) => Err(IblBakeArtifactReadbackError::MissingSection { section }),
        (None, Some(bytes)) => Err(IblBakeArtifactReadbackError::UnexpectedSection {
            section,
            actual: bytes.len(),
        }),
        (None, None) => Ok(()),
    }
}
