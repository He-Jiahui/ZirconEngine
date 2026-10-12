#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum ChartKind {
    Aggregate,
    Line,
    Bar,
    Pie,
    Sparkline,
    Gauge,
}

/// 分类结果由 MUI X 总入口消费；角色不匹配时必须留给后续组件或通用表面。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chart_kind(
    component_role: &str,
    role: &str,
) -> Option<ChartKind> {
    if super::super::matches_any_role(component_role, role, &["mui-x-line-chart", "LineChart"]) {
        Some(ChartKind::Line)
    } else if super::super::matches_any_role(component_role, role, &["mui-x-bar-chart", "BarChart"])
    {
        Some(ChartKind::Bar)
    } else if super::super::matches_any_role(component_role, role, &["mui-x-pie-chart", "PieChart"])
    {
        Some(ChartKind::Pie)
    } else if super::super::matches_any_role(
        component_role,
        role,
        &["mui-x-sparkline", "SparkLineChart"],
    ) {
        Some(ChartKind::Sparkline)
    } else if super::super::matches_any_role(component_role, role, &["mui-x-gauge", "Gauge"]) {
        Some(ChartKind::Gauge)
    } else if super::super::matches_any_role(component_role, role, &["mui-x-charts", "Charts"]) {
        Some(ChartKind::Aggregate)
    } else {
        None
    }
}
