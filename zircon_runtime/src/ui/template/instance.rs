use serde::{Deserialize, Serialize};

use zircon_runtime_interface::ui::template::{
    UiBindingRef, UiCompiledBindingProgram, UiTemplateNode,
};

/// 供表面构造与编辑器实例缓存复用的展开后节点树及其对应绑定程序。
/// 手工构造只带作者绑定元数据；需要执行编译绑定时应使用编译器产物，后续修改根树不会自动重编程序。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateInstance {
    pub root: UiTemplateNode,
    #[serde(default)]
    binding_program: UiCompiledBindingProgram,
}

impl UiTemplateInstance {
    /// 创建不带编译绑定的实例，适用于手写树及只验证树布局或交互元数据的调用。
    pub fn new(root: UiTemplateNode) -> Self {
        Self {
            root,
            binding_program: UiCompiledBindingProgram::default(),
        }
    }

    // 编译器在参数、控件作用域和样式全部确定后配对这两份数据，节点先序必须与绑定索引一致。
    pub(crate) fn with_binding_program(
        root: UiTemplateNode,
        binding_program: UiCompiledBindingProgram,
    ) -> Self {
        Self {
            root,
            binding_program,
        }
    }

    pub fn binding_program(&self) -> &UiCompiledBindingProgram {
        &self.binding_program
    }

    // TODO: [CR-UI-TEMPLATE-0001] 确认深层手工实例的绑定收集预算；树构建已有深链测试，此递归入口尚无对应覆盖，下一步在适配器实例解析中补深链用例。
    /// 为适配器检查作者绑定提供先序借用视图；它不生成运行时句柄，也不修改绑定程序。
    pub fn binding_refs(&self) -> Vec<&UiBindingRef> {
        let mut bindings = Vec::new();
        collect_binding_refs(&self.root, &mut bindings);
        bindings
    }
}

fn collect_binding_refs<'a>(node: &'a UiTemplateNode, bindings: &mut Vec<&'a UiBindingRef>) {
    bindings.extend(node.bindings.iter());
    for child in &node.children {
        collect_binding_refs(child, bindings);
    }
}
