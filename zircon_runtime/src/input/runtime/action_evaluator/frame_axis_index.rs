//! 为本帧轴状态和转换分别建立可复用的查询索引。
//!
//! # 入口与约束
//! [`FrameAxisIndex::load_frame`] 从两个源向量分别复制记录，按轴身份和源位置排序后保留同轴最后条目。
//! 两个列表独立处理，不核验当前值与转换末值是否一致；索引不归一化或拒绝原始浮点值。
//!
//! # 生命周期
//! 索引拥有记录，不借用帧；查询返回副本。下一次加载替换逻辑内容，清空仍保留向量容量。
//! 工作区在配置没有轴绑定时清空索引，否则为当前帧加载；这里不自行加锁。
//!
//! # 参考
//! 参见 [`FrameAxisIndex`]、[`retain_latest_value`] 与 [`retain_latest_transition`]。
use crate::input::{GamepadAxisInput, GamepadAxisTransition, InputFrameSnapshot};

// 单帧摇杆状态和边沿的可复用查询索引；同一设备轴重复上报时，以源序最后一项供动作评估。
/// 拥有本帧轴当前值和转换的两个独立索引。
///
/// # 约束与生命周期
/// 派生 `Default` 创建空索引；应在查询本次帧数据前调用 [`Self::load_frame`]，重复轴以各源向量最后条目为准。
/// 两个索引不互相补齐或验证，保存的浮点内容保持原样；查询副本不借用索引。
/// 加载和清空保留可复用容量；自身无锁，由调用方的独占可变访问控制准备阶段。
///
/// # 参考
/// 参见 [`Self::value()`]、[`Self::transition()`] 与 [`Self::clear`]。
#[derive(Debug, Default)]
pub(super) struct FrameAxisIndex {
    /// 按轴身份排序、每轴最多一项的当前值记录；加载时从 gamepad_axes 复制，保留该源列表最后同轴项。
    ///
    /// # 生命周期
    /// 字段由索引拥有，加载更新逻辑内容；向量可跨帧保留容量，清空仅移除其逻辑记录。
    ///
    /// # 参考
    /// 参见 [`AxisValue`]、[`FrameAxisIndex::value()`]。
    values: Vec<AxisValue>,
    /// 按轴身份排序、每轴最多一项的转换记录；从独立转换列表复制，保留其最后同轴项，不与当前值列表协调。
    ///
    /// # 生命周期
    /// 字段由索引拥有，加载更新逻辑内容；向量可跨帧保留容量，清空仅移除其逻辑记录。
    ///
    /// # 参考
    /// 参见 [`AxisTransition`]、[`FrameAxisIndex::transition()`]。
    transitions: Vec<AxisTransition>,
    /// 仅测试构建的最近加载源条目数，两份源向量长度饱和相加；不是唯一轴数、比较次数或耗时，clear 置零。
    ///
    /// # 生命周期
    /// 该标量在每次 load_frame 时以两个源向量的长度更新，clear 将其归零；它不拥有轴数据或缓冲容量，生产构建不包含此字段。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::source_visit_count`]。
    #[cfg(test)]
    source_visits: usize,
}

/// 保存一个源当前值的轴身份、原始源位置和复制内容。
///
/// # 约束与生命周期
/// 加载时按值形成记录，`Copy` 不保留源帧借用；排序后原始源位置用于决定最后同轴项。
/// 记录用于最近一次帧加载建立的索引，不代表时间戳或转换事件编号。
///
/// # 参考
/// 参见 [`FrameAxisIndex::load_frame`]。
#[derive(Clone, Copy, Debug)]
struct AxisValue {
    /// 设备与轴的复合身份，作为查询及分组键。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisValue`] 与 [`FrameAxisIndex::load_frame`]。
    input: GamepadAxisInput,
    /// 在对应当前值源向量中的零基位置；两种源列表分别编号，排序后不重新编号。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisValue`] 与 [`FrameAxisIndex::load_frame`]。
    source_index: usize,
    /// 复制的原始 f32 当前值，索引不执行范围限制、死区或有限性检查。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisValue`] 与 [`FrameAxisIndex::load_frame`]。
    value: f32,
}

/// 保存一个源转换的轴身份、原始源位置和复制内容。
///
/// # 约束与生命周期
/// 加载时按值形成记录，`Copy` 不保留源帧借用；排序后原始源位置用于决定最后同轴项。
/// 记录用于最近一次帧加载建立的索引，不代表时间戳或转换事件编号。
///
/// # 参考
/// 参见 [`FrameAxisIndex::load_frame`]。
#[derive(Clone, Copy, Debug)]
struct AxisTransition {
    /// 设备与轴的复合身份，作为查询及分组键。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisTransition`] 与 [`FrameAxisIndex::load_frame`]。
    input: GamepadAxisInput,
    /// 在对应转换源向量中的零基位置；两种源列表分别编号，排序后不重新编号。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisTransition`] 与 [`FrameAxisIndex::load_frame`]。
    source_index: usize,
    /// 完整复制的 GamepadAxisTransition，保留原始前值和末值，不与当前值向量核验。
    ///
    /// # 生命周期
    /// 字段随所属记录复制和替换，无源帧借用。
    ///
    /// # 参考
    /// 参见 [`AxisTransition`] 与 [`FrameAxisIndex::load_frame`]。
    transition: GamepadAxisTransition,
}

/// 提供一次帧准备及准备后的只读查询。
///
/// # 约束与生命周期
/// 加载建立排序及去重不变式；返回查询值为副本，容量指标以向量元素计。
/// 测试源指标在生产构建中不存在。
///
/// # 参考
/// 参见 [`FrameAxisIndex`]。
impl FrameAxisIndex {
    /// 替换本索引的帧当前值和转换，并分别建立有序、同轴唯一的记录。
    ///
    /// # 参数
    /// `frame` 只在调用期间借用；状态和转换源可分别含重复轴，原源顺序中的最后项优先。
    /// 内容原样复制，不限制浮点范围或有限性，也不核验两个列表一致性。
    ///
    /// # 返回
    /// 返回 `()`；两个向量先清空、复制源项、按身份和源位置排序，再压缩相邻同轴项。
    /// 测试计数是两个原源长度的饱和和，包含被去掉的重复项。
    ///
    /// # 使用时机与生命周期
    /// 用于工作区帧准备；可能因容量不足扩容，已有容量可复用。调用结束不保留帧借用。
    /// 下一次加载或清空改变索引内部内容，先前取得的查询副本仍独立；调用方提供独占可变访问。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex`]、[`retain_latest_value`]、[`retain_latest_transition`]。
    pub(super) fn load_frame(&mut self, frame: &InputFrameSnapshot) {
        self.values.clear();
        self.values.extend(
            frame
                .gamepad_axes
                .iter()
                .enumerate()
                .map(|(source_index, state)| AxisValue {
                    input: GamepadAxisInput::new(state.gamepad, state.axis),
                    source_index,
                    value: state.value,
                }),
        );
        self.values
            .sort_unstable_by_key(|value| (value.input, value.source_index));
        retain_latest_value(&mut self.values);

        self.transitions.clear();
        self.transitions
            .extend(frame.gamepad_axis_transitions.iter().enumerate().map(
                |(source_index, transition)| AxisTransition {
                    input: GamepadAxisInput::new(transition.gamepad, transition.axis),
                    source_index,
                    transition: *transition,
                },
            ));
        self.transitions
            .sort_unstable_by_key(|transition| (transition.input, transition.source_index));
        retain_latest_transition(&mut self.transitions);

        #[cfg(test)]
        {
            self.source_visits = frame
                .gamepad_axes
                .len()
                .saturating_add(frame.gamepad_axis_transitions.len());
        }
    }

    /// 查询当前已准备索引中一根轴的当前值。
    ///
    /// # 参数
    /// `axis` 是设备与轴组合的查询键；应读取已经完成当前帧加载的索引。
    ///
    /// # 返回
    /// 存在当前值记录返回 `Some` 原始 f32 副本，否则返回 `None`；不从转换记录推导当前值。
    ///
    /// # 使用时机与生命周期
    /// 只读二分查询；返回副本不借用索引，不触发帧加载或改变容量。索引清空后查询返回 `None`。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::load_frame`]、[`FrameAxisIndex::transition()`]。
    pub(super) fn value(&self, axis: GamepadAxisInput) -> Option<f32> {
        self.values
            .binary_search_by_key(&axis, |value| value.input)
            .ok()
            .map(|index| self.values[index].value)
    }

    /// 查询当前已准备索引中一根轴的转换记录。
    ///
    /// # 参数
    /// `axis` 是查询键；当前帧转换源列表独立于当前值列表。
    ///
    /// # 返回
    /// 存在转换返回 `Some` 完整记录副本，否则 `None`；不从当前值生成边沿。
    ///
    /// # 使用时机与生命周期
    /// 只读二分查询；副本可在下一次加载后继续使用，但仍描述此前准备的数据。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::load_frame`]、[`FrameAxisIndex::value()`]。
    pub(super) fn transition(&self, axis: GamepadAxisInput) -> Option<GamepadAxisTransition> {
        self.transitions
            .binary_search_by_key(&axis, |transition| transition.input)
            .ok()
            .map(|index| self.transitions[index].transition)
    }

    /// 清空当前帧索引的逻辑记录，并把测试源计数归零。
    ///
    /// # 参数
    /// `self` 为要重置的索引；工作区在无轴绑定或配置重置时调用。
    ///
    /// # 返回
    /// 返回 `()`；两个记录向量长度为零，测试源计数为零。
    ///
    /// # 使用时机与生命周期
    /// 向量容量保留供后续加载复用；之前返回的复制值不受影响，之后查询没有记录。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::load_frame`]、[`FrameAxisIndex::storage_capacity`]。
    pub(super) fn clear(&mut self) {
        self.values.clear();
        self.transitions.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    /// 报告两份内部记录向量的保留容量之和。
    ///
    /// # 参数
    /// `self` 提供当前索引；此读取不准备帧。
    ///
    /// # 返回
    /// 返回两个向量 `capacity()` 的饱和和，单位是各向量的元素槽位。
    /// 两种元素大小不同，此和不表示字节、记录数或零分配保证。
    ///
    /// # 使用时机与生命周期
    /// 只读按值返回；清空后容量仍可保留，工作区用前后值比较记录容量增长。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::clear`]、[`FrameAxisIndex::load_frame`]。
    pub(super) fn storage_capacity(&self) -> usize {
        self.values
            .capacity()
            .saturating_add(self.transitions.capacity())
    }

    /// 在测试构建中读取最近加载记录的原源条目数量。
    ///
    /// # 参数
    /// `self` 提供测试索引；生产构建不包含此方法。
    ///
    /// # 返回
    /// 返回状态源长度与转换源长度的饱和和；它不是去重后轴数、排序比较次数或实测逐项访问计数。
    ///
    /// # 使用时机与生命周期
    /// 加载时更新，清空时归零，按值读取不修改索引。
    ///
    /// # 参考
    /// 参见 [`FrameAxisIndex::load_frame`]、[`FrameAxisIndex::clear`]。
    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}

/// 在原向量内压缩相邻同轴当前值记录，保留每组最后一项。
///
/// # 参数
/// `values` 须已按轴身份分组；若要表达最后源条目，还须每组按源位置递增排列。
/// 此助手不排序、不验证这个前提，也不选择最大的浮点值。
///
/// # 返回
/// 返回 `()`；最后记录覆盖组的已保留位置，新组向前压缩，并把向量截断到保留长度。
/// 只有写入和读取位置不同时才交换；原始源位置与载荷一起保留。
///
/// # 使用时机与生命周期
/// 生产加载先排序再调用；只独占借用向量，不保留其它借用，截断保留容量。
/// 未分组的输入可留下重复组，因此不能把助手单独当作通用去重或配置验证入口。
///
/// # 参考
/// 参见 [`FrameAxisIndex::load_frame`]、[`AxisValue`]、[`retain_latest_transition`]。
fn retain_latest_value(values: &mut Vec<AxisValue>) {
    let mut retained = 0;
    for index in 0..values.len() {
        if retained > 0 && values[retained - 1].input == values[index].input {
            values[retained - 1] = values[index];
        } else {
            if retained != index {
                values.swap(retained, index);
            }
            retained += 1;
        }
    }
    values.truncate(retained);
}

/// 在原向量内压缩相邻同轴转换记录，保留每组最后一项。
///
/// # 参数
/// `transitions` 须已按轴身份分组；若要表达最后源条目，还须每组按源位置递增排列。
/// 此助手不排序、不验证这个前提，也不选择最大的浮点值。
///
/// # 返回
/// 返回 `()`；最后记录覆盖组的已保留位置，新组向前压缩，并把向量截断到保留长度。
/// 只有写入和读取位置不同时才交换；原始源位置与载荷一起保留。
///
/// # 使用时机与生命周期
/// 生产加载先排序再调用；只独占借用向量，不保留其它借用，截断保留容量。
/// 未分组的输入可留下重复组，因此不能把助手单独当作通用去重或配置验证入口。
///
/// # 参考
/// 参见 [`FrameAxisIndex::load_frame`]、[`AxisTransition`]、[`retain_latest_value`]。
fn retain_latest_transition(transitions: &mut Vec<AxisTransition>) {
    let mut retained = 0;
    for index in 0..transitions.len() {
        if retained > 0 && transitions[retained - 1].input == transitions[index].input {
            transitions[retained - 1] = transitions[index];
        } else {
            if retained != index {
                transitions.swap(retained, index);
            }
            retained += 1;
        }
    }
    transitions.truncate(retained);
}

/// 仅在测试构建中挂载轴记录压缩的比较、源码守卫和计时样例。
///
/// # 约束与生命周期
/// 测试可访问私有记录和助手；计时样例默认忽略，其注解不检查实际构建配置，结果只覆盖实际执行的压缩样例。
/// 源码守卫期待相邻的条件属性和模块声明，实际还有路径属性；首个分隔查找为空时，测试在 expect 处失败，尚未进入形状断言。
///
/// # 参考
/// 参见 [`FrameAxisIndex`]、[`retain_latest_value`] 与 [`retain_latest_transition`]。
#[cfg(test)]
#[path = "tests/frame_axis_index.rs"]
mod tests;
