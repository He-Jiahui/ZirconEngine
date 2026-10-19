//! 段落脚本与 emoji 呈现分析，供成形分段和后备字体查询共用。
//! Cosmic 后端与文本服务构建分析后复用查询；结果拥有区间和可复制的快照标识，
//! 不保留原文借用。区间均使用构建时原文的半开 UTF-8 字节坐标。
//! 快照标识用于后续核对；属性查询仍使用编译期提供方，这里不切换 Unicode 表。

use crate::text::TextRange;
use unicode_bidi::{BidiDataSource, HardcodedBidiData};
use unicode_script::{Script, ScriptExtension, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::language::{text_language_script_subtag, TextLanguageScriptSubtag};
use crate::text::{compiled_unicode_data_snapshot_id, UnicodeDataSnapshotId};
use crate::text::{FontScript, Iso15924Tag, ShapedGlyphScript};

use super::emoji_presentation::cluster_uses_emoji_presentation;

const EMOJI_SCRIPT_TAG: Iso15924Tag = Iso15924Tag::EMOJI;

/// 同一脚本身份对应的原文字节区间，由分段器按源顺序生成。
/// 相邻同脚本区间会合并；这两个值不持有原文或字体资源。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScriptSegment {
    /// 相对于分析输入的半开 UTF-8 字节范围。
    pub range: TextRange,
    /// 本段解析出的脚本身份；字体覆盖由后续字体解析处理。
    pub script: ShapedGlyphScript,
}

/// 一次原文分析的自有结果，供同一文本的后续成形步骤反复查询。
/// 构建结束后只保留区间与标识；调用方须保持查询坐标与该输入一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParagraphTextAnalysis {
    /// 按原文顺序排列并合并相邻同脚本区间的结果。
    scripts: Vec<ScriptSegment>,
    /// 按字素簇判定并合并相邻命中的 emoji 呈现区间。
    emoji_presentation_ranges: Vec<TextRange>,
    /// 构建时传入的快照身份副本，不是可替换的数据表句柄。
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

impl ParagraphTextAnalysis {
    /// 使用编译期快照标识构建段落分析。
    ///
    /// # 参数
    /// - `text`：本次借用的原文；返回区间以它的 UTF-8 字节位置为坐标。
    /// - `language`：可选语言标签，只提取显式脚本子标签作为歧义候选的偏好。
    ///
    /// # 返回与参考
    /// 返回不借用输入的分析值；构建及快照边界见 [Self::for_snapshot]。
    pub(crate) fn new(text: &str, language: Option<&str>) -> Self {
        Self::for_snapshot(
            text,
            text_language_script_subtag(language),
            compiled_unicode_data_snapshot_id(),
        )
    }

    /// 在请求绑定的快照身份下，建立脚本分段和 emoji 呈现区间。
    ///
    /// # 参数
    /// - `text`：仅在构建期间借用的原文，所有区间均保留其 UTF-8 字节坐标。
    /// - `explicit_language_script`：已解析的显式脚本子标签，不根据语言补全脚本。
    /// - `unicode_data_snapshot`：按值保留的身份副本，供后续请求一致性核对。
    ///
    /// # 返回与约束
    /// 返回自有集合及标识；传入身份不会切换属性查询使用的数据表。
    /// 观测入口按 test/feature 编译；实际累计仍取决于请求级观测状态。
    /// 分析结果的生成不依赖观测是否活动。
    pub(crate) fn for_snapshot(
        text: &str,
        explicit_language_script: Option<TextLanguageScriptSubtag>,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        let profile_started = super::analysis_profile::start_build();
        let analysis = Self {
            scripts: script_segments_with_preferred_script(
                text,
                unicode_script_for_language(explicit_language_script),
            ),
            emoji_presentation_ranges: emoji_presentation_ranges(text),
            unicode_data_snapshot,
        };
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        super::analysis_profile::record_script_emoji_build(text.len(), profile_started);
        analysis
    }

    /// 返回构建时保留的快照身份副本，供请求一致性核对。
    /// `self` 仅作只读借用；返回标识不选择属性数据表。
    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    /// 查询原脚本身份，emoji 呈现覆盖由 [Self::shaped_script_for_range] 处理。
    ///
    /// # 参数与返回
    /// `self` 只读借用，`range` 使用分析输入的半开 UTF-8 字节坐标。
    /// 返回区间查询的身份副本；中点、起点回退规则见本模块的区间查询函数。
    pub(crate) fn script_for_range(&self, range: TextRange) -> ShapedGlyphScript {
        script_for_range(&self.scripts, range)
    }

    /// 在原脚本查询上叠加 emoji 呈现身份，供字形分段使用。
    ///
    /// # 参数与返回
    /// `self` 只读借用，`range` 为分析输入的 UTF-8 字节区间。
    /// 非空查询与任一 emoji 呈现区间重叠即返回 emoji 标签，否则查询原脚本。
    /// 这里不要求整个查询区间都被同一呈现区间包含，也不验证整个区间。
    pub(crate) fn shaped_script_for_range(&self, range: TextRange) -> ShapedGlyphScript {
        if range_overlaps_any(&self.emoji_presentation_ranges, range) {
            ShapedGlyphScript {
                iso15924: EMOJI_SCRIPT_TAG,
            }
        } else {
            self.script_for_range(range)
        }
    }

    /// 将成形脚本身份转换为后备字体解析使用的查询键。
    ///
    /// # 参数与返回
    /// `self` 只读借用；`range` 使用分析输入的半开 UTF-8 字节坐标。
    /// 返回包含 emoji 呈现覆盖的 [FontScript] 值，身份本身不保证字体覆盖。
    pub(crate) fn font_script_for_range(&self, range: TextRange) -> FontScript {
        FontScript::from_iso15924_tag(self.shaped_script_for_range(range).iso15924.as_str())
    }
}

/// 单次扫描中尚未配对的开括号上下文；随本次分段调用一起释放。
/// 没有脚本的待定项形成栈后缀，后续具体脚本可统一补齐该后缀。
#[derive(Clone, Copy)]
struct PairedBracketContext {
    /// 提供方归一化的开括号字符，用于反向寻找配对项。
    opening: char,
    /// 开括号所处的具体脚本；暂未确定时等待后续上下文。
    script: Option<Script>,
}

/// 按原文标量的脚本扩展候选生成有序分段。
///
/// # 参数
/// - `text`：仅在扫描期间借用的原文，区间单位为 UTF-8 字节。
/// - `language`：可选语言标签；只使用其中显式的脚本子标签作兼容候选偏好。
///
/// # 返回
/// 返回自有区间与身份；空文本返回空集合，不保留输入借用。
pub(crate) fn script_segments(text: &str, language: Option<&str>) -> Vec<ScriptSegment> {
    script_segments_with_preferred_script(text, preferred_script_for_language(language))
}

/// 扫描候选交集和括号上下文，在交集不兼容处结束当前段。
///
/// # 参数与返回
/// `text` 仅在本次扫描借用；`preferred_script` 是按值传入的可选偏好，
/// 只有包含在候选中时才参与选择。返回按 UTF-8 字节位置排列的自有分段。
/// 括号规则只提供本算法的脚本上下文，不承诺完整 Unicode 括号规范实现。
fn script_segments_with_preferred_script(
    text: &str,
    preferred_script: Option<Script>,
) -> Vec<ScriptSegment> {
    // 当前候选、主脚本和前一具体脚本共同决定段界；括号栈仅在本次扫描存在。
    let mut segments = Vec::new();
    let mut current_start = 0;
    let mut current_candidates = ScriptExtension::default();
    let mut current_primary = None;
    let mut previous_script = None;
    let mut has_current = false;
    let mut bracket_stack = Vec::<PairedBracketContext>::new();
    let mut unresolved_bracket_start = None;

    for (start, ch) in text.char_indices() {
        let primary = script_for_char(ch);
        let mut extensions = script_extension_for_char(ch);
        let paired_bracket = is_common_like(primary)
            .then(|| HardcodedBidiData.bidi_matched_opening_bracket(ch))
            .flatten();
        let mut matched_bracket_index = None;

        // 开括号保存当时上下文；没有具体脚本时从首个待定栈项开始跟踪后缀。
        if let Some(bracket) = paired_bracket {
            if bracket.is_open {
                let script = contextual_script(
                    current_candidates,
                    current_primary,
                    preferred_script,
                    previous_script,
                );
                if script.is_none() && unresolved_bracket_start.is_none() {
                    unresolved_bracket_start = Some(bracket_stack.len());
                }
                bracket_stack.push(PairedBracketContext {
                    opening: bracket.opening,
                    script,
                });
            } else if let Some(index) = bracket_stack
                .iter()
                .rposition(|entry| entry.opening == bracket.opening)
            {
                let opening_script = bracket_stack[index].script.or_else(|| {
                    contextual_script(
                        current_candidates,
                        current_primary,
                        preferred_script,
                        previous_script,
                    )
                });
                if let Some(opening_script) = opening_script {
                    extensions = opening_script.into();
                }
                matched_bracket_index = Some(index);
            } else {
                // 未找到匹配开括号时丢弃旧上下文，避免把失配栈带入后续字符。
                bracket_stack.clear();
                unresolved_bracket_start = None;
            }
        }

        // 先扩展当前兼容候选；不兼容时先提交旧段，再用当前字符启动新段。
        if !has_current {
            current_start = start;
            current_candidates = extensions;
            current_primary = specific_script(primary);
            has_current = true;
        } else {
            match compatible_intersection(current_candidates, extensions) {
                Some(intersection) => {
                    current_candidates = intersection;
                    if current_primary.is_none()
                        && specific_script(primary)
                            .is_some_and(|script| intersection.contains_script(script))
                    {
                        current_primary = specific_script(primary);
                    }
                }
                None => {
                    let resolved = resolve_script(
                        current_candidates,
                        current_primary,
                        preferred_script,
                        previous_script,
                    );
                    push_segment(&mut segments, current_start, start, resolved);
                    resolve_pending_brackets(
                        &mut bracket_stack,
                        &mut unresolved_bracket_start,
                        resolved,
                    );
                    previous_script = specific_script(resolved);
                    current_start = start;
                    current_candidates = extensions;
                    current_primary = specific_script(primary)
                        .filter(|script| extensions.contains_script(*script));
                }
            }
        }

        // 一旦得到具体上下文，补齐仍待定的开括号，供未来闭括号恢复。
        if let Some(resolved) = contextual_script(
            current_candidates,
            current_primary,
            preferred_script,
            previous_script,
        ) {
            resolve_pending_brackets(&mut bracket_stack, &mut unresolved_bracket_start, resolved);
        }
        // 配对闭合后移除匹配开括号及其之后的项，待定游标也须同步收缩。
        if let Some(index) = matched_bracket_index {
            truncate_bracket_stack(&mut bracket_stack, &mut unresolved_bracket_start, index);
        }
    }

    // 扫描结束只提交已启动的尾段，因此空输入保持空结果。
    if has_current {
        let resolved = resolve_script(
            current_candidates,
            current_primary,
            preferred_script,
            previous_script,
        );
        push_segment(&mut segments, current_start, text.len(), resolved);
    }

    segments
}

/// 按扩展字素簇判定 emoji 呈现，并合并相邻命中的区间。
///
/// # 参数与返回
/// `text` 只在本次调用借用；返回自有的半开 UTF-8 字节区间。
/// 相邻命中可以合并，未命中的簇保持间隔；判定依据见
/// [cluster_uses_emoji_presentation]，不据此保证字体覆盖或实际字形输出。
fn emoji_presentation_ranges(text: &str) -> Vec<TextRange> {
    let mut ranges = Vec::<TextRange>::new();
    for (start, cluster) in text.grapheme_indices(true) {
        if !cluster_uses_emoji_presentation(cluster) {
            continue;
        }
        let end = start + cluster.len();
        if let Some(previous) = ranges.last_mut().filter(|range| range.end == start) {
            previous.end = end;
        } else {
            ranges.push(TextRange { start, end });
        }
    }
    ranges
}

/// 在有序且不重叠的区间集中判断是否存在非空交集。
///
/// # 参数与返回
/// `ranges` 是构建器持有的只读字节区间；`range` 与其使用同一原文坐标。
/// 起点不小于终点时返回 `false`，否则查首个结束位置越过查询起点的候选。
/// 命中只表示有重叠，不要求候选包含整个查询区间。
fn range_overlaps_any(ranges: &[TextRange], range: TextRange) -> bool {
    if range.start >= range.end {
        return false;
    }
    ranges
        .get(ranges.partition_point(|candidate| candidate.end <= range.start))
        .is_some_and(|candidate| candidate.start < range.end)
}

/// 从有序脚本分段中选取查询区间的代表脚本。
///
/// # 参数
/// - `segments`：按源位置排列的只读分段，使用调用方同一文本的坐标。
/// - `range`：半开 UTF-8 字节区间；本函数不验证整段边界或脚本一致性。
///
/// # 返回
/// 优先选中点所在段，未命中则按起点回退，仍无候选时返回默认身份。
/// 返回身份副本，不借用分段；空区间也不在此单独拒绝。
pub(crate) fn script_for_range(segments: &[ScriptSegment], range: TextRange) -> ShapedGlyphScript {
    let midpoint = range.start + range.end.saturating_sub(range.start) / 2;
    segments
        .get(segments.partition_point(|segment| segment.range.end <= midpoint))
        .filter(|segment| midpoint >= segment.range.start && midpoint < segment.range.end)
        .or_else(|| {
            segments
                .get(segments.partition_point(|segment| segment.range.end <= range.start))
                .filter(|segment| range.start < segment.range.end)
        })
        .map(|segment| segment.script)
        .unwrap_or_default()
}

/// 追加非空段，并尽量与输出尾部的相邻同脚本段合并。
///
/// # 参数与状态变化
/// `segments` 由调用方独占借用；`start`、`end` 为原文半开 UTF-8 字节位置，
/// `script` 为已选身份。空区间不改输出；其余情况延长尾段或追加自有段值。
fn push_segment(segments: &mut Vec<ScriptSegment>, start: usize, end: usize, script: Script) {
    if start >= end {
        return;
    }
    let script = shaped_script(script);
    if let Some(last) = segments.last_mut() {
        if last.script == script && last.range.end == start {
            last.range.end = end;
            return;
        }
    }
    segments.push(ScriptSegment {
        range: TextRange { start, end },
        script,
    });
}

/// 保留两组候选的兼容交集，决定扫描是否继续当前段。
///
/// # 参数与返回
/// `left`、`right` 均按值传入；非空交集返回 `Some`。
/// 双方都为空时仍视为兼容，其余空交集返回 `None` 并触发上层分段。
fn compatible_intersection(
    left: ScriptExtension,
    right: ScriptExtension,
) -> Option<ScriptExtension> {
    let intersection = left.intersection(right);
    if !intersection.is_empty() {
        Some(intersection)
    } else if left.is_empty() && right.is_empty() {
        Some(left)
    } else {
        None
    }
}

/// 将 Common、Inherited、Unknown 统一视为尚不能充当具体上下文的身份。
fn is_common_like(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}

/// 通过 `UnicodeScript` 的 `char` 实现查询编译期主脚本属性。
fn script_for_char(ch: char) -> Script {
    ch.script()
}

/// 通过同一 `UnicodeScript` 实现取得标量的脚本扩展候选。
fn script_extension_for_char(ch: char) -> ScriptExtension {
    ch.script_extension()
}

/// 将按值传入的身份筛为具体脚本，通用、继承与未知身份返回 `None`。
fn specific_script(script: Script) -> Option<Script> {
    (!is_common_like(script)).then_some(script)
}

/// 在候选内选择可确定的具体上下文，供分段与括号继承共用。
///
/// # 参数
/// `candidates`、`primary`、`preferred`、`previous` 均按值传入，分别是当前
/// 候选、当前主脚本、显式语言偏好和前一具体脚本；它们都不是字体覆盖信息。
///
/// # 返回与顺序
/// 单个具体候选优先；多候选依次尝试兼容的前一脚本、偏好、主脚本。
/// 空集及 Common/Inherited 候选返回 `None`，最终兜底由 [resolve_script] 完成。
fn contextual_script(
    candidates: ScriptExtension,
    primary: Option<Script>,
    preferred: Option<Script>,
    previous: Option<Script>,
) -> Option<Script> {
    if candidates.is_empty() || candidates.is_common() || candidates.is_inherited() {
        return None;
    }
    if candidates.len() == 1 {
        return candidates.iter().next().and_then(specific_script);
    }
    previous
        .filter(|script| candidates.contains_script(*script))
        .or_else(|| preferred.filter(|script| candidates.contains_script(*script)))
        .or_else(|| primary.filter(|script| candidates.contains_script(*script)))
}

/// 为即将提交的段取得最终脚本，包括无法形成具体上下文时的兜底。
///
/// # 参数与返回
/// 四个按值参数沿用 [contextual_script] 的含义与优先级。
/// 未选出具体上下文时保留 Unknown/Inherited/Common；其他候选按迭代顺序
/// 取首个具体脚本，仍无结果则为 Unknown，不据此选择字体。
fn resolve_script(
    candidates: ScriptExtension,
    primary: Option<Script>,
    preferred: Option<Script>,
    previous: Option<Script>,
) -> Script {
    contextual_script(candidates, primary, preferred, previous).unwrap_or_else(|| {
        if candidates.is_empty() {
            Script::Unknown
        } else if candidates.is_inherited() {
            Script::Inherited
        } else if candidates.is_common() {
            Script::Common
        } else {
            candidates
                .iter()
                .find_map(specific_script)
                .unwrap_or(Script::Unknown)
        }
    })
}

/// 将已经确定的具体脚本写入待定开括号后缀。
///
/// # 参数与状态变化
/// - `stack`：仅在本次扫描中独占借用的括号条目切片。
/// - `unresolved_start`：首个待定条目的索引，单位是栈项而非原文字节。
/// - `script`：按值传入的决议；非具体脚本不会消费待定游标。
///
/// 游标存在时先取走它，再补齐后缀；后缀尚未赋值是调用方维护的不变量，
/// 调试断言只在调试配置检查该前提。
fn resolve_pending_brackets(
    stack: &mut [PairedBracketContext],
    unresolved_start: &mut Option<usize>,
    script: Script,
) {
    let Some(script) = specific_script(script) else {
        return;
    };
    let Some(start) = unresolved_start.take() else {
        return;
    };
    for entry in &mut stack[start..] {
        debug_assert!(entry.script.is_none());
        entry.script = Some(script);
    }
}

/// 收缩配对后的括号上下文，并清除落在被移除后缀中的待定游标。
///
/// # 参数与状态变化
/// `stack` 与 `unresolved_start` 均独占借用；`len` 是保留的前缀项数。
/// 匹配闭括号时传入开括号索引，因此该开括号及其之后的栈项一起移除。
fn truncate_bracket_stack(
    stack: &mut Vec<PairedBracketContext>,
    unresolved_start: &mut Option<usize>,
    len: usize,
) {
    stack.truncate(len);
    if unresolved_start.is_some_and(|start| start >= len) {
        *unresolved_start = None;
    }
}

/// 从借用的语言标签提取显式脚本偏好；缺失或无法解析时不提供偏好。
fn preferred_script_for_language(language: Option<&str>) -> Option<Script> {
    unicode_script_for_language(text_language_script_subtag(language))
}

/// 将按值传入的显式子标签映射为具体脚本；无映射或非具体身份返回 `None`。
fn unicode_script_for_language(script: Option<TextLanguageScriptSubtag>) -> Option<Script> {
    script
        .and_then(|script| script.as_str().and_then(Script::from_short_name))
        .and_then(specific_script)
}

/// 将按值传入的脚本转换为成形产物保留的四字母身份。
///
/// # 返回
/// 返回不借用输入的 [ShapedGlyphScript]；短名不能形成标签时回退 Common。
fn shaped_script(script: Script) -> ShapedGlyphScript {
    ShapedGlyphScript {
        iso15924: match Iso15924Tag::parse(script.short_name()) {
            Some(tag) => tag,
            None => Iso15924Tag::COMMON,
        },
    }
}

#[cfg(test)]
#[path = "tests/script_segment.rs"]
mod tests;
