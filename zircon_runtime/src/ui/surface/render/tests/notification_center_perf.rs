use toml::Value;

use super::collect_visible_notification_rows;

#[test]
fn visible_rows_stop_after_the_first_valid_depth_first_entries() {
    let notifications = Value::Array(vec![
        Value::String(String::new()),
        Value::String("first|title=First".to_string()),
        Value::Array(vec![
            Value::String("second|title=Second".to_string()),
            Value::String("third|title=Third".to_string()),
        ]),
    ]);
    let mut rows = Vec::new();

    collect_visible_notification_rows(&notifications, 2, &mut rows);

    assert_eq!(
        rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        vec!["first", "second"]
    );
}
