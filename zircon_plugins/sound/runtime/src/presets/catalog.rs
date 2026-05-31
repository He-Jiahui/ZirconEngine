use zircon_runtime::core::framework::sound::SoundMixerPresetDescriptor;

use crate::SoundConfig;

use super::default::default_graph;
use super::locators::{
    DEFAULT_MIXER_PRESET_LOCATOR, MUSIC_SFX_MIXER_PRESET_LOCATOR, SPATIAL_ROOM_MIXER_PRESET_LOCATOR,
};
use super::music_sfx::music_sfx_graph;
use super::spatial_room::spatial_room_graph;

pub(crate) fn built_in_mixer_presets(config: &SoundConfig) -> Vec<SoundMixerPresetDescriptor> {
    vec![
        SoundMixerPresetDescriptor::new(
            DEFAULT_MIXER_PRESET_LOCATOR,
            "Default",
            default_graph(config),
        ),
        SoundMixerPresetDescriptor::new(
            MUSIC_SFX_MIXER_PRESET_LOCATOR,
            "Music and SFX",
            music_sfx_graph(config),
        ),
        SoundMixerPresetDescriptor::new(
            SPATIAL_ROOM_MIXER_PRESET_LOCATOR,
            "Spatial Room",
            spatial_room_graph(config),
        ),
    ]
}
