use super::*;

#[test]
fn editor_trigger_is_nonblocking_and_does_not_own_runtime_resources() {
    let source = include_str!("../trigger.rs");
    assert!(source.contains("pub fn submit"));
    assert!(source.contains("pub fn poll"));
    assert!(source.contains("pub fn cancel"));
    assert!(source.contains("pub fn take_source_payload"));
    assert!(!source.contains(&["Scene", "Renderer"].concat()));
    assert!(!source.contains(&["cache", "_root"].concat()));
    assert!(!source.contains(&["ProjectAsset", "Manager"].concat()));
    assert!(!source.contains(&["capture_and_persist", "_reflection_probe"].concat()));
}

#[test]
fn editor_command_keeps_runtime_capture_request_serialized() {
    let request = ReflectionProbeCaptureRequest::new(
        "atrium",
        AssetUri::parse("res://generated/reflection_probes/atrium.zcube").unwrap(),
        [3.0, 1.5, -2.0],
        9,
    );
    let command = ReflectionProbeCaptureEditorCommand::from_request(&request).unwrap();

    assert_eq!(command.request().unwrap(), request);
    assert!(command.request_json().contains("\"schema_version\": 2"));
    assert!(command.placement().unwrap().is_none());
}
