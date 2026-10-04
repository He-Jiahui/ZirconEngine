---
status: in_progress
review_date: 2026-09-07
source_recheck_required: true
plan_sources:
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/00-ui-architecture-performance-reassessment-2026-09-02.md
  - docs/plans/optimize/zircon_runtime/300-runtime-engineering-gap-synthesis-review.md
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/optimize/zircon_editor/300-editor-engineering-gap-synthesis-review.md
  - docs/plans/optimize/zircon_runtime_interface/300-interface-engineering-gap-synthesis-review.md
---

# Astra 全域修复与性能达标计划

## 目标与范围

覆盖 `docs/plans/optimize` 中全部非 tooling 能力缺口，以 Windows 产品验收为主，包含必要的 `zr_vm`、WOC 依赖修复，以及本地可部署的账号、团队、商城和云同步服务。保留现有架构和有效实现，按依赖推进高级能力。用户已明确授权全域实施，因此本轮不限于 MVP 修补。公网运营、商业上线和 tooling 改进不在范围内；本次接续遵循协调器退役规则；本地验证证据与 Jenkins、里程碑及产品验收分别记录。

五条只读审查线已复核关键计划、生产调用链和现有测试；这不代表逐篇重审或全目录验收。最初审查 HEAD 为 `9588d9dcfd44fadd9f841ff0abeed4a65493ca07`，本次实施接续时观测 HEAD 为 `084c50501b6cb3c4f794b36e2c64c4af977aa4dd`。共享工作树包含大量其他会话修改，HEAD 不能单独代表待验证源码。每批验证必须封存内容和依赖指纹、实际命令、预期测试集合、实际执行数量和结果；不得覆盖或将外部 dirty 内容冒认成本轮交付。

产物按责任分为四类：本计划保存 finding 映射、优先级和依赖；`features` 保存能力、安全和一致性修复；`performance` 保存预算、等价 baseline 和测量；`layouts` 保存布局、交互及视觉验收。每项保留原 finding ID、当前源码证据、最低责任模块、实施范围、测试和性能门槛。

2026-09-05 目录清点如下，数字是物理文档数，不是独立问题数或逐篇完成审查的声明。

| 模块 | 顶层 Markdown（含索引） | 全部 Markdown | failure 文件 |
|---|---:|---:|---:|
| zircon_app | 11 | 22 | 0 |
| zircon_editor | 277 | 943 | 3 |
| zircon_hub | 8 | 23 | 0 |
| zircon_plugins | 23 | 64 | 2 |
| zircon_runtime | 254 | 1386 | 20 |
| zircon_runtime_interface | 22 | 130 | 4 |

## 计划与实现复核原则

1. 同一 canonical finding 的重审、alias 和 failure 只算同一个问题。旧报告的 Open/Partial/Closed 必须回到当前生产调用链和回归测试复核。
2. 分别记录 `confirmed_open`、`partially_implemented`、`obsolete_finding`、`implemented_pending_validation`、`accepted`。源码存在、静态计数、测试未运行、编译排队均不能升级为 accepted。
3. 新增建议必须包含触发条件、用户影响、最低责任模块、修复边界和可重复验收。缺少 baseline 的性能项先测量，不承诺相对 Unreal 或其他引擎的速度。
4. 基础可靠性优先，但高级渲染、完整领域 provider、VM 和 Hub 服务属于本轮完整目标。依赖未就绪的下游能力登记为待实施，不能删除范围，也不能用新增 facade、mock 或 capability 名称作为完成证据。
5. 当前源码复核与动态验收分别记账。重复报告通过 canonical finding 合并；仅登记到目录或综合报告的部分仍是 `recheck_required`。每个进入实施的子域先补完整责任映射，不能将聚合表当作全部问题已经去重。

## 当前证据状态

| Finding | 复核后的状态 | 当前证据与最低责任模块 |
|---|---|---|
| ABI-A1/A2/A3 | `implemented_pending_validation` | Host foreign-output unsafe 合同、checked deadline、profile files 预分配准入已有源码及回归；App/Editor 消费者和 doctest 仍须在同一快照通过 |
| LIFE-A2/A3 | `implemented_pending_validation` | App 已过滤旧 WindowId；dynamic session 已限制 action/wake 等待；task graph、module、process-log 与 owner cleanup 的共享绝对 deadline slice 已落源；受管 Cargo、真实 Windows host/DLL 与完整销毁证据仍待验证 |
| LIFE-A4 | `partially_implemented` | `shutdown_until_does_not_restart_budget_for_later_scope_drains` 补充多 scope 阶段预算回归；非合作式 module/FFI callback、Session owner command queue、TLS 硬截止时间及全部 ABI 消费者迁移仍属于 `runtime/07-lifecycle-deadline-and-census.md` 的未完成范围 |
| ASSET-A1 | `implemented_pending_validation` | manifest load 已有 metadata 预检和 max+1 有界读；保存入口另按 ASSET-A4 核对，不能据此关闭 importer 输入问题 |
| ASSET-A4 | `implemented_pending_validation` | `asset/project/manifest/save.rs` 已在原子写前以共享 4 MiB 上限拒绝超长序列化文档；exact-limit 保存重开及 +1 拒绝后旧文件不变的回归已落源。受管 Windows Cargo 执行仍待外部依赖恢复；见 `docs/plans/astra/optimize/01/2026-09-11-asset-a4-manifest-save-bound.md` |
| PLUGIN-A1 | `implemented_pending_validation` | Runtime framework 为 selection 生成 typed outcome；首方 Runtime/Editor catalog 复用 resolver，Editor catalog 对非 EditorHost 也保留逐项结果；App 的必选 Runtime/显式 Editor provider 在消费 registration 前检查失败。跨目标回归及静态合同已落源，受管 Rust 与产品 Ready 证据未闭合 |
| PLUGIN-A2 | `implemented_pending_validation` | Editor build catalog 按 `package_role.is_product_catalog_eligible()` 过滤，native preparation 与导出物化拒绝选中的 Sample/TestFixture；受管 Rust 和真实产品目录仍待验收 |
| PLUGIN-A3 | `partially_implemented` | native loader 在打开 DLL 前经 authority 校验身份、target、module kinds、capabilities、manifest 与库摘要，并装载受限 staged 路径；签名依赖闭包、真实 Windows DLL/ABI 不兼容和失败后旧代保留仍须产品验证 |
| UI-A1/A2 | `implemented_pending_validation` | 根尺寸在比较与保存前规范化；虚拟列表在物化副作用前复用 4096 槽预算；已有边界及 no-op 回归 |
| ED-A1..A4 | `implemented_pending_validation` | 有限几何预算与 drawer mount 变化判定已有源码；原生最小尺寸、逐叶分屏投影已落源但 renderer 仍为单 stream，Windows 视觉证据未完成（见 ED-A5/A6） |
| HUB-A1..A4 | `implemented_pending_validation` | 启动失败关闭操作、草稿串行屏障、目录路由隔离、紧凑导航同源宽度已有实现；旧事件顺序由 HUB-A5 补齐 |
| HUB-A5 | `implemented_pending_validation` | `HubRuntimeSession` 在锁内分配 epoch/revision，Web `hubStateChronology.ts` 以 BigInt 严格拒绝旧事件与重复版本；Chrome 浏览器回归 10/10（含延迟事件与四档布局）通过。受管 Rust 测试和真实 Tauri 发布顺序仍待验收；见 `docs/plans/astra/features/hub/01-state-publication-order.md` |
| RG-A1/A2 | `implemented_pending_validation` | const 比较修复及精确前驱 state plan 已实现；区间合并和紧凑纹理范围已落源（见 RG-A3/A4），受管回归及 release 配对测量仍待验收 |
| Plugin manifest ID/path 不一致 | `implemented_pending_validation` | discover/load 已拒绝不一致 ID/path；authority 还检查摘要、信任与能力，真实 Windows DLL 和依赖闭包验收仍开放 |
| Runtime DLL BuildSet 完全缺失 | `obsolete_finding` | 当前已有 manifest/digest/target/schema/host hash 验证与 staged library admission；签名生产工具互操作和真实 DLL fixture 仍待验证 |
| Editor export terminal | `implemented_pending_validation` | 复用 `features/editor/04-export-terminal-outcome.md` 的在途修复，先核对所有权及最新内容，不重复委派 |

Runtime 最新既有批次在 lib-test 编译阶段遇到 278 个迁移错误，未进入测试和性能采样；其最低支持层由现有 `features/runtime/04-migrated-support-validation.md` 负责。该数字是该次终态日志的事实，不是当前错误总数。ABI profile visitor handoff 也必须由后续受管终态关闭。

## 已直接核对的能力缺口

| 项目 | 当前源码证据 | 当前判断与处理 |
|---|---|---|
| Headless 产品循环 | `target-server` binary 与固定 owner 线程上的真实 DLL session/tick/drain/stop 已落源 | `implemented_pending_validation`；见 `features/app/01-headless-product-host.md`，真实 Windows BuildSet、退出和性能证据仍待执行 |
| UI 模块执行驱动 | `UiRuntimeDriver::from_core` 消费 `UiConfig::enabled`；产品 composition 激活 UiModule，`RuntimePreparedProject::load_runtime_ui_surfaces` 在加载 asset/font/layout 前进入 driver 并执行 `admit_project` | 准入和关闭接线为 `implemented_pending_validation`；Runtime300 的完整 frame/input/text/IME/无障碍退出门槛仍需逐项实施与产品验证 |
| WOC 产品入口 | `woc_runtime` 已有受能力约束的生产 `ZrVmProjectVm` adapter；headless、server、client state-only 与 bot local-deterministic main 已分别接入真实 ZrVM/事务或固定 tick driver | WOC-APP-P0-003/P0-004 由 `confirmed_open` 重判为 `partially_implemented`；各角色的真实本地 tick 与 checkpoint/replay 仍待受管 Cargo、外部依赖、client presentation、网络/持久化和 deterministic product 验收，见 `features/app/01-headless-product-host.md`、`02-woc-server-product-host.md`、`03-woc-client-product-host.md`、`04-woc-bot-product-host.md` |
| 线程观测边界 | `core/runtime/tasks/task_graph/engine_task_graph.rs::worker_inventory` 仅统计 graph 所有的三种 pool | 保留局部 owner 语义；Runtime 全线程 census 尚不能用此方法证明 |
| 整体完成状态 | 最新综合报告明确未运行 Cargo/GPU/产品压力验证 | 复用其问题分类，不复用未经当前快照验证的完成结论 |

## 首批修复与责任边界

| 优先级 / Finding | 当前源码证据与最低 owner | 实施与直接验收 |
|---|---|---|
| P0 / PLUGIN-A1 | Framework resolver 与两个 catalog 已返回 typed report；Editor catalog 非 EditorHost 早退丢失 outcome 已修，App checked consumer 已接入 | 保留 invalid/duplicate/unsupported、目标跳过、required fail-closed 回归；受管 catalog/App Rust 测试及产品 Ready 路径待验收 |
| P0 / PLUGIN-A2 | `manifest_completion/native.rs` 按 typed role 过滤；native preparation、ZIP/目录 export 均预先拒绝不合资格包 | 验证生产插件保留、Sample/TestFixture 无副作用排除及真实产品目录；不能以名称黑名单替代角色 |
| P0 / PLUGIN-A3 | `native_artifact_trust::admit` 的身份、摘要、信任及能力准入先于 `load_discovered::load_admitted_library` | 验证签名依赖闭包、ABI 不兼容、Windows DLL fixture 与失败后旧代可用，缺少产品终态时仍为部分实现 |
| P1 / LIFE-A1/A4 | task graph 已固定 Closing 的强引用 scope 集合并保存最终 census；重复关闭和丢弃外部句柄的回归已有源码；session action/wake、scope、module、graph/log 已接入共享 deadline 候选 | 状态为 `implemented_pending_validation`；受管回归、非合作式 callback、owner 终态与真实 Windows 全进程销毁仍待验证，不重复实现已有 census 修复 |
| P1 / ASSET-A4 | `asset/project/manifest/save.rs` 已在原子写前复用 load 的 4 MiB 限制；状态为 `implemented_pending_validation` | 保留 exact-limit/+1、保存重开及拒绝后旧文件完整回归，待受管 Windows Cargo 实际执行通过后方可验收 |
| P1 / ASSET-A2/A3 | glTF 与 compound 已有同代快照和 2 GiB/65,536 成员上限；compound 全量及 targeted 收集于遍历期限制成员数，普通资产扫描已跳过已识别的 compound 子树；元数据预遍历仍重复，实际打开次数与峰值待验收 | 保持 importer 消费准入快照、哈希与 companion 同代；完成 64-file 单成员打开次数、链接替换、文件数/字节预算及失败原子性验证 |
| P1 / RG-A4 | `render_graph/builder/access_scope_tracker.rs` 每次 buffer 更新扫描全部区间；texture 先枚举所有 cell | 只合并受影响的相邻区间；计数覆盖真实遍历；先验证纹理形状并紧凑表达范围，10k disjoint 不再平方退化 |
| P1 / RG-A3 | resource streamer 在 geometry resolution 前按 world/journal generation replay | 资源变更游标与 resource-to-primitive 索引触发定向更新；游标缺口有界恢复，一次变化一次 dirty，稳定帧零解析 |
| P1 / UI-A3 | base/projected hit-grid patch 对同一 cell 反复 retain/insert | 按受影响 cell 合并整批修改；保持 painter order、完整结果、COW 和失败原子性；每个 cell 一次处理 |
| P1 / ED-A5/A6 | 逐叶投影、原生 minimum-size 与空 retain 退役已有源码候选；状态为 `partially_implemented` | 当前回归尚未执行；仍须逐叶 camera/settings/focus 保存恢复、精确分屏树与比例、toolbar/input/chrome 隔离、短窗口/DPI 和真实 Windows/WGPU 验收 |
| P1 / HUB-A5 | 状态 owner 已分配 epoch/revision，Web 已统一比较 bootstrap、response、event 并以 BigInt 保持无损版本；状态为 `implemented_pending_validation` | 保留旧事件/旧 epoch/重复版本回归，等待受管 Rust 测试与真实 Tauri 发布顺序验收；浏览器夹具通过不等于原生通过 |
| P0 / WOC-APP-P0-005 | WOC Zr 状态编码器写 WOS118，而解码准入只接受至 WOS117；main 的 113 与 native 的 83 是同一 opaque WOS 格式的过时身份声明 | WOS118 reader/writer 尾部已对称，修复版本准入并统一活跃身份；保留独立的外层 WOC wire 版本，验证实际 Zr/native round-trip 和 WOS117 显式迁移 |
| P0 / PROTOCOL-P0-002 / WOC-SVC-P1-030 | `Command` 编解码及 server ingress 绕过已有 typed payload validation | known command 的 encode/decode、FixedTick 入队、replay 统一语义准入；非法输入不改队列、不进入 VM，合法批量 p95 回退不超过 5% |
| P0 / WOC-CLIENT-P0-003 | client timeline publish 失败后仍消耗 accumulator、commands、movement 和 tick | authority/VM 使用真实 prepare/commit token；timeline 校验失败丢弃 candidate，所有可观察状态保持不变并可重试 |

首批最多八条实施线：生命周期、资产输入、RenderGraph、资源几何 replay、UI hit-grid、Editor、Hub、插件。每条先写自己的编号子计划和独占文件，再实施；独立复审与验证 lane 不占实施 owner。接口由唯一 owner 修改，消费者随同迁移。同一文件已有外部租约时留下准确 handoff 并继续独立文件，不争抢或回滚。

## 实施后的复审与续接

以下状态仅对应本轮已复核源码。隔离候选在回合并前必须再次核对基线，已封存输入不能被后续修改覆盖。

2026-09-19 的首批 P0/P1 独立合同复核覆盖插件准入与产品角色、hit-grid
预算/反向索引/批量原子性、RenderScene/资产边界、Editor 原生窗口最小尺寸、
Hub 热路径以及 Frameworks05 方向边界；选定测试均通过。该复核仅证明当前
源码合同与静态行为，不能替代受管 Cargo、真实 DLL/Tauri/WGPU、产品窗口或
release p50/p95/p99 证据；完整 Frameworks05 仍保留 Text03 两项已知失败，
不在本轮越权修改。

沿用 `astra-full-domain-20260905` 的主计划归属。Headless、Hub 服务和 WOC 协议/事务候选已进入主树，后续验证须包含当前源码及完整依赖闭包；共享 dirty 工作树或历史 HEAD 均不替代受管封存输入。生产入口、跨模块生命周期和产品输出仍按下表逐项验收。

当前独立实施范围为 Core/Headless、资产输入与包恢复、Graphics、UI runtime、Editor 分屏、Hub 服务、插件准入和 WOC/VM；另设受管验证与独立复审。LIFE-A1 已有 frozen scope/census 回归；RT213 生产 collector 已消费 prepared mesh bounds，缺失或无效边界保留可见并进入空间索引 overflow 查询；WOC snapshot 安装与热重载已采用强制完整 checkpoint/rollback 合同。以上均保留未验收状态，真实 VM adapter 的隐藏状态恢复能力仍需实现。各线先声明精确写入范围并取得租约，根清单与 Cargo.lock 统一协调。

| 范围 | 当前证据状态 | 复审后的下一动作 |
|---|---|---|
| ASSET-A2/A3/A4 | `implemented_pending_validation` | 同代输入、显式根目录、打开句柄身份、Windows 大小写引用已有实现；glTF 共享解码预算与 image view 范围检查已落源。compound 全量及 targeted 入口在收集阶段限制 65,536 成员，普通资产扫描跳过 compound 子树；元数据预遍历重复、实际打开次数、峰值内存、ProjectManager 重新导入、删除竞态及失败回滚仍待受管验证或修复 |
| RG-A3/A4、UI-A3 | `implemented_pending_validation` | 定向几何 replay、紧凑子资源区间、逐 cell 合并及失败原子性已有修复与回归；RG-A3 静态契约 `16/16`，并移除 BTreeSet 目标上的冗余排序；release p95 门槛仍待实测 |
| ED-A5/A6 | `partially_implemented` | 原生 minimum、持久节点身份、精确分屏恢复、逐叶图像与 pointer/toolbar 路由已落源；renderer 仍为单 camera/session stream，独立 owner 与 Windows 产品验收未完成 |
| HUB-A5、响应式布局 | `implemented_pending_validation` | 最新真实 Chrome `hub_browser_regressions.test.mjs` 10/10、零跳过，覆盖旧事件、bootstrap 故障、四档宽度、全部路由及中英文弹窗；12 张浏览器夹具截图保存在 `target/astra-hub-browser-20260921/`。此结果只覆盖本次浏览器快照，原生 Tauri、本地服务和后续源码需各自验收 |
| Native 签名和依赖闭包 | `partially_implemented` | 不透明签名准入证明及到期策略已独立复审；Windows DLL 主体和依赖的封存装载、现有签名生产工具互操作、真实 DLL fixture 仍需整批验证 |
| Headless 和 LIFE-A4 | `partially_implemented` | Headless 创建到销毁在同一受管理线程，重复 cleanup、TLS join、日志及控制台共用首次关闭 deadline；窗口启动错误保留显式恢复入口；dynamic registry 迁移及运行验证仍未完成 |
| WOC 协议与事务 | `partially_implemented` | 协议准入和强制 checkpoint/rollback/snapshot install 已进入主树；包 schema 类型表、math.scalar 授权及 math 0.3.0 round 已补。Array<uint> ABI 已有生产传输与回归源码；完整 VM checkpoint、有界调用、有效 saveState 和产品 adapter 仍缺失。真实包加载、WOS118 codec、失败后下一 tick 等价性须实际执行 |
| Hub 本地服务 | `partially_implemented` | Windows local-service Rust 测试 78/78、受管服务 binary 构建，以及真实 Keycloak/HTTP 探测 40/40（6 项环境准入、34 项交互检查）已接受，见 `01/2026-09-07-hub-local-service-validation.md`。覆盖 schema v6、ACL、团队、签名目录、CAS 与强制终止后恢复；商城字节下载/安装、native broker、Tauri 和规模性能仍未验收 |

W2 pack promotion 的旧包误删问题已修为 `implemented_pending_validation`：`asset/pack/install/promotion.rs` 通过 `PromotionPaths` 校验并恢复 journal，再使用 `core::resource::io::transaction` 持久提交可选 backup、installed 替换及带摘要验证的 staged 退役。生产 delta installer 在读取 base 前恢复事务，已提交发布可按 `AlreadyInstalled` 恢复 receipt/plugin reload。无 backup 失败保留原件、逐文件崩溃边界、策略不匹配、损坏 journal、路径别名及重试回归已落源；实际回归和 Windows 产品故障恢复仍待受管验证，见 `features/runtime/15-pack-promotion-recovery.md`。

W4 保持原 finding 身份：Runtime185 的 directional/point/spot 场景阴影开关已接入 planner，atlas/cache/assignment 以 light type 与实体 ID 区分同实体多个光源；Rect 请求明确报告不支持，区域光和阴影能力仍开放。Runtime213 的 `RT213-P1-009/010` CPU culling 已接入 prepared local mesh bounds，并补真实空间查询 fail-open 回归，状态为 `implemented_pending_validation`；`09B-P0-6` 的虚拟几何 executor 现已在插件侧完成资源绑定、compute dispatch、feedback copy 以及 VisBuffer/debug draw 编码，并由 GPU shader/encoding 回归覆盖，状态调整为 `implemented_pending_validation`。仍需场景开关到阴影图像、同一离心网格 CPU/GPU 可见性一致、真实 GPU 输出及产品路径验收。已存在的 WGPU 自动同步和 generation submission 防护不再误报为完全缺失。

验证日志记录首个诊断 ticket `ec3063d7ad8f4243ad09126f5b3cf256` 在 Cargo 前的 artifact governance 阶段失败，不能计为编译结果。验证 lane 使用现有显式 isolated state 配置准备下一批，保留原目录与外部 dirty 依赖，不扩展 tooling；实施线继续后续任务。

## 全域能力波次

| 波次 | 能力与顺序 | 产品退出条件 |
|---|---|---|
| W0 | 完整问题去重、当前源码快照、既有修复验收、性能基线 | 每个 finding 有责任模块和证据状态；验证输入可重现，未读范围明确 |
| W1 | Core、ABI、任务、插件准入、产品生命周期 | 启动、取消、超时、热替换、退出终态一致，无遗留任务或跨代发布 |
| W2 | 项目、资产快照、导入、cook、包、World/ECS、持久化 | 创建、保存、重开、迁移、重导入、Play 恢复保持身份与数据一致 |
| W3 | Headless、窗口、时间、输入、UI runtime、文本、IME、无障碍 | 各宿主消费真实运行时服务，多窗口输入和显示生命周期完整 |
| W4 | RenderGraph/RHI、GPUScene、材质、PSO、流送、基础画质 | 实际 GPU 执行、热重载、resize、device-loss、资源回收及图像一致性通过 |
| W5 | VM、游戏框架、物理、动画、导航、音频、网络、AI | 真实项目和双进程场景验证执行、保存、回放、取消、恢复与可观测输出 |
| W6 | 虚拟几何、GI、光追、粒子、地形及其余声明领域 | 各领域具有资产、provider、调度、渲染或其他输出和完整编辑链路 |
| W7 | Editor 分屏、文档事务、资产工具、插件、导出；Hub 服务 | 正常 UI 完成编辑到发布、本地账号到团队、安装及云冲突恢复 |
| W8 | 跨域规模、故障、长时间运行、最终性能优化 | 纳入范围的 finding 全部关闭，完整产品及性能证据通过 |

每波包含实施、合并验证、失败回修和独立复审。依赖顺序约束合同，独立工作不因前一波排队而暂停。首批修复完成不等于 W0-W8 完成。

## 合同与本地服务

共享合同包括 shutdown deadline/receipt、资产快照和预算、资源变更游标、插件选择结果、分屏树、Hub 状态版本。保留既有权威 owner，消费者一次迁移；持久化格式变更提供显式迁移，ABI 不兼容变更按版本准入，不保留隐藏兼容分支。

Hub 本地服务由独立 Rust 进程承载，使用 axum、SQLite 和内容寻址文件存储；身份使用支持 Windows ZIP 部署的 Keycloak。实施前复核仓库 Hub03 的身份、授权、签名目录、安装事务和快照 CAS 合同。UI、服务和身份进程分别有生命周期、故障与恢复测试；本地真实登录、团队权限、目录签名、安装事务和冲突恢复均必须实际执行。不能把现有静态 team/catalog DTO 当作已部署服务。

## 覆盖与未读范围

| 范围 | 本次已复核 | 仍需实施前深审 |
|---|---|---|
| App/Core/Host/Interface | 任务 census、dynamic destroy、ABI 输出、窗口事件、Headless 入口、BuildSet 存在性 | 全进程 shutdown 接线、所有 ABI 家族分配准入、多窗口、完整宿主产品矩阵 |
| Asset/World/Graphics | manifest、辅助源、compound、state tracker、几何 replay | 全 importer 算法、cook/package 崩溃恢复、World/ECS 事务、RHI native lowering、各 GPU provider |
| UI/Text/Editor | 根尺寸、虚拟列表、hit-grid、drawer、有限几何、分屏投影、原生 minimums | 正半径 hit-query 显式预算合同、完整文本/IME/A11y、多文档事务、全部插件工具和产品视觉 |
| Plugins/VM/领域 | 选择和准入链、产品 fixture 混入、已有能力与闭环差异 | 每个领域 provider 的实际算法、VM 和 WOC 外部依赖、双进程执行、热替换及回放 |
| Hub | 启动/设置/目录/导航和状态顺序 | 全路由语言弹窗矩阵、真实原生窗口、账号/团队/商城/云同步服务实现与部署 |

上述聚合范围只用于安排工作。完整 finding 去重仍属 W0 的进行中工作；历史 coverage 的 E3 和测试数不自动继承为本轮验收。

## 性能与产品门槛

- 机器固定为 Ryzen 7 5800H、RTX 3060 Laptop GPU。记录 adapter 身份、驱动、分辨率、画质、workload；不得选中 GameViewer 等虚拟显示适配器作为 GPU 证据。
- 沿用有效既有门槛；一般路径 release p95 回退不超过 5%。保留根目录校验、图集命中、drawer 的专项目标；新增热点以正确性等价基线 p95 改善至少 20% 为目标。
- 预热后交错 before/after，报告 p50/p95/p99、分配、真实访问量、RSS/VRAM。验证 lane 在重编译竞争结束后采样；微基准和静态计数不能替代产品帧及交互证据。
- 正确性覆盖空值、边界、异常输入、并发、取消、超时、重试、设备丢失和崩溃恢复。禁止删测试、静默降级完整结果或通过失败提前返回制造性能提升。
- Hub 覆盖 360/768/1280/1920、长路径、中英文、全部路由和弹窗；Editor 覆盖短窗口、DPI、分屏保存恢复。浏览器截图与真实 Tauri/WGPU 窗口分别验收。

## 异步验证与继续执行

- 使用现有 Windows coordinator 的 immutable source manifest，一次提交多个任务的 package check、回归与性能命令。Cargo 输出只使用受管 D/E/F 目标根目录。
- 持久化批次 request/ticket、输入指纹、命令、预期结果和读取证据的位置；提交后立即继续下一项独立修复或复审，不等待、不持续查询编译状态。
- 同一 package、feature/profile 的多个独立任务汇总一次编译、回归及性能批次；共享接口变化加入全部消费者和必要 doctest。验证日志放在受管状态目录，子计划只保留准确证据链接与验收结论。
- 收到完成通知或后续工作结束时读取一次终态证据。失败从最低支持层修复，仅重提受影响的一组测试。
- 协调器延迟只影响依赖该操作的动作，不把整个 Goal 标成 blocked；不扩展任务去重写 tooling，也不以裸 Cargo 绕过受管验证。
- 不将排队日志写成 accepted。尚无终态证据时保留 `implemented_pending_validation`，Goal 保持未完成。
- 外部 dirty 依赖通过完整隔离快照保留并封存，不清空、回滚原工作树，不修改协调器以放行非封存输入。
- 所有可执行实现工作耗尽后完成最终复审并留下续接记录，释放执行会话交由完成通知唤醒。没有测试实际执行、性能实测及独立复审关闭证据时，保持未完成。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime/editor bounded hotpath repair (M2/M19) | manifest root/load bounds, editor asset/index/export projections, template-pack admission | `implemented_pending_validation` | 2026-09-09 | 108/108 local source-contract tests, Rustfmt and scoped diff-check pass; standalone Runtime85 model and pressure profiles retained in [.codex/state/astra-optimize-20260909-async-validation.md](../../../../.codex/state/astra-optimize-20260909-async-validation.md); immutable managed Cargo/release acceptance remains blocked by dirty `E:/Git/zr_vm` (latest `87c112d27f25e2a10e2c078d61ef638954d0f8eb`, 112 tracked/submodule + 88 untracked = 200) |
| Native plugin trust/inventory (M1/M2) | inventory preflight, selection-scoped authority, Runtime/Editor load and hot reload admission | `implemented_pending_validation` | 2026-09-09 | Runtime06/Editor12/native authority static regression batch `45/45`; public-surface audit `86 symbols / 2 locations / risks=[]`; Cargo, real DLL and release evidence remain pending under the same external dirty snapshot |
| Runtime213 visibility query | static index overflow/cell candidate normalization | `implemented_pending_validation` | 2026-09-09 | `VisibilityStaticIndex::collect_query_keys` removes per-query temporary tree nodes while preserving sorted unique candidates; cross-cell regression, rustfmt, and source contract pass; managed Runtime Cargo and release p50/p95/p99 remain pending |
| Runtime11A/M30 default-scroll scratch | retained fully validated nested-scroll candidate buffer per `UiSurface` | `implemented_pending_validation` | 2026-09-10 | Runtime UI pointer/scroll/hot-path static batch `73/73`, Rust regressions, Rustfmt and scoped diff-check pass; managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending |
| Runtime11A/M31 Taffy child-handle buffer | retained ordered `NodeId` projection reused during topology reconciliation | `implemented_pending_validation` | 2026-09-10 | Retained-product capacity/content regression and focused Runtime UI/Taffy/layout contract batch `77/77` pass; managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending |
| Runtime11A/M32 arranged-visibility scratch | retained iterative resolver state/path buffers bounded to twice the current arranged node count | `implemented_pending_validation` | 2026-09-10 | Rust regression is present for warm reuse, high-water shrink, clone scratch omission and visibility (Cargo execution deferred to managed testing); post-change combined Runtime UI/Editor contract batch `86/86`, Python syntax, and scoped Rustfmt pass; new source/plan files are whitespace-clean; managed Cargo and Windows Release p50/p95/p99 remain pending |
| Runtime11A/M33 Canvas layer capacity | bounded reservations for flattened Canvas layers and per-parent child projection | `implemented_pending_validation` | 2026-09-10 | Existing Canvas behavior coverage plus source capacity guard; combined Runtime/Editor hotpath contract batch `135/135` (0 failures, 0 errors), Python syntax, Rustfmt, and scoped diff checks pass; managed Runtime Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending |
| Editor04/M27 search normalization cache | setter-owned ASCII-lowercase cache for Asset Browser snapshots and catalog patches | `implemented_pending_validation` | 2026-09-10 | Editor asset projection/Watcher static batch `52/52`, Rust regression, Rustfmt and scoped diff-check pass; managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending |
| Editor04/M28 borrowed filter matching | allocation-free ASCII case-insensitive folder/item matching plus folder-tree capacity reservation | `implemented_pending_validation` | 2026-09-10 | Combined Runtime UI/Editor asset and evidence-contract batch `111/111`; Python syntax, Rustfmt, and scoped diff checks pass; managed Editor Cargo and Windows Release p50/p95/p99 CPU/allocation/RSS evidence remain pending |
| Editor04/M29 borrowed folder membership | borrowed `res://`/`package://` parent comparison in visible-item and catalog-patch predicates | `implemented_pending_validation` | 2026-09-10 | Combined Runtime/Editor hotpath contract batch `135/135` passed; focused asset/Taffy recheck `53/53` passed with UTF-8 boundary coverage; Python syntax, Rustfmt, source guard, and scoped diff checks pass; managed Editor Cargo and Windows Release p50/p95/p99 CPU/allocation/RSS evidence remain pending |

## 2026-09-30 当前源码续接

当前主会话为 `astra-goal-20260929-01a0f033`。用户授权后，已核验旧 W0 与 ED-A6 primary 没有关联执行句柄、活跃 Cargo 作业或有效租约，并将其标记 `cancelled`；全部源码与历史证据保留。当前进展、精确 UI 输入哈希、受管验证失败回执和未审范围见 [W0 当前源码续接](01/2026-09-30-w0-current-source-continuation.md)。此记录不关闭 W0–W8。

| Finding / 合同 | 当前状态 | 当前源码与验收边界 |
| --- | --- | --- |
| Runtime11A `P0-2` | `partially_implemented` | 真实单调输入时钟、逐 surface timer tick、frame demand 与 `tick_frame` 接线已有候选；Editor 已补消费 wake 后的失败重试、50ms 至 1s capped backoff，以及 pump 前后 owner/generation 不同的成功 demand 丢弃。十路径候选和生产 `UiHostWindow` 功能回归通过独立源码复审；受管 check 在输入封存漂移时退出，Rust 回归和真实宿主时序仍未执行通过 |
| RG-A4 | `partially_implemented` | 紧凑范围与 plane-visit 计数已有源码；`compile.rs` 仍缺少 `record_texture_plane_work(work.plane_visits)` 聚合，当前新回归不能据此宣称通过；该活跃 owner 路径保留正式 handoff |
| PLUGIN-A3 | `partially_implemented` | host-private policy/installed-slot resolver、Hub typed target 与 Play 必选失败门槛已有候选；旧 owner 核验并按用户授权精确移交后，helper 已集成 target-keyed pinned policy index 与 commit 复核迁移，格式检查通过，binary/Rust 回归待验证。Editor approved-manifest/startup 消费者及 lower host load transaction 仍在私有实施；实际 DLL、Ready 阻止与拒绝 reload 保留旧代仍待验收 |
| Hub03 `P0-05` / S6 | `partially_implemented` | native snapshot/staging/CAS bridge、预算、账户切换与 blocking lease 已有候选；terminal recovery、anchored IO 与 targetless Unknown 迁移通过源码复审。TopBar/HubPanel 修复后的浏览器证据为 native 字面投影/全部路由 17/17、account/catalog/license 16/16、Team 弹窗 8/8，四宽度与中英文分别保留 120/24/32 PNG。无桌面 Broker/Service shipping check 作业 f03235330d1440ebaf9c8c66b148063f 通过。原 24 条测试编译错误已修复，作业 e2708a5ba4e44baa8f3a1e2bd855c7e5 的 check --tests 通过，580 条输入、23 个 native 候选精确匹配；实际 Rust 测试 171 pass/1 fail、零跳过。失败是登录锁等待测试未继续轮询登出 future；并发驱动修复已按原 SID 落源归属，七项相关 Broker 原请求 `8a32bf3d11354f35a67341761e0383fb` 已在 Cargo 前因 foreign Jenkins 未登记产物拒绝，实际执行数为 0，未生成新 Cargo 作业。云恢复 r3 的六个 Rust 路径已在原 Hub SID 精确集成并完成源码字节/归属复核；实际测试数 0，受管原生门仍开放。package 根身份与 target owner 域分离、完整 128-bit 根身份仍为私有候选；独立复审 C-P2-001 要求显式迁移旧 journal owner，原安装实施线正在修复。真实 package-lock digest、冲突恢复、Tauri/Keycloak 产品与性能门槛仍开放；精确证据见当前源码续接 |
| `RT-AI-P1-015` | `partially_implemented` | SetBlackboard/EmitEvent 采用 typed staged batch 与 World receipt；Rust 未执行，跨 load exactly-once 与 service 生命周期仍开放 |
| ASSET-A3 / LIFE-A4 / ED-A6 | `partially_implemented` | one-pass discovery 保留私有补丁；World late-success commit 与空 retain/逐叶路由候选已落源，尚未执行 Rust 回归；per-leaf save/reopen 和真实产品/性能门槛仍开放 |
| WOC / ZrVM checkpoint | `partially_implemented` | Create 边界、prototype/function-cache/GC 自指针及 restore preflight 的私有候选已获 exact-hash 静态复审；仍未正式移交或执行测试，共享外部源码的 stable capture 尚未成立 |

继承库存的 576 个报告路径与 61/515 计数仅是旧快照的有界覆盖记录。27,629 个结构候选不是 canonical finding 数；未读或未去重内容仍保持 `source_unchecked`，所有产品、Windows 和 release 性能门槛保持开放。

新增有界源码复核波次为 37 个 report-qualified 原 ID 行与 12 条显式关系：5 `confirmed_open`、24 `partially_implemented`、8 `implemented_pending_validation`，测试执行数为 0。Runtime157→Runtime01 和 Interface08→Interface01 保留显式 refresh；Runtime72 的 RCL namespace 与 Interface15 certification 独立保留。该波次不提升整篇报告覆盖率、不形成 canonical/accepted 总数，精确 artifact 指纹及剩余验证入口见续接记录。

冻结的逐行证据已导入 [37 行源码复核](01/2026-09-30-source-review-wave.jsonl)、[12 条显式映射](01/2026-09-30-source-review-crosswalk.jsonl) 与 [导入摘要](01/2026-09-30-source-review-summary.json)。初次 10 个源码漂移已逐合同重新复核；导入时的 freshness 仅覆盖该摘要列明的路径/hash，不扩大未读覆盖或接受状态。

第二批已导入 [30 个原报告 ID 的源码复核](01/2026-09-30-w0-second-wave-review.jsonl)、[28 条映射观察](01/2026-09-30-w0-second-wave-crosswalk.jsonl)、[摘要](01/2026-09-30-w0-second-wave-summary.json) 与 [生产清单](01/2026-09-30-w0-second-wave-manifest.json)。12 组材料的 33 条观察只按相同报告路径和 literal ID 分组；两批共覆盖 67 个互不重复的 report-qualified ID。该数字是有界源码核验范围，测试数与 accepted 数仍为 0；576/61/515 库存边界保持不变。

2026-10-01 后续源码记录：W0 第四批增加 24 条 report-qualified 观察、14 条明确关系，前 82 行无碰撞；合计 106 条有界观察，均无本批执行测试或产品接受。未读报告 515/576 与 nested 2,316 文件边界保持不变。Core 的 20 路径组合补丁保留 foreign 事件总线 API，已形成私有源码候选，原 owner 准入、受管回归、Windows 产品和性能仍开放。


2026-10-01 20:42Z 增量已记录在当前源码续接：Hub 38 路径 package-lock/owner-migration/Reconcile UI 私有组合复用准确 TSC 与实际 8/8 JS controller/SSR 回归，Rust/helper/Tauri/Keycloak/产品门仍 open。Hub preview 原请求已终态 failed（服务重启中断），没有移交或部分源码集成。Navigation 21 路径候选补齐 weak World owner、loaded-lifetime epoch 与同步/tiled/dirty terminal publication；实际 Rust 测试数 0，pure operation prepare 与资产 collider provider 仍开放。Asset 44 路径 v4 保留 generic wire 候选，但普通 World 保存/加载仍重建 builtin registry；已返回原实施线补齐正常 provider authority，另保留一项 candidate hash drift，不接受该可变输入。Editor 实际逐叶路由和 durable transaction、Core 完整组合复审与受管回归继续进行。四批 W0 的 106 条有界观察及未读分母不变，原三条 Penpot lifecycles 和 managed lower/original/upward 门保持 open。

## 2026-10-01 22:13Z 已封存增量

W0 第五批准确核对 106 个既有 report-qualified 键后新增 24 项，五批共 130 条有界观察；
24 项无碰撞，报告库存仍为 576/61/515，nested 2,316 未读边界不变。原第五批把报告路径数
与源码路径数相加的错误已在独立 r2 更正，原四个产物保留。
[第五批 r2 manifest](01/2026-10-01-w0-batch5-manifest.json) SHA-256 `1a38ea4fe7233e36646fa3ee98536eb6e7ebebcaff3ee2676bb8e754bc991a5d`。

[浏览器覆盖回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-owner-migration-ui-r1/browser-closure-r2.json) SHA-256 `927aaeca6c73d13bace214ed20fe9b52d5b842f4c0e6ea8f17ade80fa87e2901`：
同一生产输入上十个唯一真实 Chromium 交互场景有实际通过证据，八个修正 harness 后的新
场景与两个未变化守卫分别保存原回执，32 张截图已捕获。缺失 target 的真实 input 为空、
已有 target 只读、busy 拒绝重复、未知 receipt/history error 保留终态。仅为前端受控 IPC；
Tauri/Keycloak/Rust/helper/安装/同步与性能门没有通过。

Navigation 完成准确 21 路径 source-only 独立复审，无 P0/P1；正常 operation pure backend
和全部动态门仍开放。Core shutdown/consumer 的 9 路径复审无 P0/P1 源码缺陷，直接 Session
incomplete-report retention 的 P2 回归继续补齐；完整组合复审和 active foreign Runtime
owner source consent 未闭合。WOC tick successor 形成两路径私有候选与两项行为回归，
[WOC manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-tick-exhaustion-r1/manifest.json) SHA-256 `f5a86d61948183ac01ad2be6bc8215ece235769b09a6951a9aebd46016087685`；
Rust tests 仍为 0，归属、真实 ZrVM producer、lower/original/upward 与产品门保持 open。

r4 正常 attribution 请求 `807fdc7028a047d5b28e08f10f0af6db` failed/internal_error；同 SID 的
DB matching hash 是观察到的局部效果，不能覆盖命令失败。没有重放、手工改元数据、
外部 lease/status 移交或新的 Penpot 注册/验证/route load。原 r5 私有候选保留，本次新映射与文档增量的归属终态仍需正常回执。
详细 hashes、准确范围与剩余门见对应 continuation 记录。


第五批四个 future path 的正常 preview `9badcb0486d74e26bf525f4018b5b9d0` / apply `5a318d80811647e9a4f0b882a25c0c40` 已完成，
只扩展原 W0 Session scope；没有外部 owner、lease 或 status 变更。记录写入与归属命令的最终结果单独保留，
不覆盖 r4 失败终态，也不升级 finding、Rust 或产品 acceptance。


## 2026-10-02 01:30Z 精确源码与未通过门

W0 前五批仍为 130 条有界观察、130 个不同 report-qualified 键。第五批正常 attribution
`b2c67b758029414c93fa4f0244d857fa` 已 completed，见 [第五批终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch5-publication-r1/attribution-terminal.json)。
第六批原 review 实际只有一个换行、零行；r2 虽有 18 行，但 14 个 provisional_key 错指其他
原 ID/报告，摘要也把 36 个证据路径写成 19。两版均未导入，原产物保留，原作者继续不同 r3。
报告库存 576/61/515 和未读边界保持历史含义，不把源码路径或刷新观察算成新报告验收。

WOC App03 `WOC-APP-P0-006` 的两路径精确修复已在原 W0 SID 正常移交、落源并归属：
transaction postimage `1f81e7b0875c0facb1bc1d1132d533a246dae61b99be472ccbd2d52f1d04afd8`，
tests postimage `bdd9efc99e30ba2633de4887a6c5ce93481f526520bfe3a1eb5757e46a976253`。
checked successor 在 encode/checkpoint/VM 前拒绝溢出，普通 ClientFrameDriver outer rollback
保留 terminal fault/retry 资格；独立源码复审已关闭测试计数错误。归属请求
`ac321f04860d4ae1867409e56e3d650d` completed，[WOC 精确终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-transaction-terminal-rollback-r4/attribution-terminal.json)
SHA-256 `a2227f0a76dfb26b8649a9a79318bcb8eb8ce14825850b3caf93151edc995bbd`。
声明的 21 个 lower tests 实际执行数 0；真实 ZrVM producer、原 client presentation、
lower/original/upward、Windows 产品和性能门仍 open。关联 Runtime24 finding 不合并。

Core activation r3 独立复审有两个 P0、两个 P1，原作者在不同 r4 修复；真实 Headless
Session 停机回归 r3 的三项源码问题已由独立 H 关闭，未编译或运行。Editor r4 还有 ForView
非穷尽 match、测试导入、None 保存越界及 crash pair/逐叶恢复门，原作者在不同 r5 修复。
IME r3 的 ABI 导出 P0、实际原生回调/显式请求/真实 caret publication 缺口及矩形范围门
仍待不同 r4 修复；ZrVM r1 九个源码路径复审有 14 项未关闭问题，修订 r2 继续进行。
这些完整源码复审不是 pass，精确终态见 [复审核验](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/verified-source-review-terminals-20261002-r1.json)。

Asset v5 的 48 路径 postimage/raw baseline/current 哈希核验无漂移，但普通 World 保存
物理路径越界、实际 RuntimeExtension→ProjectManager provider ingress 仍为 P1 open。
Graphics consumer 的宣布产物被作者补写测试后覆盖；root 冻结不同 r2 和漂移收据，独立
复审继续，旧 metadata/patch 字节未保留，不重建旧历史 seal。Hub 38 路径源码复审已闭合，
实际 native/helper/服务/Keycloak/Tauri 与性能验收仍 open。

六个未登记外部 Cargo 目录门仍阻止新的受管执行；没有重复提交 unchanged validation。
Core/Native 活跃外部 owner consent 与 Runtime04 compatible Resource producer 闭包仍未
建立。原三个 Penpot lifecycles、三组原 SID、route、既有 tickets 和 failure 证据保持原状。
本次没有新 Failure/Session、状态复活、外部源码/lease 回收、commit、push 或 WeCom。


## 2026-10-02 02:26Z 第六批逐项映射与完整复审边界

本批为 Runtime167 `PHY3-P1-001..006`、Runtime170 `RT-AN-01..06`、Runtime173
`NET-RT-001..006` 的 18 条有界当前源码观察。与前五批 130 个键逐项比对后，14 个新键、
4 个旧键刷新，规范观察总量为 **148 条 / 144 个不同 report-qualified 键**。这不是已验收
finding 数或全目录覆盖率。原空 batch6、错误 r2、作者 r3 与 root 不同 r4 都保留；没有
创建、导入或关闭任何 Failure。17 条 crosswalk 关系全部 `identity_merge=false`。

根线核验了三个原报告的准确段落、34 个当前文件哈希、2 个明确缺失路径、65 个源码 span、
18 个正常 caller span，并另外固定 8 个测试源码上下文。修正作者沿用的 Runtime221 metadata
为原报告 Runtime170；每个键始终等于原报告路径加原 finding ID。规范行包括逐项 lower /
original / upward / Windows 产品门、明确回归场景、存在但未执行的复用测试及性能测量范围。
见 [第六批准确输入核验](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch6-root-verification-r3/root-verification.json) 和规范 manifest `2026-10-02-w0-batch6-manifest.json`，SHA-256
`9df9c887ad0c074cc6d6dfda463323ec7652d4b919cade2ce2284f058c465af4`。测试与性能实际执行数仍为 0，全部条目 acceptance open。

当前源码结论已收窄：Physics 配置在 `set_value` 成功后才清 backend/commands、写内存，
不再沿用旧报告的 memory-before-persist 指控；完整 validate/prepare/rebuild/generation 门
仍 open。Net 已有 root 与 HTTP/WebSocket collector，这两条 factory resolve 原 root manager；
剩余四 feature、RPC/replication owner、World stages、effective config 和 Editor provider
仍待实施。Animation 的 facade 在同一模块包裹 DefaultAnimationManager，不能据两个注册
名字断言两个独立 solver；跨 builtin/plugin 目标与版本化 artifact/evaluator 仍未验收。
规范行状态为 **10 confirmed_open / 8 narrowed_open**；库存 576/61/515 仍是历史界限。

Asset v5 全 48 路径 E/F/H 复审有 5 个 P1 观察和 4 个 P2 验证缺口，其中两个 ingress
观察指向同一最低支持合同，保留原 ID 及关系。最低源码合同包括 cache wire version/迁移、
组件引用表验证、World 保存物理根准入和真实 RuntimeExtension→项目重开 codec ingress。
原作者不同 v6 修复中，见 [Asset 全 48 路径复审处置](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-scene-2d-persistence/root-independent-v5-disposition.json)；没有凭源码或 apply-check 关闭测试/产品门。

Graphics r2 独立复审确认生产 `Arc` 被 cfg(test) 隐藏，另有 defining API 补丁输入闭包、
真实绑定/queue lifecycle 回归和前序未来时间 provenance 门。原作者不同 r3 继续修复，
[Graphics 独立复审](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/rg166-consumer-frozen-r2/independent-graphics-consumer-review-e.json) 保留原 r2 和未留存旧 seal 的准确边界。Sound 的 19 列出路径冻结后，
14 个非测试与 5 个测试路径分别独立复审；Navigation 的 24 路径 operation delta 相对私有
前序 guard，57 文件基线有 15 项与共享 current 不同/缺失，组合闭包尚未直接移交构建。

原三个 Penpot lifecycle、原 UI/RG166/native SID、source-only postimage 和 tickets/routes
保持原状。六个未登记外部 Cargo 目录、活跃外部 Core/native source consent、Runtime04
compatible Resource producer 闭包及所有受管 lower/original/upward/Windows 产品/性能门
仍 open。没有新 Rust 提交、commit、push、WeCom 或外部 lease/status 变更。

规范记录：[第六批摘要](01/2026-10-02-w0-batch6-summary.json)、[第六批 manifest](01/2026-10-02-w0-batch6-manifest.json)。


## 2026-10-02 04:03Z 第七批精确源码映射与复审边界

第七批固定 Runtime213 `RT213-P1-001..006` 与 Runtime225
`MSP4-P1-025/026/027/005/019/033` 的 12 个原 P1 finding。六批规范输入仍为
148 条观察 / 144 个 report-qualified 键；本批无旧键重叠，准确发布后总量为
**160 条观察 / 156 个不同键**。它们是源码观察，全部 acceptance open，测试、产品和
性能执行数为 0。576/61/515 报告库存界限保留，不将局部行复核计为整份报告验收。

原作者四件记录与 root 核验均保留。root 固定 27 个原 source-evidence 路径，另核验
真实 caller 与测试上下文，合计 36 个源码/测试文件、6 个报告文件。历史有界 source-evidence
路径并集为 203，不能据此宣称当前全树 coverage。规范行补齐最低责任路径、实际正常调用
跨度、存在但未运行的测试函数及有限 oracle、逐项 lower/original/upward/Windows 产品、
故障和性能门。规范 manifest SHA-256 `11b99670a3104694384fa38af9b43df7ada328cca2489eff8730f3e25a7ec622`。

源码结论收窄为 **1 narrowed_open / 11 confirmed_open**：RenderScene 已有正常
renderer→registry/projector→streamer→owned GPUScene journal 调用，旧 tests-only 前提
不再成立；唯一跨产品 mutation authority 与实际 GPU generation 门仍开放。Visibility 已
传递 previous static/dynamic index 和 FrameHistoryValidationKey，剩余是广域 primitive
收集、组合 typed receipt、实际 plan executor、GI bridge 和共享 early/late/final visibility。
Shader 的 substring capture、visiting/completed cycle 混同、重复 source 所有权/预算、
variant exhaustion panic、disk 两次提交和 error-proxy 同步 compiler 等待仍有有界源码证据。

crosswalk 四条关系均不合并 finding 身份。root 将原文 `Runtime89` 别名准确关联到唯一
现存 89 编号 RenderGraph 报告；作者猜测的 `89-runtime-visibility-review.md` 缺失不等于
Runtime89 报告缺失。原错误 edge 在 root provenance 内保留，未读该报告 finding。

Editor r5 的 30 个候选路径与 current raw baseline 无差异；私有补丁重建首次被系统 Git
autocrlf 转换为 CRLF，保留该 receipt。仅为新重建命令设置 autocrlf=false 后，30 个
原始 postimage 哈希全部匹配。F7/G12/H11 完整独立复审进行中，不据此关闭 durable
Scene/workspace、序列化 startup restore 或真实 Editor 产品门。Core r4 21 路径组合
与 Nav r1 24-path delta 的原记录保留；Core 正常回滚 admission/cleanup、Nav move/ABA/
in-flight budget 的源码问题继续由其实施线修复，测试与产品门均开放。

三个 Penpot lifecycle、原 SID、source-only postimage、tickets/routes 保持原状。
Runtime04 单测 snapshot 与 compile preparation 不算 Cargo 通过，兼容 Resource producer
仍需其原 owner 封存。六个未登记外部 Cargo 目录、活跃外部 Core/native consent、原 managed
lower/original/upward、Windows 产品及性能门仍开放。没有新 Rust 编译、Failure import、
foreign owner/status/lease 变更、commit、push 或 WeCom。

规范记录：[第七批摘要](01/2026-10-02-w0-batch7-summary.json)、[manifest](01/2026-10-02-w0-batch7-manifest.json)。


## 2026-10-02 05:20Z 第八批源码更正与两批发布边界

第八批保留 Runtime187 `ECS4-P1-001..006` 和 Runtime190 `RCM6-P1-001..006`
的 12 个原 P1 身份，与前六批和第七批均无 report-qualified 键重叠。前六批仍为
148 条观察 / 144 个不同键；第七、八批准确写入且归属回执终态后，本阶段为
**172 条观察 / 168 个 report-qualified 键 / 136 个 literal ID**。这些是源码观察，
测试、产品和性能执行数为 0，全部验收门开放。576/61/515 整报告库存界限保留；
历史有界 source-evidence 路径并集为 225，不能当作当前全树或整报告验收。

第八批 root 固定 23 个原 source-evidence 路径；连同实际 caller/test 上下文，共
32 个源码/测试文件和 16 个关系证据报告。30 处实际测试定义存在但未运行，移除
一处“测试作为正常生产 caller”的标注，并将 16 条 crosswalk 指向准确列表条目。
原作者 02:00Z 标签不构成已见证的历史 seal；root 独立记录当前字节与捕获时间。
原报告和原作者记录保留，关系均不合并 finding 身份。

当前结论为 **7 narrowed_open / 5 confirmed_open**。EntityRegistry 代际检查、
StableQueryOrderIndex 稳定顺序、稀疏/密集互斥的值所有权、ComponentStorageLocation
以及 PendingComponentRow 的最终预检已有源码和有限单测。多份派生索引本身不能证明
身份损坏；剩余是跨 owner/layout generation、明确排序语义、故障与性能合同。正常
clone/load 重建是 owner-local projection reconstruction，未证明存在任意外部 reset。
Camera 仍有 11/14 reflection skip、混合 source/compiled policy、缺少本批检查的
endpoint/lens/rig/shake role、裸 active EntityId 与静默非法选择。Editor viewport
正常提供 camera snapshot，绕开全局 active selector；全局 setter 修复不等于真实
Editor endpoint 产品验收。每行已记录最低 owner、正常调用、有限 oracle 和逐项
lower/original/upward、Windows、故障与性能门。

Core r4 完整 14+7 复审确认三处回滚缺陷，原实施线 r5 继续修复；Sound r1 的 14+5
完整复审确认候选初始静音和 config/state 并发两处 P1，r2 继续修复。Editor r5
三段复审尚未全部终态，崩溃夹具新旧值相同和 ViewInstanceId 类型边界仍需关闭。
所有私有补丁、原失败、真实 Windows/native GPU/audio/Tauri/Keycloak、性能门均保留。

协调器正常查询已返回既有 W0 心跳 `e798a4769cf5410baafdb3a759a048a3` completed，
原 SID/scope/baseline 保留。旧离线 queueId 的历史 requestId 绑定没有伪造；没有
重复心跳、daemon restart、foreign status/lease 回收、Failure import、Rust 构建、
commit、push 或 WeCom。已有维护与源码 owner、六个外部 Cargo 目录的实际门仍有效。

第八批[源码观察](01/2026-10-02-w0-batch8-review.jsonl)、[关系](01/2026-10-02-w0-batch8-crosswalk.jsonl)、
[摘要](01/2026-10-02-w0-batch8-summary.json)和[manifest](01/2026-10-02-w0-batch8-manifest.json)
可重现本批原 ID、证据与剩余门；记录发布不关闭任何产品验收。

## 2026-10-02 协调器退役后的本地证据续接

八个已落源 W0 批次仍为 172 条观察 / 168 个 report-qualified 键 / 136 个 literal ID。
Core r5、Sound r2、VM r2、IME r5 的精确私有候选、独立复审和剩余源码门见
[当前续接](01/2026-09-30-w0-current-source-continuation.md)。原测试、产品、性能和三条 Penpot
生命周期保持未通过。
旧协调器的正常 claim 已返回 coordinator_retired；用户明确授权按退役后的本地证据路径
继续。保留旧数据库、原 SID、leases、attributions、pending requests/tickets 与原始失败证据；
它们不再是新本地命令的执行前提。按当前源码哈希、明确 scope/handoff 与冻结输入取得
独立本地证据，formal Jenkins 与产品验收仍分别开放。没有重启或恢复旧服务、清理 foreign
输出、重复 import/validation、commit、push 或 WeCom。

## 2026-10-02 W0 batch9–12 当前源码映射

新增四个有界复审批次；当前 canonical 记录为 **220 条观察 / 216 个 report-qualified 键 /
183 个 literal finding ID**。原八批字节保持原样，跨报告相同 literal ID 不合并身份。

| 批次 | 原报告及范围 | 当前源码状态 | 原优先级 | 记录 |
| --- | --- | --- | --- | --- |
| 9 | Runtime220 / Runtime201，各 6 条 | confirmed_open 8 / confirmed_partial 1 / narrowed_open 3 | P1 12 | [review](01/2026-10-02-w0-batch9-review.jsonl) · [manifest](01/2026-10-02-w0-batch9-manifest.json) |
| 10 | Runtime194 / Runtime202，各 6 条 | confirmed_open 5 / narrowed_open 7 | P1 12 | [review](01/2026-10-02-w0-batch10-review.jsonl) · [manifest](01/2026-10-02-w0-batch10-manifest.json) |
| 11 | Runtime217 / Runtime222，各 6 条 | confirmed_open 10 / narrowed_open 2 | P0 5 / P1 7 | [review](01/2026-10-02-w0-batch11-review.jsonl) · [manifest](01/2026-10-02-w0-batch11-manifest.json) |
| 12 | Runtime188 / Runtime189，各 6 条 | confirmed_open 7 / narrowed_open 4 / partially_narrowed_but_open 1 | P1 12 | [review](01/2026-10-02-w0-batch12-review.jsonl) · [manifest](01/2026-10-02-w0-batch12-manifest.json) |

这 48 条只补充 finding 映射、当前源码证据、具体责任类型/函数、普通调用者及未执行测试。
独立核对覆盖 155 个当前源码/报告文件，其中 133 个源码文件；不是整份报告的完成率。
全目录基线仍为 **576 份报告 / 61 份整报告源码复核 / 515 份未读**，不因有界行切片而提升。
Crosswalk 的 14 / 23 / 28 / 19 行为关系和核对元数据，不提供身份合并或实现归属授权。

[当前续接](01/2026-09-30-w0-current-source-continuation.md) 记录原稿、分代校正、责任函数和
源码变化。`tests_run=0`；原 lower/original/upward、真实 Windows 产品、性能、Jenkins 及
三条 Penpot 生命周期仍开放。按用户已授权的退役后本地证据路径落文档；旧协调器数据库、
原 SID、归属、待处理回执、锁和失败证据保持原值，没有新 Session、Failure、import 或验证重放。

## 2026-10-02 W0 batch13 与本地回归边界

Runtime186 的 `PHY4-P1-001..012` 已按原 P1 表行、37 个当前输入和独立复核追加映射。
Canonical 当前为 **232 条观察 / 228 个 report-qualified 键 / 195 个 literal ID**；
原 12 批不改写，Runtime219 仅作 currentness 关系，576 / 61 / 515 整报告库存不提升。
本批为 1 narrowed_open / 11 confirmed_open，测试执行数为 0。
[记录](01/2026-10-02-w0-batch13-review.jsonl) · [摘要](01/2026-10-02-w0-batch13-summary.json) ·
[manifest](01/2026-10-02-w0-batch13-manifest.json)。

Hub 的精确 `--locked --lib --no-default-features --features local-service service:: -j 2`
本地回归实际执行 **80 个、通过 80 个**，全部 17 个指定测试存在；16 个测试被过滤。
结果只证明本次本地 correctness，实际共享 Cargo cache 与 stage metadata 的依赖字节资格仍
在单独核对。Windows Keycloak/服务/桌面、掉电持久化、性能及 Jenkins 未通过。
WOC 原测试因最低协议借用迭代编译错误执行 0 个；现仅一行修复已落源，底层协议及原事务
回归尚待终态。[精确结果与续接](01/2026-09-30-w0-current-source-continuation.md)。
原 Session、failure/comment provenance 和三条 Penpot 生命周期保留，未调用退役 API。

## 2026-10-02 W0 batch14/15 与当前执行边界

Runtime170 新增 `RT-AN-07..11` 五项，原优先级为 P1/P1/P0/P1/P1；Runtime175 新增
28 个原 P1 literal finding 与 12 个原文未标 ID 的 P2 ordinal，另存 24 个声明验收门。
当前为 **277 条 finding 观察 / 273 个报告限定 finding 键 / 226 个 distinct 原 literal ID**；
其中 12 个 ordinal 不计作 literal ID，24 个 gate 不计作 finding，原 13 批字节保留。
[batch14摘要](01/2026-10-02-w0-batch14-summary.json) ·
[batch15摘要](01/2026-10-02-w0-batch15-summary.json) ·
[batch15验收门](01/2026-10-02-w0-batch15-gates.jsonl)。独立复核及当前38/58输入已核对，
本批测试执行0；旧30-input/hash interval和5处clock行号范围另列资格说明，不提升验收。

完整编号库存仍为576。历史61 source-observed是有限finding源码记录，515为历史未读标记，
均不代表整篇验收；独立当前清点记录Runtime174/76两篇报告哈希漂移。
RG166七个源文件已发布且41个候选postimage匹配，编译、实际GPU和原Penpot门继续开放。
Hub local-service二进制已实际本地构建通过；账号/团队/商城/云同步真实Keycloak服务部署、
正常退出重启和原生桌面仍在执行，尚无产品或性能结论。WOC协议回归正在运行，原21个
事务测试保留正常下层实际通过的依赖门。[精确续接记录](01/2026-09-30-w0-current-source-continuation.md)。

## 2026-10-02 WOC 正常下层与原事务本地终态

同一兼容closure实际执行协议22/22和原Runtime transaction21/21，合计43个测试通过；
两条正常命令均退出0，原六个协议指定回归及全部21个事务名称实际存在，未重试。
原事务另有1个既有release性能测试ignored，性能门继续开放。
这是默认feature本地correctness证据，engine-host/backend-zr-vm、真实双进程、Windows宿主、
性能及Jenkins门继续开放。[原始终态及Root核对](01/2026-09-30-w0-current-source-continuation.md)。

## 2026-10-02 W0 batch16 与新的独立源码边界

Root 核对时间 `2026-10-02T21:50:41.548627+00:00`。App09 的 53 个原 finding（3 P0 / 40 P1 / 10 P2）和 Interface07
的 61 个原 finding（1 P0 / 48 P1 / 12 P2）已按完整原 atom 记录；另存 48 个原 gate。
Interface15 的 61 个 finding refresh 与 32 个 gate refresh 单独保存，均为非新 finding 身份。
当前累计 **391 条 finding 观察 / 387 个报告限定 finding 键 / 339 个不同原 literal ID**。
12 个历史无 literal ID 的 Runtime175 P2 ordinal 保留；原声明 gate 累计72。
`RI-CERT-P0-001` 已在 Interface15 的历史记录出现，按裸 ID 去重后的增量为113，不能写成340。

[第16批 finding](01/2026-10-02-w0-batch16-review.jsonl)、[gate](01/2026-10-02-w0-batch16-gates.jsonl)、
[successor刷新](01/2026-10-02-w0-batch16-refresh.jsonl)、[摘要](01/2026-10-02-w0-batch16-summary.json)
和[manifest](01/2026-10-02-w0-batch16-manifest.json)保留255条原行分类及原报告/comment来源。
独立资格审查 SHA `8212ba9b62e8140177bf1df24677ec2379efff40d3e6520810bd9a00211253ef`，
129个capture（119仓库路径/10私有证据）与当前字节逐项相等；17个超出当前文件长度的
行号端点保留并限定为元数据，未重写历史 seal。作者60文件/9445行口径不作为语义 JSONL
数量；发布前实际为32 JSONL/538语义行/15 review-bearing/277 finding。作者 broad clean
和独立摘要将 batch15 P2 误称 Interface07/15 的文字保留并限定，实际原记录属于 Runtime175/222。
完整576编号报告库存和历史61/515有限观察边界保持；本次不宣称全目录阅读、产品或性能完成。

Core r8 的24私有postimage及9个增量已在原始字节副本重建；原Core shared→r6→r8谱系
仍待完整当前publication envelope。三个 live Session module/construction/state 的 IME 和
Nav consumer wiring由唯一合成线处理，未用旧私有整文件覆盖当前construction/shutdown。
IME r11 有56显式路径：42复用、14当前辅助上下文、1个私有routing重复import更正；
Root独立汇总 `1b013d50586428d4472368315de24c6f029af584fee554e9ef87378a28eff7e4`。
Nav r3 的59路径/48原字节复用/11独立delta的汇总为
`a7fe76285a1220f7b69b277508c8084ddfbce05d1b402b32ef85166bbcc46db3`；
实际provider、丢失worker、deadline、Editor/Session及MAX边界oracle门保持开放。
Sound r4 的21候选/30当前输入（含2声明尚不存在新文件）已精确重建，Root receipt
`aa39751c52cc0185fb7a3609f6844d87cc013162ac84f3f347e8d43538228a20`；
prepare静音与CPAL回调退出的两个新独立P1仍需原作者核验和修复，native PCM/性能未执行。

Hub实际本地deployment和同一SQLite/CAS重启读回receipt
`11e3100f2ce44b7e644298f63ef037d814cc120b95c4bd8bcd0bb259792d0e92`已封存，
readiness/正常退出/端口关闭和212992字节数据库/36字节store-id保持相等；realm已有且0users，
账号/团队/商城/cloud PKCE真实业务尚未验收。WOC此前22+21实际通过及1 ignored release
性能测试的精确旧receipt继续复用；native backend/VM/client/server和性能门仍开放。
以上新增候选/审查没有执行Rust测试或改写生产源；当前完整Runtime本地输入封存/验证及
Hub Desktop材料准备仍在推进。协调器退役后不调用旧claim/heartbeat/status/submit/API；
三个Penpot原生命周期、原SID/tickets/routes、build-editor及live Source/token/font/media/locale，
产品/性能/Jenkins保持开放。没有commit、push或WeCom。


## 2026-10-03 W0 第17–21批原身份归档与当前证据

Root 核对时间 `2026-10-03T04:32:50.739549+00:00`。新增510条原 finding 观察：UI Runtime76/81/82 为148，
Hub01为48，Runtime166/89为126，Editor72/54为127，Runtime85为61。
累计 **901条 finding 观察 / 897个报告限定键 / 849个不同原 literal ID**；
12个既有无literal ordinal保留。原声明gate累计428，另存Hub07的28条
当前资格gate；它们不计入原gate。新增311条successor/currentness refresh均不增加finding身份。

独立审查 `0973fe9af1c153ea17ad42fe1a7f3e7f2f0418a55a4a5bcfe95098dfcbe9430e` 的旧B17/P0及B20归属hold按原文补证和Root派生记录限定：
B17保留148个完整atom及144个原表gate，四个P0的原文closing contract另有
资格补证 `429d025a977a7035262478bc8906e1a3ee7bc4422124c0e8e3a182692d4c8ca0`。原gate表存在性更正 `e263b3a85a1451234722b5d72bf5d9ac9a582730ebac11f5188651ee161b3394`
确认所有补证引用均为已有gate；144原表行不增不减，四P0仍开放，未新增gate或验收。Editor54的67个finding与36个gate
恢复原54报告键、原行号和原文本，Editor127仅作非新refresh；`ED54-P0-03`的
`recompute_if_dirty`/`mark_host_projection_committed`差异及G01库存文字差异保留在当前刷新。
原作者packet、重写过的B17 supplement元数据限制、B20两个不同manifest和3条
第19批 report_provenance均作为原始证据保存；未重构无法恢复的历史seal。

[第17批](01/2026-10-03-w0-batch17-summary.json)、
[第18批](01/2026-10-03-w0-batch18-summary.json)、
[第19批](01/2026-10-03-w0-batch19-summary.json)、
[第20批](01/2026-10-03-w0-batch20-summary.json)、
[第21批](01/2026-10-03-w0-batch21-summary.json)分别链接原finding、原gate、刷新、
manifest及精确Source代。所有新映射仍为源码证据，动态/产品/性能门开放；完整576报告
库存、历史61/515有限观察边界继续保留，不能据此宣称全目录验收。

当前Hub S6 typed Present package-lock已落源；四路径桌面Rust编译修复的publication
为`bb83a80d37f868ee2d595b1268fa39e2da189371c637ec94150d7c23df57bb7d`，尚待当前代正常编译。
旧Desktop R2 TypeScript/Vite实际exit0，Rust actualexit101及其漏源/10类型错误保留。
Hub旧二进制的真实HTTP13项负向断言已通过，终态`72ee2d32697f009d483ca70fda52e42c905d31a75167491fd832c229b21b5a3e`；
它不验证当前S6或原生桌面。见[服务责任计划](../features/hub/02-local-service-authority.md)。
Runtime R4下层510pass/2ignored复用；原UI feature编译因正常`target/result.rs`漏源actualexit101，
八个原测试未执行。Root有限终态`aca8146fda238de08f3085067d8f06eb6644d23a91b52a1a7bb283fe533755ab`
保留原UI/Render哈希及三个原failure/SID，下一步仅准备完整新输入。Editor42路径及Nav37直接路径已精确落源，receipt分别为
`3ccda793de54d5c203c5934eb16984adc62409473d3021a189ffd13804247fb8`与`9adb98e26c37fd286f5915c49642299df702ee88c53c078ecc076fa82ed1059b`；
独立复审均为source-addressed-unrun，Core Session组合尚未据此通过。Core/Asset/IME/
Sound/VM候选独立复审、normal build-editor、live Penpot Source/token/font/media/locale、
Windows产品、性能和Jenkins仍开放；协调器已退役，无register/heartbeat/status/claim/submit，
没有新failure/import、commit、push或WeCom。

## 2026-10-03 Root B22/B23 exact original-identity publication

第22/23批新增 **291条原finding观察 /170条原始gate**，累计
**1192观察 /1188 report-qualified keys /1110个不同literal ID /12条无标签finding ordinal /598条原gate**。
原报告键、literal ID、行号和原文本按独立复审及raw captures保存；64条successor finding/gate
refresh不新增finding或原gate。B22的44个无标签gate保留`gate_id:null`与原ordinal；
09F3的M0–M15是16个roadmap milestone，不混作finding或gate。作者Runtime98的
`09F3-P0-*`对应字段仅作元数据，原09f3报告仍用其真实`P0-*` literal ID。

[第22批](01/2026-10-03-w0-batch22-summary.json)包含09f3 GI、Runtime98 current-source
addition和Runtime28硬件光追；[第23批](01/2026-10-03-w0-batch23-summary.json)包含
Runtime26原粒子问题、99d文件的Runtime103和Runtime171非新刷新。Source和参考引擎
raw generation、before/after输入以及当前单独哈希观察分别存放manifest；参考捕获不构成
竞品性能证明。B23已返回packet曾被作者覆盖，当前实际review哈希为
`91b02e48cef57b3986d1ef0d235a26a3fc2df65a5adffd4bd700f24b24c139d0`；
旧seal原字节不可恢复，保留distinct erratum并拒绝重构历史不可变seal。
独立复审JSON `7236a6c139b79a0f73ae01317583fac867e3d6d98ec3e7174b29436b8b6d7a9d`
不替代执行；其Markdown格式更正后哈希为`e7fd43d46315d3039d19a97cd240cfe3e2a184f185a9108cc53d967ad2ee3e06`，
旧Markdown字节不可恢复。

本次只补齐已有W0记录，tests_run=0，所有finding、真实GI/RT/粒子输出、GPU golden、
fault/scale/soak、Editor链路、同画质性能、Jenkins及产品门仍开放。
高级能力按Core/asset/provider/RHI与真实图调度的最低责任模块依赖推进；
Source-only和静态能力名字不算通过。原failure/SID、路由、tickets及已接受证据均保留。

### 2026-10-03 original UI/Render R5 compile and single-test Source repair

原UI SID `astra-ui-time-20260929-01a0f033-r2`及Render
`astra-rg166-device-lowering-20260930-01a0f033`不变；三个canonical failure仍开放。
R5正常同feature `+1.94.1 check --locked -p zircon_runtime --lib --tests
--no-default-features --features target-editor-host,runtime-ui-integration-tests -j 2`
在29633文件精确输入完成actual exit101，终态
`fe1da336d1f49491413d2c008c09f0a6c20df490a3e8a868fdd1db8e64461e57`；
原stderr `b15e769ad18091ab2fefe96eeab665e5c47d963158abdd90912a3023c4729beb`保存。
R4漏的正常`target/result.rs`已在R5完整封存；当前blocker是
`zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs:186`绑定未声明mut，
其219/243/263的`compute_layout`需要mutable retained Surface。八个原测试未执行。

Root在真实Source仅加该绑定`mut`，原SHA
`ad1a419f2baf869761808b0ffe07f439d7ebcf15998acce49ebc1600044a33a3`
到post `d0b74de51051844a86ff3baead08657c09fad4072c98e4bf85aa074362bc6fb2`，
保留所有production、comment和断言；publication
`b97fb16d6a8108e5da747a444f2887c502aa10a8c88f2a97bf0b982134d06430`
normal apply-check/diff-check0，shared scoped index不变，source-only并未成为验证pass。
下一次仅R5同代input加该单路径postimage，distinct R6，不改failed copy，
原RHI510pass/2ignored回执`69415347f99f363a4eedfb9cfa0632bbdfca5ec56b4214d15e810093d64ffe59`复用。
bounded handoff `a6f6c793c0c36ed03011c6bcb1144a013fc2992730fe6bbf9950a10bc27424ec`
保留旧原始编译、Source修复和下一项声明；normal compile/eight tests、build-editor/live
Penpot Source/token/font/media/locale、产品/性能/Jenkins门继续开放。无旧coordinator API、
新Session/failure/import、quiet/monitor/wake/route load、commit/push/WeCom。


## 2026-10-03 Hub R3 实际终态与 VM 精确源边界

Hub R3不可变输入 `a21f521e78b988793e99eae65e8226697869befc124978731b6cec6ad2f9c590`
普通check实际exit0，普通Hub lib实际96 passed（80service+16file_io）、0failed/0ignored。
旧expected81 service清单有一个不在该代Rust源码中的测试名，按实际集合记录，未关闭缺失名字。
Runtime Interface同批实际783passed/7failed/101ignored；只有PTY终态，原完整日志不可恢复。
原始6项package-lock通过证据保留，7项下层修复正在继续。封存输入前后0漂移；已有执行只
绑定这一代输入，不重复提交。详见现有[Hub合同记录](../features/hub/02-local-service-authority.md)，执行回执
`b58d847282c49e7056977ad99a8a6053d1bc74e5c7d17a89f5f050c38ceec714`。

VM原owner候选16个修改路径已按精确原字节落源，publication
`2dee8eaa68454da9b60ae8f62615083ec1a73d288e8d73b860456e69c9a3acc0`。
手动LIB_DIR经过物理批准路径校验，docs postimage
`3c6b135a4ae04d982d652e28fe44c89f574d862165dcb3d90998e41165be5f99`，build.rs
`b149e3451846e52bd913c7ead24bd1b667e94404ffa5e9d82b9b9766edd96d35`；foreign CMake
测试尾段、所有预存源改动和shared index保留。作者交回后覆盖了部分封存元数据/文档，
第一次publication在写源前被hash检查拒绝；当前落源采用Root已保存的原始正确37文件候选、
原535bf4封存回执和独立复审d0862a42，漂移单独登记e6181ee1，没有重构旧seal。
正常Windows shared/static原生构建输入仍在准备，实际CMake/CTest/Cargo为0，WOC/EngineHost
双进程、性能、Jenkins门开放。

Sound独立复核发现设备retirement超时后正常播放/音量等控制仍可访问保留Kira句柄的P1；
20路径原候选尚未落源。原owner正按最小共享控制准入边界准备distinct r9修复，保留pending
worker与原24路径候选。Core/IME/Asset接口组合继续，尚未据Source/apply-check宣称通过。
原failure/SID、原lower/original/upward、normal build-editor、现场Penpot Source/token/font/media/
locale、native product、性能和Jenkins门均保留。


## 2026-10-03 W0 B24 原始身份与当前报告边界

第24批新增294条原finding观察、109条原gate；累计 **1486观察 /
1482 report-qualified keys /1301不同literal ID /84条无标签finding ordinal /707条原gate**。
原始literal ID、report path、原文本及行号均保留。Runtime213的P1-001..048映射Runtime94，
48条为NONNEW；P1-049..060与P2-001..012是其当前报告24条自有finding。Runtime172和
Editor232共58条successor finding与36条gate保留为NONNEW，未新增原finding或gate。
Runtime29的5条继承owner决策不是finding；Editor16的60条P1和12条P2仍为null original ID，
只使用原ordinal定位。泛用P0/P1/P2标签可跨报告重复，原身份必须按报告限定。

Runtime09B验收矩阵25行中只有369–377行的9项9.3硬检查计gate；16条维度/指标/要求另存，
378行统计工具要求不计执行gate。封存33个当前源码与12个Unreal主参考的原始输入和
before/after字节，selected dirty状态仅限该范围，未请求whole-tree quiet。声明但缺失的
LandscapeStreamingProxy参考仍记录缺失；没有生成该文件或把参考捕获写成竞品性能证明。

[第24批记录](01/2026-10-03-w0-batch24-summary.json)与独立复审
`176a0d9b638d727087c21a390f05401115e006400454e51002e188764a45920d`
绑定原作者sealed artifact manifest `3753601a488b8659e45e4b372e73bc90308ffb14e2defc0b7ace6b3e4c3d1989`
及3份distinct errata。所有源码、GPU/Editor实测、fault/scale/soak、产品及性能/Jenkins门继续开放；
本步Rust编译/测试为0，B25尚未纳入。高级VirtualGeometry typed executor候选只按最低责任模块
准备，尚未实现或验收真实GPU调度/资源/dispatch/readback链路。


## 2026-10-03 W0 B25 物理与动画原身份复核

第25批纳入49条历史finding及95条当前报告finding，新增96条声明gate。累计为
**1630观察 /1626 report-qualified keys /1396不同literal ID /84条无标签finding ordinal /803条声明gate**。
08a/08c的原始`P1-1`等literal ID按报告限定；22条历史gate保持null原ID，以原行号定位。
当前Physics/Animation各自报告的74个literal gate不与历史行合并。Runtime219/221的15条
currentness/finding刷新和10条gate刷新仍为NONNEW，未增加finding或gate总数。

[第25批记录](01/2026-10-03-w0-batch25-summary.json)绑定Root冻结88个原始artifact（2369259B）与独立复核
`88474d1b4ee8700968fedec4acaf0502d5113d9f9870c359adf91cac3ca468b3`。
49条历史finding到当前owner只建立一对多ledger，未宣称精确逐项映射或修复完成；6份报告、
73份源码/参考捕获和原r1/r2 errata均保留。B24+B25的NONNEW刷新小计为121条finding/currentness
及46条gate，未推算更早批次的全局刷新总数。原始Physics/Animation产品合同、真实后端、
source/compiled pose、normal consumer、碰撞cook/save/load、超时/重试、Windows native、
Editor/性能及Jenkins门继续开放；此步源码修改及Rust测试为0，全目录覆盖仍未验收。


## 2026-10-03 W0 B26 原报告身份补录

第26批仅补录Runtime94与Editor138的137条原finding（P0=5/P1=108/P2=24）和80条原gate。
累计为 **1767观察 /1763 report-qualified keys /1533不同literal ID /84条无标签finding ordinal /883条声明gate**。
217条原报告行与封存字节一致；按报告路径加原ID扫描现有记录未发现重复。
Editor138的G01–G32仅与其他报告的未限定ID重名，不合并身份。

[第26批记录](01/2026-10-03-w0-batch26-summary.json)仅保存Runtime213第52行实际列出的六组Runtime94后继范围关系，均为NONNEW。
48条Runtime94原问题保持开放；不建立逐项配对。三组数量不一致为12对9、9对10和6对8，
数量相等的其余分组也未据此接受逐项语义映射。
Editor138到Editor232、Runtime94的P2与gate关系仍未确认。此步仅补录原始身份，
未宣称当前生产源码已修复或验收；源码修改与Rust测试为0，所有原GPU、Editor、
native、产品、性能及Jenkins门保持开放，全目录仍未完成。


当前源码与本地证据边界见 [原续接记录](01/2026-09-30-w0-current-source-continuation.md#2026-10-03-当前源码与独立本地证据边界)：Hub 当前二进制实际编译通过；R9 的最低 Scene 消费者修复已落源，R10 尚无通过终态；VM/Interface/Sound 和 W0 补充资料分别保留开放验收门。全域产品、性能及 Jenkins 验收未完成。


## 2026-10-03 W0 B27 与原 UI 继续验证

[第27批](01/2026-10-03-w0-batch27-summary.json)补录12份Runtime原报告的275条finding
（P0=24/P1=202/P2=49）、286处位置和212条无ID声明gate。11处重复只作为观察，
按原报告完整路径加literal ID计数。累计 **2053观察 /2038 report-qualified keys /1533不同literal ID /84 null finding ordinal /1095 gate**。
21条既有baseline/关系记录仅保留NONNEW关联，未推断逐项别名映射或当前修复。
原576份报告和额外9路径分类保持不变；身份补录未接受生产行为。

原UI/render的R10编译实际exit101、19条诊断。R11五份既有测试消费者已独立复审并落源，
新副本仅原R10加五项postimage；原回归链及受影响15项普通测试尚无通过终态。
原request/SID/生命周期、build-editor、live Penpot、产品/性能/Jenkins门继续开放。
原510项RHI证据仅复用于匹配原代次，全域目标尚未完成。


## 2026-10-04 W0 B29 插件原报告身份与原 UI 验证边界

[第29批](01/2026-10-04-w0-batch29-summary.json)补录22份插件原报告的1,448条原问题：
P0=42、P1=1,136、P2=270，其中1,256条literal ID、192条原无编号问题按报告和severity ordinal保留。
累计 **3501观察 /3486 report-qualified keys /2634不同literal ID /276 null finding ordinal /1095原gate**。
3,526条混合上下文与38条NONNEW关系保留；430条checklist分类含标题，未增加原gate分母。
原576份主报告和额外9路径分类不变，当前插件生产行为、Windows产品、性能与Jenkins仍未接受。

原R11编译实际exit101，仅剩两条E0425，均在同一个lighting测试辅助签名。
R12只将该私有辅助函数改为由唯一调用推断的T: Clone，共享句柄克隆、31处断言、5个ignored profile不变。
R12封存29,633文件/304,786,805B，manifest `6b378ea35c424412659d7af7eea4b4e966303ee0509f32b858cd8331f506f9e0`；
原编译与37项声明回归正在执行，尚无通过终态。原510项RHI证据不重跑，也未投射到当前RHI源码。
原request/SID/三生命周期、build-editor/live Penpot/source/token/font/media/locale、产品/性能/Jenkins保持开放。
