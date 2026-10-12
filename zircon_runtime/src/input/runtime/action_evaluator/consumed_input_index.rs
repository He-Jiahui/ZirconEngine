//! 复用本次已消费输入切片的排序下标，供动作求值作布尔成员查询。
//!
//! # 入口与约束
//! [`ConsumedInputIndex::load`] 分别刷新按钮和轴下标，不克隆键或保存切片借用。
//! 当前合法排列在长度相同且按新值仍有序时复用；否则重建完整下标并排序，保留重复键。
//! 查询必须配对本次准备的切片和排列；本模块不核验切片身份或自动重新准备。
//!
//! # 生命周期
//! 下标由索引拥有，可跨帧保留容量；输入仍由调用方拥有，查询时重新借用。
//! 清空移除逻辑下标和测试计数；本模块不自行加锁。
//!
//! # 参考
//! 参见 [`ConsumedInputIndex`] 与 [`refresh_sorted_indices`]。
use crate::input::{GamepadAxisInput, InputButton};

/// Reusable sorted indices into the caller-owned UI-consumed input slices.
/// 拥有调用方已消费按钮、轴切片的两份排序下标。
///
/// # 约束与生命周期
/// 派生 `Default` 创建空排列；准备不复制输入键，也不保留切片借用。
/// [`Self::load`] 生成合法完整排列；查询所传切片须继续与当前下标及排序一致。
/// 切片身份和布局不由类型或查询方法核验，换帧或改变排序后应重新准备；合法有序排列可跨次复用。
///
/// # 参考
/// 参见 [`Self::button_is_consumed`]、[`Self::axis_is_consumed`] 与 [`Self::clear`]。
#[derive(Debug, Default)]
pub(super) struct ConsumedInputIndex {
    /// 指向当前按钮源切片的完整整数排列，按该切片的按钮顺序比较排列；包含重复键的各源下标。
    ///
    /// # 约束与生命周期
    /// 字段由索引拥有，下标只适用于与其配对的当前源布局；向量容量可复用，测试计数随 load/clear 更新。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex::button_is_consumed`]。
    button_indices: Vec<usize>,
    /// 指向当前轴源切片的完整整数排列，按设备和轴身份顺序比较排列，不去重或持有输入键。
    ///
    /// # 约束与生命周期
    /// 字段由索引拥有，下标只适用于与其配对的当前源布局；向量容量可复用，测试计数随 load/clear 更新。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex::axis_is_consumed`]。
    axis_indices: Vec<usize>,
    /// 仅测试构建的最近准备源长度饱和和；不统计实际比较、下标写入或查询次数，清空时归零。
    ///
    /// # 约束与生命周期
    /// 该标量在每次 load 时更新，clear 将其归零；它不拥有下标或源切片存储，生产构建不包含此字段。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::source_visit_count`]。
    #[cfg(test)]
    source_visits: usize,
}

/// 提供调用方输入切片的准备与成员查询。
///
/// # 约束与生命周期
/// 准备生成下标，查询重新借用切片；切片配对约束必须由调用方保持。
/// 容量和原源长度计数均按值读取，不代表整个求值成本。
///
/// # 参考
/// 参见 [`ConsumedInputIndex`] 与 [`refresh_sorted_indices`]。
impl ConsumedInputIndex {
    /// 刷新当前按钮和轴源切片的排序下标，复用仍合法有序的排列。
    ///
    /// # 参数
    /// `consumed_buttons` 和 `consumed_axes` 在调用期间只读借用，输入内容仍由调用方拥有。
    /// 内部旧下标须是此前生成的合法完整排列；两个具体键类型的派生顺序用于比较。
    ///
    /// # 返回
    /// 返回 `()`；分别保留长度相同且按当前源值有序的排列，或重建并排序所有源下标。
    /// 重复输入不去重；测试计数是两份原源切片长度的饱和和。
    ///
    /// # 使用时机与生命周期
    /// 每次本帧消费记录准备时调用；不保存输入借用或克隆键，容量可复用，稳定复用仍检查相邻顺序。
    /// 准备后查询须传入与当前排列配对的切片；加载本身不执行查询，也不修改源切片。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex`]、[`refresh_sorted_indices`]。
    pub(super) fn load(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        refresh_sorted_indices(&mut self.button_indices, consumed_buttons);
        refresh_sorted_indices(&mut self.axis_indices, consumed_axes);

        #[cfg(test)]
        {
            self.source_visits = consumed_buttons.len().saturating_add(consumed_axes.len());
        }
    }

    /// 查询按钮是否出现在与当前排列配对的已消费按钮切片中。
    ///
    /// # 参数
    /// `consumed_buttons` 必须继续与最近准备的按钮下标范围和排序一致；`button` 是借用的查询身份。
    /// 方法不检查切片是否曾传给 load，也不重新排序或验证排列。
    ///
    /// # 返回
    /// 返回匹配是否存在；重复匹配只影响存在性。
    /// 错配切片可能越界 panic 或导致二分查询结果错误，返回 false 不能证明参数配对合法。
    ///
    /// # 使用时机与生命周期
    /// 只在调用期间借用输入和查询键，索引只读；不保留借用或克隆字符串。
    /// 空索引直接查询为 false；改变源布局后应先重新准备。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`InputButton`]。
    pub(super) fn button_is_consumed(
        &self,
        consumed_buttons: &[InputButton],
        button: &InputButton,
    ) -> bool {
        self.button_indices
            .binary_search_by(|index| consumed_buttons[*index].cmp(button))
            .is_ok()
    }

    /// 查询设备与轴身份是否出现在当前已消费轴集合中。
    ///
    /// # 参数
    /// `consumed_axes` 须与最近准备的轴排列在下标范围和排序上配对；`axis` 按值给出复合查询身份。
    /// 方法不验证源身份或排列。
    ///
    /// # 返回
    /// 二分匹配存在返回 true，否则 false；同名轴的不同设备是不同身份。
    /// 错配可能越界 panic 或错误查询，不提供配置验证结果。
    ///
    /// # 使用时机与生命周期
    /// 只借用调用方源切片，返回布尔副本；不保存输入或改变排列。
    /// 改变布局或换用未配对切片前应重新 load。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`GamepadAxisInput`]。
    pub(super) fn axis_is_consumed(
        &self,
        consumed_axes: &[GamepadAxisInput],
        axis: GamepadAxisInput,
    ) -> bool {
        self.axis_indices
            .binary_search_by(|index| consumed_axes[*index].cmp(&axis))
            .is_ok()
    }

    /// 移除当前消费输入的逻辑下标并归零测试源计数。
    ///
    /// # 参数
    /// `self` 提供要重置的索引；工作区配置重置时可调用。
    ///
    /// # 返回
    /// 返回 `()`，两份排列长度为零；后续成员查询没有索引项。
    ///
    /// # 使用时机与生命周期
    /// 向量容量保留，输入切片仍由调用方拥有；清空不修改其内容或保存它的借用。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex::storage_capacity`]。
    pub(super) fn clear(&mut self) {
        self.button_indices.clear();
        self.axis_indices.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    /// 报告两份下标向量的保留元素容量之和。
    ///
    /// # 参数
    /// `self` 提供当前索引；不访问或准备输入切片。
    ///
    /// # 返回
    /// 返回两个 `Vec<usize>` 容量的饱和和，单位为下标元素槽位，不是输入数量、字节预算或零分配保证。
    ///
    /// # 使用时机与生命周期
    /// 只读按值返回；清空后容量可保留，工作区比较准备前后值记录存储增长。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::clear`]、[`ConsumedInputIndex::load`]。
    pub(super) fn storage_capacity(&self) -> usize {
        self.button_indices
            .capacity()
            .saturating_add(self.axis_indices.capacity())
    }

    /// 在测试构建中报告最近一次 load 的原源条目数量。
    ///
    /// # 参数
    /// `self` 提供当前测试索引，生产构建不含此方法。
    ///
    /// # 返回
    /// 返回按钮和轴原切片长度的饱和和；重复源计入，未计实际排序比较、写入或查找次数。
    ///
    /// # 使用时机与生命周期
    /// load 更新、clear 归零，读取不触发输入准备或改变容量。
    ///
    /// # 参考
    /// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex::clear`]。
    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}

// UI 消费列表由调用方保留所有权；仅复用有效的排序下标，当前列表不再按这些下标有序时重建，避免每个动作重复线性扫描。
/// 复用对当前源值仍有序的完整下标排列，必要时重建并排序。
///
/// # 参数
/// `indices` 为独占可变排列；长度与 `values` 相同时，必须已经是合法、完整、无重复的源下标排列。
/// `values` 只读借用；`T: Ord` 的 `<=` 与 `cmp` 应提供一致全序。
/// 长度和相邻顺序检查不验证下标范围、唯一性或完整性；比较调用可有副作用或 panic，助手不隔离它们。
///
/// # 返回
/// 返回 `()`；长度相同且现有下标映射的相邻值不降序时立即返回。
/// 否则清空，写入从零到源长度的完整下标并排序；保留所有源项，不去重，相等键下标顺序无稳定保证。
///
/// # 使用时机与生命周期
/// 生产 load 和测试计时调用；返回后只保留下标和容量，不借用 `values`。
/// 合法排列可用于新的同长切片，只要按当前值仍有序；这不要求源键身份未变。
/// 单独传入损坏的同长排列可能 panic 或保留不完整索引，不能把缓存检查当作排列修复。
///
/// # 参考
/// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex`]。
fn refresh_sorted_indices<T: Ord>(indices: &mut Vec<usize>, values: &[T]) {
    if indices.len() == values.len()
        && indices
            .windows(2)
            .all(|pair| values[pair[0]] <= values[pair[1]])
    {
        return;
    }

    indices.clear();
    indices.extend(0..values.len());
    indices.sort_unstable_by(|left, right| values[*left].cmp(&values[*right]));
}

/// 仅在测试构建中挂载排列查询、源码守卫和默认忽略的计时样例。
///
/// # 约束与生命周期
/// 功能 fixture 比较稳定重载及轴布局改变后的查询，不直接统计索引写入或排序。
/// 当前源码分隔守卫未包含实际路径属性，首个分隔不匹配；计时注解也不强制 release 构建。
/// 计时结果只覆盖实际执行的固定样本；源码形状匹配不替代运行行为或排序次数的证明。
///
/// # 参考
/// 参见 [`ConsumedInputIndex`] 与 [`refresh_sorted_indices`]。
#[cfg(test)]
#[path = "tests/consumed_input_index.rs"]
mod tests;
