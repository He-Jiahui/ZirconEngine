# M5 Nanite-Like Virtual Geometry 全链收束计划

## 摘要
- 目标是把当前 M5 `Virtual Geometry` 从“cluster/page/frontier 教学型骨架”推进到“Unreal Nanite 风格的数据驱动虚拟几何系统”，同时保持它仍是 SRP 中的 opt-in `RenderFeature`，不新开平行渲染架构。
- 设计基线采用“双参考分层”：SRP/Feature/Pass/RenderGraph 组织方式跟随 Unity URP/HDRP；层级剔除、`HierarchyBuffer`、`ClusterPageData`、`VisBuffer64`、`NodeAndClusterCull`、硬件光栅化流程跟随 UE5.5.4 Nanite。
- 实施顺序固定为“Zircon 自生 Nanite 数据优先 -> 单线程教学链路 -> GPU 最小系统 -> 自动化 LOD/剔除 -> 并发提速 -> UE5.5.4 兼容验证”。UE 数据导出与解析是验证链，不是首个阻塞项。
- 双宿主同步支持 `runtime_preview` 与 `editor viewport`。所有当前后端同日可用，但通过统一外部接口 + 三档内部执行模式实现：`CpuDebug`、`CompatGpu`、`FlagshipGpu`。

## 接口与边界
- 公共特性继续沿用 `Virtual Geometry` 命名，不新增用户可见的 “Nanite” 顶层 feature；`Nanite` 只作为 `Virtual Geometry` 内部实现族与调试标签。现有路线图和 capability 开关保持不变。
- `ModelPrimitiveAsset` 从“仅 vertices/indices”升级为“基础网格 + 可选 VG cooked 引用”。新增 `VirtualGeometryAsset`，其最小稳定内容固定为：`HierarchyBuffer`、`ClusterHeaders`、`ClusterPageHeaders`、`ClusterPageData`、`RootPageTable`、`RootClusterRanges`、`DebugMetadata`。基础网格仍保留，作为兼容回退和对拍来源。
- `RenderVirtualGeometryExtract` 从当前粗粒度 `clusters/pages` 描述升级为实例驱动输入：`vg_instances`、UE 兼容相机参数、页面请求预算、调试状态 `forced_mip`、`freeze_cull`、`visualize_bvh`、`visualize_visbuffer`、`print_leaf_clusters`。
- 运行时内部新增统一执行模式 `VirtualGeometryExecutionMode::{CpuDebug, CompatGpu, FlagshipGpu}`，由 capability 决定实现细节，但对外暴露同一套 stats、debug UI、host 行为。
- 当前骨架中可复用的部分继续保留并吸收：页预算、frontier/cooling、resident slot、统一 submission authority、stats/readback。当前承载点是 [virtual_geometry_runtime_state/mod.rs](E:/Git/ZirconEngine/zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/declarations/virtual_geometry_runtime_state/mod.rs)。
- 当前基于 mesh/index 切片生成 indirect args 的教学 shader 只保留为兼容过渡，不再作为 Nanite 主实现继续扩展；主路径将替换为真正消费 `HierarchyBuffer + ClusterPageData + VisBuffer64` 的 cull/raster 管线。现状入口见 [virtual_geometry_indirect_args.wgsl](E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl)。
- 资产入口固定挂在 [model.rs](E:/Git/ZirconEngine/zircon_runtime/src/asset/assets/model.rs) 上的可选 VG cooked 引用，不替换现有基础 mesh 资产。

## 实施计划
- `N0 体系收束与文档`
  - 先补齐一份 Nanite 设计说明，固定回答理论项：Nanite 目标、观察者视角、Object 视角、无极 LOD 核心数据、四叉 BVH、LOD 主干逻辑、`StoreCluster/VisitNode` 语义、`VisBuffer64` 作用。
  - 在 SRP 中固定 pass 所属关系：`VirtualGeometryPrepare -> PageUpload/Completion -> VisBufferClear -> NodeAndClusterCull -> HardwareRasterization -> Depth/DebugOverlay -> SwapChainPresent`。
  - 双宿主共用同一 pipeline feature 与 debug model；编辑器额外提供 BVH 可视化、叶子 Cluster/Mip dump、热键调试。

- `N1 开发环境与 Zircon 自生数据链`
  - 建立 `VirtualGeometry` cook 工具链与样例场景，先从普通三角网格生成 Zircon 自生中间格式，不依赖 UE 原始 Nanite blob。
  - cook 算法第一版固定为：按 cluster 切分原始网格，生成逐层简化的 cluster LOD，构建 4-ary BVH，计算节点/cluster bounds 与 screen-space error，按 page 打包 cluster payload。第一版优先可解释性和稳定性，不追求与 UE 完全同构的压缩率。
  - 同时提供教学用导出格式：文本/二进制 dump、BVH 图输出、叶子 cluster 清单、page 映射、Mip 分层视图，支撑后续单线程对拍。

- `N2 单线程 Nanite 教学系统`
  - 先实现 CPU 单线程 reference runtime，完整覆盖：初始化全局数据源、输入输出资源、`HierarchyBuffer` 解析、BVH 全节点遍历、按层遍历、叶子 cluster 打印、MipLevel 打印、`Mip=10` 过滤、`NaniteMesh -> Page` 映射、已加载分页后的叶子验证。
  - 加入 `forced_mip` 手动控制与键盘上下箭头切换，先打通“单线程 LOD 选择 -> 数据打印 -> BVH 可视化”闭环。
  - 这一阶段同时把视图与投影矩阵切到 UE 兼容版本，并收束 view matrix 的空间变换流程；所有自动化 LOD 参数先在 CPU reference 上 dump 和解释。

- `N3 GPU 最小系统与 VisBuffer64`
  - 新增 `VisBuffer64` 抽象，逻辑接口统一为 64-bit 可见性结果；旗舰后端可用单资源实现，兼容后端允许拆成两份 32-bit 资源，但 debug/readback 行为必须一致。
  - 落地帧首 `VisBuffer64` 清空、`HardwareRasterizationPass`、深度测试、ClusterPageData/Cluster decode、首个三角形渲染、首个 Cluster 渲染、多个 Cluster 渲染、像素标记与启动参数输出。
  - 使用 `ClusterSelection` 驱动光栅化，不再依赖教学型 mesh slice 近似。最小验收是打通“LOD 选择结果 -> ClusterSelection -> Hardware Rasterization -> VisBuffer64 可视化 -> Present”。

- `N4 自动化 LOD 与层级剔除`
  - 在 CPU reference 已对拍后，落地 `VisitNode` / `StoreCluster` 两个主干函数与 `NodeAndClusterCullPass`。算法固定包含：frustum reject、背后/近平面 reject、screen-space error 判定、split/keep 决策、parent residency fallback、cluster budget/instance budget 上限、per-frame feedback 收集。
  - 先做单 pass 层级剔除，结果稳定后再切多 pass 分层 BVH 剔除，保留按层输出 Cluster 的调试能力。
  - 新增 `NaniteGlobalStateBuffer`，统一承载相机、矩阵、误差阈值、节点/cluster/page 上限、当前 debug 开关、自动 LOD 参数、每帧监控计数；按需 dump 自动化 LOD 切换相关参数。
  - 加入相机运动与 previous-frame 数据，使自动 LOD 具备稳定性与后续并发路径所需的完整输入。

- `N5 与现有 VG runtime 吸收合并`
  - 当前 frontier/residency/page table/slot authority 骨架不推倒重来，而是逐步把“真实 Nanite page/cluster payload”接入现有 runtime 主机状态与 completion 流程。
  - visibility 负责产生 authoritative node/cluster worklist，runtime 负责 resident page、upload、replacement、completion，renderer 负责 visbuffer/raster/depth resolve，stats/readback 继续统一归口。
  - 旧的“基于 segment/draw_ref 的教学 raster”在所有最小系统测试通过后退化为 compat fallback，只服务低能力或调试模式。

- `N6 并发执行与满帧率路径`
  - CPU 单线程路径永久保留为 golden reference。GPU/并发路径必须先与它在 mip 选择、cluster 选择、page request、visbuffer 结果上对齐，再允许切默认。
  - 旗舰路径改为持久化工作队列或等价并发遍历，形成 UE 风格 `Persistent NodeAndClusterCull`；兼容路径允许用更小 workgroup、分层 dispatch 或 CPU 参与 compaction，但外部行为一致。
  - 性能验收固定为：在选定教学场景和参考桌面 GPU 上，运行时与编辑器视口都能进入 VSync 限幅状态；具体硬件和分辨率在实施时记录到验证文档，不把它写死在渲染逻辑里。

- `N7 UE5.5.4 对拍与兼容导出`
  - 在 Zircon 自生链稳定后，再补 UE5.5.4 导出工具，导出 `HierarchyBuffer`、Cluster/Page 元数据、PageData、教学样例 Mesh 与 BVH 图。
  - 先做“导出到 Zircon inspection format”的验证链，而不是直接让 runtime 依赖 UE blob。对拍项固定为：层级节点、四叉 fanout、叶子 cluster、page 映射、MipLevel、自动 LOD 参数、可见 cluster 集合、visbuffer 标记结果。
  - 这一步是 correctness 与教学验证增强，不回退前面已经收束好的 Zircon 自生资产与 runtime 所有权。

## 测试与验收
- 资产/cook
  - 同一 mesh 重复 cook 生成稳定的 4-ary BVH、单调层级误差、稳定的 page 映射和 cluster 顺序。
  - 教学 dump 能独立重建 BVH 图、叶子 cluster 列表、Mip 分布和 page residency 视图。
- 单线程 reference
  - `HierarchyBuffer` 解析、全节点遍历、按层遍历、叶子打印、`Mip=10` 过滤、手动上下键切换 Mip 全部可自动回归。
  - UE 兼容 view/projection 变换下，CPU LOD 选择结果在 runtime preview 与 editor viewport 一致。
- GPU 最小系统
  - 每帧开头 `VisBuffer64` 被正确清空，首个三角形、单 Cluster、多 Cluster、深度测试、像素标记与 Present 都有截图/读回验证。
  - `ClusterSelection` 到 `HardwareRasterization` 的启动参数、Cluster decode、page load 状态可读回且与 CPU reference 一致。
- 自动化 LOD / 剔除
  - `VisitNode` / `StoreCluster`、单 pass cull、多 pass layered cull 在相同输入下得到一致 cluster 集合。
  - 相机运动、矩阵切换、missing page、parent fallback、budget clamp、forced mip、freeze cull 都有明确回归。
- 多后端/双宿主
  - `CpuDebug`、`CompatGpu`、`FlagshipGpu` 三种执行模式输出同一套 debug/stats，并在容差内给出一致的 mip/cluster/visbuffer 结果。
  - runtime preview 与 editor viewport 同场景同帧下不允许 host 分叉。
- UE 对拍
  - UE5.5.4 导出的 hierarchy/page/cluster 数据能在 Zircon inspection 工具中还原，同一视角下的 node 访问序、leaf cluster 集合与 Mip 选择在容差内一致。

## 假设与默认选择
- `Virtual Geometry` 继续是 M5 的 opt-in feature；不会改成基础渲染必经路径。
- “全后端同日兼容”解释为统一外部接口和调试面，不要求所有后端首日都实现完全相同的 GPU 内核。
- 首阶段优先做可解释、可视化、可对拍的 Nanite-like 系统；压缩率、页格式极致优化、UE 字节级同构不是第一验收目标。
- 当前 active M5 VG frontier/residency/authority 收束工作视为可复用基础，不与其方向冲突；本计划只替换其上层的 cluster 数据来源、层级剔除和 visbuffer/raster 主链。
