use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::extension::InspectorCustomizationDescriptor;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_runtime::core::framework::sound::{
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};

use crate::capability::SOUND_AUTHORING_CAPABILITY;
use crate::extension_ids::{
    SOUND_AUDIO_LISTENER_DRAWER_ID, SOUND_AUDIO_SOURCE_DRAWER_ID, SOUND_AUDIO_VOLUME_DRAWER_ID,
};

pub const SOUND_AUDIO_SOURCE_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_source.drawer.zui";
pub const SOUND_AUDIO_LISTENER_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_listener.drawer.zui";
pub const SOUND_AUDIO_VOLUME_DRAWER_TEMPLATE: &str =
    "plugins://sound/editor/audio_volume.drawer.zui";

pub const SOUND_MIXER_OPERATION_PATHS: &[&str] = &[
    "sound.mixer.track.create",
    "sound.mixer.track.update_controls",
    "sound.mixer.track.delete",
    "sound.mixer.send.upsert",
    "sound.mixer.send.delete",
    "sound.mixer.effect.add",
    "sound.mixer.effect.update",
    "sound.mixer.effect.delete",
    "sound.mixer.effect.reorder",
    "sound.mixer.preset.list",
    "sound.mixer.preset.apply",
    "sound.mixer.sidechain.set_source",
    "sound.mixer.automation.bind",
    "sound.mixer.automation.unbind",
    "sound.dynamic_event.registry.open",
    "sound.output.device.refresh",
    "sound.output.device.configure",
    "sound.output.device.start",
    "sound.output.device.stop",
    "sound.debug.acoustic.toggle_layer",
];

pub const SOUND_AUDIO_SOURCE_OPERATION_PATHS: &[&str] = &[
    "sound.component.audio_source.apply",
    "sound.component.audio_source.set_input",
    "sound.component.audio_source.set_output_track",
    "sound.component.audio_source.upsert_send",
    "sound.component.audio_source.delete_send",
    "sound.component.audio_source.bind_parameter",
    "sound.component.audio_source.unbind_parameter",
];

pub const SOUND_AUDIO_LISTENER_OPERATION_PATHS: &[&str] = &[
    "sound.component.audio_listener.apply",
    "sound.component.audio_listener.set_active",
    "sound.component.audio_listener.set_hrtf_profile",
];

pub const SOUND_AUDIO_VOLUME_OPERATION_PATHS: &[&str] = &[
    "sound.component.audio_volume.apply",
    "sound.component.audio_volume.set_shape",
    "sound.component.audio_volume.set_impulse_response",
];

pub fn register_sound_authoring_bindings(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for descriptor in sound_editor_command_descriptors() {
        registry.register_command(descriptor)?;
    }
    Ok(())
}

pub fn sound_editor_command_descriptors() -> Vec<EditorCommandDescriptor> {
    sound_editor_operation_specs()
        .into_iter()
        .map(|spec| {
            let path = EditorOperationPath::parse(spec.path).expect("valid sound operation path");
            EditorCommandDescriptor::operation(path, spec.display_name)
                .with_payload_schema_id(spec.payload_schema)
                .with_required_capabilities([SOUND_AUTHORING_CAPABILITY])
        })
        .collect()
}

pub fn sound_audio_source_inspector_customization() -> InspectorCustomizationDescriptor {
    SOUND_AUDIO_SOURCE_OPERATION_PATHS.iter().fold(
        InspectorCustomizationDescriptor::new(
            AUDIO_SOURCE_COMPONENT_TYPE,
            SOUND_AUDIO_SOURCE_DRAWER_TEMPLATE,
            SOUND_AUDIO_SOURCE_DRAWER_ID,
        ),
        |drawer, binding| drawer.with_binding(*binding),
    )
}

pub fn sound_audio_listener_inspector_customization() -> InspectorCustomizationDescriptor {
    SOUND_AUDIO_LISTENER_OPERATION_PATHS.iter().fold(
        InspectorCustomizationDescriptor::new(
            AUDIO_LISTENER_COMPONENT_TYPE,
            SOUND_AUDIO_LISTENER_DRAWER_TEMPLATE,
            SOUND_AUDIO_LISTENER_DRAWER_ID,
        ),
        |drawer, binding| drawer.with_binding(*binding),
    )
}

pub fn sound_audio_volume_inspector_customization() -> InspectorCustomizationDescriptor {
    SOUND_AUDIO_VOLUME_OPERATION_PATHS.iter().fold(
        InspectorCustomizationDescriptor::new(
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
}

fn sound_editor_operation_specs() -> Vec<SoundOperationSpec> {
    vec![
        mixer_spec("sound.mixer.track.create", "Create Sound Track"),
        mixer_spec(
            "sound.mixer.track.update_controls",
            "Update Sound Track Controls",
        ),
        mixer_spec("sound.mixer.track.delete", "Delete Sound Track"),
        mixer_spec("sound.mixer.send.upsert", "Upsert Sound Send"),
        mixer_spec("sound.mixer.send.delete", "Delete Sound Send"),
        mixer_spec("sound.mixer.effect.add", "Add Sound Effect"),
        mixer_spec("sound.mixer.effect.update", "Update Sound Effect"),
        mixer_spec("sound.mixer.effect.delete", "Delete Sound Effect"),
        mixer_spec("sound.mixer.effect.reorder", "Reorder Sound Effects"),
        mixer_spec("sound.mixer.preset.list", "List Sound Mixer Presets"),
        mixer_spec("sound.mixer.preset.apply", "Apply Sound Mixer Preset"),
        mixer_spec("sound.mixer.sidechain.set_source", "Set Sidechain Source"),
        mixer_spec("sound.mixer.automation.bind", "Bind Sound Automation"),
        mixer_spec("sound.mixer.automation.unbind", "Unbind Sound Automation"),
        mixer_spec(
            "sound.dynamic_event.registry.open",
            "Open Sound Dynamic Event Registry",
        ),
        mixer_spec("sound.output.device.refresh", "Refresh Sound Outputs"),
        mixer_spec("sound.output.device.configure", "Configure Sound Output"),
        mixer_spec("sound.output.device.start", "Start Sound Output"),
        mixer_spec("sound.output.device.stop", "Stop Sound Output"),
        mixer_spec(
            "sound.debug.acoustic.toggle_layer",
            "Toggle Acoustic Debug Layer",
        ),
        component_spec(
            "sound.component.audio_source.apply",
            "Apply AudioSource",
            "sound.component.audiosource.apply.v1",
        ),
        component_spec(
            "sound.component.audio_source.set_input",
            "Set AudioSource Input",
            "sound.component.audiosource.input.v1",
        ),
        component_spec(
            "sound.component.audio_source.set_output_track",
            "Set AudioSource Output Track",
            "sound.component.audiosource.output_track.v1",
        ),
        component_spec(
            "sound.component.audio_source.upsert_send",
            "Upsert AudioSource Send",
            "sound.component.audiosource.send.v1",
        ),
        component_spec(
            "sound.component.audio_source.delete_send",
            "Delete AudioSource Send",
            "sound.component.audiosource.send.delete.v1",
        ),
        component_spec(
            "sound.component.audio_source.bind_parameter",
            "Bind AudioSource Parameter",
            "sound.component.audiosource.parameter_binding.v1",
        ),
        component_spec(
            "sound.component.audio_source.unbind_parameter",
            "Unbind AudioSource Parameter",
            "sound.component.audiosource.parameter_binding.delete.v1",
        ),
        component_spec(
            "sound.component.audio_listener.apply",
            "Apply AudioListener",
            "sound.component.audiolistener.apply.v1",
        ),
        component_spec(
            "sound.component.audio_listener.set_active",
            "Set Active AudioListener",
            "sound.component.audiolistener.active.v1",
        ),
        component_spec(
            "sound.component.audio_listener.set_hrtf_profile",
            "Set AudioListener HRTF Profile",
            "sound.component.audiolistener.hrtf_profile.v1",
        ),
        component_spec(
            "sound.component.audio_volume.apply",
            "Apply AudioVolume",
            "sound.component.audiovolume.apply.v1",
        ),
        component_spec(
            "sound.component.audio_volume.set_shape",
            "Set AudioVolume Shape",
            "sound.component.audiovolume.shape.v1",
        ),
        component_spec(
            "sound.component.audio_volume.set_impulse_response",
            "Set AudioVolume Impulse Response",
            "sound.component.audiovolume.impulse_response.v1",
        ),
    ]
}

fn mixer_spec(path: &'static str, display_name: &'static str) -> SoundOperationSpec {
    SoundOperationSpec {
        path,
        display_name,
        payload_schema: schema_id(path),
    }
}

fn component_spec(
    path: &'static str,
    display_name: &'static str,
    suffix: &'static str,
) -> SoundOperationSpec {
    SoundOperationSpec {
        path,
        display_name,
        payload_schema: suffix,
    }
}

fn schema_id(path: &'static str) -> &'static str {
    match path {
        "sound.mixer.track.create" => "sound.mixer.track.create.v1",
        "sound.mixer.track.update_controls" => "sound.mixer.track.controls.v1",
        "sound.mixer.track.delete" => "sound.mixer.track.delete.v1",
        "sound.mixer.send.upsert" => "sound.mixer.send.upsert.v1",
        "sound.mixer.send.delete" => "sound.mixer.send.delete.v1",
        "sound.mixer.effect.add" => "sound.mixer.effect.add.v1",
        "sound.mixer.effect.update" => "sound.mixer.effect.update.v1",
        "sound.mixer.effect.delete" => "sound.mixer.effect.delete.v1",
        "sound.mixer.effect.reorder" => "sound.mixer.effect.reorder.v1",
        "sound.mixer.preset.list" => "sound.mixer.preset.list.v1",
        "sound.mixer.preset.apply" => "sound.mixer.preset.apply.v1",
        "sound.mixer.sidechain.set_source" => "sound.mixer.sidechain.source.v1",
        "sound.mixer.automation.bind" => "sound.mixer.automation.bind.v1",
        "sound.mixer.automation.unbind" => "sound.mixer.automation.unbind.v1",
        "sound.dynamic_event.registry.open" => "sound.dynamic_event.registry.open.v1",
        "sound.output.device.refresh" => "sound.output.device.refresh.v1",
        "sound.output.device.configure" => "sound.output.device.configure.v1",
        "sound.output.device.start" => "sound.output.device.start.v1",
        "sound.output.device.stop" => "sound.output.device.stop.v1",
        "sound.debug.acoustic.toggle_layer" => "sound.debug.acoustic.layer.v1",
        _ => "sound.editor.operation.v1",
    }
}
