//! 脚本分段、括号继承、呈现覆盖及快照身份的局部回归契约。
//! 夹具的范围均为 UTF-8 字节坐标；这些断言不代表字体覆盖或完整 Unicode 一致性证明。

use super::{
    preferred_script_for_language, script_for_range, script_segments, ParagraphTextAnalysis,
};
use crate::text::{compiled_unicode_data_snapshot_id, FontScript, FontScriptTag, TextRange};
use unicode_script::Script;

/// 在拉丁与阿拉伯文本交界处查询，防止代表脚本越过已有段界。
#[test]
fn script_range_lookup_preserves_segment_boundaries() {
    let text = "abcمرحبا";
    let segments = script_segments(text, None);

    assert_eq!(
        script_for_range(&segments, TextRange { start: 0, end: 1 }).iso15924,
        "Latn"
    );
    assert_eq!(
        script_for_range(
            &segments,
            TextRange {
                start: 3,
                end: text.len(),
            },
        )
        .iso15924,
        "Arab"
    );
}

/// 长音符的脚本扩展须跟随兼容片假名邻居，不能只看标量的主脚本。
#[test]
fn script_extensions_follow_the_compatible_neighbor() {
    let text = "カ\u{30fc}A";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].range, TextRange { start: 0, end: 6 });
    assert_eq!(segments[0].script.iso15924, "Kana");
    assert_eq!(segments[1].script.iso15924, "Latn");
}

/// 开头通用字符和组合标记应与随后首个具体脚本共同解析。
#[test]
fn leading_common_and_inherited_characters_follow_the_first_specific_script() {
    let text = " \u{0301}مرحبا";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Arab");
}

/// 中间与末尾通用标点保留前一具体脚本，范围仍使用原文的字节边界。
#[test]
fn intermediate_and_trailing_common_punctuation_follow_the_previous_script() {
    let text = "abc-مرحبا!";
    let arabic_start = "abc-".len();

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 2);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: arabic_start,
        }
    );
    assert_eq!(segments[0].script.iso15924, "Latn");
    assert_eq!(
        segments[1].range,
        TextRange {
            start: arabic_start,
            end: text.len(),
        }
    );
    assert_eq!(segments[1].script.iso15924, "Arab");
}

/// 仅含通用标点时保留 Common 身份，不凭空产生具体脚本。
#[test]
fn common_only_text_retains_a_common_script_segment() {
    let text = " ... ";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Zyyy");
}

/// 显式脚本子标签可解开多候选歧义，偏好来自子标签而非语言推断。
#[test]
fn explicit_language_script_subtag_resolves_an_ambiguous_extension() {
    let segments = script_segments("\u{30fc}", Some("ja-Hira-JP"));

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].script.iso15924, "Hira");
}

/// 私用扩展中的相似文本不能冒充正式脚本子标签。
#[test]
fn private_use_language_subtags_do_not_impersonate_a_script_subtag() {
    assert_eq!(
        preferred_script_for_language(Some("ja-Hira-JP")),
        Some(Script::Hiragana)
    );
    assert_eq!(preferred_script_for_language(Some("ja-x-Kana")), None);
}

/// 闭括号恢复开括号上下文，内部不同脚本不改变开括号已保存的决议。
#[test]
fn paired_brackets_return_to_the_opening_context_script() {
    let text = "abc(مرحبا)def";
    let arabic_start = "abc(".len();
    let arabic_end = arabic_start + "مرحبا".len();

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].range.end, arabic_start);
    assert_eq!(segments[0].script.iso15924, "Latn");
    assert_eq!(
        segments[1].range,
        TextRange {
            start: arabic_start,
            end: arabic_end
        }
    );
    assert_eq!(segments[1].script.iso15924, "Arab");
    assert_eq!(segments[2].range.start, arabic_end);
    assert_eq!(segments[2].script.iso15924, "Latn");
}

/// 没有前置具体脚本的嵌套开括号，可由后续首个具体脚本补齐待定后缀。
#[test]
fn leading_and_nested_brackets_inherit_the_first_resolved_script() {
    let text = "([مرحبا])";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Arab");
}

/// 两个切罗基标量得到相同 ISO 身份；这里核对查询键，不读取字体覆盖。
#[test]
fn fallback_script_identity_uses_iso15924_instead_of_the_cluster_codepoint() {
    let first = "\u{13a0}";
    let second = "\u{13a1}";
    let first_analysis = ParagraphTextAnalysis::new(first, None);
    let second_analysis = ParagraphTextAnalysis::new(second, None);
    let expected = FontScript::Other(
        FontScriptTag::from_bytes(*b"Cher").expect("canonical Cherokee script tag"),
    );

    assert_eq!(
        first_analysis.font_script_for_range(TextRange {
            start: 0,
            end: first.len(),
        }),
        expected
    );
    assert_eq!(
        second_analysis.font_script_for_range(TextRange {
            start: 0,
            end: second.len(),
        }),
        expected
    );
}

/// 未分配码点保留带类型的 Unknown 查询键，避免混成通用脚本。
#[test]
fn unassigned_codepoints_keep_a_typed_unknown_fallback_script() {
    let cluster = "\u{0378}";
    let analysis = ParagraphTextAnalysis::new(cluster, None);

    assert_eq!(
        analysis.font_script_for_range(TextRange {
            start: 0,
            end: cluster.len(),
        }),
        FontScript::Unknown
    );
}

/// 对比默认属性、文本/emoji 选择符与未分配标量，按原文字节范围查询覆盖结果。
#[test]
fn paragraph_emoji_presentation_uses_unicode_properties_and_selectors() {
    let text = "A☀☀\u{fe0f}😀\u{fe0e}\u{1f02c}";
    let analysis = ParagraphTextAnalysis::new(text, None);
    let latin = script_segments("A", None)[0].script;
    let sun_start = "A".len();
    let emoji_sun_start = sun_start + "☀".len();
    let text_face_start = emoji_sun_start + "☀\u{fe0f}".len();
    let unassigned_start = text_face_start + "😀\u{fe0e}".len();

    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: sun_start,
            end: emoji_sun_start,
        }),
        latin
    );
    assert_eq!(
        analysis
            .shaped_script_for_range(TextRange {
                start: emoji_sun_start,
                end: text_face_start,
            })
            .iso15924,
        "Zsye"
    );
    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: text_face_start,
            end: unassigned_start,
        }),
        latin
    );
    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: unassigned_start,
            end: text.len(),
        }),
        latin
    );
}

/// 无 emoji 选择符的键帽相邻模式仍应得到 emoji 呈现身份。
#[test]
fn paragraph_emoji_presentation_recognizes_keycaps_without_private_ranges() {
    let text = "A1\u{20e3}B";
    let analysis = ParagraphTextAnalysis::new(text, None);
    let keycap_start = "A".len();
    let keycap_end = keycap_start + "1\u{20e3}".len();

    assert_eq!(
        analysis
            .shaped_script_for_range(TextRange {
                start: keycap_start,
                end: keycap_end,
            })
            .iso15924,
        "Zsye"
    );
}

/// 相邻 emoji 区间合并，隔着拉丁字符的后续命中仍保留独立区间。
#[test]
fn paragraph_analysis_merges_adjacent_emoji_presentation_ranges() {
    let analysis = ParagraphTextAnalysis::new("\u{1f600}\u{1f601}A\u{1f602}", None);

    assert_eq!(analysis.emoji_presentation_ranges.len(), 2);
    assert_eq!(
        analysis.emoji_presentation_ranges[0],
        TextRange { start: 0, end: 8 }
    );
    assert_eq!(
        analysis.emoji_presentation_ranges[1],
        TextRange { start: 9, end: 13 }
    );
}

/// 改变测试用 generation 后只核对身份原样保留，不据此证明数据表切换。
#[test]
fn paragraph_analysis_retains_request_unicode_snapshot_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);
    let analysis = ParagraphTextAnalysis::for_snapshot("text", None, next);

    assert_eq!(analysis.unicode_data_snapshot(), next);
}

/// 空字节区间不因位于 emoji 内部而取得 emoji 呈现覆盖。
#[test]
fn empty_source_range_never_acquires_emoji_presentation() {
    let analysis = ParagraphTextAnalysis::new("\u{1f600}", None);

    assert_ne!(
        analysis
            .shaped_script_for_range(TextRange { start: 1, end: 1 })
            .iso15924,
        "Zsye"
    );
}

/// 对包含注释的源码去空白后检查固定禁用片段；这是拼写约束，不能证明复杂度或运行时性能。
#[test]
fn script_range_lookup_does_not_restore_linear_find() {
    let source = include_str!("../script_segment.rs");
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    assert!(!compact.contains(concat!("segments.iter()", ".find")));
}
