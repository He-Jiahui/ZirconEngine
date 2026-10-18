//! 固定字符串的呈现偏好夹具，只检查布尔结果。
//! 字体数据库、成形后端和绘制均不参与这些断言。

use super::cluster_uses_emoji_presentation;

#[test]
/// 对照默认属性、显式选择符及选择符不能单独升级非 emoji 字符的样例。
fn presentation_follows_unicode_properties_and_variation_selectors() {
    assert!(cluster_uses_emoji_presentation("\u{1f600}"));
    assert!(!cluster_uses_emoji_presentation("\u{2600}"));
    assert!(cluster_uses_emoji_presentation("\u{2600}\u{fe0f}"));
    assert!(!cluster_uses_emoji_presentation("\u{1f600}\u{fe0e}"));
    assert!(!cluster_uses_emoji_presentation("\u{1f02c}"));
    assert!(!cluster_uses_emoji_presentation("A\u{fe0f}"));
}

#[test]
/// 对照同一数字键帽带与不带 emoji 选择符的两种写法。
fn keycaps_support_both_standard_selector_forms() {
    assert!(cluster_uses_emoji_presentation("1\u{20e3}"));
    assert!(cluster_uses_emoji_presentation("1\u{fe0f}\u{20e3}"));
}
