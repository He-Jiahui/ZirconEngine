use super::super::super::super::*;
use super::descriptor::software_test_descriptor;

pub(super) fn invalid_speaker_metadata_descriptor() -> SoundOutputDeviceDescriptor {
    software_test_descriptor(
        "sound.output.bad-speakers",
        "Bad Speaker Metadata Output",
        SoundChannelLayout {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontLeft,
            ],
        },
        2,
        128,
    )
}
