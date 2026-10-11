//! 保存按相机、指针和视口区分的当前拾取射线。
//!
//! [`RayId`] 提供完整键，[`CameraRaySource`] 拥有本次相机快照与配对的视口尺寸；
//! [`RayMap`] 在重建后供 [`super::PickingBackend`] 共享借用。
//! 输入方负责选择身份和配对坐标，集合拥有计算结果，不持有相机或视口资源。
use std::collections::HashMap;

use crate::core::framework::render::{RenderViewportHandle, ViewportCameraSnapshot};
use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;

use super::{ray_from_viewport_point, PointerId, PointerLocation, PointerRay};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// 相机、指针和视口三者共同标识一条射线；同一指针可在多个视口或相机下产生不同射线。
///
/// # 身份字段与约束
///
/// - `camera`：输入源提供的相机实体数值，与快照对应，由调用方区分独立相机。
/// - `pointer`：与位置及输入事件一致的数值指针键。
/// - `viewport`：位置所属的渲染视口句柄，与相机维度一起区分射线。
///
/// 相等与散列同时比较三个字段；相同三元组只对应集合中的一个条目。
/// 通过 [`Self::new`] 或公开字段组装都只保存数值，不验证实体或视口资源。
///
/// # 生命周期与参考
///
/// 键可复制且不保活外部资源；改变或丢弃副本不会移除已存条目。
/// 查询完整键对应的射线见 [`RayMap::get`]。
pub struct RayId {
    /// 产生此射线的相机数值身份，供命中数据记录来源。
    ///
    /// 调用方应使它与本次快照一致；[`RayId::new`] 只封装数值，不查询或保活实体。
    /// 同值会参与完整键匹配，资源删除不会自动移除已有条目。
    pub camera: EntityId,
    /// 与位置及输入事件对应的 [`PointerId`] 值，报告和命中输出用它关联指针。
    ///
    /// 该字段按值拥有，不包含相机或视口身份；完整射线键仍需其它两个字段。
    /// 复制或丢弃键不会清理按指针保存的交互状态。
    pub pointer: PointerId,
    /// 像素采样所属的渲染视口身份，与相机和指针共同区分射线。
    ///
    /// 调用方负责视口和局部坐标配对；[`RayId::new`] 不验证尺寸或资源存续。
    /// 这里只拥有句柄值，不取得渲染资源所有权或触发清理。
    pub viewport: RenderViewportHandle,
}

impl RayId {
    /// 原样封装相机、指针与视口身份，供插入或查询一条射线。
    ///
    /// # 参数
    ///
    /// - `camera`：调用方选定的相机实体数值，须与使用的相机快照对应。
    /// - `pointer`：位置及输入所使用的指针键，按值传入。
    /// - `viewport`：该射线所属的渲染视口句柄，按值传入。
    ///
    /// # 返回值
    ///
    /// 返回拥有相同三值的键；相同输入产生相等键。函数不分配编号、登记射线或检查资源。
    ///
    /// # 调用时机与生命周期
    ///
    /// 在准备 [`RayMap::insert`] 或 [`RayMap::get`] 的键时使用，也可用于常量求值。
    /// 键可按值复制，构造不改变已有集合或外部指针状态。
    ///
    /// # 参考
    ///
    /// 写入与查询见 [`RayMap::insert`] 和 [`RayMap::get`]。
    pub const fn new(camera: EntityId, pointer: PointerId, viewport: RenderViewportHandle) -> Self {
        Self {
            camera,
            pointer,
            viewport,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// 拥有本次相机快照及视口配对信息，供 [`RayMap::rebuild`] 产生射线。
///
/// # 字段关系与使用约束
///
/// `camera` 标记快照对应的相机键；`viewport` 标记位置所属视口，
/// 应与 `viewport_size` 和 `snapshot` 对应同一采样场景。
/// 所有字段公开，构造或改写时由输入方维持配对和数值约束。
/// `active` 是独立的拾取源开关，重建不读取快照的 `is_active` 来替代它。
///
/// # 所有权与生命周期
///
/// 源拥有快照值，克隆得到独立副本；相机后续变化不会自动更新快照。
/// 重建只在调用期间借用源，集合保留计算出的射线数值。
/// 暂时排除这个源的便利方法见 [`Self::inactive`]。
pub struct CameraRaySource {
    /// 本次快照对应的相机数值身份，重建时写入 [`RayId`] 的相机维度。
    ///
    /// 输入方应区分独立相机并维持快照配对；构造不查实体或分配编号。
    /// 改写此字段仅影响下一次重建，不迁移旧集合条目或保活实体。
    pub camera: EntityId,
    /// 本次像素采样所属的渲染视口，供 [`RayMap::rebuild`] 与位置视口匹配。
    ///
    /// 须与源尺寸和快照对应；句柄相等不自动验证这种配对。
    /// 值的复制和丢弃不创建、保活或关闭外部视口，修改后应重新生成射线。
    pub viewport: RenderViewportHandle,
    /// 配对视口的像素宽高，用于 [`PointerLocation`] 的闭区间准入及射线宽高比计算。
    ///
    /// 应与 `viewport` 和当前快照对应。任一维度为零时 [`ray_from_viewport_point`]
    /// 不产生射线；源构造只保存尺寸，视口变化后由输入方更新并重新重建。
    pub viewport_size: UVec2,
    /// 该源拥有的 [`ViewportCameraSnapshot`] 值，包含本次射线计算所需变换及投影参数。
    ///
    /// 输入方负责快照与身份、视口尺寸一致并具有有意义的参数。
    /// [`RayMap::rebuild`] 只借用快照进行计算；快照的 `is_active` 不改变源的 `active`。
    pub snapshot: ViewportCameraSnapshot,
    /// 决定这个源是否参与下一次 [`RayMap::rebuild`]，并用于潜在组合容量的估计。
    ///
    /// 构造时为 `true`，[`Self::inactive`] 将它设为 `false`。
    /// 此标记与快照的 `is_active` 分开保存；修改它不追溯更改已计算的集合或其它源副本。
    pub active: bool,
}

impl CameraRaySource {
    /// 保存调用方配对的相机信息，并将这个拾取源设为活动。
    ///
    /// # 参数
    ///
    /// - `camera`：快照对应的相机实体数值，按值保存。
    /// - `viewport`：输入位置所属的渲染视口句柄，按值保存。
    /// - `viewport_size`：该视口的像素宽高；需要产生射线时两维均应非零。
    /// - `snapshot`：按值转移给源的当前相机快照，应与上述身份及尺寸配对。
    ///
    /// # 返回值
    ///
    /// 返回拥有四项输入且 `active` 为 `true` 的源。快照自身的 `is_active`
    /// 不影响这个初始标记；构造不验证实体、视口尺寸或投影参数。
    ///
    /// # 调用时机与生命周期
    ///
    /// 为当前采样准备 [`RayMap::rebuild`] 的相机列表时使用。
    /// 返回源拥有快照，后续场景变化须由调用方更新；需要排除该源时可使用 [`Self::inactive`]。
    ///
    /// # 参考
    ///
    /// 生成本次射线见 [`RayMap::rebuild`]；源活动标记见 [`Self::inactive`]。
    pub fn new(
        camera: EntityId,
        viewport: RenderViewportHandle,
        viewport_size: UVec2,
        snapshot: ViewportCameraSnapshot,
    ) -> Self {
        Self {
            camera,
            viewport,
            viewport_size,
            snapshot,
            active: true,
        }
    }

    /// 将这个源的拾取活动标记改为 `false`，保留其余配对信息。
    ///
    /// # 参数
    ///
    /// - `self`：要标记的源，按值移入；其拥有的快照继续保留在返回值中。
    ///
    /// # 返回值
    ///
    /// 返回相同相机、视口、尺寸与快照的非活动源。快照的 `is_active` 保持原值。
    ///
    /// # 调用时机与生命周期
    ///
    /// 把该源加入下一次相机列表前使用，随后 [`RayMap::rebuild`] 会跳过它。
    /// 该调用不关闭相机、不释放视口，也不更改其它克隆或已经计算的射线集合。
    ///
    /// # 参考
    ///
    /// 源活动标记在下一次 [`RayMap::rebuild`] 时读取。
    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// 每帧由活跃相机与视口内指针位置构造的射线集合，供所有选取后端共享。
///
/// # 存储与准入
///
/// 每个完整 [`RayId`] 最多保存一条射线。默认集合为空；克隆拥有独立条目存储。
/// [`Self::rebuild`] 根据输入配对和受检射线结果写入；[`Self::insert`] 原样写入调用方的值，
/// 使用直接插入时须自行维持 [`PointerRay::new`] 所描述的射线约束。
///
/// # 生命周期与参考
///
/// 集合拥有键和射线数值，不保活相机、视口或指针资源。
/// 清空和重建丢弃旧条目但保留已分配容量，同一实例可继续使用这些存储。
/// [`Self::iter`] 的顺序由散列表决定，不能用它还原输入次序或命中优先级。
pub struct RayMap {
    /// 拥有完整射线键与数值结果；条目数量和已分配容量是不同指标。
    ///
    /// [`Self::rebuild`] 先清空条目再复用或扩展容量；[`Self::insert`] 也可直接写入值。
    /// 容器不保存输入源借用，扩容可能重新分配；集合销毁时释放仍由它持有的表存储。
    map: HashMap<RayId, PointerRay>,
}

impl RayMap {
    /// 移除全部射线条目，为下一次写入保留已分配容量。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用要清空的集合，调用时旧条目的共享借用须已结束。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回 `()`；随后 [`Self::is_empty`] 为 `true`。旧键和值被丢弃，容量仍可用于
    /// 后续 [`Self::rebuild`]，清空不会处理相机资源或指针交互状态。
    ///
    /// # 调用时机
    ///
    /// 需要丢弃当前射线、并为后续写入保留表容量时调用。
    ///
    /// # 参考
    ///
    /// 继续生成当前输入射线见 [`Self::rebuild`]。
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// 原样保存一条射线，并在完整键重复时替换旧条目。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用要写入的集合。
    /// - `id`：按值传入的完整 [`RayId`]，由调用方保证身份与射线对应。
    /// - `ray`：按值转移给集合的世界空间射线，调用方应维持 [`PointerRay::new`] 的数值约束。
    ///
    /// # 返回值
    ///
    /// 该键此前存在时返回拥有旧射线的 `Some`，否则返回 `None`；新射线都会保存在集合中。
    /// 返回值表示替换情况，函数不验证身份资源或射线数值。
    ///
    /// # 调用时机与生命周期
    ///
    /// 手工准备后端输入时使用。相同三维键只保留最后写入的值，之后可用 [`Self::get`] 查询。
    /// 返回旧值后由调用方拥有它；忽略返回值会丢弃旧射线数值。
    ///
    /// # 参考
    ///
    /// 读取当前值见 [`Self::get`]，射线受检构造见 [`PointerRay::new`]。
    pub fn insert(&mut self, id: RayId, ray: PointerRay) -> Option<PointerRay> {
        self.map.insert(id, ray)
    }

    /// 按完整射线键借用当前保存的射线。
    ///
    /// # 调用时机
    ///
    /// 在重建或手工插入后查询精确组合，须在下次可变修改前结束引用使用。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用当前集合。
    /// - `id`：查询时借用的 [`RayId`]；相机、指针与视口三项都参与匹配。
    ///
    /// # 返回值与生命周期
    ///
    /// 找到条目时返回 `Some` 引用，其有效期受当前集合借用约束；查无键时返回 `None`。
    /// 函数不保存 `id` 的借用、不克隆或移走射线；之后修改集合须先结束返回引用的使用。
    ///
    /// # 参考
    ///
    /// 准备或替换条目见 [`Self::insert`]。
    pub fn get(&self, id: &RayId) -> Option<&PointerRay> {
        self.map.get(id)
    }

    /// 读取当前集合中完整键互异的射线条目数量，供后端预估和诊断统计。
    ///
    /// # 调用时机
    ///
    /// 在当前结果已生成后读取数量状态；查询不触发重新采样或外部资源验证。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用当前集合；本次读取不修改条目或表容量。
    ///
    /// # 返回值与使用约束
    ///
    /// 返回调用时的条目数。它不是指针数量、相机数量、已分配容量或
    /// [`Self::rebuild`] 估计的潜在组合数量；同键覆盖不增加该计数。
    ///
    /// # 生命周期
    ///
    /// 只共享读取集合，返回的 `usize` 独立于借用，后续写入可能改变数量。
    ///
    /// # 参考
    ///
    /// 只需判断有无条目时见 [`Self::is_empty`]。
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 判断当前集合是否没有射线条目。
    ///
    /// # 调用时机
    ///
    /// 在当前结果已生成后读取数量状态；查询不触发重新采样或外部资源验证。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用当前集合；本次读取不修改条目或表容量。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回 [`Self::len`] 是否为零的布尔值，只反映本次共享读取的条目状态。
    /// `true` 不说明表容量已释放，也不说明上游没有相机或指针。
    ///
    /// # 参考
    ///
    /// 清除条目并保留容量见 [`Self::clear`]。
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 借用遍历当前集合的完整射线键与世界空间射线，供后端和报告消费。
    ///
    /// # 参数
    ///
    /// - `self`：共享借用当前集合；本次读取不修改条目或表容量。
    ///
    /// # 返回值与使用约束
    ///
    /// 返回元素为 `(&RayId, &PointerRay)` 的迭代器，分别借用 [`RayId`] 与 [`PointerRay`]。
    /// 散列表不保证遍历顺序；需要顺序或优先级的调用方须自行安排。
    ///
    /// # 生命周期
    ///
    /// 迭代器及返回引用依赖集合的共享借用，不转移条目所有权。
    /// 使用这些借用期间不能清空、重建或可变写入同一集合。
    ///
    /// # 调用时机
    ///
    /// 当前射线准备完成后交给后端或诊断遍历；需要保存数据时由调用方复制自有值。
    ///
    /// # 参考
    ///
    /// 键的三维身份见 [`RayId`]，射线数值约束见 [`PointerRay`]。
    pub fn iter(&self) -> impl Iterator<Item = (&RayId, &PointerRay)> {
        self.map.iter()
    }

    /// 重建当前帧射线；调用方须先提供匹配的视口句柄与局部坐标，旧帧射线不会保留。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用要替换当前结果的集合。
    /// - `pointers`：本次采样的 [`PointerLocation`] 切片，位置须使用所属视口的局部像素坐标。
    /// - `cameras`：与本次采样配对的 [`CameraRaySource`] 切片，输入方负责身份、尺寸和快照一致。
    ///
    /// # 返回值与状态变化
    ///
    /// 返回 `()`，结果保存在集合中。先清除全部旧条目，再按活动源数量和位置数量的
    /// 饱和乘积预留潜在组合容量；已有容量可复用，请求量不等于实际桶容量或最终射线数量。
    ///
    /// 只处理源 `active` 为 `true`、视口句柄一致且位置通过闭区间准入的组合；
    /// 任一零维视口或未通过 [`ray_from_viewport_point`] 检查的组合不会写入。
    /// 快照的 `is_active` 不参与活动源筛选。
    ///
    /// # 调用时机、顺序与生命周期
    ///
    /// 先按相机切片顺序、再按位置切片顺序尝试构造；同一完整 [`RayId`]
    /// 保留本轮最后一次成功写入的值，后来被跳过的组合不会擦除本轮已有值。
    /// 旧条目在任何容量预留或构造之前已被移除；容量问题导致调用未完成时也不会恢复旧条目。
    ///
    /// 适合在后端查询前更新本次结果。两个输入切片仅在调用期间借用，
    /// 保存的射线拥有数值，不随输入源后来变化自动更新。
    ///
    /// # 参考
    ///
    /// 手工原样保存射线的不同准入入口见 [`Self::insert`]。
    ///
    /// # 相机旋转前提
    ///
    /// 相机旋转应满足底层单位四元数运算前提，详见
    /// [`crate::core::math::Quat::mul_vec3`]。启用 `glam-assert`，或在调试断言
    /// 与 `debug-glam-assert` 同时启用时，无效旋转可能在射线计算中触发 panic。
    /// 此时旧条目已清空，本轮之前成功写入的条目可能仍留在集合中；本方法不执行回滚。
    pub fn rebuild(&mut self, pointers: &[PointerLocation], cameras: &[CameraRaySource]) {
        self.map.clear();
        // 旧条目已经清空，保留的表容量可复用；预留量只按活动源估计潜在组合，
        // 后续视口匹配与射线准入会减少实际条目。预留失败也不会恢复旧帧结果。
        let active_camera_count = cameras.iter().filter(|camera| camera.active).count();
        self.map
            .reserve(pointers.len().saturating_mul(active_camera_count));
        for camera in cameras {
            if !camera.active {
                continue;
            }
            for pointer in pointers {
                if pointer.viewport != camera.viewport
                    || !pointer.is_inside_viewport(camera.viewport_size)
                {
                    continue;
                }
                if let Some(ray) = ray_from_viewport_point(
                    &camera.snapshot,
                    camera.viewport_size,
                    pointer.position,
                ) {
                    // 只有成功构造才覆盖同键：相机在外、指针在内的输入顺序决定最后成功值；
                    // 后来被跳过的组合保留本轮已有条目，散列表遍历仍不承诺该顺序。
                    self.map.insert(
                        RayId::new(camera.camera, pointer.pointer, camera.viewport),
                        ray,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "ray_map/tests/rebuild_capacity_tests.rs"]
/// 挂载 [`RayMap::rebuild`] 的空输入结果与生产源码预留表达式检查。
///
/// 性能采样测试带忽略标记，只有被显式选中时才运行；其样本每次新建映射。
/// 本模块的测试条件独立于库根全局测试入口的 native-host 条件。
mod rebuild_capacity_tests;
