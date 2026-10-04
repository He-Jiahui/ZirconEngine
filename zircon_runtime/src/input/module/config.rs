use serde::{Deserialize, Serialize};

use crate::core::framework::input::InputActionMap;

use super::super::runtime::{DefaultInputActionManager, InputActionEvaluator};

/// 输入服务的序列化启动配置；未启用时仍注册服务，但动作表对消费者呈现为空。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub action_map: InputActionMap,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            action_map: InputActionMap::default(),
        }
    }
}

impl InputConfig {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_action_map(mut self, action_map: InputActionMap) -> Self {
        self.action_map = action_map;
        self
    }

    /// 在模块工厂或独立求值器创建前读取；返回拥有所有权的表，后续重绑定须显式更新管理器。
    pub fn effective_action_map(&self) -> InputActionMap {
        if self.enabled {
            self.action_map.clone()
        } else {
            InputActionMap::default()
        }
    }

    pub fn action_evaluator(&self) -> InputActionEvaluator {
        InputActionEvaluator::new(self.effective_action_map())
    }

    pub fn action_manager(&self) -> DefaultInputActionManager {
        DefaultInputActionManager::new(self.effective_action_map())
    }
}
