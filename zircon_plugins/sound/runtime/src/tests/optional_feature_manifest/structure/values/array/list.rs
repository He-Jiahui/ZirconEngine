use super::super::super::sources::*;

#[test]
fn optional_feature_array_list_owner_uses_isolated_raw_parser() {
    assert!(
        ARRAY_LIST.contains("super::raw::string_array_values(value)"),
        "array list projection should call the raw parser from its child owner"
    );
}
