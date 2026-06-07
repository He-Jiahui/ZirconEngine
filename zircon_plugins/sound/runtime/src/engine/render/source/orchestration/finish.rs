use zircon_runtime::core::framework::sound::{
    SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished, SoundSourceId,
    SoundSourceInput,
};

pub(super) struct PendingSourceFinish {
    pub(super) source_id: SoundSourceId,
    descriptor: SoundSourceDescriptor,
    reason: SoundSourceFinishReason,
}

impl PendingSourceFinish {
    pub(super) fn new(
        source_id: SoundSourceId,
        descriptor: SoundSourceDescriptor,
        reason: SoundSourceFinishReason,
    ) -> Self {
        Self {
            source_id,
            descriptor,
            reason,
        }
    }

    pub(super) fn into_event(self) -> SoundSourceFinished {
        let input = self.descriptor.input;
        let clip = match input {
            SoundSourceInput::Clip(clip) => Some(clip),
            SoundSourceInput::External(_)
            | SoundSourceInput::SynthParameter { .. }
            | SoundSourceInput::Silence => None,
        };
        SoundSourceFinished {
            source: self.source_id,
            input,
            clip,
            reason: self.reason,
            completion_action: self.descriptor.completion_action,
            output_track: self.descriptor.output_track,
        }
    }
}
