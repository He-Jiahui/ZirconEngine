//! 文本布局几何的有限值恢复、数值预算与请求约束。
//!
//! 恢复工具把异常数值转换为有限发布值，预算入口则显式拒绝不合要求的几何，两者承担不同职责。
//! 预算按本次数值检查上限，不随检查次数扣减；它不校验来源范围、容器包含关系或内存消耗。

/// Largest integer-valued logical-pixel extent that remains exactly representable by `f32`.
///
/// This is a numeric safety ceiling, not a product-tuned layout limit. Product policy may choose
/// a lower session budget after representative workload profiling.
///
/// 默认值对应非负连续整数的 f32 精度边界；不表示更大的整数一概不可表示。
const DEFAULT_MAX_EXACT_LAYOUT_EXTENT: f32 = 16_777_216.0;

/// Publish finite layout geometry from a wider intermediate without changing ordinary `f32`
/// arithmetic. This is the shared recovery path for finite-input overflow across text layout.
///
/// 宽精度中间结果的 NaN 映射到零，其余值夹至有限的 f32 范围。
/// 此恢复不返回预算错误，也不保证转换精度；需要拒绝超限几何时仍须经过预算入口。
pub(crate) fn finite_geometry(value: f64) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
    }
}

/// 优先保留正常的 f32 候选值，仅在候选非有限时由宽精度结果恢复。
///
/// `candidate` 有限时原样返回，不与 `exact` 比较；否则按 `finite_geometry` 的规则发布后者。
pub(crate) fn finite_f32_or_geometry(candidate: f32, exact: f64) -> f32 {
    candidate
        .is_finite()
        .then_some(candidate)
        .unwrap_or_else(|| finite_geometry(exact))
}

/// Retains exact accumulation history after the published `f32` value first overflows.
///
/// Before recovery, ordinary `f32` arithmetic remains authoritative. After recovery, every
/// publication is derived from the wider history so a later sign change cannot accidentally use
/// a previously saturated value as if it were the exact sum.
///
/// 以默认零状态开始，按布局顺序累计；复制此值会同时复制当前发布值与历史状态。
/// 宽精度历史仍存在浮点舍入，不能把它理解为任意输入序列的数学精确和或预算准入结果。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FiniteGeometryAccumulator {
    /// 当前发布值：恢复前沿用逐项 f32 结果，恢复后由宽精度历史得出。
    value: f32,
    /// 持续累积各输入的 f64 表示，恢复前也同步推进。
    exact: f64,
    /// 任意一次非有限候选后即置位；后续累计保持使用宽精度恢复分支。
    recovered: bool,
}

impl FiniteGeometryAccumulator {
    /// 追加一项并返回最新发布值；同时推进普通候选与宽精度历史。
    ///
    /// 输入不会在此检查预算；任何非有限候选都会永久进入恢复状态，后续不再以先前饱和值作为历史和。
    pub(crate) fn add(&mut self, value: f32) -> f32 {
        self.exact += f64::from(value);
        let candidate = self.value + value;
        if self.recovered || !candidate.is_finite() {
            self.recovered = true;
            self.value = finite_geometry(self.exact);
        } else {
            self.value = candidate;
        }
        self.value
    }

    pub(crate) const fn value(&self) -> f32 {
        self.value
    }

    /// 读取当前 f64 累计历史；该值仍可能非有限，也未经过几何预算检查。
    pub(crate) const fn exact(&self) -> f64 {
        self.exact
    }
}

/// 按迭代顺序消费 `values`，返回累计器的发布值；空迭代返回零。
///
/// 此入口不会先收集各项，也不执行预算准入；恢复规则与逐项调用累计器相同。
pub(crate) fn finite_sum(values: impl IntoIterator<Item = f32>) -> f32 {
    let mut sum = FiniteGeometryAccumulator::default();
    for value in values {
        sum.add(value);
    }
    sum.value()
}

/// 测量和发布检查使用的数值上限副本。
///
/// # 使用与约束
/// 通过 `new` 构造合法预算后可按值传递；各次检查互不扣减额度。
/// extent 必须有限且非负；坐标可为负，但绝对值不得越过累计上限。
/// 这不是来源映射、裁剪、精度或内存预算的完整证明。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryBudget {
    /// 单个轴向大小的上限。
    max_axis_extent: f32,
    /// 累计大小及坐标绝对值的上限，不是可耗尽的剩余额度。
    max_accumulated_extent: f32,
}

/// 单次准入失败的数值证据，由调用层补充 owner 与源范围后形成会话回执。
///
/// 坐标失败记录其绝对值；宽精度超限转换后也可能把无穷值留在尝试值中。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryViolation {
    /// 被拒绝的尝试值，保留 NaN 或无穷等异常数值。
    pub(crate) attempted_extent: f32,
    /// 本次比较采用的数值上限。
    pub(crate) admitted_extent: f32,
}

/// 请求侧可用空间的有界或无界元数据，不是可直接发布的成品大小。
///
/// `from_request_extent` 仅把正无穷解释为无界；直接构造 `Bounded` 不会自动执行预算检查。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TextLayoutAxisConstraint {
    /// 有界请求值；直接构造时由调用方维护预算前提。
    Bounded(f32),
    /// 保留无界请求，后续生成的成品几何仍需预算准入。
    Unbounded,
}

/// 几何拒绝的调用层归因类别，用于会话诊断区分测量、发布和表格阶段。
///
/// 枚举值本身不执行检查或布局调度；实际失败位置由调用方填写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextLayoutGeometryOwner {
    IntrinsicMeasurement,
    ResolvedLayoutPublication,
    TableAvailableTrackExtent,
    TablePreferredCell,
    TableColumnTracks,
    TableRowTracks,
    TableCellFrame,
    TableAggregate,
}

impl TextLayoutGeometryBudget {
    /// 构造可按值复制的几何预算。
    ///
    /// # 参数与返回
    /// 两个上限都须有限，`max_axis_extent > 0` 且累计上限不小于轴上限。
    /// 满足条件返回 `Some`；否则返回 `None`，由调用方决定拒绝或回退策略。
    pub(crate) fn new(max_axis_extent: f32, max_accumulated_extent: f32) -> Option<Self> {
        if !max_axis_extent.is_finite()
            || max_axis_extent <= 0.0
            || !max_accumulated_extent.is_finite()
            || max_accumulated_extent < max_axis_extent
        {
            return None;
        }
        Some(Self {
            max_axis_extent,
            max_accumulated_extent,
        })
    }

    pub(crate) const fn max_axis_extent(self) -> f32 {
        self.max_axis_extent
    }

    pub(crate) const fn max_accumulated_extent(self) -> f32 {
        self.max_accumulated_extent
    }

    /// 有限且落在闭区间 `[0, max_axis_extent]` 时返回 `extent` 原值，否则返回拒绝数值。
    pub(crate) fn admit_axis_extent(self, extent: f32) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit(extent, self.max_axis_extent)
    }

    /// 对一个已累计的非负大小使用累计上限准入，成功返回原值。
    ///
    /// 此调用不维护已用额度；负数、非有限值或超上限值立即返回错误。
    pub(crate) fn admit_accumulated_extent(
        self,
        extent: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        // violation 只记录尝试值与预算上限；调用层再补充 owner 和源范围，形成可归因的拒绝回执。
        self.admit(extent, self.max_accumulated_extent)
    }

    /// 准入可带符号的坐标，成功保留原符号和值。
    ///
    /// 坐标须有限且落在 `[-max_accumulated_extent, max_accumulated_extent]`；失败回执记录绝对值。
    pub(crate) fn admit_coordinate(
        self,
        coordinate: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        let magnitude = coordinate.abs();
        if coordinate.is_finite() && magnitude <= self.max_accumulated_extent {
            Ok(coordinate)
        } else {
            Err(self.violation(magnitude, self.max_accumulated_extent))
        }
    }

    /// 先准入两个累计大小，再用 f64 中间量检查其和。
    ///
    /// # 返回与约束
    /// 任一操作数或总和越界即返回错误；成功返回转换为 f32 后再次准入的和。
    /// 该计算不会改变预算，也不保证任意小数求和完全无舍入。
    pub(crate) fn checked_add_accumulated(
        self,
        left: f32,
        right: f32,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit_accumulated_extent(left)?;
        self.admit_accumulated_extent(right)?;
        let attempted = f64::from(left) + f64::from(right);
        if attempted > f64::from(self.max_accumulated_extent) {
            return Err(self.violation(attempted as f32, self.max_accumulated_extent));
        }
        self.admit_accumulated_extent(attempted as f32)
    }

    /// 先按轴上限准入单项大小，再检查重复 `count` 次的累计大小。
    ///
    /// 即使 `count` 为零，单项也必须先合法；成功返回经 f32 转换后再次准入的乘积。
    /// `count` 表示重复次数，此入口不约束实际分配或迭代工作量。
    pub(crate) fn checked_scale_accumulated(
        self,
        extent: f32,
        count: usize,
    ) -> Result<f32, TextLayoutGeometryViolation> {
        self.admit_axis_extent(extent)?;
        let attempted = f64::from(extent) * count as f64;
        if !attempted.is_finite() || attempted > f64::from(self.max_accumulated_extent) {
            return Err(self.violation(attempted as f32, self.max_accumulated_extent));
        }
        self.admit_accumulated_extent(attempted as f32)
    }

    fn admit(self, extent: f32, admitted_extent: f32) -> Result<f32, TextLayoutGeometryViolation> {
        if extent.is_finite() && extent >= 0.0 && extent <= admitted_extent {
            Ok(extent)
        } else {
            Err(self.violation(extent, admitted_extent))
        }
    }

    const fn violation(
        self,
        attempted_extent: f32,
        admitted_extent: f32,
    ) -> TextLayoutGeometryViolation {
        TextLayoutGeometryViolation {
            attempted_extent,
            admitted_extent,
        }
    }
}

impl TextLayoutAxisConstraint {
    /// 从请求大小建立约束：正无穷保留为无界，其余值必须通过轴预算。
    ///
    /// 返回有界或无界元数据；负无穷、NaN、负数与超限有限值返回准入错误。
    pub(crate) fn from_request_extent(
        extent: f32,
        budget: TextLayoutGeometryBudget,
    ) -> Result<Self, TextLayoutGeometryViolation> {
        if extent == f32::INFINITY {
            return Ok(Self::Unbounded);
        }
        budget.admit_axis_extent(extent).map(Self::Bounded)
    }

    /// 还原请求参数；无界分支返回正无穷，不能据此把它当作已准入的发布大小。
    pub(crate) const fn request_extent(self) -> f32 {
        match self {
            Self::Bounded(extent) => extent,
            Self::Unbounded => f32::INFINITY,
        }
    }

    pub(crate) const fn bounded_extent(self) -> Option<f32> {
        match self {
            Self::Bounded(extent) => Some(extent),
            Self::Unbounded => None,
        }
    }

    /// 从有界请求中扣除已消费大小，结果最低为零；无界请求保持无界。
    ///
    /// `consumed` 在两种分支中都先接受累计预算检查；此处不重新准入直接构造的有界值。
    pub(crate) fn subtract_accumulated(
        self,
        consumed: f32,
        budget: TextLayoutGeometryBudget,
    ) -> Result<Self, TextLayoutGeometryViolation> {
        budget.admit_accumulated_extent(consumed)?;
        match self {
            Self::Bounded(extent) => Ok(Self::Bounded((extent - consumed).max(0.0))),
            Self::Unbounded => Ok(Self::Unbounded),
        }
    }
}

impl Default for TextLayoutGeometryBudget {
    fn default() -> Self {
        Self {
            max_axis_extent: DEFAULT_MAX_EXACT_LAYOUT_EXTENT,
            max_accumulated_extent: DEFAULT_MAX_EXACT_LAYOUT_EXTENT,
        }
    }
}

#[cfg(test)]
#[path = "tests/layout_geometry.rs"]
mod tests;
