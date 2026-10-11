//! 定义拾取后端的能力元数据与命中收集接口。
//!
//! 实现通过 [`PickingBackend`] 接收共享的 [`RayMap`]，返回由调用者拥有的
//! [`PointerHits`] 批次。能力标签仅描述声明的来源；本模块不验证标签是否对应实际实现。
//! [`super::run_picking_pipeline`] 负责调用后端并继续排序、悬停解析和事件派发。
//! 实际排序读取每个命中批次的 [`PointerHits::order`]，元数据中的顺序不会自动传入结果。
//! trait 方法同步返回值，但内部状态变化、缓存与后台工作的约定由具体实现提供。
use super::{PointerHits, RayMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 描述后端声明的拾取来源或算法能力。
///
/// 标签按值保存在 [`PickingBackendInfo::capabilities`] 中；
/// [`PickingBackendInfo::supports`] 只查询列表成员，不验证实现，也不执行拾取。
/// 当前管线不会依据能力标签筛选后端，实际命中由 [`PickingBackend::collect_hits`] 返回。
///
/// # 值与生命周期
/// 无载荷标签可按值复制并存入能力列表；它不持有后端、场景或 GPU 资源。
///
/// # 参考
/// 参见 [`PickingBackendInfo`] 与 [`PickingBackend::collect_hits`]。
pub enum PickingBackendCapability {
    /// 声明能够在 CPU 上利用射线求交提供候选。
    ///
    /// [`super::PrimitivePickingBackend`] 使用此标签描述球形求交能力；
    /// 标签本身只保存在 [`PickingBackendInfo::capabilities`] 中，不能证明已产生任何命中。
    CpuRayCast,
    /// 声明能提供叠加辅助形状的拾取候选。
    ///
    /// [`super::PrimitivePickingBackend`] 声明此标签，并由具体形状和目标配置产生候选；
    /// 标签不携带形状数据，管线仍通过 [`PickingBackend::collect_hits`] 获取结果。
    OverlayShapes,
    /// 声明能以可渲染对象的包围边界提供候选。
    ///
    /// 此值不携带边界或命中数据；声明保存在 [`PickingBackendInfo::capabilities`] 中，
    /// 具体边界获取、更新和命中生成由实现定义。
    RenderableBounds,
    /// 声明能提供 UI 拾取候选。
    ///
    /// 此值不指定 UI 坐标空间、路由或几何数据；这些约定由具体实现提供，
    /// 再通过 [`PickingBackend::collect_hits`] 返回供管线解析的命中批次。
    Ui,
    /// 声明能借助 GPU 拾取提供候选。
    ///
    /// 能力标签不记录 GPU 提交、读回或完成状态；这些边界由具体后端定义。
    /// [`PickingBackend::collect_hits`] 的返回值只携带本次交付给管线的命中批次。
    GpuPicking,
}

#[derive(Clone, Debug, PartialEq)]
/// 后端身份、能力与排序层级；合并器依据输出中的 order 协调多个命中来源。
///
/// 保存后端提供的名称、能力声明和来源排序配置，供调用者按值取得并解释。
///
/// # 字段关系
/// [`Self::capabilities`] 是声明列表；[`Self::supports`] 不检查实现的实际行为。
/// [`Self::order`] 需要由后端写入 [`PointerHits::order`] 才影响命中排序，管线不会自动复制元数据。
/// 目标优先级先于来源顺序，具体比较规则见 [`super::sorted_hits_for_pointer`]。
///
/// # 所有权与生命周期
/// 实例拥有名称字符串、能力列表与标量。克隆产生独立的元数据值；
/// [`PickingBackend::info`] 返回的值不借用后端，也不会随之后的后端变化自动更新。
/// 公开字段与构造方法不验证名称唯一性、能力真实性或顺序值范围。
///
/// # 参考
/// 参见 [`Self::new`]、[`PickingBackend::info`] 与 [`PointerHits::order`]。
pub struct PickingBackendInfo {
    /// 后端提供的名称或诊断身份，按值存为字符串。
    ///
    /// [`Self::new`] 通过调用参数的 `Into<String>` 转换取得它；空字符串与重复名称均可保存。
    /// 当前管线不按该字段分组或调用后端，来源识别规则由使用方约定。
    pub name: String,
    /// 后端声明的能力列表，不等同于实际查询结果或实现验证。
    ///
    /// [`Self::with_capability`] 按调用顺序追加，保留重复项；[`Self::supports`] 查询精确标签成员资格。
    /// 声明是否准确由生成元数据的后端负责，当前管线不会以此决定调用或跳过后端。
    pub capabilities: Vec<PickingBackendCapability>,
    /// 后端的来源顺序配置，供实现写入返回批次的 [`PointerHits::order`]。
    ///
    /// [`super::PrimitivePickingBackend`] 在收集时复制该值；trait 与元数据类型不会自动传播它。
    /// 统一排序先按目标优先级，再用 `total_cmp` 按批次顺序降序比较；
    /// 顺序相同后才比较命中深度和来源索引。字段不验证有限性、单位或范围。
    pub order: f32,
}

impl PickingBackendInfo {
    /// 把名称转换为拥有的字符串，并创建能力列表为空、顺序为 `0.0` 的元数据。
    ///
    /// # 参数
    /// - `name`：生成元数据的调用方提供的名称，按值调用其 `Into<String>` 实现；不要求非空或唯一。
    ///
    /// # 返回
    /// 返回可继续声明能力或设置顺序的 [`Self`]，并拥有转换后的名称。
    /// 函数不返回 `Result`，也不验证名称；转换的副作用或 panic 由参数的转换实现决定。
    ///
    /// # 所有权与调用边界
    /// 参数被消费。本函数设置元数据字段，不调用拾取管线；
    /// 自定义 `Into<String>` 转换是否访问外部状态或启动工作不由此接口限制。
    ///
    /// # 参考
    /// 参见 [`Self::with_capability`]、[`Self::with_order`] 与 [`PickingBackend::info`]。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capabilities: Vec::new(),
            order: 0.0,
        }
    }

    /// 消费当前元数据，向能力列表追加一个标签并返回更新后的值。
    ///
    /// # 参数
    /// - `self`：要更新的元数据，其所有权进入本次调用。
    /// - `capability`：本次要声明的标签；追加不去重、不排序，也不验证实际能力。
    ///
    /// # 返回
    /// 返回保留原名称和顺序、列表末尾增加该标签的 [`Self`]，可继续链式构造。
    /// 其它已克隆的元数据值与已产生的命中批次不会被更新；此操作不触发后端查询。
    ///
    /// # 参考
    /// 参见 [`Self::capabilities`]、[`Self::supports`] 与 [`PickingBackendCapability`]。
    pub fn with_capability(mut self, capability: PickingBackendCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// 消费当前元数据并替换来源顺序配置，返回更新后的值。
    ///
    /// # 参数
    /// - `self`：要更新的元数据，按值取得所有权。
    /// - `order`：新的 `f32` 排序配置；原样保存，包含非有限值在内的取值不会被拒绝。
    ///
    /// # 返回
    /// 返回名称和能力列表保持原值、顺序已替换的 [`Self`]。
    /// 已返回的 [`PointerHits`] 批次不受此次修改影响；后端须在下一次构造批次时采用该配置。
    ///
    /// # 参考
    /// 参见 [`Self::order`]、[`PointerHits::order`] 与 [`super::sorted_hits_for_pointer`]。
    pub fn with_order(mut self, order: f32) -> Self {
        self.order = order;
        self
    }

    /// 查询当前元数据的能力列表是否包含指定标签。
    ///
    /// # 参数
    /// - `capability`：要以精确枚举相等关系查询的 [`PickingBackendCapability`]。
    ///
    /// # 返回
    /// 列表含有该标签时返回 `true`，空列表或未声明时返回 `false`。
    /// 重复声明不影响此布尔结果；查询仅借用元数据，不检查实际后端实现或修改列表。
    ///
    /// # 参考
    /// 参见 [`Self::capabilities`] 与 [`Self::with_capability`]。
    pub fn supports(&self, capability: PickingBackendCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// 从共享 RayMap 产生每指针命中组的后端边界；管线负责跨后端排序、遮挡与事件派发。
///
/// # 调用者与实现者责任
/// 当前 [`super::run_picking_pipeline`] 在射线阶段后按输入后端次序依次调用
/// [`Self::collect_hits`]。实现负责正确填写命中批次的指针、目标、深度和来源顺序，
/// 并为几何坐标及排序键提供可供调用者解释的约定；能力声明不会代替结果校验。
///
/// # 所有权与并发
/// 输入射线通过共享借用提供，返回的元数据与命中批次由调用者拥有。
/// `Send + Sync` 要求实现满足共享访问的线程安全边界；此处不规定具体算法、内部缓存或后台任务。
/// 方法同步返回不表示所有内部工作已结束，后端资源、提交和取消语义须由具体实现说明。
///
/// # 参考
/// 参见 [`PickingBackendInfo`]、[`RayMap`]、[`PointerHits`] 与 [`super::run_picking_pipeline`]。
pub trait PickingBackend: Send + Sync {
    /// 返回后端本次提供的身份、能力声明和来源顺序配置。
    ///
    /// # 返回
    /// 返回拥有名称与能力列表的 [`PickingBackendInfo`]，不借用后端内部存储。
    /// 当前 [`super::PrimitivePickingBackend`] 返回其元数据克隆，标准管线不调用本方法作能力筛选或排序。
    ///
    /// # 调用边界
    /// 方法使用共享借用；具体实现可以采用内部状态机制。
    /// 返回值不证明能力已验证、命中已产生或后台工作已完成；调用副作用与资源时序由实现定义。
    ///
    /// # 参考
    /// 参见 [`Self::collect_hits`] 与 [`PickingBackendInfo::supports`]。
    fn info(&self) -> PickingBackendInfo;
    /// 根据共享射线输入返回后端本次交付给管线的命中批次。
    ///
    /// # 参数
    /// - `rays`：调用期间可读取的 [`RayMap`]。需要保留输入信息的实现应保存自有数据，
    ///   本次共享借用不会将射线集合的所有权交给后端。
    ///
    /// # 返回
    /// 返回由调用者拥有的 [`PointerHits`] 列表；没有可交付命中时可返回空列表。
    /// 接口没有独立错误通道，空列表本身不说明未命中、内部不可用或尚无可交付结果的原因。
    /// 实现负责批次中的指针归属、命中数据和来源排序约定；管线继续合并排序、悬停与事件处理。
    ///
    /// # 调用时机与生命周期
    /// 总开关和射线开关开启时，标准管线逐个调用后端，即使本帧射线集合为空也会调用。
    /// 方法同步返回只表示结果值已交付，不能据此推断具体实现没有 GPU、异步或其它后台工作。
    /// 这些工作的完成、取消和资源释放边界由实现提供。
    ///
    /// # 参考
    /// 参见 [`Self::info`]、[`super::run_picking_pipeline`] 与 [`super::sorted_hits_for_pointer`]。
    fn collect_hits(&self, rays: &RayMap) -> Vec<PointerHits>;
}
