//! 公开的特性接口需要与当前描述符驱动的图编译路径保持一致。
use crate::core::framework::render::RenderFrameExtract;
use crate::render_graph::RenderGraphBuilder;

use crate::graphics::feature::RenderFeatureDescriptor;

// TODO: [CR-GRAPHICS-RUNTIME-0001] 确认动态注册通道接口的预期入口；当前第一方图编译只消费描述符，未找到此回调调用点。
/// 向框架声明图契约；动态注册入口的生命周期需与描述符编译路径对齐。
pub trait RenderFeature: Send + Sync {
    fn descriptor(&self) -> RenderFeatureDescriptor;

    fn register_passes(
        &self,
        _graph: &mut RenderGraphBuilder,
        _extract: &RenderFrameExtract,
    ) -> Result<(), String> {
        Ok(())
    }
}
