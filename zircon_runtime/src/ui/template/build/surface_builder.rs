use crate::ui::surface::UiSurface;
use crate::ui::template::UiCompiledDocument;
use crate::ui::template::UiTemplateInstance;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::build_error::UiTemplateBuildError;
use super::tree_builder::UiTemplateTreeBuilder;

/// 将模板实例接入 retained surface 的入口，供模板服务、预览和热重载构建替换对象。
/// 资产调用方先完成编译与样式解析；直接实例入口也接纳手工构建的节点树，不计算几何或发布渲染帧。
#[derive(Default)]
pub struct UiTemplateSurfaceBuilder;

impl UiTemplateSurfaceBuilder {
    /// 为指定 surface 身份构建独立运行状态；已编译实例的根与绑定程序须同源，手工实例可使用空程序。
    /// 失败时调用方仍持有原实例，且收不到部分初始化的 surface。
    pub fn build_surface(
        tree_id: UiTreeId,
        instance: &UiTemplateInstance,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        let tree = UiTemplateTreeBuilder::build_tree(tree_id.clone(), instance)?;
        let mut surface = UiSurface::new(tree_id);
        surface.tree = tree;
        surface.install_compiled_binding_program(instance.binding_program().clone());
        Ok(surface)
    }

    /// 复用编译文档中已解析的实例；随后由宿主按目标 viewport 调用布局计算。
    pub fn build_surface_from_compiled_document(
        tree_id: UiTreeId,
        document: &UiCompiledDocument,
    ) -> Result<UiSurface, UiTemplateBuildError> {
        Self::build_surface(tree_id, document.template_instance())
    }
}
