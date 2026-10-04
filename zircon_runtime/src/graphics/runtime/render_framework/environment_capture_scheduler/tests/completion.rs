const SOURCE: &str = include_str!("../completion.rs");

#[test]
fn physical_publication_and_source_payload_precede_terminal_success() {
    let publish = SOURCE
        .find("publication(EnvironmentCapturePublication::Publish, self)")
        .expect("physical publication callback");
    let source_payload = SOURCE[publish..]
        .find("self.publish_source_payload(source_payload, active.bake_key)")
        .map(|offset| publish + offset)
        .expect("source payload publication");
    let terminal = SOURCE[source_payload..]
        .find("self.publish_terminal(")
        .map(|offset| source_payload + offset)
        .expect("terminal success publication");

    assert!(publish < source_payload);
    assert!(source_payload < terminal);
}

#[test]
fn discard_callback_precedes_cancelled_or_superseded_terminal_status() {
    let discard = SOURCE
        .find("publication(EnvironmentCapturePublication::Discard, self)")
        .expect("physical discard callback");
    let terminal = SOURCE[discard..]
        .find("self.publish_terminal(")
        .map(|offset| discard + offset)
        .expect("terminal intent publication");

    assert!(discard < terminal);
}
