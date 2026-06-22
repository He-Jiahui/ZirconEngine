use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::declaration::UiAssetStyleRuleDeclarationData;
use super::matched_rule::UiAssetMatchedStyleRuleData;
use super::rule::UiAssetStyleRuleData;
use super::state::UiAssetStyleStateData;
use super::theme_source::UiAssetThemeSourceData;
use super::token::UiAssetStyleTokenData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetStylePanelData {
    pub states: UiAssetStyleStateData,
    pub class_items: ModelRc<SharedString>,
    pub theme_source: UiAssetThemeSourceData,
    pub rule: UiAssetStyleRuleData,
    pub matched_rule: UiAssetMatchedStyleRuleData,
    pub rule_declaration: UiAssetStyleRuleDeclarationData,
    pub token: UiAssetStyleTokenData,
    pub can_create_rule: bool,
    pub can_extract_rule: bool,
    pub stylesheet_items: ModelRc<SharedString>,
}
