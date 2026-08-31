use serde::{Deserialize, Serialize};

use super::UiModelProviderKey;

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
    pub fn resolve(parent: Option<&Self>, patch: &UiModelContextPatch) -> Self {
        let mut resolved = parent.cloned().unwrap_or_default();
        for layer in UiModelContextLayer::ALL {
            match patch.override_for(layer) {
                None => {}
                Some(UiModelContextOverride::Bind { provider }) => {
                    *resolved.provider_mut(layer) = Some(provider.clone());
                }
                Some(UiModelContextOverride::Clear) => {
                    *resolved.provider_mut(layer) = None;
                }
            }
        }
        resolved
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

    fn provider_mut(&mut self, layer: UiModelContextLayer) -> &mut Option<UiModelProviderKey> {
        match layer {
            UiModelContextLayer::Surface => &mut self.surface,
            UiModelContextLayer::Component => &mut self.component,
            UiModelContextLayer::Row => &mut self.row,
            UiModelContextLayer::Item => &mut self.item,
        }
    }
}
