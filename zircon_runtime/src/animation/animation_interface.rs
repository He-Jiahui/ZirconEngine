use crate::core::framework::animation::AnimationManager;

pub trait AnimationInterface: AnimationManager {
    fn is_enabled(&self) -> bool {
        self.playback_settings().enabled
    }
}
