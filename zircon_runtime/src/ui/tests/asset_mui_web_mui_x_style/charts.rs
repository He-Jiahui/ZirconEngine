use super::*;

#[test]
fn mui_x_chart_and_gauge_utility_classes_match_retained_targets() {
    let style = UiAssetLoader::load_toml_str(MUI_X_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_X_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_asset(style)
        .expect("style asset registration succeeds");
    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let chart = find_node(root, "LineChartRoot");
    assert_eq!(
        str_attr(chart, "surface_variant"),
        Some("line-chart-loading")
    );
    assert_eq!(str_attr(chart, "text_tone"), Some("chart-configured"));
    assert_eq!(
        str_attr(chart, "validation_level"),
        Some("chart-customized")
    );
    assert_classes(
        chart,
        &[
            "MuiLineChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-loading",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );
    let chart_legend = find_node(root, "LineChartLegend");
    assert_eq!(
        str_attr(chart_legend, "text_tone"),
        Some("chart-legend-state")
    );
    assert_classes(
        chart_legend,
        &[
            "MuiChartsLegend",
            "MuiChartsLegend-hasSeries",
            "MuiChartsLegend-hasCustomColors",
            "MuiChartsLegend-loading",
        ],
    );
    let chart_tooltip = find_node(root, "LineChartTooltip");
    assert_eq!(
        str_attr(chart_tooltip, "surface_variant"),
        Some("chart-tooltip-state")
    );
    assert_classes(
        chart_tooltip,
        &[
            "MuiChartsTooltip",
            "MuiChartsTooltip-loading",
            "MuiChartsTooltip-hasSeries",
            "MuiChartsTooltip-hasAxes",
            "MuiChartsTooltip-hasMargin",
            "MuiChartsTooltip-interactionHover",
        ],
    );

    let bar_chart = find_node(root, "BarChartRoot");
    assert_eq!(
        str_attr(bar_chart, "text_tone"),
        Some("bar-chart-configured")
    );
    assert_classes(
        bar_chart,
        &[
            "MuiBarChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let pie_chart = find_node(root, "PieChartRoot");
    assert_eq!(
        str_attr(pie_chart, "text_tone"),
        Some("pie-chart-configured")
    );
    assert_classes(
        pie_chart,
        &[
            "MuiPieChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let sparkline_chart = find_node(root, "SparkLineChartRoot");
    assert_eq!(
        str_attr(sparkline_chart, "text_tone"),
        Some("sparkline-configured")
    );
    assert_classes(
        sparkline_chart,
        &[
            "MuiSparkLineChart-root",
            "MuiChartsSurface-root",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );

    let gauge = find_node(root, "GaugeRoot");
    assert_eq!(str_attr(gauge, "validation_level"), Some("gauge-valued"));
    assert_classes(
        gauge,
        &[
            "MuiGauge-root",
            "MuiChartsSurface-root",
            "MuiGauge-hasValue",
            "MuiCharts-hasSeries",
            "MuiCharts-hasAxes",
            "MuiCharts-hasCustomColors",
            "MuiCharts-hasMargin",
        ],
    );
    let gauge_tooltip = find_node(root, "GaugeTooltip");
    assert_eq!(
        str_attr(gauge_tooltip, "validation_level"),
        Some("gauge-tooltip-valued")
    );
    assert_classes(
        gauge_tooltip,
        &[
            "MuiChartsTooltip",
            "MuiChartsTooltip-hasSeries",
            "MuiChartsTooltip-hasAxes",
            "MuiChartsTooltip-hasMargin",
            "MuiChartsTooltip-hasValue",
        ],
    );
}
