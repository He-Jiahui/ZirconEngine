use serde::{Deserialize, Deserializer, Serialize};

use super::{RandomServiceCheckpointError, RandomServiceState, RandomStreamCheckpoint};

/// Canonical replay checkpoint for the seed authority and every registered stream.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RandomServiceCheckpoint {
    format_version: u16,
    service: RandomServiceState,
    streams: Vec<RandomStreamCheckpoint>,
}

#[derive(Deserialize)]
struct RandomServiceCheckpointWire {
    format_version: u16,
    service: RandomServiceState,
    streams: Vec<RandomStreamCheckpoint>,
}

impl<'de> Deserialize<'de> for RandomServiceCheckpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RandomServiceCheckpointWire::deserialize(deserializer)?;
        Self::validate(wire.format_version, wire.service, &wire.streams)
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            format_version: wire.format_version,
            service: wire.service,
            streams: wire.streams,
        })
    }
}

impl RandomServiceCheckpoint {
    pub const FORMAT_VERSION: u16 = 1;

    pub fn try_new(
        service: RandomServiceState,
        streams: Vec<RandomStreamCheckpoint>,
    ) -> Result<Self, RandomServiceCheckpointError> {
        Self::validate(Self::FORMAT_VERSION, service, &streams)?;
        Ok(Self {
            format_version: Self::FORMAT_VERSION,
            service,
            streams,
        })
    }

    pub const fn format_version(&self) -> u16 {
        self.format_version
    }

    pub const fn service_state(&self) -> RandomServiceState {
        self.service
    }

    pub fn streams(&self) -> &[RandomStreamCheckpoint] {
        &self.streams
    }

    pub fn into_parts(self) -> (RandomServiceState, Vec<RandomStreamCheckpoint>) {
        (self.service, self.streams)
    }

    fn validate(
        format_version: u16,
        service: RandomServiceState,
        streams: &[RandomStreamCheckpoint],
    ) -> Result<(), RandomServiceCheckpointError> {
        if format_version != Self::FORMAT_VERSION {
            return Err(RandomServiceCheckpointError::UnsupportedFormatVersion {
                version: format_version,
            });
        }

        for (index, stream) in streams.iter().copied().enumerate() {
            if stream.state().algorithm() != service.algorithm() {
                return Err(RandomServiceCheckpointError::StreamAlgorithmMismatch {
                    index,
                    service_algorithm: service.algorithm(),
                    stream_algorithm: stream.state().algorithm(),
                });
            }
            if index > 0 && streams[index - 1].key() >= stream.key() {
                return Err(RandomServiceCheckpointError::NonCanonicalStreamOrder { index });
            }
        }
        Ok(())
    }
}
