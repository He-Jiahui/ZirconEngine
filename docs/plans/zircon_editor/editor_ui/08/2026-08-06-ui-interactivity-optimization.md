---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/record.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
---

# Editor UI 可交互性优化计划

## 1. 目标与边界

本计划承接 EditorUI08 的 Runtime UI 承载切换与 `editor-event-full-reflection-rebuild` 失败交接，优先恢复主 Workbench 在 pointer move、hover、scroll、keyboard 与局部状态更新下的可交互性。最终路径必须由已提交 generation 持有 presentation、hit/control/paint index 与编译后的绘制段；输入和绘制 consumer 只读取该 generation 或提交窄 interaction patch，不再在事件回调中反射、深拷贝或重建整棵 UI。

本计划只优化现有产品路径，不通过以下方式制造表面改善：降低输入采样率、丢弃 press/release/scroll、限制刷新率、关闭诊断功能、只要求 release 构建可用、增加第二份无失效约束的 host-local cache，或把全树扫描下移到另一个 helper。

## 2. 当前基线

- 主窗口采用 `ControlFlow::Wait` 与 OnDemand edit mode，空闲约 `0.004` core，未发现空转事件循环；主要开销发生在真实输入触发的同步 projection/paint 链。
- 修复前约 33 Hz hover 采样：top workbench `0.43` core、document controls `0.83` core、status `0.55` core。
- 当前止损已让 Workbench hit 直接借用已提交 node model，不再为每次 hit 构建 `UiSurface`。修复前 debug bundle 在两个 document control 间交替 600 次、30 ms 间隔时耗时 `21.343 s`、约 `0.964` core。
- 内建 profile 已把剩余 P0 定位到 damage paint 内的重复文本布局：422 次实际 region paint 中，每次约访问 96 个 template nodes、damage 拒绝 77 个，只 clone 约 19 个；但 `paint_command_text` 仍在 8491 次调用中累计约 `15.481 s`。viewport RGBA 深拷贝、全 hover projection、event-time hit surface rebuild 与 damage 前 node clone 也是已确认放大项。
- 当前切片已将 viewport RGBA 改为 shared immutable payload，把 hover 收束为 paint-time transient state，在 damage 检查后才 clone node，并让稳定文本投影复用有界缓存；同时把输入、菜单、滚动与绘制切换到 generation-owned spatial/paint index，主题切换到 immutable snapshot。最终 debug bundle 预热后三次相同 600-event 采样分别为 `19.466 s / 0.306 core`、`19.242 s / 0.348 core` 与 `19.426 s / 0.307 core`，窗口均保持 responding，达到本计划 debug 产品 gate。
- 当前 `zircon_editor` package build 与 bundle smoke 通过；`FrameRect::right()` / `bottom()` 支撑层阻断已修复。focused lib test 的首轮共享基线有 569 个 test-only 编译错误；补回 `workbench_projection` 父模块遗漏的 support 导入后，级联错误降至 148 个。剩余错误分散在 plugin/settings/scene/world 等并行 API 迁移，目标用例仍未开始执行，不能将其误报为本切片测试通过。

当前直接 borrowed hit 修改是 M2 之前的止损，不是最终验收形态：它消除了 event-time surface rebuild，但仍会线性扫描 Workbench nodes，且事件入口仍读取 owned presentation。

## 3. 依赖顺序与目标架构

```text
业务状态 / interaction / viewport / theme / diagnostics dirty domains
  -> 单次 generation commit
  -> immutable HostPresentationGeneration
       + control/hit index
       + section/spatial paint index
       + compiled ordered paint segments
       + shared resource handles
  -> input consumer: borrowed route + typed interaction patch
  -> damage consumer: changed section/segment patch
  -> presenter: bounded damage regions + GPU submit
```

底层顺序不可颠倒：先建立 dirty-domain generation 与唯一提交 authority，再迁移 input consumer，随后才能让 damage/paint 复用同一代索引。事件合并只能作为最终调度优化，不能代替前三层的零重建合同。

## 4. 统一预算与观测

所有里程碑复用同一组计数器，计数器只累计数值，不在每帧同步导出 JSON/PNG：

| 预算项 | 稳定 generation 目标 | 变化 generation 目标 |
|---|---:|---:|
| full presentation clone / viewport RGBA copied bytes | `0` | 结构 snapshot `<= 1`，RGBA 只传 shared handle |
| reflection / hit surface / control index rebuild | `0` | 每个 dirty generation `<= 1` |
| unchanged node visit / command build / handler probe | `0` | 与 changed nodes 和 damage sections 线性 |
| same-target hover property write / layout dirty | `0` | 只更新 old/new target |
| normal ordered stream fallback sort | `0` | 显式乱序输入才允许 fallback |
| profile artifact encode / file I/O | `0` | 仅显式 one-shot capture，在有界 worker 执行 |

产品 gate 同时覆盖 debug 与 profiling 构建。固定 1672x941 Workbench、预热后在两个真实 document controls 间交替投递 600 次 pointer move（30 ms 间隔）：debug 总时长 `<= 20.5 s`、进程 CPU `<= 0.35` core、无 `NotResponding`；主线程 input-to-damage p95 `<= 8 ms`、damage-to-submit p95 `<= 16.7 ms`。profiling 构建的 input-to-damage p95 `<= 4 ms`。RSS/Private Bytes 在 10 分钟交互压力下不得持续增长，像素、clip、z-order、popup 和焦点语义必须与 full repaint 对拍一致。

## 5. 里程碑

### M0 可复现基线与性能合同

- Goal：让当前卡顿可以被自动复现、分段归因，并为后续里程碑提供不可弱化的预算合同。
- Dependencies：现有 profiling scopes、geometry artifact 与 Windows bundle 构建脚本。
- Implementation slices：补齐 full snapshot clone bytes、reflection/index build、node visit、command build、damage area、fallback sort、artifact export 次数；把固定 pointer storm 驱动与产品采样参数固化到受管验证入口；修复阻断 focused tests 的最低共享测试支撑问题。
- Testing stage `M0-Baseline-Gate`：运行性能计数器单测、geometry/hit route 合同、`zircon_editor` package build，并用当前 debug bundle 复跑 idle 与 600-event 场景。失败预算可以作为基线结果，但采样必须稳定复现已知放大且计数器本身不得改变产品路径。
- Exit evidence：同一场景连续三次的 wall time/core/p95 波动在 15% 内；P0 放大能被上述计数器直接定位；focused test batch 能实际开始执行。

### M1 Dirty-domain generation 与 immutable presentation authority

- Goal：状态变化只重建受影响 domain，并原子发布一个可共享、可借用的 `HostPresentationGeneration`。
- Dependencies：M0 计数合同；runtime `UiSurfaceFrame` 与现有 invalidation bitset。
- Implementation slices：把 structure/layout、interaction、viewport image、theme、render stats/diagnostics 分代；event effects 不再把空或窄变化默认升级为 `PRESENTATION_DATA`；一次 frame 内相同 captured generation 最多 apply 一次；main/native windows 共享结构与 pane artifact；viewport bytes、文本和资源改用 immutable/shared handle；在 generation commit 时同步发布 control/hit、popup stack、section/paint index，禁止 consumer 私建第二权威。
- Testing stage `M1-Generation-Gate`：覆盖 dirty mask 到 domain generation 的完整映射、同值 no-op、重入 mutation、旧 generation 拒绝 apply、main/native cursor、viewport/resource 生命周期；运行 package build、focused generation/invalidation/host lifecycle tests，再跑 1/100/10k node 和 1k stable read 计数矩阵。
- Exit evidence：stable generation 的 full presentation clone、RGBA copy、reflection/index rebuild 均为 `0`；每个真实 dirty generation 的 snapshot/index publish `<= 1`；现有 paint-only fast path与 post-submit diagnostics 语义保持。

### M2 输入、命中与 transient interaction 硬切换

- Goal：pointer/keyboard/scroll 只查询已提交 generation 的稳定索引，same-target move 成为零写入路径。
- Dependencies：M1 generation-owned control/hit index、popup z stack 与 shared presentation handle。
- Implementation slices：删除 event-time Workbench/Panes `UiSurface` rebuild 和全 presentation owned read；统一一次 event 一次 route，popup/base/pane 不重复命中；hover/focus/pressed/capture 形成 typed interaction patch 与 old/new damage；菜单、table row、floating window、viewport toolbar 使用同代索引；在语义稳定后才允许 event pump 合并连续 pointer move，且不得跨越 press/release/scroll/focus 边界。
- Testing stage `M2-Input-Gate`：覆盖 clip、reverse paint order、disabled/separator、popup blocking、table row、capture、drag、context menu 与 viewport handoff；运行 1/100/10k nodes、1k same-target 与 alternating-target storm，随后执行真实窗口输入采样。
- Exit evidence：1k moves 的 surface/bounds/index rebuild、整 node/row clone、RGBA copy均为 `0`；无 popup 时全 node scan `0`，10k popup rows 每 hit visited `<= 2`；same-target property write/layout dirty `0`；产品 input-to-damage p95 达到统一预算。

### M3 Damage-driven command extraction 与 compiled paint segments

- Goal：局部 damage 只访问相交且发生变化的 section/segment，不再进入完整 Workbench painter。
- Dependencies：M1 section/spatial index、theme/resource generation；M2 typed interaction damage。
- Implementation slices：从 dirty generation 直接形成 patch command stream；按 top chrome、dock、document、status、floating、popup/overlay 划分 section；每个 node/component 在变化时编译 already-ordered paint segment，稳定帧不再进行字符串分类、theme lock、文本测量、资源解析或 render-command 多次转换；componentized extension 的 parent/subtree/root/clip 索引在 generation commit 时构建一次；damage 使用固定容量 region set，只有明确阈值才升级 Full；image/atlas 只携带 resource handle/UV/generation。
- Testing stage `M3-Paint-Gate`：覆盖 full/patch pixel parity、clip、z-order、popup overflow、viewport 合成、typed Border、乱序 fallback、资源失效与 theme generation；运行 1/1k/10k nodes 的 single-section damage 计数矩阵、package tests 与 GPU/Softbuffer 对拍，再复跑产品 pointer storm。
- Exit evidence：single-section damage 的 unchanged visited/build/clone/handler probe/theme lock/filesystem access为 `0`；changed commands 与 changed nodes/damage 面积近线性；正常 stream fallback sort `0`；damage-to-submit p95 与 CPU 达到统一预算。

### M4 生命周期收敛与产品验收

- Goal：把前三个里程碑接入 tick、dispatch side-effect、native window、viewport submit 与 diagnostics，移除旧宽刷新路径并完成可执行 bundle 验收。
- Dependencies：M1-M3 全部 gate 通过。
- Implementation slices：render stats 不再无条件 mark structure presentation dirty；pending decision 每 frame 最多按 captured generation apply 一次；native window 持 applied target/presentation/bounds cursors，stable OS property calls 为零；profiling artifact 改为显式 one-shot + 有界后台编码；删除已经没有调用者的旧 reflection/presentation/paint fallback 分支和临时 borrowed full scan。
- Testing stage `M4-Product-Gate`：按 `docs/plans/milestone-validation-policy.md` 执行一次 batched `zircon_editor` package check/build、所有相关 focused lib/integration contracts、debug 与 profiling bundle smoke；进行 10 分钟真实 hover/scroll/typing/docking 压力、WPR/内建 profile 采样和 full/patch screenshot parity。
- Exit evidence：统一产品预算全部达标；无非响应、输入丢失、焦点/拖拽/菜单回归；旧宽刷新与 event-time rebuild 源码守卫归零；构建脚本产出的 exe 可独立启动并携带完整资产/DLL。

## 6. 调试与纠偏规则

- 上层场景失败时先回查最低共享层：generation publish/失效 -> stable index -> typed damage -> command segment -> presenter，不在 event handler 添加例外缓存或专用旁路。
- 任一里程碑的 correctness parity 或预算未满足时保持 open，只重跑受影响 focused batch；修复后再执行该里程碑完整测试阶段。
- M1-M3 不允许并行修改同一 presentation/index authority；可以并行补独立测试与采样工具，但合入以 dependency order 为准。
- 产品测量必须关闭每帧 artifact export，保留 profiler counter；诊断采集本身的 CPU、I/O 和分配必须单列，不能计入或污染交互样本。

## 7. 状态与产出记录

最新产品验收又发现并修复了一个独立的 runtime 启动崩溃：Vulkan backend 存在 seed data 时，旧 gate 会在 device 未启用 `wgpu::Features::PIPELINE_CACHE` 的情况下调用 `create_pipeline_cache`。`RuntimePipelineCache` 现在同时要求 backend、seed data 与 device feature 三项成立，否则返回显式 `UnsupportedDeviceFeature` 并关闭 cache；`cargo check -p zircon_runtime --lib --locked` 通过。精确 bundle 位于 `C:\Users\HeJiahui\ZirconBuilds\ui-perf-validation\pipeline-cache-fixed-20260807-0752`，smoke 通过，`zircon_editor.exe` SHA-256 为 `284D5F2FB4910BB66701B519AF5BD2161740310D95E8A12395979A46E8C7E614`，`zircon_runtime.dll` SHA-256 为 `B2D41A1886D0CEFCDA062E8F47E0F112BEC444C52140321F7DD587BA2A811EF2`。该 EXE 在真实 GPU 产品路径持续运行 30 秒，窗口保持 responding，stderr 为 0，正常退出且不再触发 pipeline cache validation error。

同一精确 bundle 的进程内部 reference 与实际桌面 GPU 客户区均为 `1672x941`。从可信全桌面捕获按客户区边界裁切后，GPU 帧文件 `parity/screenshot_gpu_desktop_crop.png` 的 SHA-256 为 `3158AB31C088650B0E93DAAE5AA90238C582B50EF4917F99D4BD81B2923E6118`；相对内部 reference 的 differing-pixel ratio 为 `0.0165621`，average-channel delta 为 `0.977978/255`。目视复核确认 Workbench、Hierarchy、Inspector、Console、状态栏与 popup/chrome 层级完整，Inspector 文本无覆盖。该证据关闭当前版本 reference-vs-GPU 门槛，但不替代最新版本 GPU-vs-Softbuffer 对拍。

随后用同一源码构建启用 `target-editor-host,profiling,profiling-chrome` 的独立 bundle，并通过 `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` 捕获精确 `1672x941` 客户区。bundle 位于 `C:\Users\HeJiahui\ZirconBuilds\ui-perf-validation\profile-softbuffer-fixed-20260807-0818`，`zircon_editor.exe` SHA-256 为 `4159E72ACA758C4F81C046F0EB1F4EC98DC6DB81A307E05D7AD918A07D763EFD`，`zircon_runtime.dll` SHA-256 为 `3DD843F1BA6C4E507102868168974C89CD67ACB8AE0191553E0B3A09EE151217`。Softbuffer 窗口在捕获时 responding，stderr 为 0，正常退出；reference vs Softbuffer 的 differing-pixel ratio 为 `0.000509104`、average-channel delta 为 `0.0312068/255`，GPU vs Softbuffer 为 `0.0165112` 与 `0.976091/255`。三组画面结构、clip、z-order 与内容一致，当前精确版本三向产品像素门槛关闭。

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
| M0 | 可复现基线、计数器与构建入口 | 执行中 | - | 性能采样与构建脚本已稳定；共享 support 导入修复把 `lib test` 错误从 569 降到 148，但其余并行 API 漂移仍阻止 focused test 进入目标用例。 |
| M1 | immutable presentation 与 dirty domains | 执行中 | - | shared viewport/theme/generation 与 diagnostics 独立分代已接入；主题快照现在只在启动或 token 变化时同步，稳定 generation 读取不再访问全局主题 authority；产品配置 check 通过，完整 generation 合同测试仍受共享测试基线阻断。 |
| M2 | generation-owned input/hit/hover index | 执行中 | - | 产品 input-to-damage p95 已达标，same-target/transient hover 与稳定索引已接入；完整 popup/clip/focus focused batch 待共享测试基线恢复。 |
| M3 | damage paint 与有序命令提取 | 执行中 | - | 正常 hover 帧 fallback sort 为 0，模板访问已降至 72/frame；当前精确版本 reference/GPU/Softbuffer 三向对拍通过，Inspector 容器标题重叠已修复，完整 focused full/patch contracts 仍受共享测试基线阻断。 |
| M4 | 生命周期、旧路径删除与产品验收 | 执行中 | - | bundle、三向像素、600-event、10 分钟压力、30 秒真实 GPU 存活与产物审计通过；旧宽刷新源码守卫与完整 focused tests 仍开放。 |

当前执行切片（不等同于里程碑完成）：2026-08-07 已完成 generation-owned shared presentation、damage-driven spatial/paint index、transient interaction、state-owned immutable theme snapshot、有界文本缓存、显式一次性异步 profile 导出、native window 同值 no-op，以及 diagnostics 独立 generation。主题 authority 只在启动或设计 token 变化时同步到 host state，普通 pointer/keyboard generation read 只克隆已有 `Arc`。Windows 原生 `cargo check -p zircon_editor --lib --locked` 在 Inspector 修复后再次通过，profiling 产品配置 `cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host,profiling --locked` 通过；`tools/tests/build-editor.Tests.ps1` 两次复核均为 `3/3` 通过；协调器 artifact audit 返回 `unmanaged: []`。

profiling 产品构建在 600 次交替 pointer move 下记录 `input-to-damage p95 = 83.4 us`、`damage-to-submit p95 = 2254.1 us`、约 `0.214 core`，两项端到端 p95 均已关闭。随后 generation paint-index catalog 的 debug + profiling-feature 对照把 `template_node_visit_count` 从 `1336/frame` 降至 `72/frame`（`-94.6%`），clone 从 `177/frame` 降至 `49/frame`，damage reject 从 `1159/frame` 降至 `23/frame`，正常 hover 帧 fallback sort 从 `1/frame` 降为 `0`；同配置 `damage-to-submit p95` 从 `10.520 ms` 降至 `6.456 ms`，CPU 从 `0.625 core` 降至 `0.512 core`。最新已采样可执行 bundle 为 `C:\Users\HeJiahui\ZirconBuilds\ui-perf-validation\index-catalog-20260807-053410`，600-event 墙钟 `19.024 s`、窗口全程 responding、正常退出；`zircon_editor.exe` SHA-256 为 `D35711B0148A40A910B35CF91DB593BAD4CBDE174021AF4D84C72F4FE12B922B`。

此前切片的首帧 full repaint 对拍通过现有阈值：software reference vs GPU 的 differing-pixel ratio 为 `0.024788`、average-channel delta 为 `0.8704`，GPU vs Softbuffer 仅 `16` 个像素不同（ratio `0.0000102`、average delta `0.0000194`）。当前精确版本改用验证目录中的最小临时工程，移除会触发共享 IDE shader stub 的项目级 shader 资产，并把无效的 Static-under-Dynamic fixture mobility 修正为 Dynamic；产品路径未改动。由此成功生成 `theme-inspector-fixed-20260807-0713-first-frame.png`，同时暴露并修复了 `InspectorEditableFieldsPanel` 大容器文本被通用 painter 垂直居中、覆盖 `Transform` 行的问题。修复后的独立 bundle 位于 `C:\Users\HeJiahui\ZirconBuilds\ui-perf-validation\theme-inspector-fixed-20260807-0713`，smoke 通过，`zircon_editor.exe` SHA-256 为 `5CA8A1915E764DCA8ED9ED5FB243D84FE74C4AB1DF90DFFCD3EA67F7310394D8`，`zircon_runtime.dll` SHA-256 为 `D53988727C86CF4540038AF5A6688B381779B7667A96C0EF8DAC1525CD6C0E1F`。当前精确版本的 full/patch GPU/Softbuffer 三向对拍尚未重跑，因此 M3 像素门槛仍保持开放。10 分钟压力证据仍为 `14,406` 次 hover/正向 scroll/keyboard/短 document-tab drag 循环、`0` 次无响应、平均 `0.034 core`，Private Bytes 后 5 分钟保持平台；反向滚轮因 harness 的 PowerShell 无符号转换告警不计入证据。聚焦命令 `cargo test -p zircon_editor --lib inspector_pane_projects_editable_field_nodes_and_actions` 仍未进入目标用例：共享 support 导入修复后，全量 `lib test` 配置仍先因 148 个其他测试 API 漂移错误编译失败。
