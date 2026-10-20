//! 汇集位图页阴影的补丁与整页初始化标记，供后续存储端接纳。
//! 本模块只拥有记录和字节引用，不执行纹理上传，也不核验当前驻留页或世代。
//! 记录由 bitmap_run::staged_upload 生成，经帧接受入口交给页阴影存储；值的合并不代表 GPU 完成。

use std::collections::BTreeSet;

use super::super::GlyphAtlasPageKey;
use super::GlyphAtlasBitmapPageShadowPatch;

/// 作为拥有所有权的值传递的一批页阴影更新。
///
/// 补丁保留向量顺序，两个页键集合各自去重；同一页可同时出现在两个集合中，
/// 失败优先的筛选留给存储端。派生克隆会复制集合，补丁像素仍通过 Arc 共享。
/// 本值不持有页面或 GPU 租约；接纳时仍需检查驻留页、补丁世代和可用预算。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowCommit {
    /// 顺序保存的区域补丁；本层不检查内容、尺寸或页世代。
    pub(crate) patches: Vec<GlyphAtlasBitmapPageShadowPatch>,
    /// 生产者标记可尝试建立全零阴影的页键，能否建立由存储端决定。
    pub(crate) zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
    /// 需排除全零建立的页键；合并时保留，交由消费者过滤成功标记。
    pub(crate) failed_zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
}

impl GlyphAtlasBitmapPageShadowCommit {
    /// 消费另一份记录，将其集合合入当前值，不应用其中的像素。
    ///
    /// # 参数
    /// - `self`：调用期间独占借用的接收方，返回后持有合并结果。
    /// - `other`：按值消费的输入，补丁及页键移入接收方，不深拷贝 Arc 字节。
    ///
    /// # 返回与约束
    /// 返回 `()`。补丁保持目标在前、输入在后的顺序，两个页键集合分别合并去重。
    /// 不消解两集合之间的冲突；参见 `GlyphAtlasBitmapPageShadowStore::apply` 的失败过滤和世代准入。
    pub(crate) fn extend(&mut self, other: Self) {
        let Self {
            patches,
            mut zero_initialized_pages,
            mut failed_zero_initialized_pages,
        } = other;
        // 两类初始化标记分别保留，避免合并时丢失消费者需要的失败信息。
        append_or_adopt_owned_vec(&mut self.patches, patches);
        self.zero_initialized_pages
            .append(&mut zero_initialized_pages);
        self.failed_zero_initialized_pages
            .append(&mut failed_zero_initialized_pages);
    }
}

/// 将输入元素移入目标；空且零容量时直接接管输入向量。
///
/// # 泛型与参数
/// - `T`：不要求 Clone 或 Copy，元素通过所有权转移合并。
/// - `target`：调用期间独占借用的目标；容量以元素数计。
/// - `source`：按值消费的输入，调用后其元素归目标所有。
///
/// # 返回与约束
/// 返回 `()`。仅空且零容量的目标直接接管；其余情况追加，容量不足仍可重新分配。
/// 该分支保护已有预留容量，不承诺整个提交过程零分配。
fn append_or_adopt_owned_vec<T>(target: &mut Vec<T>, mut source: Vec<T>) {
    // 空目标可直接接管输入缓冲；已有预留容量时继续追加，以复用目标的分配。
    if target.is_empty() && target.capacity() == 0 {
        *target = source;
        return;
    }
    target.append(&mut source);
}

#[cfg(test)]
#[path = "tests/commit.rs"]
mod tests;
