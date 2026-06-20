use super::super::ChartKind;

pub(super) fn chart_kind_name(kind: ChartKind) -> &'static str {
    match kind {
        ChartKind::Aggregate => "aggregate",
        ChartKind::Line => "line",
        ChartKind::Bar => "bar",
        ChartKind::Pie => "pie",
        ChartKind::Sparkline => "sparkline",
        ChartKind::Gauge => "gauge",
    }
}
