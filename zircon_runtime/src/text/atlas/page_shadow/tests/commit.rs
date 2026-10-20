//! 核对拥有式集合合并、特定缓冲接管分支和实现的源码形态。
//! 夹具只构造内存记录，不创建 GPU 页面；被忽略的性能夹具需要显式选择才运行。

use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasRect};

/// 用非空输入和默认目标核对缓冲接管及两类页键的保留。
/// 指针与容量断言只约束此夹具，不能外推整条图集流程无分配。
#[test]
fn optimization_batch_ds_shadow_commit_adopts_owned_patch_storage() {
    let mut incoming = GlyphAtlasBitmapPageShadowCommit::default();
    incoming.patches = vec![patch(3), patch(4)];
    incoming
        .zero_initialized_pages
        .insert(GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3));
    incoming
        .failed_zero_initialized_pages
        .insert(GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 4));
    let incoming_pointer = incoming.patches.as_ptr();
    let incoming_capacity = incoming.patches.capacity();

    let mut target = GlyphAtlasBitmapPageShadowCommit::default();
    target.extend(incoming);

    assert_eq!(target.patches.len(), 2);
    assert_eq!(target.patches.as_ptr(), incoming_pointer);
    assert_eq!(target.patches.capacity(), incoming_capacity);
    assert!(target
        .zero_initialized_pages
        .contains(&GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3)));
    assert!(target
        .failed_zero_initialized_pages
        .contains(&GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 4)));
}

/// 要求拥有式合并锚点存在，并拒绝旧的逐元素扩展写法。
/// 只读测试挂载前的原文，注释也参与匹配，不能借新增文字满足守卫。
#[test]
fn optimization_batch_ds_shadow_commit_uses_owned_collection_merges() {
    let production = include_str!("../commit.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("shadow commit production source");

    assert!(production.contains("append_or_adopt_owned_vec(&mut self.patches, patches)"));
    assert!(production.contains(".append(&mut zero_initialized_pages)"));
    assert!(production.contains(".append(&mut failed_zero_initialized_pages)"));
    assert!(!production.contains("self.patches.extend(other.patches)"));
}

/// 交替采样两条向量合并路径，以纳秒样本比较本批次的 p95。
/// 计时包含每轮输入构造与合并，不涵盖补丁结构、页键集合或 GPU 上传。
#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_ds_shadow_commit_owned_patch_storage_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const MERGES_PER_SAMPLE: usize = 2_048;
    const PATCHES_PER_MERGE: usize = 4_096;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_owned_merges(
                MERGES_PER_SAMPLE,
                PATCHES_PER_MERGE,
                false,
            ));
            optimized_samples.push(measure_owned_merges(
                MERGES_PER_SAMPLE,
                PATCHES_PER_MERGE,
                true,
            ));
        } else {
            optimized_samples.push(measure_owned_merges(
                MERGES_PER_SAMPLE,
                PATCHES_PER_MERGE,
                true,
            ));
            legacy_samples.push(measure_owned_merges(
                MERGES_PER_SAMPLE,
                PATCHES_PER_MERGE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME427_SHADOW_COMMIT_OWNED_PATCH_STORAGE_BENCH_V1 merges_per_sample={MERGES_PER_SAMPLE} patches_per_merge={PATCHES_PER_MERGE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "shadow commit owned patch storage p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );

    /// 重复构造 u64 输入并移入空目标，返回总耗时，单位为纳秒。
    ///
    /// # 参数
    /// - `merge_count`：本次计时内重复构造和合并的次数。
    /// - `patch_count`：每轮向量的元素数，此处元素不是实际补丁结构。
    /// - `optimized`：选择缓冲接管 helper；为 false 时走向量扩展对照。
    ///
    /// # 约束
    /// 每轮重新创建目标，校验和仅用于阻止工作被消除，不验证像素或页世代。
    fn measure_owned_merges(merge_count: usize, patch_count: usize, optimized: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for merge_index in 0..merge_count {
            let source = vec![merge_index as u64; patch_count];
            let mut target = Vec::new();
            if optimized {
                super::append_or_adopt_owned_vec(&mut target, source);
            } else {
                target.extend(source);
            }
            checksum = checksum.wrapping_add(target.len() ^ target.capacity());
            black_box(&target);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    /// 原地排序非空纳秒样本，返回向上取整的 95% 秩位置值。
    /// `samples` 的顺序会改变且不能为空；本夹具的两个调用均传入固定 17 项。
    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}

/// 构造世代为 1、内容为单像素的 AlphaMask 补丁值，不申请实际页面。
/// `page_index` 原值用于页键，示例像素取其低 8 位；这里不核验图集状态。
fn patch(page_index: u32) -> GlyphAtlasBitmapPageShadowPatch {
    GlyphAtlasBitmapPageShadowPatch {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, page_index),
        page_generation: 1,
        target_rect: GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        bytes_per_row: 1,
        bytes: vec![page_index as u8].into(),
    }
}
