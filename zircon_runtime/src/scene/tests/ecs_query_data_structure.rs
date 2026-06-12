fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn query_data_component_location_fetches_use_direct_tuple_branches() {
    let source = include_str!("../ecs/query/query_data.rs");
    let read_fetchers = section_between(
        source,
        "impl<'query, T> QueryData for &'query T",
        "impl<'query, T> QueryDataAccess for Ref<'query, T>",
    );
    let optional_fetcher = section_between(
        source,
        "impl<'query, T> QueryData for Option<&'query T>",
        "impl QueryDataAccess for EntityId",
    );

    assert!(
        read_fetchers
            .matches(
                "let (value, _) = world.component_ref_with_ticks_at_location::<T>(*location)?;"
            )
            .count()
            == 2
            && !read_fetchers.contains(".map(|(value, _)| value)"),
        "read-only and mutable-query read projections must unwrap location values without tuple-map adapters"
    );
    assert!(
        optional_fetcher.contains(
            "let Some((value, _)) = world.component_ref_with_ticks_at_location::<T>(*location)"
        ) && optional_fetcher.contains("return Some(None);")
            && optional_fetcher.contains("Some(Some(value))")
            && !optional_fetcher.contains(".map(|(value, _)| value)"),
        "optional component-location fetches must preserve missing-component semantics through direct branches"
    );
}
