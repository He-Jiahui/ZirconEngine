use super::ChartRasterCacheKey;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::mui_x_primitives::charts::ChartKind;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn chart_resource_key_separates_dynamic_chart_content() {
    let base = TemplatePaneNodeData::default();
    let mut changed_value = base.clone();
    changed_value.value_percent = 0.8;
    let mut selected = base.clone();
    selected.selected = true;

    assert_ne!(
        ChartRasterCacheKey::new(&base, 64, 32, ChartKind::Gauge, PALETTE),
        ChartRasterCacheKey::new(&changed_value, 64, 32, ChartKind::Gauge, PALETTE),
    );
    assert_ne!(
        ChartRasterCacheKey::new(&base, 64, 32, ChartKind::Pie, PALETTE),
        ChartRasterCacheKey::new(&selected, 64, 32, ChartKind::Pie, PALETTE),
    );
}
