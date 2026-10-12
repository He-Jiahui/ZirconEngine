// 保留立体声名称和数量但逆置扬声器顺序，验证规范布局不能仅按名称接受。
use super::super::super::super::*;
use super::descriptor::software_test_descriptor;

pub(crate) fn invalid_speaker_metadata_descriptor() -> SoundOutputDeviceDescriptor {
    software_test_descriptor(
        "sound.output.bad-speakers",
        "Bad Speaker Metadata Output",
        AudioChannelLayout {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontLeft,
            ],
        },
        2,
        128,
    )
}
