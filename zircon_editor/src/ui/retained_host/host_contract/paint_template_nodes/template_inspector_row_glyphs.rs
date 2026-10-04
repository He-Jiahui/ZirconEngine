//! Inspector 图标入口统一转交打包 shell 资源；各资源行决定何时显示，图标模块不创建交互区域。

mod checks;
mod chevrons;
mod cubes;
mod swatches;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use checks::push_inspector_check_tick;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chevrons::push_inspector_down_chevron;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use cubes::push_inspector_cube_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use swatches::push_inspector_swatch;
