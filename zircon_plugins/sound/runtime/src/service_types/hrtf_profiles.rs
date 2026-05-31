use zircon_runtime::core::framework::sound::{SoundError, SoundHrtfProfileDescriptor};

use crate::descriptor_validation::hrtf::validate_hrtf_profile_descriptor;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn load_hrtf_profile_impl(
        &self,
        profile: SoundHrtfProfileDescriptor,
    ) -> Result<(), SoundError> {
        validate_hrtf_profile_descriptor(&profile)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .insert(profile.profile_id.clone(), profile);
        state.hrtf_states.clear();
        Ok(())
    }

    pub(super) fn remove_hrtf_profile_impl(&self, profile_id: &str) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .remove(profile_id)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownHrtfProfile {
                profile_id: profile_id.to_string(),
            })?;
        state.hrtf_states.clear();
        Ok(())
    }

    pub(super) fn hrtf_profiles_impl(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
        let mut profiles = self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .hrtf_profiles
            .values()
            .cloned()
            .collect::<Vec<_>>();
        profiles.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(profiles)
    }
}
