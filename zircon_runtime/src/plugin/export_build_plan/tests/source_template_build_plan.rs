use super::source_template_command;

#[test]
fn preallocated_source_template_command_preserves_contract() {
    assert_eq!(
        source_template_command(true),
        vec![
            "cargo".to_string(),
            "build".to_string(),
            "--manifest-path".to_string(),
            "Cargo.toml".to_string(),
            "--target-dir".to_string(),
            "stages/source_template/target".to_string(),
            "--release".to_string(),
        ]
    );
}
