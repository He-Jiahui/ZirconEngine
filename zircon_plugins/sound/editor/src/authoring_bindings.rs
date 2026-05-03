use zircon_editor::core::editor_extension::{
    ComponentDrawerDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
};
use zircon_runtime::core::framework::sound::{
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};

use crate::{
    SOUND_AUDIO_LISTENER_DRAWER_ID, SOUND_AUDIO_SOURCE_DRAWER_ID, SOUND_AUDIO_VOLUME_DRAWER_ID,
};

pub const SOUND_AUDIO_SOURCE_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_source.drawer.ui.toml";
pub const SOUND_AUDIO_LISTENER_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_listener.drawer.ui.toml";
pub const SOUND_AUDIO_VOLUME_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_volume.drawer.ui.toml";

pub const SOUND_MIXER_OPERATION_PATHS: &[&str] = &[
    "Sound.Mixer.Track.Create",
    "Sound.Mixer.Track.UpdateControls",
    "Sound.Mixer.Track.Delete",
    "Sound.Mixer.Send.Upsert",
    "Sound.Mixer.Send.Delete",
    "Sound.Mixer.Effect.Add",
    "Sound.Mixer.Effect.Update",
    "Sound.Mixer.Effect.Delete",
    "Sound.Mixer.Effect.Reorder",
    "Sound.Mixer.Preset.List",
    "Sound.Mixer.Preset.Apply",
    "Sound.Mixer.Sidechain.SetSource",
    "Sound.Mixer.Automation.Bind",
    "Sound.Mixer.Automation.Unbind",
    "Sound.DynamicEvent.Registry.Open",
    "Sound.Output.Device.Configure",
    "Sound.Output.Device.Start",
    "Sound.Output.Device.Stop",
    "Sound.Debug.Acoustic.ToggleLayer",
];

pub const SOUND_AUDIO_SOURCE_OPERATION_PATHS: &[&str] = &[
    "Sound.Component.AudioSource.Apply",
    "Sound.Component.AudioSource.SetInput",
    "Sound.Component.AudioSource.SetOutputTrack",
    "Sound.Component.AudioSource.UpsertSend",
    "Sound.Component.AudioSource.DeleteSend",
    "Sound.Component.AudioSource.BindParameter",
    "Sound.Component.AudioSource.UnbindParameter",
];

pub const SOUND_AUDIO_LISTENER_OPERATION_PATHS: &[&str] = &[
    "Sound.Component.AudioListener.Apply",
    "Sound.Component.AudioListener.SetActive",
    "Sound.Component.AudioListener.SetHrtfProfile",
];

pub const SOUND_AUDIO_VOLUME_OPERATION_PATHS: &[&str] = &[
    "Sound.Component.AudioVolume.Apply",
    "Sound.Component.AudioVolume.SetShape",
    "Sound.Component.AudioVolume.SetImpulseResponse",
];

pub fn register_sound_authoring_bindings(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for descriptor in sound_editor_operation_descriptors() {
        registry.register_operation(descriptor)?;
    }
    Ok(())
}

pub fn sound_editor_operation_descriptors() -> Vec<EditorOperationDescriptor> {
    sound_editor_operation_specs()
        .into_iter()
        .map(|spec| {
            let path = EditorOperationPath::parse(spec.path).expect("valid sound operation path");
            let mut descriptor = EditorOperationDescriptor::new(path, spec.display_name)
                .with_payload_schema_id(spec.payload_schema)
                .with_required_capabilities(["editor.extension.sound_authoring"]);
            if spec.undoable {
                descriptor =
                    descriptor.with_undoable(UndoableEditorOperation::new(spec.display_name));
            }
            descriptor
        })
        .collect()
}

pub fn sound_audio_source_drawer_descriptor() -> ComponentDrawerDescriptor {
    SOUND_AUDIO_SOURCE_OPERATION_PATHS.iter().fold(
        ComponentDrawerDescriptor::new(
            AUDIO_SOURCE_COMPONENT_TYPE,
            SOUND_AUDIO_SOURCE_DRAWER_TEMPLATE,
            SOUND_AUDIO_SOURCE_DRAWER_ID,
        ),
        |drawer, binding| drawer.with_binding(*binding),
    )
}

pub fn sound_audio_listener_drawer_descriptor() -> ComponentDrawerDescriptor {
    SOUND_AUDIO_LISTENER_OPERATION_PATHS.iter().fold(
        ComponentDrawerDescriptor::new(
            AUDIO_LISTENER_COMPONENT_TYPE,
            SOUND_AUDIO_LISTENER_DRAWER_TEMPLATE,
            SOUND_AUDIO_LISTENER_DRAWER_ID,
        ),
        |drawer, binding| drawer.with_binding(*binding),
    )
}

pub fn sound_audio_volume_drawer_descriptor() -> ComponentDrawerDescriptor {
    SOUND_AUDIO_VOLUME_OPERATION_PATHS.iter().fold(
        ComponentDrawerDescriptor::new(
            AUDIO_VOLUME_COMPONENT_TYPE,
            SOUND_AUDIO_VOLUME_DRAWER_TEMPLATE,
            SOUND_AUDIO_VOLUME_DRAWER_ID,
        ),
        |drawer, binding| drawer.with_binding(*binding),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SoundOperationSpec {
    path: &'static str,
    display_name: &'static str,
    payload_schema: &'static str,
    undoable: bool,
}

fn sound_editor_operation_specs() -> Vec<SoundOperationSpec> {
    vec![
        mixer_spec("Sound.Mixer.Track.Create", "Create Sound Track", true),
        mixer_spec(
            "Sound.Mixer.Track.UpdateControls",
            "Update Sound Track Controls",
            true,
        ),
        mixer_spec("Sound.Mixer.Track.Delete", "Delete Sound Track", true),
        mixer_spec("Sound.Mixer.Send.Upsert", "Upsert Sound Send", true),
        mixer_spec("Sound.Mixer.Send.Delete", "Delete Sound Send", true),
        mixer_spec("Sound.Mixer.Effect.Add", "Add Sound Effect", true),
        mixer_spec("Sound.Mixer.Effect.Update", "Update Sound Effect", true),
        mixer_spec("Sound.Mixer.Effect.Delete", "Delete Sound Effect", true),
        mixer_spec("Sound.Mixer.Effect.Reorder", "Reorder Sound Effects", true),
        mixer_spec("Sound.Mixer.Preset.List", "List Sound Mixer Presets", false),
        mixer_spec("Sound.Mixer.Preset.Apply", "Apply Sound Mixer Preset", true),
        mixer_spec(
            "Sound.Mixer.Sidechain.SetSource",
            "Set Sidechain Source",
            true,
        ),
        mixer_spec("Sound.Mixer.Automation.Bind", "Bind Sound Automation", true),
        mixer_spec(
            "Sound.Mixer.Automation.Unbind",
            "Unbind Sound Automation",
            true,
        ),
        mixer_spec(
            "Sound.DynamicEvent.Registry.Open",
            "Open Sound Dynamic Event Registry",
            false,
        ),
        mixer_spec(
            "Sound.Output.Device.Configure",
            "Configure Sound Output",
            true,
        ),
        mixer_spec("Sound.Output.Device.Start", "Start Sound Output", false),
        mixer_spec("Sound.Output.Device.Stop", "Stop Sound Output", false),
        mixer_spec(
            "Sound.Debug.Acoustic.ToggleLayer",
            "Toggle Acoustic Debug Layer",
            false,
        ),
        component_spec(
            "Sound.Component.AudioSource.Apply",
            "Apply AudioSource",
            "sound.component.audiosource.apply.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.SetInput",
            "Set AudioSource Input",
            "sound.component.audiosource.input.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.SetOutputTrack",
            "Set AudioSource Output Track",
            "sound.component.audiosource.output_track.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.UpsertSend",
            "Upsert AudioSource Send",
            "sound.component.audiosource.send.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.DeleteSend",
            "Delete AudioSource Send",
            "sound.component.audiosource.send.delete.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.BindParameter",
            "Bind AudioSource Parameter",
            "sound.component.audiosource.parameter_binding.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioSource.UnbindParameter",
            "Unbind AudioSource Parameter",
            "sound.component.audiosource.parameter_binding.delete.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioListener.Apply",
            "Apply AudioListener",
            "sound.component.audiolistener.apply.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioListener.SetActive",
            "Set Active AudioListener",
            "sound.component.audiolistener.active.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioListener.SetHrtfProfile",
            "Set AudioListener HRTF Profile",
            "sound.component.audiolistener.hrtf_profile.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioVolume.Apply",
            "Apply AudioVolume",
            "sound.component.audiovolume.apply.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioVolume.SetShape",
            "Set AudioVolume Shape",
            "sound.component.audiovolume.shape.v1",
            true,
        ),
        component_spec(
            "Sound.Component.AudioVolume.SetImpulseResponse",
            "Set AudioVolume Impulse Response",
            "sound.component.audiovolume.impulse_response.v1",
            true,
        ),
    ]
}

fn mixer_spec(
    path: &'static str,
    display_name: &'static str,
    undoable: bool,
) -> SoundOperationSpec {
    SoundOperationSpec {
        path,
        display_name,
        payload_schema: schema_id(path),
        undoable,
    }
}

fn component_spec(
    path: &'static str,
    display_name: &'static str,
    suffix: &'static str,
    undoable: bool,
) -> SoundOperationSpec {
    SoundOperationSpec {
        path,
        display_name,
        payload_schema: suffix,
        undoable,
    }
}

fn schema_id(path: &'static str) -> &'static str {
    match path {
        "Sound.Mixer.Track.Create" => "sound.mixer.track.create.v1",
        "Sound.Mixer.Track.UpdateControls" => "sound.mixer.track.controls.v1",
        "Sound.Mixer.Track.Delete" => "sound.mixer.track.delete.v1",
        "Sound.Mixer.Send.Upsert" => "sound.mixer.send.upsert.v1",
        "Sound.Mixer.Send.Delete" => "sound.mixer.send.delete.v1",
        "Sound.Mixer.Effect.Add" => "sound.mixer.effect.add.v1",
        "Sound.Mixer.Effect.Update" => "sound.mixer.effect.update.v1",
        "Sound.Mixer.Effect.Delete" => "sound.mixer.effect.delete.v1",
        "Sound.Mixer.Effect.Reorder" => "sound.mixer.effect.reorder.v1",
        "Sound.Mixer.Preset.List" => "sound.mixer.preset.list.v1",
        "Sound.Mixer.Preset.Apply" => "sound.mixer.preset.apply.v1",
        "Sound.Mixer.Sidechain.SetSource" => "sound.mixer.sidechain.source.v1",
        "Sound.Mixer.Automation.Bind" => "sound.mixer.automation.bind.v1",
        "Sound.Mixer.Automation.Unbind" => "sound.mixer.automation.unbind.v1",
        "Sound.DynamicEvent.Registry.Open" => "sound.dynamic_event.registry.open.v1",
        "Sound.Output.Device.Configure" => "sound.output.device.configure.v1",
        "Sound.Output.Device.Start" => "sound.output.device.start.v1",
        "Sound.Output.Device.Stop" => "sound.output.device.stop.v1",
        "Sound.Debug.Acoustic.ToggleLayer" => "sound.debug.acoustic.layer.v1",
        _ => "sound.editor.operation.v1",
    }
}
