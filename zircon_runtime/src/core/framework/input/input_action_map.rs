use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{InputAction, InputActionContext, InputBinding};

/// 声明动作、上下文和物理绑定的可序列化配置；求值器会编译它，修改后须重新设置映射。
/// 公共字段允许外部构造；新增上下文通过 `add_context` 维护优先级顺序，同名添加保留原项。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionMap {
    #[serde(default)]
    pub contexts: Vec<InputActionContext>,
    pub actions: Vec<InputAction>,
    pub bindings: Vec<InputBinding>,
}

impl InputActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_action(mut self, action: InputAction) -> Self {
        self.add_action(action);
        self
    }

    pub fn with_context(mut self, context: InputActionContext) -> Self {
        self.add_context(context);
        self
    }

    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bind(binding);
        self
    }

    /// 按优先级和标识稳定插入；读取外部构造的未排序上下文时先修正顺序。
    pub fn add_context(&mut self, context: InputActionContext) -> &mut Self {
        if self.has_context(&context.id) {
            return self;
        }
        if !contexts_are_sorted(&self.contexts) {
            self.contexts.sort_by(context_order);
        }
        let insertion_index = self
            .contexts
            .partition_point(|candidate| context_order(candidate, &context).is_lt());
        self.contexts.insert(insertion_index, context);
        self
    }

    pub fn add_action(&mut self, action: InputAction) -> &mut Self {
        if !self.has_action(&action.id) {
            self.actions.push(action);
        }
        self
    }

    pub fn bind(&mut self, binding: InputBinding) -> &mut Self {
        if !binding.is_empty() {
            self.bindings.push(binding);
        }
        self
    }

    pub fn clear_bindings(&mut self, action: impl AsRef<str>) -> &mut Self {
        let action = action.as_ref();
        self.bindings.retain(|binding| binding.action != action);
        self
    }

    pub fn has_action(&self, action: impl AsRef<str>) -> bool {
        let action = action.as_ref();
        self.actions.iter().any(|candidate| candidate.id == action)
    }

    pub fn has_context(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .any(|candidate| candidate.id == context)
    }

    /// 未显式声明的上下文默认可用，使只在动作上命名的旧配置仍能求值。
    pub fn context_enabled(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .find(|candidate| candidate.id == context)
            .map(|candidate| candidate.enabled)
            .unwrap_or(true)
    }

    pub fn bindings_for_action<'a>(
        &'a self,
        action: &'a str,
    ) -> impl Iterator<Item = &'a InputBinding> + 'a {
        self.bindings
            .iter()
            .filter(move |binding| binding.action == action)
    }
}

fn contexts_are_sorted(contexts: &[InputActionContext]) -> bool {
    contexts
        .windows(2)
        .all(|pair| !context_order(&pair[0], &pair[1]).is_gt())
}

fn context_order(left: &InputActionContext, right: &InputActionContext) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then(left.id.cmp(&right.id))
}

#[cfg(test)]
#[path = "tests/input_action_map.rs"]
mod tests;
