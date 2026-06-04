use super::super::super::super::*;

use super::invocation::weapon_fire_invocation;
use super::registration::{register_weapon_fire_event, register_weapon_fire_handlers};

pub(crate) struct FanoutFixture {
    pub(crate) sound: DefaultSoundManager,
    pub(crate) invocation: SoundDynamicEventInvocation,
}

pub(crate) fn fanout_fixture() -> FanoutFixture {
    let sound = DefaultSoundManager::default();
    register_weapon_fire_event(&sound);
    register_weapon_fire_handlers(&sound);

    let invocation = weapon_fire_invocation();
    sound.submit_dynamic_event(invocation.clone()).unwrap();

    FanoutFixture { sound, invocation }
}
