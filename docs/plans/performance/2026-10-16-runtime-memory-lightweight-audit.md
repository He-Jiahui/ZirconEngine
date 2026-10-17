---
title: Rust 内存轻量化与 Wasm 边界排查
date: 2026-10-16
status: implemented_pending_validation
scope: Runtime first; Rust product roots inventoried, selected data owners inspected
---

# Rust 内存轻量化与 Wasm 边界排查

## 结论与证据边界

本次检查以游戏运行时为优先，关注大数组复制、同一载荷的重复持有、线程交接和 Wasm 可移植性。已修改切线生成、morph 重建、CPU 蒙皮和材质 uniform 上传四条路径；未改变公共 API、项目/缓存格式，也未引入 unsafe。修改涉及五个 Rust 文件，保留共享 checkout 的既有修改。

源码检查能够证明移除了哪些显式复制和预分配，不能证明实际 RSS、allocator retained pages、GPU 驻留或 Wasm 运行内存下降。Rust 构建、回归运行、产品内存测量和浏览器验收仍待完成，状态保持 `implemented_pending_validation`。

本轮选择“单线程浏览器可运行，多线程按能力启用”作为后续 Wasm 基线；不是已经取得的运行能力。既有内存控制面问题仍沿用 Runtime196 / Tooling25 的 MEM 编号，不把此次局部修改作为全局预算、OOM 或回收控制面已完成的证据。

## 全仓词法扫描

扫描 Runtime（含内部 crates）、RuntimeHost、RuntimeInterface、App、Editor、Hub 和 Plugins，共 13,365 个 Rust 文件、1,614,594 行、58,529,900 字节，无读取失败。排除了 test/tests/test_support、bench/benches、examples、fixtures、generated、vendor、target 路径段及明显的测试文件名；生产文件内的条件测试块仍可能计入。

| Rust 根 | 文件数 | to_vec 出现数 | clone 出现数 | Arc 类型出现数 |
| --- | ---: | ---: | ---: | ---: |
| zircon_runtime | 6,134 | 192 | 6,610 | 2,490 |
| zircon_editor | 4,868 | 99 | 6,058 | 1,249 |
| zircon_plugins | 1,470 | 140 | 1,016 | 363 |
| zircon_app | 196 | 13 | 150 | 99 |
| zircon_hub | 178 | 12 | 554 | 57 |
| zircon_runtime_interface | 506 | 8 | 218 | 48 |
| zircon_runtime_host | 13 | 0 | 0 | 0 |

按载荷字段名与复制操作筛出 114 处候选；另有 57 个嵌套 Vec 类型表达式、5 处无界通道构造。它们是人工跟踪的数据流入口，不是已确认缺陷数或实际分配次数。共享句柄、按值移动、必要的独立可写结果均不能计为多份物理载荷。

扫描输入按路径 casefold 排序，以 `path|SHA256(file)` 加 LF 聚合的 SHA256 为 `733d70e6fa7ef6ecbc310e44dda6e401eeedba0fd5437bac27e889af9bbfe4b0`。这是逐文件观测指纹，不是冻结的源码快照；最终格式调整后的修改文件哈希单独记录在交接元数据。

## 已修改的数据路径

以下字节公式描述被移除的源码级载荷和临时缓冲，不是进程内存实测。V 为顶点数，I 为索引数，S 为实际拆分的顶点数。

| 路径 | 原有问题与修改 | 可核对的载荷变化 |
| --- | --- | --- |
| morph 法线/切线重建 | 基础位置、法线、UV、切线和索引先复制，实际只在所有目标计算成功后才修改原网格。改为借用基础流，只保留两个可写工作数组与待发布结果。 | 请求切线时移除 `48*V + 原索引字节数`；不请求切线时移除 `24*V + 原索引字节数` 的基础流副本。百万顶点的 48*V 约为 45.8 MiB。 |
| 切线接缝拆分 | 原代码先建立完整 u32 源索引、完整重写索引，并按 `I-V` 预留哈希表。改为直接读取原索引，实际出现接缝时才建立重写索引与分组；首次拆分前的索引按原顺序补入。属性追加直接 reserve/push。 | 无接缝时不再分配两份 `4*I` 的索引缓冲和预留分组表；有接缝时消除源索引 `4*I` 及每属性一次 `S*属性步长` 的中间数组，保留必要的重写结果。 |
| 每帧 CPU 蒙皮 | 直接网格转换已经拥有顶点/索引，却通过借用接口再次构造网格。改为在自有顶点缓冲上计算并移动索引；同时准备 GPU 原始源与 CPU 回退时，将自有源移入结果。 | 直接路径消除一份 `V*size_of::<MeshVertex>() + 4*I` 的临时载荷；双路径的原始源从复制改为移动，仍保留必要的 GPU 源和 CPU 回退。 |
| 材质 uniform 上传 | 每个属性载荷先 clone 成 Vec，再扩展到 256 字节。达到 ABI 下限时直接借用载荷，短载荷使用固定 256 字节栈缓冲补零。 | 消除该准备步骤的临时堆分配；大载荷不再整段复制，短载荷仅复制有效字节。GPU 上传本身仍需要传输。 |

所有 morph 结果仍在整批计算成功后发布；接缝拆分继续保留 u16/u32 索引格式选择和校验失败回滚；GPU 调色板超限继续使用原 CPU 回退。

主要生产 owner：

- `zircon_runtime/src/asset/assets/mesh/tangents.rs`
- `zircon_runtime/src/graphics/scene/renderer/mesh/build/skinning.rs`
- `zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/resource.rs`

## 仍需处理的具体问题

| 优先级 / owner | 当前源码证据 | 后续处理与验收 |
| --- | --- | --- |
| P1 · Wasm 功能依赖 | `zircon_runtime/src/lib.rs` 明确拒绝 `native-host + wasm32`；Cargo 中 `graphics`、`text`、`script`、`cooked-assets` 等仍引入 native-host。`platform-web` 不能单独解除该限制。 | 将 cooked 读取和运行时消费者与本地导入、文件监听、动态库宿主分开；建立真实浏览器运行配方，分别验证单线程与可选共享内存线程路径。不能通过删除编译保护声明支持。 |
| P1 · 线程调度可移植性 | `core/runtime/tasks/parallel_for.rs` 依赖 Rayon，pool/owned workers 创建原生线程；渲染流水线也有原生 worker。 | 审查所有创建线程的入口，单线程后端复用同一数据所有权接口；Wasm 多线程单独声明能力、内存和部署要求。 |
| P1 · 无界订阅 | `asset/pipeline/manager/assets/runtime.rs::subscribe_asset_changes_internal` 使用 unbounded channel，慢消费者可持续积累事件。 | 结合已有 bounded(1) generation wake 设计合并/溢出后重新同步；覆盖订阅者停滞、突发导入、销毁和事件完整性。直接改成阻塞发送可能造成锁内阻塞，必须联动消费者。 |
| P1 · 缓存编码前深复制 | `asset/artifact/cache_payload/model.rs::From<&ModelPrimitiveAsset>` clone 顶点、索引、SDF 和虚拟几何后才序列化。 | 为编码路径提供借用的序列化视图，解码保留拥有数据的类型；用相同工件字节、roundtrip 和大模型加载/保存峰值验收。 |
| P1 · 大型固定数组内联与历史复制 | `graphics/scene/renderer/mesh/build/pending_mesh_draw.rs` 每条记录内联两个 `Option<SkinnedMeshJointPaletteStorage>`；该 POD 由 256 个 4×4 f32 矩阵和 16 B 参数组成，即 16,400 B。仅两份数组内容就有 32,800 B/条，`None` 仍占结构体空间，标签与其他字段另计。`gpu_scene/prev_skinned_palette.rs` 的 `.copied()` 及提交后逐项复制还会复制整个快照。 | 待绘制记录保存堆上不可变快照的共享引用，或保存带 generation 的 arena 句柄；同一姿态在多次绘制间共享。保留 GPU POD 布局及成功提交后推进历史的协议，验证无蒙皮记录大小、共享指针身份、跨帧姿态及失败帧历史。 |
| P1 · CPU/GPU 驻留策略 | GPU 原始源、CPU 回退、MeshAsset/ModelPrimitive 等表示需要按消费者逐一对账。双表示可能有业务必要，不能仅凭 clone 判定冗余。 | 区分权威资产、可重建表示、in-flight 上传和碰撞/拾取使用者；验证场景卸载、热重载、最后租约释放，按产品配置决定可丢弃的 CPU 副本。 |
| P2 · 导入上下文复制 | `AssetImportContext` derive Clone，主源为 Vec，companion map 也持有 Vec；外部模型导入 `source_plan.rs` 同时准备源、写入数据与 companion 文件。 | 优先移动一次性所有权；确需多消费者时在既有 owner 内共享不可变载荷，并保留事务文件一致性与错误定位。公开字段类型调整需同步插件和调用者。 |
| P2 · 容量与回收 | 既有 Runtime196 记录 asset leases、arena、UI pool、transient GPU pool 的局部计量/trim，不能替代全进程账本。RHI reference_1080p_mid 预算是桌面参考上限。 | 复核峰值后容量、外部 Arc 租约与卸载回收，建立 Wasm 产品预算。上限不是已分配字节，逻辑 trim 也不等于 RSS 或 Wasm 线性内存缩小。 |

已核对的合理共享：`graphics/runtime/framework/pipelined/queue.rs` 使用复用的单槽有界通道、按值移动帧包和 `Arc<UiRenderSubmission>`；GPU 句柄和 atlas 上传页的共享引用不能计作深复制。`AssetImportContext::source_str` 的正常 UTF-8 路径已经借用，错误路径的字节复制用于现有拥有错误数据的接口，不能把它当作正常导入的整块复制。

原则是按生命周期选类型：短期只读使用借用，独占数据按值移动，需要多消费者时共享只读载荷；可增长数据继续使用 Vec，并管理容量和退出条件。大数组不迁入栈，不用全局锁包住全部资源，也不以永久缓存掩盖分配次数。

## 验证与交接

新增四个回归覆盖：材质 ABI 下限/以上的载荷指针身份，短/空载荷的补零和重复使用，拥有网格蒙皮时顶点/索引分配复用，以及 GPU 源保持原始内容、CPU 回退仍产生正确姿态。

Jenkins 待执行本批 `module_unit` 验证：绑定确切测试全名，要求覆盖四个新增回归以及既有切线、蒙皮和调色板超限回退测试。当前正式配方执行受影响包的编译、测试发现和测试阶段；交接预检不代表这些阶段已运行。Wasm 目标矩阵和浏览器产品验证分别记录，不从原生测试结果推导通过。

产品测量使用相同场景/资源、构建配置和硬件，分别记录冷加载峰值、稳态、突发导入/慢消费者、场景卸载后和重复装卸后的 live bytes、分配次数/字节、复制字节、容量、RSS/commit、GPU/staging 与帧耗时。共享载荷按分配身份计一次，引用/租约另计；Wasm 区分存活分配和已增长的线性内存。基线完成前不承诺总内存下降百分比。

本轮未创建源码副本、构建树、临时脚本或夹具。清理释放磁盘 0 B；保留本报告、精确补丁/源哈希与验证交接元数据。记录位于 `.codex/outbox/reports/runtime-memory-20261016/artifacts.json`；待验收项保持开放。
