//! 词边界视图的固定示例：身份保存、完整词前缀与导航查询。
//! 这些断言不验证指定身份与实际分词版本的一致性，也未覆盖非法或超长字节偏移。

use super::*;

// 只验证传入身份原样保留，不把身份变化解释成切换了分词数据。
#[test]
fn map_retains_the_unicode_snapshot_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);

    assert_eq!(
        WordBoundaryMap::for_snapshot("text", next).unicode_data_snapshot(),
        next
    );
}

// 连字符与撇号示例区分完整词边界和简单空白切分。
#[test]
fn completed_prefix_uses_unicode_words_instead_of_whitespace_tokens() {
    let hyphenated = WordBoundaryMap::new("alpha-beta");
    assert_eq!(hyphenated.completed_prefix_end(8), 5);

    let apostrophe = WordBoundaryMap::new("go can't");
    assert_eq!(apostrophe.completed_prefix_end(6), 2);
    assert_eq!(apostrophe.completed_prefix_end(8), 8);
}

// 此夹具验证无空格中文输入的一个已拟合字节前缀。
#[test]
fn completed_prefix_supports_text_without_spaces() {
    let text = "中文文本";
    let third_ideograph_end = "中文文".len();

    assert_eq!(
        WordBoundaryMap::new(text).completed_prefix_end(third_ideograph_end),
        third_ideograph_end
    );
}

// 导航示例验证严格前后关系和词内命中，未覆盖词尾相等或共享端点的选择。
#[test]
fn navigation_queries_share_the_same_word_ranges() {
    let map = WordBoundaryMap::new("alpha-beta");

    assert_eq!(map.previous_word_start(8), Some(6));
    assert_eq!(map.previous_word_start(6), Some(0));
    assert_eq!(map.next_word_end(5), Some(10));
    assert_eq!(map.word_range_at(7), Some(TextRange { start: 6, end: 10 }));
}
