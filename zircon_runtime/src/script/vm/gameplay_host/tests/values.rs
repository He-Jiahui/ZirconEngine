use crate::core::framework::script::{
    ScriptHostArguments, ScriptHostCallFrame, ScriptHostOwnedArgumentSource, ScriptHostValue,
};

use super::with_string;

#[test]
fn runtime13_string_extractor_borrows_the_argument_payload() {
    let arguments = vec![ScriptHostValue::String("player.hp".to_string())];
    let argument_source = ScriptHostOwnedArgumentSource::new(&arguments);
    let capabilities = Vec::new();
    let context = ScriptHostCallFrame::new(
        "zr.gameplay.component",
        "component_json",
        ScriptHostArguments::new(&argument_source),
        &capabilities,
        None,
    );

    let value_length = with_string(&context, 0, |value: &str| {
        assert_eq!(value, "player.hp");
        Ok(value.len())
    })
    .expect("string argument is accepted");
    let ScriptHostValue::String(argument) = &arguments[0] else {
        panic!("fixture must contain a string argument");
    };

    assert_eq!(value_length, argument.len());
}

#[test]
fn gameplay_host_consumers_keep_transient_strings_borrowed() {
    let values = include_str!("../values.rs");
    let production_values = values
        .split_once("#[cfg(test)]")
        .map_or(values, |(head, _)| head);

    assert!(
        !production_values.contains("fn expect_string("),
        "gameplay host must not retain an owned string extractor beside with_string"
    );
    for source in [
        include_str!("../combat.rs"),
        include_str!("../components.rs"),
        include_str!("../input.rs"),
        include_str!("../lifecycle.rs"),
        include_str!("../scene_transition.rs"),
    ] {
        let production_source = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(head, _)| head);

        assert!(production_source.contains("with_string("));
        assert!(!production_source.contains("expect_string("));
    }
}
