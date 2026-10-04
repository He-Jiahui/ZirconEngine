//! 查询数据的已编译位置路径须保留可选组件缺失语义，避免回退到逐实体注册表查询。
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
        "unsafe impl<'query, T> QueryDataAccess for Ref<'query, T>",
    );
    let optional_fetcher = section_between(
        source,
        "impl<'query, T> QueryData for Option<&'query T>",
        "unsafe impl QueryDataAccess for EntityId",
    );

    let read_fetchers_compact: String = read_fetchers.split_whitespace().collect();
    let optional_fetcher_compact: String = optional_fetcher.split_whitespace().collect();
    assert!(
        read_fetchers_compact
            .matches(
                "let(value,_)=World::query_component_ref_with_ticks_at_location::<T>(world,*location)?;"
            )
            .count()
            == 2
            && !read_fetchers.contains(".map(|(value, _)| value)"),
        "read-only and mutable-query read projections must unwrap location values without tuple-map adapters"
    );
    assert!(
        optional_fetcher_compact.contains(
            "letSome((value,_))=World::query_component_ref_with_ticks_at_location::<T>(world,*location)"
        ) && optional_fetcher.contains("return Some(None);")
            && optional_fetcher.contains("Some(Some(value))")
            && !optional_fetcher.contains(".map(|(value, _)| value)"),
        "optional component-location fetches must preserve missing-component semantics through direct branches"
    );
}
