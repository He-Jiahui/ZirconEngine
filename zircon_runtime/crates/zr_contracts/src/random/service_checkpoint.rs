use std::fmt;

use serde::de::{DeserializeSeed, SeqAccess, Visitor};
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
    #[serde(deserialize_with = "deserialize_bounded_streams")]
    streams: Vec<RandomStreamCheckpoint>,
}

fn deserialize_bounded_streams<'de, D>(
    deserializer: D,
) -> Result<Vec<RandomStreamCheckpoint>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedStreamsVisitor;

    impl<'de> Visitor<'de> for BoundedStreamsVisitor {
        type Value = Vec<RandomStreamCheckpoint>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter
                .write_str("a random stream checkpoint sequence within the stream count budget")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // A wire-provided size hint is not an allocation budget. Grow only for admitted entries.
            let mut streams = Vec::new();
            for _ in 0..RandomServiceCheckpoint::MAX_STREAMS {
                match sequence.next_element()? {
                    Some(stream) => streams.push(stream),
                    None => return Ok(streams),
                }
            }

            // Probe for element N+1 without deserializing or retaining its payload.
            let _: Option<()> = sequence.next_element_seed(RejectExtraStream)?;
            Ok(streams)
        }
    }

    struct RejectExtraStream;

    impl<'de> DeserializeSeed<'de> for RejectExtraStream {
        type Value = ();

        fn deserialize<D>(self, _: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Err(serde::de::Error::custom(
                RandomServiceCheckpointError::TooManyStreams {
                    max: RandomServiceCheckpoint::MAX_STREAMS,
                    actual: RandomServiceCheckpoint::MAX_STREAMS + 1,
                },
            ))
        }
    }

    deserializer.deserialize_seq(BoundedStreamsVisitor)
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
    pub const FORMAT_VERSION: u16 = 2;
    /// Maximum number of persisted streams accepted by the v2 checkpoint contract.
    pub const MAX_STREAMS: usize = 65_536;

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
        if streams.len() > Self::MAX_STREAMS {
            return Err(RandomServiceCheckpointError::TooManyStreams {
                max: Self::MAX_STREAMS,
                actual: streams.len(),
            });
        }

        for (index, stream) in streams.iter().copied().enumerate() {
            if stream.master_seed_generation() != service.master_seed_generation() {
                return Err(
                    RandomServiceCheckpointError::StreamAuthorityGenerationMismatch {
                        index,
                        service_generation: service.master_seed_generation(),
                        stream_generation: stream.master_seed_generation(),
                    },
                );
            }
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
