use super::*;

#[test]
fn runtime_15_review_guard_source_inventory_sources_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_REVIEW_GUARD_SOURCES, 35usize),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[0], 80),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[1], 85),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[2], 85),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[3], 45),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[4], 140),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[5], 35),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[6], 115),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[7], 70),
        (STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN[8], 100),
    ] {
        assert_line_budget(path, limit, "review-guard source inventory");
    }
    for (path, limit) in [
        (REVIEW_GUARD_STRUCTURE_ROWS, 50usize),
        (STATUS_REVIEW_FOUNDATION_CHILD, 45),
        (DATE_REVIEW_FOUNDATION_CHILD, 45),
        (CODE_REVIEW_STATUS_MAP_SOURCE, 85),
        (TYPED_ERROR_STRUCTURE_STATUS_MAP_SOURCE, 85),
    ] {
        assert_line_budget(path, limit, "review-guard source status");
    }
    for path in STATUS_REVIEW_FOUNDATION_CHILDREN
        .iter()
        .chain(DATE_REVIEW_FOUNDATION_CHILDREN.iter())
    {
        assert_line_budget(path, 90, "review-guard foundation child map");
    }

    for path in [
        STATUS_REVIEW_CODE_REVIEW_CHILD,
        DATE_REVIEW_CODE_REVIEW_CHILD,
    ] {
        assert_line_budget(path, 45, "code-review map parent");
    }

    for path in STATUS_REVIEW_CODE_REVIEW_CHILDREN
        .iter()
        .chain(DATE_REVIEW_CODE_REVIEW_CHILDREN.iter())
    {
        assert_line_budget(path, 90, "review-guard code-review child map");
    }

    for path in STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN {
        assert_line_budget(path, 100, "review-guard foundation map guard child");
    }

    for path in STRUCTURE_REVIEW_FOUNDATION_ROUTE_MOUNT_CHILDREN {
        assert_line_budget(path, 95, "review-guard foundation route-mount child");
    }

    for path in STRUCTURE_REVIEW_FOUNDATION_STATUS_MIRROR_CHILDREN {
        assert_line_budget(path, 95, "review-guard foundation status-mirror child");
    }

    for path in REVIEW_GUARD_STRUCTURE_ROW_CHILDREN {
        assert_line_budget(path, 140, "review-guard row-data child");
    }

    for path in REVIEW_GUARD_STRUCTURE_ROW_GRANDCHILDREN
        .iter()
        .chain(STRUCTURE_SUPPORT_ROW_DATA_OWNER_ROW_CHILDREN.iter())
    {
        assert_line_budget(path, 80, "review-guard nested row-data child");
    }
}

fn assert_line_budget(path: &str, limit: usize, label: &str) {
    let line_count = read_runtime_src(path).lines().count();
    assert!(
        line_count < limit,
        "{path} should stay below the {label} budget {limit}; got {line_count} lines"
    );
}
