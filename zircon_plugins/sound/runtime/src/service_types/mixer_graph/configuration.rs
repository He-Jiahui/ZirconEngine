use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraph};

use crate::mixer_configuration::configure::configure_mixer_graph;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn configure_mixer_impl(
        &self,
        graph: SoundMixerGraph,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        configure_mixer_graph(&mut state, graph)
    }
}
