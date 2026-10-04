//! 结构守卫锁定缓存查询的缺失组件语义及直接读取路径；行为等价性由查询行为测试覆盖。
fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

fn section_after<'a>(source: &'a str, start: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .unwrap_or_else(|| panic!("read section after {start}"))
}

#[test]
fn cached_query_filters_and_fetches_use_direct_branches() {
    let source = include_str!("../ecs/query/cached_query_iter.rs");
    let added_filter = section_between(
        source,
        "impl<T> CachedQueryFilter for Added<T>",
        "impl<T> CachedQueryFilter for Changed<T>",
    );
    let changed_filter = section_between(
        source,
        "impl<T> CachedQueryFilter for Changed<T>",
        "impl CachedQueryFilter for ()",
    );
    let data_fetchers = section_between(
        source,
        "impl<'query, T> CachedQueryData for &'query T",
        "impl CachedQueryData for EntityId",
    );
    let tick_helper = section_after(source, "fn component_ticks_at_location<T>");

    assert!(
        added_filter.contains("let Some(component_ticks) = component_ticks_at_location::<T>")
            && added_filter.contains("return false;")
            && added_filter.contains("component_ticks.is_added(ticks)")
            && !added_filter.contains(".is_some_and("),
        "cached Added<T> filters must branch directly on missing component ticks"
    );
    assert!(
        changed_filter.contains("let Some(component_ticks) = component_ticks_at_location::<T>")
            && changed_filter.contains("return false;")
            && changed_filter.contains("component_ticks.is_changed(ticks)")
            && !changed_filter.contains(".is_some_and("),
        "cached Changed<T> filters must branch directly on missing component ticks"
    );
    let data_fetchers_compact: String = data_fetchers.split_whitespace().collect();
    assert!(
        data_fetchers_compact
            .matches(
                "let(value,_)=World::query_component_ref_with_ticks_at_location::<T>(world,*location)?;"
            )
            .count()
            == 2
            && data_fetchers_compact.contains(
                "letSome((value,_))=World::query_component_ref_with_ticks_at_location::<T>(world,*location)"
            )
            && data_fetchers.contains("Some(Some(value))")
            && !data_fetchers.contains(".map(|(value, _)| value)"),
        "cached query data fetches must unwrap value/tick pairs without tuple-map adapters"
    );
    assert!(
        tick_helper.contains("world.component_ticks_at_location::<T>(*location)")
            && !tick_helper.contains("component_ref_with_ticks_at_location"),
        "cached component tick lookup must use typed tick access without fetching the value"
    );
}
