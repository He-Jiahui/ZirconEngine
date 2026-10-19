//! 核对机会表的范围投影、来源回执及请求身份保留。
//!
//! 人工机会表隔离边界规则；源码拼写检查单独读取主文件全文，不能证明运行时性能。

use super::{ClusterLineBreakFlags, LineBreakKind, LineBreakOpportunity, LineBreakOpportunityMap};
use crate::text::{
    compiled_unicode_data_snapshot_id, LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity,
    ShapedGlyphLineBreakReceipt,
};

/// 构造带已分析规则的期望值，使“没有机会”与未知默认回执可在断言中区分。
const fn unicode_default_receipt(
    opportunity: ShapedGlyphLineBreakOpportunity,
) -> ShapedGlyphLineBreakReceipt {
    ShapedGlyphLineBreakReceipt {
        profile: LineBreakTailoringProfile::UnicodeDefault,
        opportunity,
    }
}

// 用排序后的人工机会分别覆盖软末端、强制末端和跳过前段；不依赖实际文本提供方分词。
#[test]
fn cluster_flags_only_visit_the_cluster_opportunity_window() {
    let map = LineBreakOpportunityMap {
        opportunities: vec![
            LineBreakOpportunity {
                byte_index: 2,
                kind: LineBreakKind::Soft,
            },
            LineBreakOpportunity {
                byte_index: 4,
                kind: LineBreakKind::Mandatory,
            },
            LineBreakOpportunity {
                byte_index: 8,
                kind: LineBreakKind::Soft,
            },
        ],
        unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
        tailoring_profile: LineBreakTailoringProfile::UnicodeDefault,
    };

    assert_eq!(
        map.flags_for_cluster(0, 2),
        ClusterLineBreakFlags {
            soft_break: true,
            mandatory_break: false,
            receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderAllowed),
        }
    );
    assert_eq!(
        map.flags_for_cluster(2, 4),
        ClusterLineBreakFlags {
            soft_break: false,
            mandatory_break: true,
            receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderMandatory,),
        }
    );
    assert_eq!(
        map.flags_for_cluster(5, 8),
        ClusterLineBreakFlags {
            soft_break: true,
            mandatory_break: false,
            receipt: unicode_default_receipt(ShapedGlyphLineBreakOpportunity::ProviderAllowed),
        }
    );
}

// 无机会的范围仍来自一次有效分析，回执应保留规则身份，而非退回未知默认值。
#[test]
fn cluster_without_an_opportunity_still_records_the_analysis_profile() {
    let map = LineBreakOpportunityMap::new("alpha beta");

    assert_eq!(
        map.flags_for_cluster(0, 1).receipt,
        unicode_default_receipt(ShapedGlyphLineBreakOpportunity::None)
    );
}

// 全文读取包含注释，随后删除所有空白；这里只约束一种被禁用的源码拼写，
// 没有测量查询耗时、分配或实际访问次数。
#[test]
fn cluster_flags_do_not_restore_a_full_opportunity_fold() {
    let source = include_str!("../line_break.rs");
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    assert!(!compact.contains(concat!("self.opportunities.iter()", ".fold")));
}

// 改变测试身份而保持相同文本，只核对传入身份被保留，不据此推断运行时数据表切换。
#[test]
fn line_break_analysis_retains_request_unicode_snapshot() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);
    let map = LineBreakOpportunityMap::for_snapshot("alpha beta", next);

    assert_eq!(map.unicode_data_snapshot(), next);
}
