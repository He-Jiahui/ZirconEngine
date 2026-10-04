use super::*;

#[test]
fn v2_explicit_layout_usize_values_use_the_runtime_layout_bound() {
    let maximum = Value::Integer(MAX_UI_LAYOUT_DISCRETE_VALUE as i64);
    assert_eq!(
        parse_usize("ui.test.bound", Some(&maximum), "root", "container.rows").unwrap(),
        Some(MAX_UI_LAYOUT_DISCRETE_VALUE)
    );

    let oversized = Value::Integer((MAX_UI_LAYOUT_DISCRETE_VALUE + 1) as i64);
    let error =
        parse_usize("ui.test.bound", Some(&oversized), "root", "container.rows").unwrap_err();
    assert!(error.to_string().contains(&format!(
        "container.rows must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"
    )));
}
