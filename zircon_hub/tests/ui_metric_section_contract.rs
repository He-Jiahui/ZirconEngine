//! Static contracts for shared Hub metric-section layout policy.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

#[test]
fn metric_card_delegates_copy_to_text_stack_helper() {
    let data_display = read_ui_file("data_display.slint");

    let metric_text_stack = data_display
        .split("component MetricCardTextStack")
        .nth(1)
        .and_then(|source| source.split("export component MetricCard").next())
        .expect("data_display.slint must declare MetricCardTextStack before MetricCard");
    for snippet in [
        "inherits VerticalLayout",
        "in property <string> label;",
        "in property <string> primary;",
        "in property <string> secondary;",
        "in property <bool> compact: false;",
        "padding-left: MaterialStyleMetrics.padding_16;",
        "padding-right: MaterialStyleMetrics.padding_16;",
        "padding-top: root.compact ? HubTokens.space-2 : MaterialStyleMetrics.padding_16;",
        "padding-bottom: root.compact ? HubTokens.space-2 : MaterialStyleMetrics.padding_16;",
        "spacing: root.compact ? HubTokens.space-1 : MaterialStyleMetrics.spacing_6;",
        "text: root.label;",
        "text: root.primary;",
        "text: root.secondary;",
        "style: MaterialTypography.title_small;",
        "color: MaterialPalette.on_surface;",
        "overflow: elide;",
    ] {
        assert!(
            metric_text_stack.contains(snippet),
            "MetricCardTextStack must own metric-card copy and typography; missing {snippet}"
        );
    }
    assert_eq!(
        metric_text_stack.matches("MutedText {").count(),
        2,
        "MetricCardTextStack should own label and secondary muted text nodes"
    );
    assert_eq!(
        metric_text_stack.matches("MaterialText {").count(),
        1,
        "MetricCardTextStack should own the primary metric value text node"
    );

    let metric_card = data_display
        .split("export component MetricCard")
        .nth(1)
        .and_then(|source| source.split("export component HubMetricSlot").next())
        .expect("data_display.slint must declare MetricCard before HubMetricSlot");
    for snippet in [
        "inherits HubPanel",
        "in property <string> label;",
        "in property <string> primary;",
        "in property <string> secondary;",
        "in property <bool> compact: false;",
        "min-height: root.compact ? HubTokens.list-row-md + HubTokens.space-2 : HubTokens.list-row-md;",
        "horizontal-stretch: 1;",
        "MetricCardTextStack {",
        "width: parent.width;",
        "height: parent.height;",
        "label: root.label;",
        "primary: root.primary;",
        "secondary: root.secondary;",
        "compact: root.compact;",
    ] {
        assert!(
            metric_card.contains(snippet),
            "MetricCard must stay a HubPanel shell that delegates metric copy to MetricCardTextStack; missing {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "MutedText {",
        "text: root.label;",
        "text: root.primary;",
        "text: root.secondary;",
        "style: MaterialTypography.title_small;",
    ] {
        assert!(
            !metric_card.contains(forbidden),
            "MetricCard should not own direct metric-card text internals after helper extraction: {forbidden}"
        );
    }
}

#[test]
fn metric_section_state_centralizes_four_card_responsive_grid_policy() {
    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let cloud = read_ui_file("cloud.slint");
    let team = read_ui_file("team.slint");

    assert!(
        components.contains("HubMetricSectionState,"),
        "components.slint must re-export HubMetricSectionState with the data-display metric primitives"
    );

    let metric_state = data_display
        .split("export component HubMetricSectionState")
        .nth(1)
        .and_then(|source| source.split("export component BuildHistoryRow").next())
        .expect("data_display.slint must declare HubMetricSectionState before BuildHistoryRow");
    for snippet in [
        "in property <length> content-width;",
        "in property <length> metric-gap: HubTokens.panel-gap;",
        "in property <length> metric-min-width: HubTokens.panel-min-sm * 3 / 4;",
        "in property <length> regular-row-height: HubTokens.workspace-row-cloud-metrics;",
        "in property <length> compact-row-height: HubTokens.list-row-md + HubTokens.space-2;",
        "in property <length> wide-breakpoint: root.metric-min-width * 4 + root.metric-gap * 4;",
        "in property <length> medium-breakpoint: root.metric-min-width * 2 + root.metric-gap;",
        "in property <length> compact-card-breakpoint: HubTokens.panel-min-md * 2 + root.metric-gap;",
        "in property <bool> allow-two-columns: true;",
        "out property <bool> compact-card: root.content-width < root.compact-card-breakpoint;",
        "out property <bool> four-columns: root.content-width >= root.wide-breakpoint;",
        "out property <bool> two-columns: !root.four-columns && root.allow-two-columns && root.content-width >= root.medium-breakpoint;",
        "out property <int> row-count: root.four-columns ? 1 : (root.two-columns ? 2 : 4);",
        "out property <length> row-height: root.compact-card ? root.compact-row-height : root.regular-row-height;",
        "out property <length> slot-basis: root.four-columns || root.two-columns ? root.metric-min-width : root.content-width;",
        "out property <length> slot-min-width: root.slot-basis;",
        "out property <float> slot-grow: 1;",
        "out property <length> section-height: root.row-height * root.row-count + root.metric-gap * (root.row-count - 1);",
        "width: 0px;",
        "height: 0px;",
    ] {
        assert!(
            metric_state.contains(snippet),
            "HubMetricSectionState must own the shared metric-grid sizing rule; missing {snippet}"
        );
    }
    assert!(
        !metric_state.contains("MetricCard {") && !metric_state.contains("MaterialText {"),
        "HubMetricSectionState should own only layout state; HubMetricSlot remains the metric-card renderer"
    );

    for (page_name, source, state_id, slot_name) in [
        ("CloudPage", &cloud, "cloud-metrics", "CloudMetricSlot"),
        ("TeamPage", &team, "summary-metrics", "TeamSummarySlot"),
    ] {
        for snippet in [
            &format!("{state_id} := HubMetricSectionState {{"),
            "content-width: root.content-width;",
            "compact: ",
            "compact-rows: ",
            "row-height: ",
            "spacing-horizontal: ",
            "spacing-vertical: ",
            &format!("{slot_name} {{"),
            &format!("basis: {state_id}.slot-basis;"),
            &format!("flex-basis: {state_id}.slot-basis;"),
            &format!("grow: {state_id}.slot-grow;"),
            &format!("flex-grow: {state_id}.slot-grow;"),
            &format!("min-width: {state_id}.slot-min-width;"),
            &format!("height: {state_id}.row-height;"),
            &format!("compact-card: {state_id}.compact-card;"),
        ] {
            assert!(
                source.contains(snippet),
                "{page_name} must consume HubMetricSectionState for summary metric sizing; missing {snippet}"
            );
        }
        for forbidden in [
            "metrics-four-columns:",
            "metrics-two-columns:",
            "metric-row-count:",
            "metric-slot-basis:",
            "metric-slot-min-width:",
            "metric-slot-grow:",
            "metric-section-height:",
            "summary-compact:",
            "summary-section-height:",
        ] {
            assert!(
                !source.contains(forbidden),
                "{page_name} should not keep page-local metric grid sizing policy after HubMetricSectionState extraction: {forbidden}"
            );
        }
    }

    assert!(
        cloud.contains("metric-gap: HubTokens.space-3;")
            && cloud.contains("regular-row-height: HubTokens.workspace-row-cloud-metrics;")
            && cloud.contains("compact-row-height: HubTokens.list-row-md + HubTokens.space-2;")
            && cloud.contains("compact-card-breakpoint: HubTokens.panel-min-md * 2 + HubTokens.space-3;")
            && cloud.contains("service-available-height: max(root.service-panel-chrome-height + root.service-row-slot-height, root.content-height - root.header-height - cloud-metrics.section-height - HubTokens.panel-gap * 2);"),
        "CloudPage should configure the shared metric state for the Cloud reference row and service-list budget"
    );
    assert!(
        team.contains("metric-gap: HubTokens.panel-gap;")
            && team.contains("regular-row-height: HubTokens.workspace-row-team-summary;")
            && team.contains("compact-row-height: HubTokens.workspace-row-team-summary;")
            && team.contains("allow-two-columns: false;")
            && team
                .contains("wide-breakpoint: HubTokens.panel-min-sm * 3 + HubTokens.panel-gap * 3;")
            && team.contains(
                "compact-card-breakpoint: HubTokens.panel-min-sm * 3 + HubTokens.panel-gap * 3;",
            ),
        "TeamPage should configure the same metric state for the Team four-card reference row"
    );
}
