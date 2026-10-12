use super::update_optional_stat_string;

#[test]
fn optional_stat_string_reuses_storage_for_stable_and_shorter_values() {
    let mut value = Some(String::from("default-render-profile"));
    let allocation = value.as_ref().unwrap().as_ptr();

    update_optional_stat_string(&mut value, Some("default-render-profile"));
    assert_eq!(value.as_ref().unwrap().as_ptr(), allocation);

    update_optional_stat_string(&mut value, Some("low"));
    assert_eq!(value.as_deref(), Some("low"));
    assert_eq!(value.as_ref().unwrap().as_ptr(), allocation);
}
