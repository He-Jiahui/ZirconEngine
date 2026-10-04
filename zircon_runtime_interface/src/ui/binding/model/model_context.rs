use serde::{Deserialize, Serialize};

use super::UiModelProviderKey;

#[cfg(test)]
#[path = "model_context/tests/resolve_performance_tests.rs"]
mod resolve_performance_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiModelContextLayer {
    Surface,
    Component,
    Row,
    Item,
}

impl UiModelContextLayer {
    pub const ALL: [Self; 4] = [Self::Surface, Self::Component, Self::Row, Self::Item];
}

/// 单层上下文的显式操作：缺少 patch 字段表示继承，Bind 替换父级 provider，Clear 移除该层 provider。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum UiModelContextOverride {
    Bind { provider: UiModelProviderKey },
    Clear,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiModelContextPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    surface: Option<UiModelContextOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    component: Option<UiModelContextOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    row: Option<UiModelContextOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    item: Option<UiModelContextOverride>,
}

impl UiModelContextPatch {
    pub fn with_binding(
        mut self,
        layer: UiModelContextLayer,
        provider: UiModelProviderKey,
    ) -> Self {
        *self.override_for_mut(layer) = Some(UiModelContextOverride::Bind { provider });
        self
    }

    pub fn with_clear(mut self, layer: UiModelContextLayer) -> Self {
        *self.override_for_mut(layer) = Some(UiModelContextOverride::Clear);
        self
    }

    pub fn override_for(&self, layer: UiModelContextLayer) -> Option<&UiModelContextOverride> {
        match layer {
            UiModelContextLayer::Surface => self.surface.as_ref(),
            UiModelContextLayer::Component => self.component.as_ref(),
            UiModelContextLayer::Row => self.row.as_ref(),
            UiModelContextLayer::Item => self.item.as_ref(),
        }
    }

    fn override_for_mut(
        &mut self,
        layer: UiModelContextLayer,
    ) -> &mut Option<UiModelContextOverride> {
        match layer {
            UiModelContextLayer::Surface => &mut self.surface,
            UiModelContextLayer::Component => &mut self.component,
            UiModelContextLayer::Row => &mut self.row,
            UiModelContextLayer::Item => &mut self.item,
        }
    }
}

/// 已解析的逐层 provider 集合，供后续注册表解析字段与 provider schema 时使用。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResolvedModelContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    surface: Option<UiModelProviderKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    component: Option<UiModelProviderKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    row: Option<UiModelProviderKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    item: Option<UiModelProviderKey>,
}

impl UiResolvedModelContext {
    /// 对每层独立应用 patch，产生新有效上下文；未覆盖层从 parent 继承。
    pub fn resolve(parent: Option<&Self>, patch: &UiModelContextPatch) -> Self {
        Self {
            surface: resolve_provider(
                parent.and_then(|parent| parent.surface.as_ref()),
                patch.surface.as_ref(),
            ),
            component: resolve_provider(
                parent.and_then(|parent| parent.component.as_ref()),
                patch.component.as_ref(),
            ),
            row: resolve_provider(
                parent.and_then(|parent| parent.row.as_ref()),
                patch.row.as_ref(),
            ),
            item: resolve_provider(
                parent.and_then(|parent| parent.item.as_ref()),
                patch.item.as_ref(),
            ),
        }
    }

    pub fn provider(&self, layer: UiModelContextLayer) -> Option<&UiModelProviderKey> {
        match layer {
            UiModelContextLayer::Surface => self.surface.as_ref(),
            UiModelContextLayer::Component => self.component.as_ref(),
            UiModelContextLayer::Row => self.row.as_ref(),
            UiModelContextLayer::Item => self.item.as_ref(),
        }
    }

    pub fn providers(&self) -> impl Iterator<Item = (UiModelContextLayer, &UiModelProviderKey)> {
        UiModelContextLayer::ALL
            .into_iter()
            .filter_map(|layer| self.provider(layer).map(|provider| (layer, provider)))
    }

    #[cfg(test)]
    fn provider_mut(&mut self, layer: UiModelContextLayer) -> &mut Option<UiModelProviderKey> {
        match layer {
            UiModelContextLayer::Surface => &mut self.surface,
            UiModelContextLayer::Component => &mut self.component,
            UiModelContextLayer::Row => &mut self.row,
            UiModelContextLayer::Item => &mut self.item,
        }
    }
}

fn resolve_provider(
    parent: Option<&UiModelProviderKey>,
    override_value: Option<&UiModelContextOverride>,
) -> Option<UiModelProviderKey> {
    match override_value {
        None => parent.cloned(),
        Some(UiModelContextOverride::Bind { provider }) => Some(provider.clone()),
        Some(UiModelContextOverride::Clear) => None,
    }
}
