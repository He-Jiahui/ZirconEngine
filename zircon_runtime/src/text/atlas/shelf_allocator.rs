//! 为单页内容分配有间隔的矩形，供持久槽缓存复用同一页面的空间。
//! 不单独回收矩形；页重建时整体重置，失败分配不得推进游标以免挤掉后续可容纳字形。
//! 分配器只保存单页的像素配置与行游标，不拥有页面、像素或 GPU 租约。
//! 失败保留的是本分配器状态；外层可能已预约页面，不表示整个图集操作回滚。

use crate::core::math::UVec2;

use super::{GlyphAtlasPageKey, GlyphAtlasRect};

/// 成功分配后得到的页键与内容矩形值快照。
///
/// 可独立复制，不持有页面租约；使用前的页世代核验仍由调用方负责。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasAllocation {
    /// 归属页面的值标识，不包含页面资源或世代。
    pub(crate) page_key: GlyphAtlasPageKey,
    /// 内容占用的像素矩形，宽高不含后置间隔。
    pub(crate) rect: GlyphAtlasRect,
}

/// 固定单页配置及按行推进的分配游标。
///
/// 水平放不下时尝试下一行；行高取该行已分配矩形的最大高度。
/// 克隆会复制配置和当前位置，副本独立推进，不保证它们占用的页面区域互斥。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasShelfAllocator {
    /// 固定页标识；不随行游标推进而改变。
    page_key: GlyphAtlasPageKey,
    /// 内容可使用的像素宽高；构造时不归一或校验。
    page_size: UVec2,
    /// 相邻内容和行之间的间隔，不要求本次矩形的尾部间隔仍落在页内。
    padding_px: u32,
    /// 下一次横向尝试位置；后置间隔溢出时保存 `u32::MAX` 哨兵。
    cursor_x: u32,
    /// 当前行顶端的像素纵坐标。
    cursor_y: u32,
    /// 当前行成功分配矩形的最大高度，供下一次换行使用。
    shelf_height: u32,
}

impl GlyphAtlasShelfAllocator {
    /// 保存页面配置并从原点开始分配，不创建页面或验证页世代。
    ///
    /// # 参数
    /// - `page_key`：分配结果中携带的页面值标识。
    /// - `page_size`：单页像素宽高；此处不拒绝零值或归一尺寸。
    /// - `padding_px`：相邻内容与行之间追加的像素间隔。
    ///
    /// # 返回与使用时机
    /// 返回三个游标状态均为零的新分配器；实际尺寸准入发生在 `allocate`。
    /// 槽缓存复用前还会以 `matches_configuration` 比较固定配置。
    pub(crate) fn new(page_key: GlyphAtlasPageKey, page_size: UVec2, padding_px: u32) -> Self {
        Self {
            page_key,
            page_size,
            padding_px,
            cursor_x: 0,
            cursor_y: 0,
            shelf_height: 0,
        }
    }

    /// 尝试分配不含间隔的内容矩形，成功后才推进本分配器的游标。
    ///
    /// # 参数
    /// - `size`：内容像素宽高；任一为零或超过页面对应边长时返回 `None`。
    ///
    /// # 返回
    /// 成功值拥有页键和矩形，不借用分配器；`None` 表示内容不适配或行坐标受检加法失败。
    /// 所有失败路径都在三个状态字段提交前返回，保留本分配器的当前位置。
    ///
    /// # 约束与调用时机
    /// 只约束内容矩形落在页内，不保证四周均留间隔。成功后的水平间隔加法溢出时，
    /// 仍返回本次矩形，并以 `u32::MAX` 保存下一横坐标，供后续分配尝试换行。
    /// 这不承诺外层操作回滚：位图或渲染调用方可能已预约页面，离线打包也可能在后续拷贝失败。
    /// 参见 `slot_cache::GlyphAtlasSlotCache::allocate` 的克隆试分配与成功替换边界。
    pub(crate) fn allocate(&mut self, size: UVec2) -> Option<GlyphAtlasAllocation> {
        if size.x == 0 || size.y == 0 || size.x > self.page_size.x || size.y > self.page_size.y {
            return None;
        }

        // 换行与适配检查先使用局部状态，提前返回不提交本分配器游标。
        let mut cursor_x = self.cursor_x;
        let mut cursor_y = self.cursor_y;
        let mut shelf_height = self.shelf_height;
        if cursor_x > 0
            && cursor_x
                .checked_add(size.x)
                .is_none_or(|right| right > self.page_size.x)
        {
            cursor_x = 0;
            cursor_y = cursor_y
                .checked_add(self.shelf_height)
                .and_then(|next_y| next_y.checked_add(self.padding_px))?;
            shelf_height = 0;
        }

        if cursor_y
            .checked_add(size.y)
            .is_none_or(|bottom| bottom > self.page_size.y)
        {
            return None;
        }

        let rect = GlyphAtlasRect {
            x: cursor_x,
            y: cursor_y,
            width: size.x,
            height: size.y,
        };
        // 内容已通过边界检查；尾部间隔溢出只保存下一位置哨兵，不撤销当前矩形。
        self.cursor_x = cursor_x
            .checked_add(size.x)
            .and_then(|next_x| next_x.checked_add(self.padding_px))
            .unwrap_or(u32::MAX);
        self.cursor_y = cursor_y;
        self.shelf_height = shelf_height.max(size.y);

        Some(GlyphAtlasAllocation {
            page_key: self.page_key,
            rect,
        })
    }

    /// 比较复用所需的固定配置，不检查或改变当前分配进度。
    ///
    /// # 参数与返回
    /// 仅当 `page_key`、`page_size`、`padding_px` 均逐值相等时返回 `true`。
    /// 槽缓存用此决定能否继续试分配；`false` 不会重建页面或重置游标。
    pub(super) fn matches_configuration(
        &self,
        page_key: GlyphAtlasPageKey,
        page_size: UVec2,
        padding_px: u32,
    ) -> bool {
        self.page_key == page_key && self.page_size == page_size && self.padding_px == padding_px
    }
}

#[cfg(test)]
#[path = "tests/shelf_allocator.rs"]
mod tests;
