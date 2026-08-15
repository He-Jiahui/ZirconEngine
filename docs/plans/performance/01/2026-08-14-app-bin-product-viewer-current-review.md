---
related_code:
  - zircon_app/src/bin
  - zircon_runtime/src/asset
  - zircon_runtime/src/graphics
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetCompilingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp
  - dev/bevy/crates/bevy_winit/src/state.rs
tests:
  - current-source hash stability 17/17 passed
  - direct rustfmt 14/17 passed; three foreign-dirty formatting failures retained
  - renderdoccmd x64 v1.44 tool probe passed
  - managed Windows Cargo and WPR/xperf/RenderDoc current-source matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App产品入口与PBR Viewer current-source性能审查（2026-08-14）

## 范围与快照

`zircon_app/src/bin/**`当前源码 **17/17** 个Rust文件、**7,168** 行、**6,552** 个非空行、**129** 条`#[test]`已逐文件完整阅读。复核前后17个SHA-256前缀全部不变；入口目录存在其它会话的大量tracked/untracked修改，本轮不覆盖。

| 子域 | 文件 | 行/非空行 | 静态结论 |
|---|---|---:|---|
| 产品入口 | `editor.rs`、`runtime_preview.rs`、`main.rs` | 162/144 | 薄委托；无独立帧循环 |
| Viewer事件与交互 | `app.rs`、`app_tests.rs`、`args.rs`、`background_load.rs`、`base_pipeline_recheck.rs`、`camera.rs` | 2,843/2,603 | 按需redraw、后台加载、退避重检 |
| 呈现与证据 | `frame_io.rs`、`gpu_timing_evidence.rs`、`presenter.rs`、`renderdoc.rs`、`work_paths.rs` | 1,766/1,617 | 交互直呈；CPU帧仅截图/fallback；证据链有缺口 |
| 资产与场景 | `hdri.rs`、`project_assets.rs`、`scene.rs` | 2,397/2,188 | 版本化fixture与单次project open；启动阶段仍串行 |

文件哈希前缀为：`editor B4DA86322C09`、`runtime_preview 1C4484407966`、`app_tests 9B7CE9842E8D`、`app A0499DADEB39`、`args 6F133EFA4A44`、`background_load 0D291044B77B`、`base_pipeline_recheck 8382A3FFBA72`、`camera B7DBEF4E6BE3`、`frame_io 6D284E9F915F`、`gpu_timing_evidence CF38761889AA`、`hdri C8D1611CF7D8`、`main 3888903C8AF0`、`presenter 102A202F894D`、`project_assets B8C1E1C0F007`、`renderdoc 9E21A663B75C`、`scene D569A09B1AFA`、`work_paths 7D5D580BBA6F`。

直接`rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`为14/17通过；`app.rs`、`app_tests.rs`、`base_pipeline_recheck.rs`仅有当前外部改动的格式差异。本轮不改它们。测试结构扫描还确认129条测试中 **40条** 直接读取生产`.rs`文本并断言源码形状（`app_tests` 22、`gpu_timing_evidence` 1、`hdri` 3、`project_assets` 3、`scene` 11）；这些guard可防结构回退，但不能替代行为counter、Cargo执行或性能证据。

## 已确认的当前行为

1. `editor.rs`与`runtime_preview.rs`只委托统一entry；Viewer在`about_to_wait`使用`WaitUntil`，redraw请求由布尔状态合并，loading title最多1 Hz，base pipeline以16/32/64/128/250 ms上限退避并有45 s一次性超时。静态调用图没有idle `Poll`或持续空转渲染。
2. 交互路径优先`render_to_viewport_surface`原生GPU呈现；完整CPU readback、PNG/hash/metadata只在截图、证据或native fallback发生。单球按需Viewer不是逐帧CPU呈现热点。
3. Viewer project fixture已版本化为v4。球面为8,385 vertices、49,152 indices、16,384 triangles；warm readiness报告generation=0、serialized source bytes=0、asset filesystem writes=0，project只由Runtime AssetManager执行一次`open_project`。旧报告中每启动生成18,721 vertices/110,592 indices的`PERF-MVP-428`静态根因已经消除，但动态验收尚未完成。
4. `PbrMirrorScene::new`在名为`zircon-pbr-scene-loader`的独立OS线程执行，完成后用event-loop proxy唤醒UI。UI响应问题已经从主线程移出；线程内部仍按HDRI preflight -> project fixture -> runtime bootstrap -> project open -> world load -> renderer init -> IBL restore顺序执行，并已经记录各阶段wall time与生成/打开计数。

## 发现与结构方向

### P0验收缺陷：默认RenderDoc模板在新work-dir上启动前失败

`main.rs:28-36`先计算`<work-dir>/renderdoc/zircon_shader_pbr_viewer`并预加载DLL；这早于EventLoop、App与scene创建任何目录。`renderdoc.rs:345-362`只接受已经存在的父目录，不创建它。因此“新work-dir + 显式DLL + 默认capture path”会在产品启动前确定失败。另有`app.rs:947-966`在没有直连bridge时只打印“without a direct RenderDoc evidence record”并返回成功，调用方随后仍把capture标记完成；这不能作为自动化`.rdc`验收。

Render17必须把capture请求收敛为一个evidence transaction：准备E/D/F盘输出目录 -> preload/configure -> surface-ready后capture -> 停止 -> 查询latest path -> 验证非空小写`.rdc` -> 关联frame profile/graph dump。无bridge模式可保留人工调试，但自动验收必须失败或明确返回`Unverified`，不得记为完成。

### P1候选：后台线程内仍是串行启动关键路径

当前阶段化计时足以定位，不足以证明应并行哪一段。不能直接并发`ProjectAssetManager`、world与renderer owner；先对cold/warm数据建立依赖图，再只并发无共享可变owner的阶段。目标结构是：fixture/cache identity先决；runtime bootstrap、只读HDRI preflight和可独立的renderer backend准备按依赖并行；project open后world load；IBL artifact restore/compute使用受Runtime管理的task pool；必须带优先级、内存预算、取消和shutdown join，而不是再创建无界线程。

### P1候选：warm IBL仍完整读取源HDRI

`hdri.rs:54-74`在任何restore查询前都`fs::read`整个HDRI并保留bytes；`source_cubemap_environment`随后才在`118-143`检查artifact。该路径是确定的O(source bytes) warm I/O和峰值内存，但2K下是否值得改变、8K/32K下占比多大仍需File I/O与allocation数据。Runtime04应把source identity/metadata快速判定与完整内容读取分层；若artifact key必须内容哈希，则由持久文件fingerprint（size/mtime/file-id + 已验证hash）或异步流式hash负责，不能用不可靠mtime直接声明命中。

### 不立项：入口局部微优化

camera算术、单结果`mpsc`、标题字符串、一次性PNG/metadata和单球snapshot不是当前已证实热点。`render_snapshot`会复制小型descriptor/Vec，但大HDRI/PMREM/irradiance texel由`Arc`共享；在规模trace证明alloc/copy占比前，不改API或增加缓存失效复杂度。

## 参考引擎依据

- Unreal `ShaderPipelineCache.cpp:55-112`为background/fast/precompile/PSO thread-pool分别定义batch size与每帧时间预算；`808-845`按模式切换预算；`1933-2059`记录任务、wall/CPU时间并根据实测耗时增减batch。Zircon的pipeline/IBL启动工作也应有阶段、优先级和预算，不应一次性堆积到任意线程。
- Unreal `AssetCompilingManager.cpp:386-493`同时按优先级与可用内存动态限制并发，`570-599`把后台并发限制为worker约一半并给blocking工作前台余量，`768-791`统一处理各asset manager的异步完成。Zircon要借鉴的是受管理的资源预算与owner，不是简单提高线程数。
- Unreal `AsyncLoading2.cpp:3653-3740`通过thread-state time limit在工作段检查超时，并在GC等待时主动让出。若Viewer阶段未来进入Runtime任务图，必须保留取消/抢占点和主线程预算。
- Bevy `bevy_winit/src/state.rs:670-732`在Continuous/Reactive模式间选择`Wait`/`Poll`/`WaitUntil`，并在一次request后清除`redraw_requested`；这与当前Viewer的按需redraw方向一致，无依据改成持续轮询。

## 动态验收矩阵

1. 仅使用E/D/F盘work、cache、target、WPR ETL与`.rdc`。构建并运行current-source受管Viewer：HDRI 2K/8K/32K x fixture cold/warm x IBL cold/warm x native direct/forced CPU fallback x normal/screenshot/RenderDoc capture；每格至少10次cold、30次warm，报告median/p95而非单次。
2. WPR/xperf采集CPU sampling、Disk/File I/O、CSwitch/ReadyThread、heap/VirtualAlloc、DPC/ISR与energy；关联现有`hdri_decode/project_assets/runtime_bootstrap/project_open/world_load/renderer_init/ibl_restore/total`，记录读写bytes、open/count、alloc bytes/peak RSS、线程ready/wait及package/process energy。
3. RenderDoc只接受current-source产品capture：`.rdc`存在、非空且bridge latest path一致；`renderdoccmd` v1.44 replay成功，记录actions/draw/dispatch/copy/clear/present、资源峰值与marker。配对cold/warm capture，并使用wgpu timestamp时明确区分GPU duration与CPU renderer-call wall。
4. 静态硬门槛：warm fixture generation=0、serialized bytes=0、filesystem writes=0、project open=1；稳定idle redraw/render/present=0；native interactive CPU full-frame readback=0；每次自动capture有且只有一个可验证artifact。40条源码形状测试逐步以行为counter/临时E盘fixture测试补齐，但保留必要的architecture guard。
5. 只有paired before/after证明被改阶段wall、CPU、I/O/alloc或energy显著下降，且像素、交互、fallback、shutdown、device-loss与Cargo回归通过，才能声明收益。UE数据只作结构参照；不同硬件/场景不能据此声称功耗或耗时“接近UE”。

## 跨计划交付与本轮决策

| Owner计划 | 必须解决的合同 | Performance验收 |
|---|---|---|
| `zircon_runtime/render/17` | capture evidence transaction；pipeline/IBL阶段marker、timestamp、批量/帧预算 | 新work-dir可捕获；无bridge不误报；`.rdc`与profile/graph同generation |
| `zircon_runtime/runtime/04` | HDRI source identity、warm artifact快速判定、受管理异步I/O/compute与取消 | warm完整源read bytes按证据归零或有明确hash必要性；cold结果一致 |
| Performance `01` / App入口owner | 保留按需redraw、单project open与版本化fixture；建立cold/warm产品矩阵 | F0/F2/F4 gate、WPR/xperf/RenderDoc及行为counter全部通过 |

本轮没有修改源码：RenderDoc缺陷涉及正在被其它会话修改的`main/app/renderdoc`，启动并行又必须等待阶段profile与跨owner设计。静态审查完成但current-source受管Cargo、WPR/xperf、allocation/energy与RenderDoc产品capture均未完成，因此整个`zircon_app/src/bin/**`继续留在`pending.md`，不得进入`review.md`、提交里程碑或发送验收企微。
