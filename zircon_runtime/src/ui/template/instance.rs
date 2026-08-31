use serde::{Deserialize, Serialize};

use zircon_runtime_interface::ui::template::{
    UiBindingRef, UiCompiledBindingProgram, UiTemplateNode,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateInstance {
    pub root: UiTemplateNode,
    #[serde(default)]
    binding_program: UiCompiledBindingProgram,
}

impl UiTemplateInstance {
    pub fn new(root: UiTemplateNode) -> Self {
        Self {
            root,
            binding_program: UiCompiledBindingProgram::default(),
        }
    }

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
