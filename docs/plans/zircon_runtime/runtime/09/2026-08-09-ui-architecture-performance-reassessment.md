# Zircon UI 架构与性能重评及增量刷新实施计划

状态：`ACTIVE / PRODUCT HOT PATH OPTIMIZED / FULL-SYSTEM GATES STILL OPEN`

日期：2026-08-09

范围：`zircon_editor`、`zircon_runtime`、`zircon_runtime_interface`、`zr_rhi_wgpu` 的编辑器 UI 输入、状态投影、布局、命中、绘制、图片资源和 GPU 提交路径。

本报告重新评估 `2026-08-07-runtime-ui-incremental-refresh.md` 的完成状态，并记录随后完成的产品热路径优化。按钮和原生窗口缩放已从同步全量投影/重建转为局部 patch、事务门控和跨帧 GPU 产物复用；这不等于整个 UI 子系统已经完成增量化，未关闭项见第 12 节。

## 0. 执行摘要

截至 2026-08-09 19:20，本轮已经贯通编辑器产品中的按钮、窗口缩放、资源 watcher、SVG CPU cache 和 GPU present 热路径，不再只依赖 runtime 局部单测推断性能。

| 产品场景 | 优化前 | 当前有效 profile | 结果 |
| --- | ---: | ---: | --- |
| 按钮点击 tick p95 | 10.19 ms | 6.44 ms | 下降 36.8% |
| 按钮点击 slow/presentation/workbench/full-command | 4 / 8 / 4 / 4 | 0 / 0 / 0 / 0 | 点击改为 command range patch |
| 24 次 UIAutomation resize 平均返回 | 21.84 ms | 13.76 ms | 下降 37.0%，进入 16.67 ms 平均预算 |
| 24 次 resize 最大返回 | 28.43 ms | 18.48 ms | 无 `>100 ms` spike |
| resize batch plan / vertex upload / text build | 25 / 25 / 25 | 1 / 1 / 1 | generation + projection cache 命中 24 次 |
| resize retained texture copy | 25 次 / 124,517,408 B | 1 次 / 6,293,408 B | 事务内直接交换链提交，复制字节下降 94.9% |
| SVG/visual raster 调用 | 344 | 46 | 下降 86.6% |
| resize GPU image upload | 逐次 prepare 风险 | 1 次 / 131,072 B | 预热上传后 24 次 cache hit，不重复上传 |

当前 profiling 产物：

- 按钮：`E:\zircon-profiles\runtime09-svg-m7\runtime09-chrome-patch-button-20260809-164757`
- resize：`E:\zircon-profiles\runtime09-svg-m7\runtime09-resize-direct-swapchain-uia-20260809-1920`
- profiling EXE：`E:\Git\ZirconEngine\target\profiling\zircon_editor.exe`
- runtime DLL SHA-256：`1AE9896AAD3000BA1C5BA7117DDB444F15290D1AE8D7B8DCBBBFA61ED4788996`

有效结论是：已测按钮交互不再触发全 Workbench 重建，连续 native resize 不再在每个 `WM_SIZE` 上重跑布局/模型/命令提取、批处理、顶点上传、文本构建或 retained texture copy。最终稳定尺寸仍执行一次正式 reflow 和 retained cache 重建，这是正确性要求，不应被消除。

## 1. 结论

当前卡顿不是单一布局函数、SVG 解码器或 GPU 上传函数造成的，而是 UI 数据流中存在多套可变权威状态，窄事件最终被扩张为整套编辑器 UI 的重复投影和重建。

最主要的因果链为：

```text
按钮/窗口/资源事件
  -> 全局 HostInvalidationMask
  -> recompute_if_dirty
  -> 全量 Shell snapshot / WorkbenchViewModel / geometry
  -> Workbench template bridge 多次布局和状态投影
  -> 全量 HostWindowPresentationData 构造与替换
  -> hit index / render commands / image preparation 级联更新
```

已确认的核心问题：

1. `HostInvalidationRoot` 只有全局位和计数器，没有 changed node、pane、window、resource 集合，也没有可提交的帧事务。
2. `recompute_if_dirty` 将 presentation/layout/window 级脏标记汇聚为全量 shell、model、geometry、template bridge 和 presentation 工作。
3. Workbench 模板桥在一次调用中可能先执行 mount layout，再投影 toolbar/menu/drawer/responsive 状态，最后再次布局；同一 surface 存在重复布局和重复状态同步。
4. 反射控制树已经有 typed delta queue，但 tick 中的 view refresh report 被丢弃；它没有成为 retained editor UI 的脏传播来源。
5. `HostPresentationGeneration` 优化了读侧共享，但写侧仍物化和替换整份 `HostWindowPresentationData`，所以它不是增量 presentation authority。
6. 资源事件按时间预算分批 drain，却对每个不完整批次同步执行一次 runtime project refresh 和 presentation commit。启动积压因此被放大为多帧长阻塞。
7. CPU SVG/raster 缓存和 GPU image cache 都已经存在。重复加载现象的根因主要是全局缓存失效、重复模板节点访问、重复命令准备和不稳定/过宽的 generation，而不是“完全没有缓存”。
8. runtime 的局部 arranged/hit/render patch 已存在，但编辑器 Workbench 主路径仍由另一套 retained host/presentation 流程主导，局部机制在产品中的可达率不足。
9. 鼠标移动的 native callback 目前读取上一代已提交 bridge frame，空闲 hover 已较轻；按钮点击仍可能通过宽脏标记进入上述全量路径。
10. 根窗口尺寸变化通常需要 `O(N)` 更新依赖根尺寸的节点几何，这是不可消除的；当前错误在于一次几何变化触发多次 `O(N)` 布局及额外的模型、模板、presentation、命中和绘制全量工作。

因此，正确方向不是继续为每一个全量重建结果叠加缓存，而是收敛 UI 权威状态，建立从 typed delta 到 dirty set、局部阶段产物和不可变 frame generation 的单一提交管线。

## 2. 审查范围和方法

本次静态审查覆盖：

| 域 | 规模 | 重点 |
| --- | ---: | --- |
| `zircon_runtime/src/ui` | 804 个 Rust 文件 | surface、layout、hit、render、text、template、component |
| `zircon_runtime_interface/src/ui` | 224 个 Rust 文件 | 公共树、布局、render command、generation 契约 |
| `zircon_editor/src/ui` | 4357 个 Rust 文件 | retained host、Workbench、host contract、asset editor |
| `zr_rhi_wgpu` UI 路径 | 约 15 个 Rust 文件 | GPU image cache、atlas、surface present |
| `.zui` 模板 | 316 个文件 | 编辑器模板规模和投影面 |

采用了以下证据：

- 静态追踪 UIAutomation 鼠标/按钮/窗口缩放、资源 watcher 到 GPU 提交的完整调用链。
- 对 dirty domain、generation、缓存 key、全局扫描、重复布局和资源失效逐项审计。
- 使用 profiling release 编辑器执行窗口缩放、点击、空闲 hover、asset refresh 场景。
- 将实现与 Unreal Slate、Slint、Bevy UI/Taffy 和 Godot Control 的 retained/invalidation 算法逐项对照。
- 检查已有计划和尚未关闭的 failure records，避免将局部单测通过误判为产品链路完成。

重评基线和当前性能产物均位于 E 盘：

- 初始重评：`E:\zircon-profiles\runtime09-svg-m7\runtime09-postfix-resize-20260809-040843`
- 初始基线：`E:\zircon-profiles\runtime09-svg-m7\runtime09-clean-resize-20260809-033706`
- 当前 resize：`E:\zircon-profiles\runtime09-svg-m7\runtime09-resize-direct-swapchain-uia-20260809-1920`
- 当前 profiling EXE：`E:\Git\ZirconEngine\target\profiling\zircon_editor.exe`

禁止后续性能构建、trace、临时输出写入 C 盘。

## 3. 重评时的基线结果

### 3.1 窗口交互

预热后的 8 次 UIAutomation Resize 调用耗时为：

```text
24.596, 15.021, 37.180, 37.229, 42.684, 70.567, 1315.495, 1457.272 ms
```

当前 80 ms resize debounce 只能推迟一部分工作。连续操作跨过 debounce 边界后，昂贵提交仍在交互期间同步发生，因此最后两次出现超过 1.3 秒的阻塞。debounce 只能作为临时 admission control，不能作为架构修复。

### 3.2 产品热区

| 热区 | 当前结果 | 说明 |
| --- | ---: | --- |
| `recompute_if_dirty` p95 | 1352.29 ms | 宽脏标记进入全量重算 |
| `recompute_workbench_window_bridge` p95 | 931.52 ms | 模板桥是主要热点 |
| `recompute_apply_presentation` | 11 次 / 4867.59 ms | 平均 442.51 ms，p95 1684.38 ms |
| `apply_build_host_scene_data` | 11 次 / 2544.29 ms | 平均 231.30 ms，p95 1332.83 ms |
| `asset_refresh_runtime_project` | 10 次 | 平均 169.28 ms，p95 219.23 ms |
| 空闲 hover tick p95 | 0.62 ms | 当前 pointer read path 基本健康 |
| 点击 tick p95 | 215 ms | 点击后宽失效仍形成长帧 |

asset refresh 的 10 个记录帧全部超预算，tick p95 为 1814.12 ms。事件 drain 的时间预算没有降低总成本，反而把一次积压拆成了 10 次昂贵提交。

### 3.3 SVG、CPU 图片和 GPU 缓存

| 指标 | 基线 | 当前 | 判断 |
| --- | ---: | ---: | --- |
| `visual_assets_load_pixels` | 344 | 46 | 像素复用已明显生效，调用约下降 86.6% |
| raster calls | 344 | 46 | 重复 raster 明显下降 |
| SVG parse calls | 28 | 29 | 主要是唯一源解析，不是逐帧全量重复解析 |
| `template_node_image_pixels` | 210 / 678 ms | 374 / 1006 ms | 上游节点访问/绘制次数反而增加 |

GPU 侧 `UiImageCache` 已使用 `(resource_key, generation)` 保留纹理并仅在 generation 变化时 `queue.write_texture`。当前问题是：

- 编辑器资源事件会清除过宽范围的 CPU/atlas 状态；
- presentation 和 render command 反复重建，导致 image preparation 被重新访问；
- SVG cache lookup 仍包含 canonicalize 和 metadata 文件系统查询；
- CPU RGBA 同时保存在编辑器 cache 和 GPU resource 侧，存在驻留重复；
- atlas 接入较晚，不能抵消上游全量投影。

结论：需要稳定资源 identity、targeted generation 和 atlas residency；不能把 GPU cache 当作修复 UI 状态流的替代品。

## 4. 当前算法审计

### 4.1 输入和帧事务

`EditorUiDeltaQueue` 已实现属性路径 latest-wins 合并，并为 press/release 等离散事件保留 barrier 顺序。这一算法方向正确，但其应用目标是另一套 runtime reflection/control service。

编辑器 tick 调用 `drain_pending_view_refreshes()` 后丢弃返回 report。retained Workbench 没有消费 changed control/property 集合，最终只能用 `HostInvalidationMask` 表示“presentation/layout/paint 是否脏”。这会在入口处丢失局部性。

目标算法：一个 frame transaction 同时容纳连续值合并、离散事件 barrier、changed node/pane/window/resource 集合和 domain generation。只有 transaction commit 可以推进 generation。

### 4.2 UI 权威状态过多

当前相关状态包括：

1. 编辑器 authoring/domain model；
2. runtime reflection/control tree；
3. `WorkbenchViewModel` snapshot；
4. template surface/tree；
5. `HostWindowPresentationData`；
6. pointer bridge surfaces/hit views；
7. extracted render command stream；
8. CPU pixels、icon atlas、GPU image resources。

这些对象不是单向、局部派生的稳定阶段缓存，而会在多条路径中被整份重建或相互投影。任何一级 generation 变化都会扩大后续工作集。

目标角色划分：

- authoring/domain model 只保存编辑器业务状态；
- 一个持久 `EditorUiScene`/`UiSurface` 成为 presentation 的唯一可变权威；
- reflection/accessibility/automation 是同一 transaction 的只读派生索引，不拥有另一棵可变 UI 树；
- `HostWindowPresentationData` 收敛为不可变 frame view 或迁移期兼容 adapter，不再是第二 presentation authority；
- pointer routing 只读最近提交的 hit generation；
- command stream 和 resource residency 是按 generation 缓存的派生产物。

### 4.3 布局

runtime 已有 propagated roots 和 subtree incremental layout，但 Workbench 模板桥仍可能在一次调用内重复 `template_surface.recompute_layout`。此外，responsive、drawer、toolbar 和 popup anchor 的状态投影分散在布局前后，容易因中间结果依赖而追加第二次 layout。

目标算法：

1. 在 transaction apply 阶段一次性写入 toolbar/menu/drawer/responsive/mount 状态；
2. 计算 layout dirty roots，按 ancestor collapse 去重；
3. 每个 surface 每帧最多执行一次 layout pass；
4. popup/overlay anchor 作为 layout 输出依赖，在同一 pass 的后序阶段计算；
5. 仅在根尺寸、全局字体/主题度量、拓扑或索引不一致时走显式 full fallback；
6. full fallback 必须记录 reason 和 visited count。

复杂度目标：

- 稳定帧：`O(1)`；
- 单节点 paint/property：`O(k + affected damage cells)`；
- 局部 layout：`O(affected ancestors + affected subtrees)`；
- 根 resize：允许 `O(N)`，但整个 surface 只能有一次 measure/arrange，不允许附带模型和 presentation 多次 `O(N)` 重建。

### 4.4 命中测试

runtime 已有 cell grid 和反向索引，局部 frame 变化可更新受影响 cell。该方向正确，但前提是 changed node 集合必须从产品事件完整传到 hit stage，且 grid 索引与 topology generation 一致。

目标算法：按 changed node 的旧/新 cell membership 做 remove/insert；未改变 membership 时只更新几何和必要排序键。新增/删除、clip 变化、pointer policy 变化必须是独立 typed reason。索引失配时保守 full rebuild 并计数，不能静默继续。

### 4.5 绘制命令和 damage

当前 render cache 的局部 patch 机制存在，但产品 presentation 经常重新创建上游场景数据，使稳定 command identity 难以持续命中。

目标算法：command identity 固定为 `(surface_id, node_id, role)`；节点的 command range 和资源引用持久化。paint-only 只重建对应 ranges；geometry-only 仅平移明确声明可平移的命令，文本和内部绝对几何必须重新 extract。damage 使用旧/新 bounds 并集，按有限脏矩形或 tile 集合提交。

### 4.6 图片和 SVG

目标资源链：

```text
canonical source id
  -> parsed SVG/document cache (content generation)
  -> raster/decoded pixel cache (size/scale/theme generation)
  -> atlas allocation or standalone texture (GPU generation)
  -> stable command resource handle
```

每一级只由其直接依赖推进 generation：布局位置变化不能使 SVG/raster/GPU generation 变化；主题变化只影响声明依赖主题的矢量资源；单个文件变化只能失效该 resource id；atlas page 更新只上传变化区域。

资源 watcher 必须先合并 backlog，再在一个 frame transaction 中提交。连续事件按 canonical resource id latest-wins，rename/delete 保留有序 barrier。队列积压时允许延迟 commit，但必须有最大等待时间，不能每个部分批次做一次全量 project refresh。

## 5. 参考引擎对照

### 5.1 Unreal Slate：主要架构基准

Slate 的 `FSlateInvalidationRoot` 保存 persistent fast widget list、typed invalidation reason、preupdate/prepass/postupdate heaps、final update list、cached hit-test grid 和 cached draw elements。`InvalidateWidget` 将 reason 合并到稳定 widget proxy，只把 proxy 推入对应 heap；fast paint 消费 final update list，而不是重新遍历整棵树。

`EInvalidateWidgetReason` 明确区分 Layout、Paint、Volatility、ChildOrder、RenderTransform、Visibility、AttributeRegistration 和 Prepass。Layout 被视为昂贵操作，paint-only 不允许升级成 layout。

Slate hit grid 维护稳定 WidgetMap 和 cell membership；边界变化只从旧 cell 移除并加入新 cell，未跨 cell 的节点只更新排序/用户信息。

Slate RHI resource manager 为 static、dynamic 和 vector resources 使用稳定 map；小图进入 atlas，vector graphics 有独立 cache，atlas 只在 dirty 时更新。

Zircon 应复制的是这些约束，不是 Unreal 的具体类层次：单一 retained authority、typed invalidation、dirty heaps/sets、cached stage output、稳定资源句柄和显式 slow-path fallback。

### 5.2 Slint

Slint partial renderer 使用 property tracker 记录渲染属性依赖，保存旧/新 bounds，将两者并集加入 dirty region，并过滤不相交 item。它证明 damage tracking 必须与属性依赖和缓存几何绑定，而不是仅靠一个全局 paint bit。

### 5.3 Bevy UI / Taffy

Bevy `UiSurface` 持有 entity 到 Taffy node 的稳定映射，已有节点通过 update style/context/children 修改，而不是每帧重建 layout tree。ECS change detection 提供 changed component 集合。

### 5.4 Godot Control

Godot setter 先做 same-value early return。minimum-size cache 仅沿父链向上失效，并在已失效处停止；实际更新延迟合并，仅在尺寸真的变化时通知和重新布局。

### 5.5 共同原则

所有参考实现都满足以下性质：

- stable identity；
- exact dirty reason；
- changed set/queue，而非每帧全树扫描；
- stage-local cache；
- old/new geometry damage；
- conservative fallback 有明确条件；
- resource identity 与 widget/layout identity 解耦。

## 6. 目标架构

```text
OS / editor / asset events
          |
          v
EditorUiFrameTransaction
  - latest-wins property deltas
  - discrete barriers
  - changed node/pane/window/resource sets
  - typed dirty reasons
          |
          v
Persistent EditorUiScene / UiSurface authority
          |
          +--> local style/text/layout patch
          +--> local hit-grid patch
          +--> local render-command + damage patch
          +--> targeted resource generation update
          |
          v
Arc<UiFrameGeneration>
  - structure/layout/hit/paint/resource cursors
  - immutable arrays and stable ranges
          |
          +--> renderer
          +--> pointer routing
          +--> native windows
          +--> accessibility/automation/reflection views
```

`UiFrameGeneration` 只在 transaction commit 时发布。消费者通过各自 cursor 判断是否需要工作，不再通过 materialize 全量 DTO 判断差异。

### 6.1 Full fallback 矩阵

| 原因 | 允许的 full stage | 不应连带的 stage |
| --- | --- | --- |
| root size 改变 | 该 surface layout，必要的 hit/command geometry | model、SVG parse、resource upload |
| topology/child order | 受影响 surface structure/layout/hit/order | 无关窗口和资源 |
| global font metrics | 依赖字体的 text/layout | 图片资源、无关 surface |
| global style/theme metrics | 依赖该 token 的 nodes；索引不可用时 surface fallback | authoring model、asset catalog |
| clip/order index mismatch | hit/command fallback | model、resource decode |
| resource watcher lag/overflow | resource registry reconciliation | 不自动重建全部 UI model |
| serialization/cache index 缺失 | 对应 cache rebuild | 其他 cache |

每次 fallback 必须记录 `reason、surface、visited_nodes、rebuilt_commands、uploaded_bytes、duration`。没有 reason 的 full rebuild 视为缺陷。

## 7. 分阶段实施计划

### M0：建立真实性基线和架构护栏

- 固化本报告和 E 盘 profile 基线。
- 为 full model、template layout、presentation、hit、command、resource upload 建立同帧计数。
- 增加 fallback reason、visited work 和 generation cursor 观测。
- 禁止用只统计外层 changed IDs 的计数冒充复杂度证据。

完成门：一次点击、一次 resize 和一次资源变化能够说明每个阶段为什么运行、实际访问多少对象。

### M1：统一帧事务和 invalidation root

- 将 `EditorUiDeltaQueue` drain report 接入 retained editor UI，不再丢弃。
- `HostInvalidationRoot` 增加 typed changed scope：node、pane、window、resource 和 topology。
- 同一帧 latest-wins 合并连续状态，离散输入保持 barrier。
- generation 只在 transaction commit 推进。

完成门：按钮 hover/pressed/checked、局部文字和 paint 属性不再触发 full Workbench model/presentation。

### M2：资源事件聚合和定向失效

- 跨帧积累 bounded watcher batch，队列 drain 或最大延迟到达时只提交一次。
- 按 canonical resource id 合并事件并保留 rename/delete 顺序。
- 删除全局 visual cache clear；只推进变化资源的 content generation。
- SVG parse、raster、atlas、GPU resource 分级缓存和统计。

完成门：1 万资源事件不会形成 1 万次 refresh；重复显示同一 SVG 不重复解析、raster 或上传；单资源变化不清除其他资源。

### M3：收敛 Workbench presentation authority

- 将 toolbar/menu/drawer/responsive/mount mutations 合并到一次 surface transaction。
- 每个 surface 每帧最多一次 layout。
- 逐步停止完整 `WorkbenchViewModel -> HostWindowPresentationData` 重建；改为稳定节点和字段级 patch。
- reflection/accessibility/automation 从已提交 generation 派生。

完成门：根 resize 允许一次 `O(N)` layout，但 `model_build、chrome_snapshot、presentation_materialize` 为零；模板布局每 surface 每帧不超过一次。

### M4：布局与命中的真实局部化

- dirty root 使用稳定索引和 ancestor collapse，不扫描全树发现 dirty。
- responsive cache 使用 topology generation 校验，不能只比较 cardinality。
- arranged、slot、hit reverse indices 与 topology generation 同步。
- 增加跨 cell、zero-area、clip、detach/attach、popup、scroll、serialization regressions。

完成门：10k nodes 单节点变化的实际访问量与 changed subtree 成线性关系；测试不得 ignored。

### M5：绘制命令、damage 和 GPU residency

- 稳定 command identity/range；text 和自定义内部几何只走正确 extract path。
- old/new bounds union damage；限制 dirty rect 数并显式合并。
- icon atlas 和 GPU image cache 使用稳定 resource generation；atlas 只上传变化区域。
- 移除 CPU RGBA 的不必要重复驻留。

完成门：paint-only 不运行 layout/hit；位置-only 不 parse/raster/upload 图片；单 icon 更新只上传对应 atlas region。

### M6：不可变 frame generation 和多消费者 cursor

- 发布 `Arc<UiFrameGeneration>`，结构数组和命令 buffer 使用共享稳定段。
- renderer、pointer、native windows、automation 分别维护 cursor。
- 删除读侧 `materialize()` 整份 presentation 的产品热路径。
- 大列表和资源浏览器接入 virtualization。

完成门：稳定帧无整份 DTO clone；慢消费者不阻塞 authoring transaction；内存随稳定工作集达到平台期。

### M7：产品验证和旧路径删除

- 用真实编辑器验证按钮、拖拽、菜单、dock/drawer、窗口缩放、DPI、资源导入、SVG、多个 native window。
- 删除旧的 duplicate authority、兼容性全量投影和无 owner 的 fallback。
- 更新原计划完成状态，关闭或迁移相关 failure records。

完成门：所有验收场景、复杂度门、CPU/GPU/RSS 门通过，且旧路径不可达。

## 8. 性能和正确性验收门

### 8.1 压力矩阵

| 场景 | 规模 |
| --- | --- |
| UI tree | 1 / 100 / 1k / 10k nodes |
| button/pointer storm | 1000 连续输入事件 |
| resize | 200 次，8-16 ms 间隔 |
| asset backlog | 10k watcher events |
| icon/SVG | 1k resources，重复和唯一源混合 |
| soak | 10 分钟持续交互和资源变更 |

### 8.2 目标门

- input callback p95 `<= 2 ms`；callback 不做同步 rebuild。
- input-to-damage p95 `<= 8 ms`。
- damage-to-submit p95 `<= 16.7 ms`，60 Hz 目标下不允许稳定长帧。
- 预热后 resize 不允许 `> 100 ms` spike；同一 redraw admission 最多一个 frame commit。
- hover/paint-only 场景 full model、full layout、full presentation、SVG parse 和 GPU upload 必须为 0。
- 10k nodes 单节点测试报告 actual nodes/slots/cells/commands visits，而非只报告 changed ID 数。
- 重复 SVG 在 content generation 不变时：parse 0、raster 0、upload 0。
- RSS/GPU resident bytes 在固定工作集 soak 中达到平台期；缓存 eviction 后可回收。
- 所有 fallback 有原因，未知 full fallback 为 0。

参考引擎用于验证算法性质和复杂度，不在没有同机器、同 UI、同构建配置时声称绝对耗时优于 Unreal/Slint/Bevy/Godot。

## 9. 当前临时改动的定位

工作树中已有 resource event invalidation narrowing、visual cache invalidation narrowing、resize debounce/cached presentation、UiPerf resize scenario、icon atlas 和共享 pixel payload 等改动。这些改动可保留为候选优化，但必须按本报告重新验证：

- debounce 不是 resize 算法的完成条件；
- cache hit 不能替代稳定 presentation/resource identity；
- atlas 不能掩盖重复模板投影；
- outer changed count 不能证明内部没有 `O(N)` 扫描；
- 局部 runtime tests 不能替代编辑器产品链路性能门。

## 10. 第一实施切片

报告落地后先实施 M1 的最小闭环，而不是继续调 debounce：

1. 保留并消费 `drain_pending_view_refreshes()` 的 report；
2. 将 changed control/property scope 映射到 retained host transaction；
3. 为点击/paint-only 路径增加“禁止 full model/presentation”的回归和计数；
4. 同帧合并后只提交一次 generation；
5. 用真实按钮点击和窗口缩放 profile 验证局部性，再进入资源 backlog 和 Workbench authority 收敛。

任何发现必须回写本报告；阶段只有在正确性、复杂度、CPU、GPU、内存和产品交互门同时通过后才能标记完成。

## 11. 已落地的算法和实现记录

### 11.1 按钮和窄状态变化

- Workbench 状态投影合并为一次提交；移除重复 host projection。
- 模板节点、row topology 和 presentation 字段使用稳定 identity/signature，未变化段共享复用。
- 按钮 hover/pressed/checked 进入局部 chrome command range patch，不再扩张成 full model、presentation 和 command rebuild。
- 真实点击 profile 中 slow path、presentation、Workbench model 和 full command 均为 0；GPU image upload 为 0。

### 11.2 原生窗口缩放事务

- native resize 开始后只更新物理 surface size，并设置 `native_resize_reflow_pending`。
- resize 事务内复用上一代已提交 presentation，不同步构建新的 Workbench model；debounce 只负责决定最终正式 reflow 的提交时机。
- `GpuChromePresenter` 首帧建立一次不可变 command/draw-list snapshot，后续 23 帧只 retarget 当前交换链尺寸。
- `UiSurfaceDrawList` 分离 `surface_size` 和 `projection_size`。generation 的批处理、顶点和文本坐标使用稳定 projection；scissor 和 swapchain 使用当前 surface。
- WGPU compiled batch plan、draw buffer 和 text cache 使用 `(generation, projection_size)`，target-only resize 不使缓存失效。
- resize 临时帧显式绕过 retained surface cache，直接画到 swapchain；cache 只失效，不随每个尺寸重建。最终正式 reflow 懒重建并复制一次 retained cache。

在同一 24 次 UIAutomation 协议下，当前计数为：command snapshot build/reuse `1/23`，batch plan build/hit `1/24`，vertex upload `1`，text build/hit `1/24`，overlap planner `1`，retained copy `1`。GPU time p95 为 `186 us`、最大 `1025 us`，说明 13.76 ms 的窗口 API 平均耗时不是通过把大量 GPU 工作隐藏到后台得到的。

### 11.3 SVG、图片和 watcher

- SVG source/document、raster pixels、image resource 和 GPU texture 使用分层 generation，不再由布局位置变化推进资源 generation。
- visual asset 加载从 344 次降到 46 次；已预热 SVG 在按钮/resize 中没有重复 GPU upload。
- 发现并修复 atomic write 的自激 watcher 环：`.zr-staging-` 和 `.zr-backup-` 临时兄弟文件曾被 scanner/watcher 当成新资源，再触发下一轮写入和 refresh。scanner 与 watcher 现在统一排除事务路径。
- GPU `UiImageCache` 仍按 `(resource_key, generation)` 保留 texture；profile 中两张图首次写入 2 次、131,072 B，之后 24 帧命中 preparation cache。

### 11.4 runtime 局部阶段

- arranged tree、hit entry/cell 和 render command range 已建立反向索引，局部 geometry patch 不再按 changed ID 重复线性查整棵 Vec。
- geometry fast path 只接受 owner frame/clip 完全匹配且没有 absolute text layout 的命令；文本、内部偏移、自定义 clip 自动回退正式 extract。
- zero-area pointer 变为正面积、mixed layout/input、跨 cell 排序和 non-pointer 变化均有保守 fallback/回归路径。

## 12. 未关闭问题和边界

本轮不能标记 `M4-M7 complete`，以下问题仍是后续阻断项：

1. `UiSurface.last_layout_root_size` 为 serde skip。反序列化 clean surface 后第一次以不同 root size rebuild 可能把新尺寸写入后直接返回，发布旧 arranged/hit/render 几何。
2. UI tree 仍公开可变，直接写 `node.dirty` 可以绕过 tracked dirty IDs；`dirty_summary`、incremental root selection、`clear_dirty_flags` 和部分 responsive 路径仍存在全树扫描，10k 单节点复杂度门尚未关闭。
3. responsive MUI cache 仍有 topology 同 cardinality 的 detach/attach identity 风险；模板节点变化还可能触发全局 responsive resolution。
4. layout engine selection 真正替换时，`recompute_counts()` 会全扫所有 selection 并重建 fallback map。
5. 最终 geometry 的 138 个 hit samples 中有 1 个既存失败：`activity_rail.right.ActivityRailButton0` 的中心点在 client 右边界外。相同失败存在于两份优化前 profile，不是本轮回归，但属于真实 correctness debt。
6. 当前 resize 仍需要每帧 rasterize 已缓存 draw buffers 到 swapchain；这是 live resize 的必要显示工作。`painted_pixels` 仍按 full surface 记录，但 CPU command/batch/text/resource 工作已复用，GPU p95 低于 0.2 ms。
7. `viewport_image` profile 仍报告 region request full-frame repaint 和 presentation/layout 脏化，startup 也有 `gpu_presenter_recorded_no_draw_calls` 诊断；它们不属于按钮/resize/SVG 修复完成证据。
8. `zr_rhi_wgpu` 的 test target 当前被工作树中既存测试编译错误阻断，包括私有 image resource import、缺失测试 helper 和 vertex equality；WGPU library 与 editor 产品编译通过，但新增 WGPU 单测尚未形成可执行 gate。
9. editor 聚焦测试 `gpu_presenter_builds_one_command_snapshot_per_native_resize_transaction` 在 15 分钟内未完成 test binary 链接并超时，未取得执行结果。该行为说明当前 editor test target 过重，需要拆出更小的 presenter contract gate。

无效 profile 不作为证据：`runtime09-resize-command-cache-20260809-180315` 使用了 `NOSIZE`；`runtime09-resize-command-cache-valid-20260809-180455` 被 OS redraw 合并；`runtime09-resize-command-cache-uia-20260809-180600` 在 maximized 状态执行；`runtime09-resize-direct-swapchain-uia-20260809-1915` 因 automation harness 异常退出。

## 13. 后续实施顺序

1. 修复右侧 activity rail 边界和命中一致性，补 resize 后真实 click regression。
2. 用 topology generation 闭合 dirty/responsive/slot cache，禁止公开 mutation 绕过 dirty index；修复反序列化后首次 root resize。
3. 将 responsive resolution 和 layout selection report 改为受影响 container 增量更新，计数覆盖内部 nodes/slots/cells 实际访问量。
4. 收敛 resize/full-command 诊断语义，区分“本帧 full-surface present”和“重新构建 command stream”。
5. 修复 `zr_rhi_wgpu` test target 后执行 projection、retained bypass、text、relative command、image residency 的 GPU 单测。
6. 完成 200 次 resize、1000 次 button/pointer、10k resource backlog 和 10 分钟 soak；只有 CPU、GPU、RSS、fallback 和 correctness gates 同时通过后，才关闭 M4-M7。

## 14. 本轮验证记录

- `cargo test -p zr_rhi --profile profiling retargeted_surface_preserves_the_generation_projection_extent`：通过，1/1。
- `cargo test -p zircon_editor --profile profiling gpu_presenter_builds_one_command_snapshot_per_native_resize_transaction -- --exact`：15 分钟链接超时，未执行测试，不计为通过。
- `cargo check -p zircon_editor --profile profiling --no-default-features`：通过。
- `cargo build -p zircon_app --bin zircon_editor --profile profiling --no-default-features --features target-editor-host,profiling`：通过，19m54s。
- profiling EXE：`E:\Git\ZirconEngine\target\profiling\zircon_editor.exe`，91,507,712 B。
- runtime DLL：`E:\Git\ZirconEngine\target\profiling\zircon_runtime.dll`，44,408,832 B；与 `deps` 产物 SHA-256 一致。
- 24 次 UIAutomation resize：进程退出码 0，平均 13.7605 ms，最大 18.4788 ms。
- 最终 profile：`E:\zircon-profiles\runtime09-svg-m7\runtime09-resize-direct-swapchain-uia-20260809-1920`。
