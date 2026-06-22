pub(super) fn collection_type_is_generic(normalized_type: &str) -> bool {
    normalized_type.is_empty()
        || matches!(
            normalized_type,
            "any" | "value" | "uivalue" | "variant" | "unknown"
        )
}

pub(super) fn collection_type_is_numeric(normalized_type: &str) -> bool {
    normalized_type.contains("int")
        || normalized_type.contains("float")
        || normalized_type.contains("double")
        || normalized_type.contains("number")
}

pub(super) fn collection_type_is_reference_like(normalized_type: &str) -> bool {
    normalized_type.contains("asset")
        || normalized_type.contains("instance")
        || normalized_type.contains("object")
        || normalized_type.contains("ref")
}
