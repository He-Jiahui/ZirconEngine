// TODO: [CR-EDITOR-PAINT-CONTROLSTYLE-0005] 确认列间隙分隔线应使用哪个裁剪域；负偏移把线放到列框外，
// 当前 shell_panel 入口先把 clip 限于本列，竖线 helper 随即剔除。根入口/几何测试正有外来修改，需复核最新调用契约。
// 该偏移用于列间隙的外置分隔位置，不是内容面板内部留白。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DRAWER_COLUMN_SEPARATOR_OFFSET: f32 = -6.0;
