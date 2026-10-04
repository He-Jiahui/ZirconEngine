use super::*;

#[test]
fn validation_batch_arguments_require_one_pinned_script() {
    let script = PathBuf::from("zircon-validation-runtime74-batch.ps1");
    assert_eq!(
        parse_validation_batch_script(vec![script.display().to_string()]).unwrap(),
        script
    );
    assert!(parse_validation_batch_script(Vec::new()).is_err());
    assert!(parse_validation_batch_script(vec![
        "zircon-validation-runtime74-batch.ps1".to_string(),
        "unexpected".to_string(),
    ])
    .is_err());
    assert!(parse_validation_batch_script(vec!["unscoped.ps1".to_string()]).is_err());
}
