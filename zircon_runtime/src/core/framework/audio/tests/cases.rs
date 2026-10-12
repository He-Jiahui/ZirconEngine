//! 保留布局校验的局部源码结构约束，作为语义测试之外的实现回归提示。
//!
//! 检查只读取嵌入源码，不打开音频设备，也不测量分配或性能；实现改名时须核对检查的作用域。
/// 约束命名布局校验保留静态契约查询，避免恢复通过预设构造器生成临时布局的旧表达式。
///
/// # 使用时机与返回值
///
/// 测试框架调用，成功返回单元值；检查源码中的辅助函数标记及被排除的旧表达式，失败会触发断言。
/// 这是文本结构检查，不能独立证明所有临时分配均已消除或预设语义全部正确。
///
/// # 参考
///
/// 参见 [`super::AudioChannelLayout::is_canonical_named_layout`] 与声音布局的语义测试。
#[test]
fn named_layout_validation_uses_borrowed_static_metadata() {
    let source = include_str!("../channel_layout.rs");

    assert!(source.contains("fn named_layout_contract("));
    assert!(!source.contains("Self::from_name(&self.name)"));
}

/// 约束位置去重保留内部位映射，避免恢复逐项扫描后续列表的旧表达式。
///
/// # 使用时机与返回值
///
/// 测试框架调用，成功返回单元值；借用编译时嵌入源码检查标记和旧表达式，失败会触发断言。
/// 文本检查不测量复杂度或运行时行为，位置互异性仍由对应的语义测试与完整契约共同约束。
///
/// # 参考
///
/// 参见 [`super::AudioChannelLayout::has_unique_speakers`] 与声音布局的重复位置测试。
#[test]
fn speaker_uniqueness_validation_is_single_pass() {
    let source = include_str!("../channel_layout.rs");

    assert!(source.contains("fn speaker_bit("));
    assert!(!source.contains("self.speakers[index + 1..].contains(speaker)"));
}
