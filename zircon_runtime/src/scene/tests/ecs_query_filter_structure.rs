fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn added_and_changed_filters_use_direct_tick_branches() {
    let source = include_str!("../ecs/query/query_filter.rs");
    let added_filter = section_between(
        source,
        "impl<T> QueryFilter for Added<T>",
        "pub struct Changed<T>",
    );
    let changed_filter = section_between(
        source,
        "impl<T> QueryFilter for Changed<T>",
        "impl QueryFilter for ()",
    );
    let location_helper = section_between(source, "fn component_ticks_at_location<T>", "}");

    assert!(
        added_filter
            .contains("let Some(component_ticks) = world.component_change_ticks::<T>(entity) else")
            && added_filter.contains("component_ticks.is_added(ticks)")
            && !added_filter.contains(".is_some_and("),
        "Added<T> must branch directly on missing component ticks"
    );
    assert!(
        changed_filter
            .contains("let Some(component_ticks) = world.component_change_ticks::<T>(entity) else")
            && changed_filter.contains("component_ticks.is_changed(ticks)")
            && !changed_filter.contains(".is_some_and("),
        "Changed<T> must branch directly on missing component ticks"
    );
    assert!(
        added_filter.contains("let Some(component_ticks) = component_ticks_at_location::<T>")
            && changed_filter
                .contains("let Some(component_ticks) = component_ticks_at_location::<T>")
            && source.matches(".is_some_and(").count() == 0,
        "cached query filter locations must use direct missing-location branches"
    );
    assert!(
        location_helper.contains(
            "let (_, ticks) = world.component_ref_with_ticks_at_location::<T>(*location)?;"
        ) && location_helper.contains("Some(ticks)")
            && !location_helper.contains(".map(|(_, ticks)| ticks)"),
        "component tick location lookup must return ticks without a tuple-map adapter"
    );
}
