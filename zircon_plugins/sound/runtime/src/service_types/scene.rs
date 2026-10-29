use std::sync::Arc;

use zircon_runtime::core::framework::sound::{SoundError, SoundSourceDescriptor, SoundSourceId};

use crate::descriptor_validation::source::validate_source_descriptor;
use crate::poison_recovery::lock_recover;

use super::DefaultSoundManager;

pub(crate) struct SceneSource {
    pub(crate) id: SoundSourceId,
    descriptor: SoundSourceDescriptor,
}

#[cfg(test)]
#[path = "tests/scene.rs"]
mod tests;

impl DefaultSoundManager {
    pub(crate) fn same_session(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    pub(crate) fn sync_scene_source(
        &self,
        previous: Option<&SceneSource>,
        mut descriptor: SoundSourceDescriptor,
    ) -> Result<SceneSource, SoundError> {
        if let Some(previous) =
            previous.filter(|source| source.descriptor.input == descriptor.input)
        {
            descriptor.id = Some(previous.id);
            {
                let state = lock_recover(&self.state);
                validate_source_descriptor(&state, &descriptor)?;
            }
            let update = self.source_status_impl(previous.id).and_then(|_| {
                let mut structural = descriptor.clone();
                structural.gain = previous.descriptor.gain;
                structural.speed = previous.descriptor.speed;
                structural.muted = previous.descriptor.muted;
                if structural != previous.descriptor {
                    // Loop, routing and range edits use the existing rebind
                    // contract, retaining the same service source identity.
                    self.update_source_impl(descriptor.clone())?;
                } else {
                    if descriptor.gain != previous.descriptor.gain {
                        self.set_source_gain_impl(previous.id, descriptor.gain)?;
                    }
                    if descriptor.speed != previous.descriptor.speed {
                        self.set_source_speed_impl(previous.id, descriptor.speed)?;
                    }
                    if descriptor.muted != previous.descriptor.muted {
                        if descriptor.muted {
                            self.mute_source_impl(previous.id)?;
                        } else {
                            self.unmute_source_impl(previous.id)?;
                        }
                    }
                }
                Ok(())
            });
            match update {
                Ok(()) => {}
                // Completion (or an explicit service stop) must not turn an
                // unchanged playing=true component into a repeating one-shot.
                // Keep its identity until stop/disable/removal or input change.
                // A callback can also finish between status and a control's
                // second completion poll; that race retains the same tombstone.
                Err(SoundError::UnknownSource { .. }) => {}
                Err(error) => return Err(error),
            }
            return Ok(SceneSource {
                id: previous.id,
                descriptor,
            });
        }
        if let Some(previous) = previous {
            self.release_scene_source(previous.id)?;
        }
        descriptor.id = None;
        let id = self.create_source_impl(descriptor.clone())?;
        lock_recover(&self.scene_sources).insert(id);
        descriptor.id = Some(id);
        Ok(SceneSource { id, descriptor })
    }

    pub(crate) fn release_scene_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        match self.remove_source_impl(source) {
            Ok(()) | Err(SoundError::UnknownSource { .. }) => {
                lock_recover(&self.scene_sources).remove(&source);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn release_scene_sources(&self) -> Result<(), SoundError> {
        let sources = lock_recover(&self.scene_sources)
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for source in sources {
            self.release_scene_source(source)?;
        }
        Ok(())
    }
}
