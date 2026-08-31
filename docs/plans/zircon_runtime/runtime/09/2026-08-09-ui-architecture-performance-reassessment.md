# Zircon UI 架构与性能重评及增量刷新实施计划

状态：`ACTIVE / PRODUCT HOT PATH OPTIMIZED / FULL-SYSTEM GATES STILL OPEN`

日期：2026-08-14（持续复核）

范围：`zircon_editor`、`zircon_runtime`、`zircon_runtime_interface`、`zr_rhi_wgpu` 的编辑器 UI 输入、状态投影、布局、命中、绘制、图片资源和 GPU 提交路径。

本报告重新评估 `2026-08-07-runtime-ui-incremental-refresh.md` 的完成状态，并记录产品热路径优化。旧 profiling EXE 已证明一部分按钮和原生窗口缩放可以命中局部 patch、事务门控和跨帧 GPU 产物复用；当前源码还增加了 typed host transaction、Runtime changed-node transaction、目标化视觉资源失效、内容寻址 GPU identity 和 frame-authoritative popup hit grid，但尚未由同一份当前源码构建出的新 Editor EXE 验证。历史二进制、静态源码和当前产品动态三类证据不得混用，这也不等于整个 UI 子系统已经完成增量化，未关闭项见第 12 节。

## 0. 执行摘要

截至 2026-08-12，本轮已经贯通编辑器产品中的基础按钮、窗口缩放、资源 watcher、SVG CPU cache 和 GPU present 热路径，不再只依赖 runtime 局部单测推断性能。长间隔点击复测发现并关闭了 Toast 剩余时长造成的约 100-148 ms 周期性宿主投影重建；修复后空闲段 presentation/model rebuild 均为 0。稳定跨 Drawer 切换仍不可接受：最新旧架构样本的输入 callback p95 低于 6 ms，但事件后的 `recompute_if_dirty`/presentation 仍为数百毫秒并伴随秒级离群值。进一步逐层复核确认，原所谓 dock patch 位于完整 Workbench model、全部 pane payload 和完整 `ShellPresentation` 构造之后；即使 patch 命中，也不能消除前置全量成本。本轮因此将 `SHELL_CONTENT` 提升为正式重算目标，在收集全量 pane payload 之前尝试单 dock/单 pane 原子提交。

| 历史产品场景（旧 profiling EXE） | 优化前 | 已落盘 profile | 结果 |
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
- Toast 修复后混合 Drawer 点击：`E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch\20260811-031434-click-dock-patch-spaced`
- Drawer 区域修复前稳定切换：`E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch\20260811-032345-click-dock-patch-spaced`
- Drawer geometry gate 后仍回退：`E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch\20260811-154349-click-dock-patch-spaced`
- 最新旧架构秒级离群样本：`E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch\20260811-201002-click-dock-patch-spaced`
- profiling EXE：`E:\Git\ZirconEngine\target\profiling\zircon_editor.exe`
- runtime DLL SHA-256：`1AE9896AAD3000BA1C5BA7117DDB444F15290D1AE8D7B8DCBBBFA61ED4788996`

这些历史样本的有效结论是：当相应局部路径命中时，已测按钮交互可以避免全 Workbench 重建，连续 native resize 可以避免在每个 `WM_SIZE` 上重跑布局/模型/命令提取、批处理、顶点上传、文本构建或 retained texture copy。它们不能证明当前工作树中的新早期分支已经可达或性能达标。最终稳定尺寸仍执行一次正式 reflow 和 retained cache 重建，这是正确性要求，不应被消除。

## 1. 结论

当前卡顿不是单一布局函数、SVG 解码器或 GPU 上传函数造成的，而是 UI 数据流中存在多套可变权威状态，窄事件最终被扩张为整套编辑器 UI 的重复投影和重建。

最主要的因果链为：

```text
按钮/窗口/资源事件
  -> HostInvalidationTransaction(All / View / ShellContent) + legacy dirty flags
  -> 单一受支持 scope 命中 scoped target；多 scope、混合 reason 或 legacy flag 扩大到 full
  -> full 时重建 Shell snapshot / WorkbenchViewModel / geometry / pane payloads
  -> Workbench template bridge 布局和状态投影
  -> HostWindowPresentationData 提交
  -> Runtime per-node layout/hit/render patch 或显式 full fallback
  -> damage-scissored GPU present 与 generation-keyed resource/text cache
```

已确认的核心问题：

1. `HostInvalidationRoot` 已有可提交的 `HostInvalidationTransaction`，以 `BTreeMap<HostInvalidationScope, HostInvalidationMask>` 合并 `All`、`View` 和 `ShellContent` scope；当前缺口是 host 层还没有 node/resource/window changed set，多 `ShellContent` scope 或混合 legacy flag 会扩大到 full。
2. `recompute_if_dirty` 已能选择 `ShellContent`、`WorkbenchProjection`、`ViewPresentation`、`WindowMetrics` 或 `Full` target；只有受支持的窄事务命中 fast path。Full fallback 仍同步构造 shell snapshot、model、geometry、全部 pane payload、presentation 和 pointer/native presenter 派生状态，是当前最需由产品 profile 量化的扩大点。
3. Workbench 模板桥在一次调用中可能先执行 mount layout，再投影 toolbar/menu/drawer/responsive 状态，最后再次布局；同一 surface 存在重复布局和重复状态同步。
4. 反射控制树已经有 typed delta queue，但 tick 中的 view refresh report 被丢弃；它没有成为 retained editor UI 的脏传播来源。
5. `HostPresentationGeneration` 优化了读侧共享，但写侧仍物化和替换整份 `HostWindowPresentationData`，所以它不是增量 presentation authority。
6. 资源事件按时间预算分批 drain，却对每个不完整批次同步执行一次 runtime project refresh 和 presentation commit。启动积压因此被放大为多帧长阻塞。
7. CPU SVG tree/raster cache、immutable icon-atlas page、presenter-local image cache 和 device-shared WGPU texture registry 都已经存在；当前风险是缓存失效/重建频率、稳定 generation 可达率和冷 miss 同步成本，而不是“完全没有 GPU 图像缓存”。
8. Runtime 已有 per-node invalidation transaction、局部 arranged/hit/render patch、slot/node 反向索引和 projected popup hit authority；Editor Workbench 仍由另一套 host transaction/presentation 流程主导，两层粒度不一致时会丢失 Runtime 已有局部性。
9. 鼠标事件现在应只查询已发布 `UiSurfaceFrame` 的 authoritative projected hit grid；popup 的 arranged/render 扫描已经移到 rebuild/publication 阶段。输入热路不得再为每个事件扫描 arranged nodes、render commands 或 popup template nodes。
10. 根窗口尺寸变化通常需要 `O(N)` 更新依赖根尺寸的节点几何，这是不可消除的；当前错误在于一次几何变化触发多次 `O(N)` 布局及额外的模型、模板、presentation、命中和绘制全量工作。

因此，正确方向不是继续为每一个全量重建结果叠加缓存，而是收敛 UI 权威状态，建立从 typed delta 到 dirty set、局部阶段产物和不可变 frame generation 的单一提交管线。

## 2. 审查范围和方法

本次静态审查覆盖：

| 域 | 规模 | 重点 |
| --- | ---: | --- |
| `zircon_runtime/src/ui` | 821 个 Rust 文件 / 183,614 行 | surface、layout、hit、render、text、template、component |
| `zircon_runtime_interface/src/ui` | 232 个 Rust 文件 / 23,160 行 | 公共树、布局、render command、generation 契约 |
| `zircon_editor/src/ui` | 4,437 个 Rust 文件 / 338,700 行 | retained host、Workbench、host contract、asset editor |
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

GPU 侧 `WgpuUiImageCache` 已使用 `(resource_key, generation)` 保留窗口局部 bind group，并仅在 generation 变化时准备新资源。2026-08-13 的当前源码进一步将静态纹理所有权提升到 `RenderBackend` 持有的 `WgpuUiSharedImageRegistry`：同一 WGPU device/context 下的所有原生 presenter 共享纹理，窗口只保留自己的 bind group 和绘制状态。该接线尚未由新 EXE 验证，因此已有证据仍只证明单窗体跨帧驻留。当前问题和边界是：

- 编辑器资源事件原先会在任一视觉资源变化时清除全部 CPU raster/atlas 状态；当前实现已增加 `source path -> logical raster key` 反向索引，普通 SVG/PNG/JPEG 等变化只删除依赖该路径的 raster variants，资源事件丢失、无 locator 的 Texture 事件以及 sprite-atlas manifest/texture 变化仍保守全量失效；
- presentation 和 render command 反复重建，导致 image preparation 被重新访问；
- 稳定 raster 命中不会进入 SVG tree lookup；raster 被失效后，SVG tree cache 现在先查询 `normalized source alias -> unique canonical path` 的纯内存索引，唯一命中不再执行 `canonicalize/stat`。冷路径、同相对路径歧义或显式失效后才读取 metadata/文件并解析；watcher 会直接删除目标路径的 tree，避免依赖同长度/时间戳变化，并且同一 canonical path 只保留最新 tree generation；
- CPU RGBA 同时保存在编辑器 cache 和 GPU resource 侧，存在驻留重复；
- 非 atlas raster 的 GPU identity 已收敛为最终 `(width, height, RGBA)` 内容哈希。无关资源变化不再改变其 identity；内容不变继续命中 GPU，内容变化自然得到新 key。sprite atlas 继续使用 atlas generation；
- GPU texture registry 已是 device-local，presenter-local cache 只持有 texture clone 和 bind group。同一 command stream 对重复 `(resource_key,generation)` 只探测一次，稳态本地命中不获取全局锁；新 presenter 在本地 bind group 建立前保留 CPU payload，present 阶段先查询共享 registry，共享命中不执行 `queue.write_texture`，LRU 在 extract/present 间淘汰时仍能用 payload 正确重建；
- device registry 与 presenter cache 都有 256 entries / 64 MiB 上限。registry 淘汰只释放全局引用，已提交窗口仍持有 texture clone，因此不会破坏在飞帧；后续冷窗口可能在全局淘汰后重新上传，这是有界缓存的预期行为，需要在 soak 中计数；
- atlas 接入较晚，不能抵消上游全量投影。

结论：CPU targeted invalidation、内容寻址 identity、device-level texture registry 和 presenter-local bind group 分层已经落地；仍需新 EXE 的双原生窗口 residency/upload/eviction 证据。GPU cache 不能替代 UI 状态流和布局算法修复。

## 4. 当前算法审计

### 4.1 输入和帧事务

`EditorUiDeltaQueue` 已实现属性路径 latest-wins 合并，并为 press/release 等离散事件保留 barrier 顺序。`HostInvalidationRoot` 也已有 transaction：同 scope 的 reasons 做 union，任意数量的纯 `View + PRESENTATION_DATA` 可以局部提交，单个 `ShellContent` scope 可以进入早期 shell-content target。

当前粒度断点在两层之间：Runtime `UiInvalidationTransaction` 已按 `UiNodeId` 保存 dirty domains 和 typed reasons，commit 输出 changed nodes 与每域 generation；Editor host transaction 仍只表达 `All/View/ShellContent`，并与 legacy boolean dirty flags 并存。多 shell scope、混合 reason 或 legacy flag 会走 full，Runtime 的 changed-node authority也尚未成为 Editor 产品提交的唯一来源。

目标算法：保留现有两套 transaction 的正确合并语义，同时让 host scope 携带或引用 Runtime changed set，并显式记录 fast-path hit、fallback reason、scope cardinality和被扩大的工作集。只有 transaction commit 可以推进 generation。

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

Runtime 已有 propagated roots、subtree incremental layout、`UiLayoutSlotIndex` 和 responsive candidate index。局部 pass 从 changed nodes 向祖先求最小覆盖 roots，再只 measure/arrange 被选子树；当前集合实现主要使用 `BTreeSet/BTreeMap`，算法成本约为 `O(k*h + V log V)`，其中 `k` 是 changed nodes、`h` 是树高、`V` 是被访问子树。根尺寸变化允许一次完整 responsive/layout `O(N)`。Workbench 模板桥若在同一 transaction 内重复 `template_surface.recompute_layout`，仍会把这条必要的一次 `O(N)` 放大。

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

Runtime 已有 cell grid、node/entry/cell 反向索引，局部 frame/input 变化可更新受影响 cell。control-anchored popup 现由 rebuild-owned `UiProjectedHitTestIndex` 在 frame publication 前把 popup subtree 的 frame/clip 做双轴仿射投影，并以稳定 popup stack + subtree order 覆盖底层内容；`UiSurfaceFrame` 和 instance hit 都读取同一 projected grid。增量 patch 只遍历 `changed_node_ids ∪ projected_node_ids`，base hit-grid full rebuild 或 z baseline 越界会显式重建 projected index，事件热路不做 arranged/render 扫描。

目标算法：按 changed node 的旧/新 cell membership 做 remove/insert；未改变 membership 时只更新几何和必要排序键。新增/删除、clip 变化、pointer policy 变化必须是独立 typed reason。索引失配时保守 full rebuild 并计数，不能静默继续。

### 4.5 绘制命令和 damage

当前 render cache 已保存 per-node command bucket、command range 和 geometry-patchable 集合，局部 geometry/render patch 只访问 changed nodes 与固定 ranges。Editor 产品链也已经把一个裁剪到 surface 的 damage union rectangle传给 `UiSurfaceDrawList`；WGPU 在 retained cache ready 时使用 `Load` + scissor，并剔除不相交 draw op/batch，target-only resize 可直接 `RetainedProjectionCopy`。这是真实的局部 GPU redraw，不只是统计；边界是 damage 目前只有一个 union rect，多个远离区域会被合并成大范围 overdraw。

目标算法：继续保持 command identity `(surface_id, node_id, role)`、持久 command range 和资源引用；paint-only 只重建对应 ranges，geometry-only 仅平移明确声明可平移的命令，文本和内部绝对几何必须重新 extract。先在产品 profile 中记录 `damage area / surface area` 和被 scissor 接受的 draw work；只有单 union rect 的 overdraw 被证明确为主因后，才参考 Slint 的最多 3 个 dirty rect、按最小额外面积合并策略升级为有界多矩形，而不是先增加 GPU 提交复杂度。

### 4.6 图片和 SVG

当前资源链已经按下列层级缓存；目标是用 current-source 产品证据证明暖路径真实命中，而不是再增加一套并行缓存：

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

### 5.6 精确源码证据与 Zircon 验收映射

| 参考实现 | 实现证据 | 测试证据 | 对 Zircon 的强制不变量 |
| --- | --- | --- | --- |
| Unreal Slate invalidation | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`：`PreInvalidationReason`/`PostInvalidationReason`、`HeapPushUnique`、`PaintFastPath`/`PaintSlowPath`、cached hit grid/elements | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Test/SlateInvalidationWidgetListTest.cpp` | dirty reason 必须分域；同节点同阶段唯一入队；只有 root/topology/cache-invalid 才允许 slow path；fallback reason 必须计数 |
| Unreal hit grid | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp` | Slate invalidation debug path 会执行 cached hit grid verification | changed widget 只修改旧/新 cell membership；索引代际不一致必须 full rebuild，不能静默发布陈旧 hit 结果 |
| Unreal RHI resources | `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.cpp` 与 `SlateRHIResourceManager.h`：稳定 static/dynamic/vector resource map、atlas、显式 release/update | Slate renderer/resource automation 与统计接口 | 资源 handle 必须独立于控件位置和 presentation generation；单资源变化不得推进无关图像 generation；共享 device 下应只有一份 texture residency |
| Slint partial renderer | `dev/slint/internal/core/partial_renderer.rs`：cached geometry/property tracker、old/new bounds、bounded dirty region、cache generation | `dev/slint/api/rs/slint/tests/partial_renderer.rs`：same-value 不 redraw、属性变化只提交期望区域 | damage 必须来自旧/新 bounds 并集；稳定属性写入不得 redraw；cache index 必须带 generation 防陈旧引用 |
| Bevy UI/Taffy | `dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs`：稳定 entity-to-Taffy node 映射和原位 style/context/children 更新 | 同 crate 的 UI layout tests/change-detection 调用点 | layout tree identity 跨帧稳定；普通节点变化只更新映射节点/祖先，不重建全树 |
| Godot Control | `dev/godot/scene/gui/control.cpp`、`dev/godot/scene/gui/container.cpp`：same-value early return、minimum-size 向上失效、deferred sort | Godot scene GUI tests | setter no-op 不得产生 layout/presentation effect；祖先传播在已 dirty 边界停止；同帧布局请求必须合并 |

这里参考的是可验证的算法性质，而不是照搬类层次。任何 Zircon fast path 都必须有与 full path 等价的结果测试、真实访问量计数和产品 profile；仅存在一个名为 `incremental` 的函数不构成通过证据。

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

### 11.5 后续源码复核纠正

里程碑提交后再次按当前共享工作树逐行复核，确认早期独立 review 的四项结论已经被后续实现关闭，不能继续作为现存瓶颈：

- `last_layout_root_size=None` 且已有 arranged nodes 时现在显式视为 root size changed；新增 serialization -> different root size 回归，防止发布旧 geometry。
- `UiTree.nodes` 已由私有 `BTreeMap` 的 `UiTreeNodes` 包装，所有公开 mutable entry point 都登记 pending mutation node IDs；dirty summary 和 clear 在索引初始化后只消费 tracked candidates。
- responsive slot cache 不再只比较 cardinality；缺失 slot index、越界 slot、child/parent identity mismatch 都保守回退 full responsive resolution。
- layout engine selection 的稳定 route replacement 使用 `replace_selection_at()` 原地扣除/增加 backend、support、fallback reason 和 Taffy aggregates，不再为单 route 变化调用 `recompute_counts()` 全扫 selections。

### 11.6 control geometry 与 paint primitive 解耦

- 新一轮产品几何采样确认 activity rail 的 34px 外壳本身正确，但 `ActivityRailButton*` 被投影为 `1x32`。问题不在 hit grid，而在 editor chrome projection：同一 logical control 会产生 owner surface、border/separator、state indicator 和 icon/text 等多个 render primitive。
- 第一轮修复将 `control_frame()` 从“取首个同 control command”改为合并全部有效 primitive bounds；包含该 Rust 修改的新 EXE 仍稳定报告 `1x32`，证明丢失发生得更早。`expand_activity_rail_button_nodes()` 以 `BTreeMap<row, node>` 收集 stencil 时按 paint order 后写覆盖，1px separator 在 owner surface 之后写入，28px 主几何在进入 `control_frame()` 前已经消失。
- stencil composition 现在统一保留同一行中有效面积最大的 control template，Activity Rail、menu slot 和 menu popup 不再由最后一个 paint primitive 决定逻辑几何；新增正反 render order 都选择 `28x32` owner surface 的回归。`control_frame()` 的 bounds union 继续作为下游防御。
- 34px dense rail 的 authored geometry 同步收敛为 `x=3,width=28`，icon 为 `x=8,width=18`，保留左右各 3px 的稳定命中余量。旧 44px token 的 `x=6,width=32` 已删除。
- 面积优选是当前兼容层的保守纠错，不是最终数据模型。根治目标是让 `view_template_nodes_from_surface()` 按 arranged/tree node 或显式 control geometry 一控件一行投影，render command 只提供 paint/style 数据；paint primitive 顺序不得再参与 control identity、hit frame 或 automation frame。
- 包含最新 stencil selection 的 profiling EXE 已验证四个 Activity Rail 按钮均为 `28x32`；138 个 hit samples 的 expected/actual mismatch 为 0。旧 EXE 和只包含 `control_frame()` union 的中间产物仍是 `1x32`，仅作为定位反证，不再代表当前状态。

### 11.7 Toast 稳定身份与宿主投影风暴

- 3 秒间隔点击 profile 暴露了短点击测试没有覆盖的周期性长帧：35 个 slow frames、91 次 dirty presentation，36 次重算集中在约 100-148 ms；146 次增量 host projection 共 patch 7018 个节点。
- `sync_activity_notifications` 每 tick 获取 `live_toast_snapshot()`；`activity_toast_queue_entry()` 又把 `remaining_lifetime()` 序列化成 `auto_hide_duration_ms`。即使 Toast 的 id、标题、消息和严重级别均未变化，剩余时长也会使整个 `toast_queue` 字符串数组不相等。
- 这个差异触发 template refresh、host projection 和 `PRESENTATION_DATA` invalidation；runtime 还将 `auto_hide_duration_ms` 分类为 layout/hit/render/text/input 全域 dirty，因此一个计时字段被放大成 Workbench 重建。
- 修复将 Toast queue 的业务身份与 volatile duration 字段分离：稳定字段决定是否刷新 retained projection，倒计时继续由已有 runtime timer/reducer 消费。初始 payload 仍保留时长，不改变自动隐藏语义；真正的新 id、标题、消息或 severity 仍会更新。
- 修复后 3 秒间隔混合点击 profile 中，空闲 hover 的 presentation/model rebuild 均为 0，未再出现此前每隔约 100-148 ms 的 36 次周期性重算；SVG/GPU upload 也没有在空闲段重复发生。这关闭了 Toast 风暴根因，但没有关闭 Drawer 切换自身的全量路径。
- 这是一项有边界的止血修复。最终架构仍应由通知中心提供 snapshot generation 和 next-expiry deadline，让 host 在 generation 变化或到期时才同步，而不是每 tick 构造快照后再做语义比较。

### 11.8 Activity Drawer 区域所有权与几何安全复用

- 稳定序列 `LeftBottom -> LeftTop -> LeftBottom` 的修复前 profile 仍有 7/7 over-budget tick；`tick/recompute_if_dirty` p95 为 `1884.74 ms`，`recompute_apply_presentation` p95 为 `1835.02 ms`，left dock conversion p95 为 `1816.84 ms`。输入回调本身 p95 仅 `2.39 ms`，真正的延迟发生在事件后的同步重算和呈现。
- `LeftTop` 与 `LeftBottom` 共用一个物理 Left region，但原布局模型允许两个 Drawer 同时保持 `Pinned/AutoHide`。投影和 autolayout 又只从 `[LeftTop, LeftBottom]` 选择首个展开 stack，造成状态模型、Activity Rail active 状态、实际 pane 和 geometry authority 不一致。
- `ActivateDrawerTab` 已负责将折叠目标设为 `Pinned`，回调却额外发送一次 `SetDrawerMode(Pinned)`；布局命令即使没有状态变化仍发布 `LayoutChanged + PresentationChanged`，第二条命令因此会把已经降级为 `SHELL_CONTENT` 的失效重新升级成全量路径。
- 当前实现将物理 region 排他放入 `ActivityWindowLayout`：`ActivateDrawerTab`、`FocusView` 和非折叠 `SetDrawerMode` 在提交目标前统一折叠同区域 sibling 并清除其 active selection。回调删除冗余 reopen 命令；跨 stack 切换仅在区域切换前后都保持展开时申请 shell-content 复用。
- 布局复用不是仅凭 dirty 标签生效。每次仍计算轻量 `WorkbenchShellGeometry`，只有实际 mounted frame 集合与上一代相等时才复用 component frames；extent、scale 或可见/floating geometry 任一变化都会记录 geometry fallback 并执行正式布局。presentation patch 继续要求 shell/layout/status/document 均稳定且恰好一个 dock 改变，并新增分原因 fallback counters。
- 第一轮区域排他 profile `20260811-103300-click-dock-patch-spaced` 将 `tick/recompute_if_dirty` p95 从 `1884.74 ms` 降到 `465.30 ms`，click counter p95 为 `15.67 ms`，但仍有 9 个 over-budget frame；layout cache 仅 hit 1 次、geometry fallback 3 次，presentation patch 因 shell fallback 1 次而成功数为 0，因此不通过。
- 进一步核对确认 3 次 geometry fallback 的可见 region/splitter/status/viewport frames 实际相同，差异来自 Hierarchy 与 Module Plugins descriptor 导致的 `window_min_width/height`。窗口最小尺寸是 native constraint metadata，不是 mounted frame。第二版等价门只忽略这两个字段，仍比较 center/status/region/splitter/floating/viewport 全部 frame；dock patch 同步写回新的 min-size metadata，任何可见或 floating frame 变化仍回退。
- `20260811-154349-click-dock-patch-spaced` 中 click callback p95 仅 `1.91 ms`、空闲 hover p95 `0.07 ms`，但 `tick/recompute_if_dirty` p95 `344.09 ms`、apply presentation p95 `298.60 ms`；6 次点击仍全部进入 full model/hit/command 路径，dock patch 命中为 0。
- `20260811-201002-click-dock-patch-spaced` 仍无 patch hit，click callback p95 `5.71 ms`，而 tick 出现 `2359.93 ms` p95 离群值。GPU 侧每次点击只保留 4 次、65,536 B 的唯一资源写入，没有 SVG 周期性重传；这进一步证明当前卡顿主因是 CPU 侧全量 presentation，而不是鼠标派发或 GPU image upload。
- 继续定位发现 `get_host_presentation()` 会把实时 diagnostics overlay 写入 `host_shell.debug_refresh_rate`，而每次完整 `ShellPresentation` 构造都写回 startup placeholder。旧 `same_shell` 把这个每帧变化的显示字段当作结构字段，导致从第二帧起持续以 `Shell` 原因回退。当前结构等价门已排除该 volatile 字段，并将 shell identity/status/commands/presets/theme/native 分组计数；局部提交还会保留上一代实时 diagnostics 文本。
- 本节在真实 profile、GPU upload、damage 和退出状态共同通过前不标记为完成。

### 11.9 ShellContent 早期单 dock 提交

- 旧流程顺序为：构造完整 Workbench model/geometry -> floating projection -> 同步 viewport/pointer layout -> 收集所有可见 pane payload -> 构造完整 `ShellPresentation` -> 最后尝试 dock patch。这个顺序在算法上仍是全量重建，patch 只减少最后一次赋值，不能解决数百毫秒前置工作。
- `RecomputeInvalidationTarget` 现在包含正式的 `ShellContent` 分支。只有 pending 原因属于 `SHELL_CONTENT | PRESENTATION_DATA`、legacy mirror 至多为 `PRESENTATION_DATA`、没有 layout/window/render 合并工作，并且 committed geometry cache 仍有效时，才允许进入早期路径；否则保持完整回退。
- Activity region 的目标选择和正式 pane projection 共用同一个 `side_pane_selection()`，消除 fast/full 两套 active-tab 优先级漂移。目标包含精确 dock、`ViewContentKind` 和 `instance_id`。
- pane payload 按目标类型和实例收集：UI Asset/Animation 直接请求一个实例，不再枚举所有打开 view；runtime diagnostics、module plugins、build export 只在目标类型匹配时采集；template-v2 数据按目标 pane template document id 获取单个 snapshot，不再复制全部模板源。
- 展示层只构造目标 pane 及目标 dock 的 rail/header/tab/paint models。非目标 pane、floating windows、document pane 和完整 host scene 不参与该路径；最终沿用 retained paint-model reverse index 原子替换对应子树，并只提交该 dock damage。
- 提交前仍验证 Shell 结构分组、完整 host layout、status、document pane、恰好一个 dock 变化、目标 payload 完整性和 paint-model replacement cardinality。任一失败都记录精确 fallback 并执行现有全量路径，不发布部分状态。
- callback effects 现在携带 canonical dock 和 `instance_id`；同批事件只有恰好一个 scope 且原因仅为 `SHELL_CONTENT | PRESENTATION_DATA` 时，才写入 scoped invalidation transaction。不同 scope、全局 presentation/layout/render 或其他宽失效会清除局部资格。recompute 不再通过三 dock presentation diff 反推目标。
- 早期分支现在位于 `build_recompute_shell_snapshot()` 之前。上一代成功提交的 layout/chrome/model/geometry/layout frames/descriptors 被保存为 `CommittedShellState`；窄点击只读取新的 authoritative layout，严格验证除目标物理 drawer region 的 selection/mode 外其他结构、extent、可见性和 tab identity 均未变化。更新采用两阶段 preflight/commit：先只读验证目标 region 的 chrome/tool-stack/drawer-ring 映射，再原位同步 active-tab/mode/visible/extent；不会克隆全模型，也不会在中途失败时留下半更新缓存。
- 早期成功后不再执行 full shell snapshot、full pane payload collection、full `ShellPresentation`、native floating presenter sync 或全量 pointer surface sync。目标 dock 仍沿用 retained paint-model reverse index 做原子替换并提交 bounded damage；下一阶段需要用产品 profile 证明该分支真实可达并量化 fallback reason。
- 该实现已通过相关文件 `rustfmt --check` 和 `git diff --check`。正式 Windows 编译和新 EXE profile 尚未完成：首次验证被 coordinator SQLite 短暂锁定，重试被另一个 Session 的 CPU lane reservation 拒绝；不能将静态检查记为产品性能通过。

### 11.10 视觉资源按路径失效与内容寻址

- 旧视觉像素 cache key 混入单个全局 generation；任一图片变化都会清空全部 raster variants、推进所有逻辑资源 identity，并让后续 presentation 重新访问全部 SVG/图片来源。这个算法把“一个文件变化”扩张成“所有图标重栅格和新 GPU identity”。
- `VisualAssetCache` 现在在 bounded raster entries 之外维护 `base logical key -> candidate source paths` 反向索引。加载成功、负缓存和缺失图标 fallback 共用同一 logical key，因此后来新出现的文件也能通过 watcher 路径命中并删除全部相关 size/tint variants。
- 反向索引进一步分为 `source alias -> logical base key` 与 `logical base key -> raster variant keys`，单路径事件不扫描全部 raster entries。目标复杂度为 `O(changed aliases * log R + affected variants)`；缓存 admission/eviction 仍受 4096 entries、64 MiB 上限约束。
- 普通视觉资源事件收集 current/previous locator，排序去重后执行 targeted invalidation；资源事件 gap、Texture 无 locator、sprite-atlas manifest/texture 变化继续执行全量失效。targeted/full 次数以及被删除的 raster/tree 数量都有独立 profile counter。
- SVG tree cache 同时接受路径级失效；显式 watcher 事件不再仅依赖 `mtime + len` 推断变化。存入新 tree 时同 canonical path 的旧 generation 被移除，避免编辑同一 SVG 后 Arc tree 累积。
- SVG tree cache 维护 `source alias -> canonical path` 反向索引并限制为 1024 entries；稳定唯一 alias 命中为纯内存 lookup，不再每次 `canonicalize/stat`。同相对路径映射多个 canonical path 时拒绝快命中并回到带 stamp 的保守路径；目标失效只移除受影响 path。LRU admission 允许在这个固定上限内扫描，不能出现在逐帧 paint 热路。
- Windows `Path::canonicalize()` 返回的 verbatim path（`\\?\E:\...` / `\\?\UNC\...`）现在与普通绝对路径归一为同一 alias；否则 cache 虽已存入 canonical tree，下一帧普通 locator 仍会错过 memory index 并重复 `canonicalize/stat`。新增 verbatim/regular absolute query 等价回归。
- 非 atlas raster 保留 `retained_image_resource_key(width, height, rgba)` 内容寻址结果，不再被逻辑 cache key 覆盖。CPU targeted invalidation 后，内容未变则 GPU identity 不变，内容变化则自然获得新 key；atlas 仍使用 manifest/texture generation。
- 该切片已通过相关文件 `rustfmt --check` 与 `git diff --check`，尚未完成当前共享主线上的 Windows 编译和运行时 asset-change/GPU upload profile，因此不标记为产品验收完成。

### 11.11 Device 级 GPU UI 纹理注册表

- 根因不是“完全没有 GPU cache”，而是 cache ownership 错误。旧 `WgpuUiImageCache` 由每个 `WgpuUiSurfaceRenderer` 独立创建；主窗口和 floating native windows 虽共享同一 `Device/Queue`，仍会为同一 SVG raster 分别创建 texture 并上传。
- `RenderBackend` 现在持有唯一 `Arc<WgpuUiSharedImageRegistry>`，并通过 `WgpuUiSurfaceContext` 克隆到每个 presenter。注册表按 `(content-addressed resource_key, generation)` 保存 texture；窗口局部 cache 只创建自己的 view/bind group。
- prepare 顺序为：动态 external viewport product、窗口局部 bind group、device 级静态 texture、CPU payload 首次上传。动态 viewport texture 不会误入静态 LRU，稳态图标不获取全局 registry 锁，新窗口可以复用已上传 texture。
- registry admission 使用确定性 LRU，限制为 256 entries / 64 MiB。generation 是独立 entry；registry eviction 只释放全局引用，不会使 presenter 已持有的 texture clone 或在飞帧悬空。
- 新窗口在自己的 bind group 尚未建立前仍携带 CPU payload。如果 registry 在 extract 与 present 之间被淘汰，仍能重新上传而不会缺图；共享项仍驻留时则直接复用，upload write 为 0。
- device registry resolve 还会校验当前 staged/draw-list 的宽高；presenter-local admission 失败时不会把共享项误判为已准备，而是继续 CPU payload 本地上传兜底。新增 `gpu_image_shared_resolves`、`gpu_image_shared_upload_writes`、`gpu_image_shared_upload_bytes` 三个 profiler 指标，用于区分本地命中、跨窗口 resolve 和真实 device upload。
- `UiSurfacePresenter::is_image_resource_resident()` 有意只查询 presenter-local cache 与生命周期受 provider 约束的动态 external image，不能把可随时被全局 LRU 淘汰的 device registry 暴露成无条件 bool residency；否则 extract 后淘汰会造成无 payload 缺图。跨窗口复用由 present 阶段的 registry resolve 保证，command-stream compact 则按 `(resource_key,generation)` 缓存本地驻留判定，避免一页 atlas 的多个图标重复探测。
- 新增 admission 回归约束最旧跨窗口纹理先淘汰、replacement target 不被自身淘汰。当前相关文件 `rustfmt` 和 `git diff --check` 通过；托管 Windows 构建尚未得到源码结果，见第 14 节。
- profiler 除 shared resolve/upload 次数与字节外，新增 `gpu_image_shared_resident_bytes`，直接报告 device registry 的全局驻留量。每 presenter 的 `gpu_image_cache_resident_bytes` 仍是局部 admission 预算；presenter 持有的 texture clone 会延长实际 GPU 生命周期，因此只有双窗口和 10 分钟 soak 的两个指标共同达到平台期，才能声明内存 gate 通过。

### 11.12 WindowMetrics 阶段缓存与 viewport 增量投影

- `sync_shell_size()` 现在只发布 `WINDOW_METRICS`，不再显式合并通用 `PRESENTATION_DATA`。legacy compatibility flags 也不再把 window metrics 镜像成普通 layout dirty，因此纯窗口尺寸事务可以被精确识别；同帧一旦合并业务 presentation/layout/render，仍走 full fallback。
- 纯 `WindowMetrics` 重算从 `CommittedShellState` 取得 layout/model/chrome/descriptors，跳过 `runtime.current_layout()`、descriptor clone、`chrome_snapshot()` 和完整 `WorkbenchViewModel::build_with_context()`；只按新 shell size/scale 重算 `WorkbenchShellGeometry` 和模板 layout frames。根尺寸驱动的几何布局仍允许 `O(N)`，但不再附带无关业务模型重建。
- viewport content frame 改变后，`EditorViewportEvent::Resized` 只有在 effects 精确包含 presentation + render、且没有 layout/scope/side effects 时才进入增量投影：直接更新 cached chrome viewport size 和 `StatusBarModel`，保留 render dirty 交给提交阶段消费。任何不完整或扩大的 effects 都会记录 fallback，并重建正式 chrome/model。
- 当前实现包含 source guards 和 focused unit tests，但尚未由当前工作树构建出的 EXE 验证。验收时必须同时观察 `ui.viewport_resize.incremental_projection_count`、fallback count、chrome/model build count、布局 pass 数与真实 UIAutomation latency。

### 11.13 Bottom drawer token authority 收敛

- UI12 的 componentized narrow workbench 回归暴露出 Runtime09 bridge 仍私有持有 `AUTHORED_DRAWER_HEADER_HEIGHT = 42.0`。声明式 `.zui`、`WorkbenchChromeMetrics` 和 autolayout collapsed constraints 已统一为 `panel_header_height`，但 live bridge 的模型输入和 Ultra/Narrow compaction 会再次把它覆盖为 42px。
- 该漂移会在窗口缩放进入窄 tier 时制造两套几何 authority：声明式节点期望 token 高度，bridge 却发布 42px shell。它既破坏 bottom reopen strip correctness，也会让 committed layout/cache equivalence 因 runtime override 发生额外 fallback。
- hard cutover 已删除私有常量。`WorkbenchDrawerLayoutInputs::from_workbench_model()` 与 `compacted_bottom_region_input()` 现在都消费调用方的 `metrics.panel_header_height`；generic `drawer_region_input()` 仍只接受通用 collapsed extent，没有引入 editor token 或兼容别名。
- 同文件增加 custom metric 低层合同，分别覆盖 collapsed model input 与 Ultra/Narrow compaction；来源 UI12 已提供默认 metric 的 componentized shell focused contract。验证顺序固定为：低层 custom metric -> componentized shell reopen strip -> 单次 layout-pass regression，不能以源码 grep 代替行为测试。

## 12. 未关闭问题和边界

本轮不能标记 `M4-M7 complete`，以下问题仍是后续阻断项：

1. root size 变化仍必须全局解析 responsive metadata；普通 dirty template node 已走受影响 ID，但包含 template metadata 的真实 10k 规模门和内部 node/slot/cell 访问计数尚未建立。
2. hit grid 在大量 pointer nodes 重叠于同一 spatial cell 时，单节点跨 cell/order 更新仍可能复制或排序大 cell；现有 outer counter 不能代表这个最坏情况的实际工作量。
3. Activity Rail 的 `28x32` control geometry 与 138/138 hit samples 已关闭此前的 `1x32` correctness 缺口；但 template projection 仍从 paint primitive 反推 semantic control，架构上仍需收敛到 arranged/tree control geometry。
4. 当前 resize 仍需要每帧 rasterize 已缓存 draw buffers 到 swapchain；这是 live resize 的必要显示工作。`painted_pixels` 仍按 full surface 记录，但 CPU command/batch/text/resource 工作已复用，GPU p95 低于 0.2 ms。
5. `viewport_image` profile 仍报告 region request full-frame repaint 和 presentation/layout 脏化，startup 也有 `gpu_presenter_recorded_no_draw_calls` 诊断；它们不属于按钮/resize/SVG 修复完成证据。
6. `zr_rhi_wgpu` 的 test target 当前被工作树中既存测试编译错误阻断，包括私有 image resource import、缺失测试 helper 和 vertex equality；WGPU library 与 editor 产品编译通过，但新增 WGPU 单测尚未形成可执行 gate。
7. editor 聚焦测试 `gpu_presenter_builds_one_command_snapshot_per_native_resize_transaction` 在 15 分钟内未完成 test binary 链接并超时，未取得执行结果。该行为说明当前 editor test target 过重，需要拆出更小的 presenter contract gate。
8. 通知中心目前没有可供 retained host 消费的 snapshot generation 和 next-expiry deadline；语义比较避免了重建，但每 tick snapshot/polling 仍是额外工作。
9. `auto_hide_duration_ms` 当前仍属于 layout/hit/render/text/input 全域 dirty。只有在运行时确实需要修改 timer metadata 时才会触发；后续应把计时状态移出模板布局属性，而不是继续扩大 dirty 例外表。
10. 新增 Toast countdown 回归在 editor `--lib` test target 编译阶段被 225 个既存测试错误阻断，92m19.8s 后仍未执行；主要是 `ViewTemplateFrameData` move、trait/constructor 漂移和既存 E0716。不能将源码回归视为已运行通过，需拆出可独立编译的 notification projection contract test。
11. `execute_layout_command` 目前仍会为其他 `changed=false` 命令无条件发布 layout/presentation effects，`EditorUiHost::apply_layout_command_inner` 也会在普通 no-op 后重算 session metadata。本次热路已删除冗余 reopen 命令，但通用 no-op invalidation gate 仍需收口。
12. `ShellContent` 已移到 full shell snapshot 之前并使用 committed derived state；当前未关闭的是产品可达性和 cache equivalence 证据：需证明连续跨 drawer 切换不会因 descriptor/payload/native presenter 差异持续回退，也不会留下 model/chrome/layout 漂移。
13. 2026-08-12 当前共享主线的正式 profiling build 仍被 34 个非本切片编译错误阻断，涉及 GPU timer export、IBL readback/private export、ECS `ChangeTick`、mesh SDF、query const、property closure 和 model geometry lifetime。随后验证器又因共享 CPU lane reservation 拒绝启动。本轮新增 UI 文件未在已有错误输出中出现，但在得到完整编译和运行证据前仍视为未验收。
14. device-level registry 已落盘，但双 native-window 同资源只上传一次、64 MiB LRU 压力、窗口销毁后资源回收和 10 分钟 GPU resident bytes 平台期尚未用当前 EXE 验证。
15. `UiSurface::rebuild()` 与 `compute_layout()` 仍直接调用全树 `dirty_summary()`；产品主路径应优先使用 `rebuild_dirty()` 的 tracked candidate 集合，但 legacy/full API 一旦可达就会恢复 `O(N)` dirty discovery。必须在产品 profile 中分别计数调用者，不能只看局部 patch 的 outer changed count。
16. Bottom drawer token hard cutover 已完成源码修改，但失败交接仍保持 open，直到 managed Windows 实际执行 custom metric、componentized shell 和一次 layout-pass 三层回归；共享 Cargo lane 或 target 生命周期故障导致目标测试数为 0 时不得关闭。

无效 profile 不作为证据：`runtime09-resize-command-cache-20260809-180315` 使用了 `NOSIZE`；`runtime09-resize-command-cache-valid-20260809-180455` 被 OS redraw 合并；`runtime09-resize-command-cache-uia-20260809-180600` 在 maximized 状态执行；`runtime09-resize-direct-swapchain-uia-20260809-1915` 因 automation harness 异常退出。

## 13. 后续实施顺序

1. CPU lane 释放且共享主线编译恢复后，先构建包含 committed `ShellContent` 与 `WindowMetrics` stage cache 的 profiling EXE；执行长间隔/快速 Activity Rail 点击和连续 resize，要求 early patch hit、full pane payload/full presentation/model build 为 0，并同时检查 slow frames、SVG CPU load、GPU upload、damage 和退出状态。
2. 根据新 EXE 的 fallback reason 收紧 committed-state equivalence；若产品热路仍因未建模字段回退，则把该字段纳入 target shell delta，而不是恢复完整 shell snapshot。
3. 让 no-op layout command 返回空 effects，并在 `diff.changed=false` 时跳过 session metadata recompute；为该契约建立独立窄测试。
4. 为通知中心增加 monotonic snapshot generation 与 next-expiry deadline；tick 仅在 generation 变化或 deadline 到期时同步，不再按帧构造/比较完整字符串快照。
5. 将 template projection 从 render-command rows 收敛为 semantic control rows：arranged geometry 提供 control frame，command range 只提供 paint/style/resource 派生信息。
6. 为 responsive resolution 增加 responsive-node index/topology generation，使 root resize 只遍历实际包含响应式属性的节点和隐式 Grid slots。
7. 将性能计数扩展到 responsive nodes、slots、hit cell entries 和排序工作量，移除 ignored 10k gate。
8. 收敛 resize/full-command 诊断语义，区分“本帧 full-surface present”和“重新构建 command stream”。
9. 修复 editor 和 `zr_rhi_wgpu` test targets 后执行 notification、projection、retained bypass、text、relative command、image residency 的窄 contract tests。
10. 执行 device-level UI image registry 的双 native-window contract/profile gate：同一 `(resource_key, generation)` 全 device 只允许 1 次 upload；第二窗口仅创建轻量 bind group；窗口销毁/LRU 后 GPU resident bytes 必须回落或达到平台期。
11. 完成 200 次 resize、1000 次 button/pointer、10k resource backlog 和 10 分钟 soak；只有 CPU、GPU、RSS、fallback 和 correctness gates 同时通过后，才关闭 M4-M7。

## 14. 本轮验证记录

- 2026-08-12 相关 Rust 文件 `rustfmt --edition 2021 --check`：通过；`git diff --check`：通过，仅有仓库 CRLF 提示。
- 当前主线 `validate-matrix.ps1 -Package zircon_app -Features 'target-editor-host,profiling' -Bin zircon_editor -CargoProfile profiling -SkipTest`：前一次完整尝试在 12m13s 后因 34 个共享主线既存 runtime/graphics/asset/ECS/plugin 错误失败；缓存重试 4m13s 得到相同错误集合，本切片 UI 文件未出现在错误中。
- 早期 `ShellContent` 实现后的验证器重试：第一次在 session register 阶段返回 `database is locked`；协调器随后报告 `healthy/read_write`，第二次因 `cargo_cpu_lane_reserved`（reservation `6d7d4be0bf1947f1b4564a1e0eed72a9`）拒绝启动，未进入编译。该结果不计为通过或源码失败。
- 2026-08-13 device-level registry 接线后的 `zr_rhi_wgpu` 托管 Windows build：第一次由 coordinator 分配 `D:\cargo-targets\zircon-engine\pool\f9fef...`，编译依赖期间 active target 被后台删除，Cargo 以多项 fingerprint/path `os error 3` 失败，未进入本 crate 源码；随后 `cargo.release` preflight 超时。第二次显式申请 `D:\cargo-targets\zircon-ui09-current` 时被前述残留兼容 job 标记为 busy；状态查询一度返回 SQLite `database is locked`，之后 job 自动恢复为 released。这些均为协调器生命周期故障，不计为源码通过或失败，也没有绕过协调器直接运行 Cargo。
- 2026-08-13 committed ShellContent、WindowMetrics stage cache、viewport 增量投影和共享纹理 profiler 接线完成后，相关 Rust 文件已执行 `rustfmt --edition 2021`，scoped `git diff --check` 通过（仅 CRLF 提示）。当前协调器报告 Cargo blocker `f0403a1606764c6d934dcfb3e4fa1e28` 仍由 `validate-matrix:text-runtime-mvp-baseline-r1-20260813` 占用，因此本轮尚未启动新的 Cargo 编译，也未绕过单兼容池规则。
- 2026-08-13 二次源级复审关闭两个 SVG 热路漏口：稳定 SVG tree lookup 不再逐次访问文件系统；command producer 按资源代次去重本地驻留查询，跨窗口 device registry 仍在 present 阶段解析以保留 LRU 竞态兜底。相关实现已格式化，正式 Windows 构建和双窗口/连续点击 profile 仍等待共享 Cargo 兼容池释放，不计为通过。
- 2026-08-13 Windows 路径复核进一步关闭 verbatim canonical alias 漏口；相关 watcher/cache Python 合同 10/10 通过。device registry 删除了未使用且会把可淘汰 LRU 瞬时状态误表述成承诺的 public `is_resident()`，只保留 present 阶段原子 resolve/prepare，并贯通 shared resident-bytes profiler。Rust 动态验证仍受同一 managed lane blocker 限制。
- 2026-08-13 导入 Runtime09-owned failure `failure-2026-08-13-workbench-drawer-header-token-drift.md`。已删除 42px 私有常量并统一消费 `WorkbenchChromeMetrics::panel_header_height`；`rustfmt --check`、scoped `git diff --check` 和常量零引用检查通过。动态 focused/layout-pass 回归仍待 managed Windows lane，交接保持 open。
- 2026-08-13 首次 managed lower-layer 测试申请在执行前被 `cargo_reuse_pool_busy` 拒绝；兼容池 owner 仍为 job `f0403a1606764c6d934dcfb3e4fa1e28` / session `validate-matrix:text-runtime-mvp-baseline-r1-20260813`，coordinator 记录为 running 但当前 `live_process_pids` 为空。未接管或 finish 外部 job，目标测试执行数仍为 0。
- 2026-08-13 最后一轮 `zr_rhi_wgpu` shared-registry focused test 申请同样没有启动 Cargo；coordinator 报告下一条 managed CPU lane 已预留给 session `coordinator01-benchmark-identity-review-maintenance-20260811`（reservation `d365245d296743efab520a23815b0c20`）。因此本轮新增 SVG/device-registry/drawer Rust 行为测试的当前源码执行数仍为 0，不能据此声明动态 green。
- 2026-08-13 无 Cargo 源码/性能合同实际执行 18 条：drawer resize 1/1、workspace watcher 2/2、bounded refresh 3/3、watcher generation 5/5、hit index cutover 2/2、presentation generation 4/4、plan output audit 1/1 均通过。`test_runtime_ui_architecture_boundary` 另有 1 条共享主线清单漂移失败，报告 `surface_unexpected_entries = [focus, invalidation.rs]`；它不作为本切片回归或通过证据。
- `cargo test -p zr_rhi --profile profiling retargeted_surface_preserves_the_generation_projection_extent`：通过，1/1。
- `cargo test -p zircon_editor --profile profiling gpu_presenter_builds_one_command_snapshot_per_native_resize_transaction -- --exact`：15 分钟链接超时，未执行测试，不计为通过。
- 里程碑构建时 `cargo check -p zircon_editor --profile profiling --no-default-features`：通过；随后共享工作树继续变化，当前同命令在 4m50s 后被环境 IBL 代码的 11 个既存错误阻断，包括缺失 `build_source_cubemap_irradiance_cube_with_parallel_executor`、缺失/不可见 `IblBakeArtifactProducer` 和 `SourceCubemapIrradianceCube`，以及当前工具链拒绝 const `u32::max`。不能把先前通过外推到当前源码。
- `cargo build -p zircon_app --bin zircon_editor --profile profiling --no-default-features --features target-editor-host,profiling`：通过，19m54s。
- `cargo test -p zircon_runtime --profile profiling deserialized_surface_rebuilds_geometry_for_the_first_new_root_size -- --exact --nocapture`：13m38s 后在 test target 编译阶段失败，未执行新增测试；阻断为 native bitmap atlas 缺失 `tests/missing_raster.rs`、环境 IBL 类型/helper 缺失和 rich cache `String`/`Arc<str>` 不匹配，共 7 个既存错误。
- `cargo fmt --all -- --check`：被 35s 执行上限截断并触发 rustfmt broken-pipe panic，不计为通过；缩小到相关模块后仅报告共享工作树既有 import 排序/格式差异，新增反序列化测试的唯一格式差异已修正。
- profiling EXE：`E:\Git\ZirconEngine\target\profiling\zircon_editor.exe`，91,507,712 B。
- runtime DLL：`E:\Git\ZirconEngine\target\profiling\zircon_runtime.dll`，44,408,832 B；与 `deps` 产物 SHA-256 一致。
- 24 次 UIAutomation resize：进程退出码 0，平均 13.7605 ms，最大 18.4788 ms。
- 最终 profile：`E:\zircon-profiles\runtime09-svg-m7\runtime09-resize-direct-swapchain-uia-20260809-1920`。
- asset-only activity rail profile：`E:\zircon-profiles\runtime09-svg-m7\rail-template-fix-manual\20260809-224529-startup`。窗口 client `1672x941`；四个 rail button 仍为 `1x32`，但右侧按钮由旧样本的 `x=1675` 移到 `x=1668`，138 个 hit samples 的 expected/actual mismatch 为 0。进程在关闭窗口超时后被终止，因此只采信已落盘 geometry/screenshot，不将退出生命周期作为通过证据。
- `control_frame()` union EXE：91,601,920 B，SHA-256 `56965EEC9D80CF9660145B743EC1CFD453EFAE25314F95FFE762FED8F1083888`；runtime DLL 44,381,696 B，SHA-256 `E02DDC475061C517260EA0115F9928410E818794E5C35D01577F5667FBA25847`。
- `control_frame()` union profile：`E:\zircon-profiles\runtime09-svg-m7\rail-control-frame-union\20260810-010953-startup`。进程退出码 0，client `1672x941`，四个 button 仍为 `1x32`；该反证将根因定位到 stencil composition 的 last-paint-wins 覆盖。startup GPU image upload 仅 2 次、131072 B，均为预热驻留，没有观察到帧间 SVG 重传。

## 15. 2026-08-13 当前工作树复审裁决

本节覆盖第 12、13 节中由较早源码版本得出的风险。复审以当前共享工作树为准；旧 profile 仍只作为问题定位证据，不能外推为当前源码验收结果。

### 15.1 已关闭或需要降级的旧风险

1. `rebuild_dirty()` 的正常增量路径不再通过 `dirty_summary()` 和 `clear_dirty_flags()` 全树扫描。首次建立 dirty index 后，候选集合来自 `dirty_node_ids`、invalidation transaction 和 `UiTreeNodes::pending_mutation_node_ids()`；清理也只访问这些节点。
2. `UiTreeNodes` 不再公开无跟踪的 mutable map。`get_mut`、`entry`、`insert`、`remove`、`IndexMut`、`iter_mut` 和 `values_mut` 都登记 mutation node；编辑器现存直接 `root.dirty.* = true` 的写法因此不会静默绕过 dirty index。
3. 生产 `mark_layout_dirty` / `invalidate_node(Layout)` 虽同时设置 layout-derived hit-test dirty，但 `rebuild_dirty()` 会识别“hit-test dirty 仅由 layout 派生”，因此真实 API 可以进入 arranged / hit grid / render 的局部 patch，不再是测试直接写 `dirty.layout` 才可达。
4. `last_layout_root_size` 是 serde-skip 状态，但当前 `None` 且已有 arranged nodes 时会被视为 root-size change，反序列化后的新尺寸不会再无操作返回。
5. layout engine selection report 的稳定替换通过 `replace_selection_at()` 增减聚合计数，不再为单节点 backend route 变化调用 `recompute_counts()` 全扫全部 layout containers。
6. editor 中仍存在 `surface.rebuild()`，但本次逐调用点核对后，主点击相关实例大多是 4 到几十节点的独立 pointer proxy surface；resize proxy 固定为 root 加 3 个 splitter。它们是可继续收束的 API debt，不是 Workbench 2 秒级 presentation 延迟的主要来源。

### 15.2 当前仍成立的结构问题

1. 根尺寸变化会使所有 surface roots layout dirty，因此实际几何约束传播通常仍是 `O(N)`。这是窗口 reflow 的合理上界；错误在于历史实现把业务 model、descriptor、chrome snapshot、文本、资源和 command stream 的无关重建也绑在每个 resize event 上。第 11.2、11.12 节的 transaction 与 committed stage cache 必须用当前 EXE 证明这些附加工作已归零。
2. `apply_mui_responsive_layout()` 在 root-size change 时仍遍历全部 nodes 多轮并遍历全部 slots。若真实 Workbench 中只有少量节点带 viewport/variant/density metadata，这部分属于可消除的前置 `O(N)`；但在 fresh resize profile 证明它占主导前，不应先引入新的 topology index。
3. hit grid 的 node / entry 反向索引已经消除按 total nodes 查找，但一个 cell 内仍用顺序数组 remove / insert。其复杂度是 `O(cell occupancy)`，与 Unreal `FHittestGrid::FCell` 的局部数组策略一致；只有重叠压力 profile 证明真实 cell occupancy 很高时，才需要更复杂的容器。
4. full fallback 仍是正确性边界：拓扑变化、visible-range、input policy、clip/bounds 扩张、非 owner-frame render command 和 resolved text geometry 都不得伪装成 geometry-only patch。优化目标是降低 fallback 的产品触发率，而不是删除这些保护。
5. `ShellContent` early commit、`WindowMetrics` committed snapshot 和 device-level image registry 均缺当前源码构建及产品动态证据。它们现在是“实现完成、验收未完成”，不能因静态合同通过就宣称按钮、resize 或跨窗口 SVG 问题已解决。

### 15.3 与 Unreal Slate 的当前差距

Unreal 的关键不是简单使用 dirty flag，而是让 invalidation root 同时拥有稳定 widget list/index、按依赖顺序处理的 pre/prepass/post update heap、唯一且有序的 final update list、cached element data 以及可重建比对的验证开关。`FSlateInvalidationRoot` 还显式验证 widget list/index、hit-test grid、visibility、volatile list、attribute 一帧只更新一次和 cached elements；`FHittestGrid` 通过 `WidgetMap -> WidgetArray index -> cells` 更新同一 widget，cell membership 未变时只更新排序和元数据。

Zircon 当前已具备 dirty candidate、arranged index、hit reverse index、render command ranges 和 conservative fallback，但还缺等价的产品级不变量门：

- 同一 node 在一次 transaction 中最多完成一次 layout / extract / publish；
- local patch 后 arranged、hit grid、render extract 与强制 full rebuild 结果等价；
- stable click 不得进入 full Workbench model / descriptor / chrome / command construction；
- resize 中只允许 root geometry propagation 和 target present，不允许资源 identity、SVG decode/raster、文本 shaping 或业务 model 重建；
- device image registry 的 shared upload、resident bytes、eviction 和 window-local bind-group ownership必须可观测。

### 15.4 修订后的实施顺序

1. 先恢复托管 Windows 构建，生成与当前 source manifest 绑定的 profiling EXE；旧 EXE 不参与当前验收。
2. 执行长间隔与快速 Activity Rail / drawer 点击，要求 callback、recompute、presentation、model、command、SVG 和 GPU upload 的计数能区分局部命中与 full fallback，并记录 p50/p95/p99/max。
3. 执行 200 次 native resize，分开报告 OS resize 调用耗时、UI transaction 耗时、layout visited nodes、model/chrome build、command snapshot/batch/text cache、GPU time 和 RSS 峰值。根 geometry `O(N)` 可以存在，无关阶段 rebuild 必须为 0。
4. 执行同 SVG 双 native-window、内容不变 watcher event、单文件内容变化、64 MiB LRU 压力和窗口销毁；同一 `(resource_key, generation)` 的 device upload 必须为 1，resident bytes 最终达到平台或回落。
5. 只有 profile 显示 root resize 的 responsive resolution 占显著比例时，才增加 `responsive-node / implicit-grid-slot` index，并以 topology generation、同 cardinality detach/attach、serde rebuild 和 direct mutation 回归保证新索引不陈旧。
6. 增加 Unreal 风格的 debug equivalence gate：局部 arranged/hit/render 结果与同输入 full rebuild 做结构化比较；压力测试不得只看 outer visit counter，还要记录 cell occupancy、fallback reason、responsive visited count 和实际耗时。
7. 完成 1000 次 pointer/click、200 次 resize、10k resource backlog 和 10 分钟 soak 后再关闭 M4-M7；任何当前源码未执行的测试都不得写成通过。

### 15.5 复审确认的当前热路缺口

1. 第 11.9 节“早期路径不再构造完整 presentation”只对 model/projection builder 成立，对提交载体不成立。`patch_shell_content_presentation_from_state()` 仍调用 `UiHostWindow::get_host_presentation()`；后者经 `HostPresentationGeneration::materialize()` 克隆整个 `HostWindowPresentationData`，再由 `patch_host_presentation_paint_models()` 用新的整窗 `Arc` 替换旧结构。`ModelRc` 字段只是引用计数克隆，但 shell、scene、pane 和大量 `SharedString=String` 仍按值复制；这不是 Unreal invalidation-root 的局部 proxy update。
2. GPU presenter 的 region path 会把 damage 传入 command extraction，primitive 输出受 paint clip 限制，但 `record_host_frame_commands()` 仍无条件进入 `draw_workbench_presentation_commands()`；componentized 顶层继续调用 top chrome、四个 dock、resize、floating、menu/prompt 和 root overlay。部分 template subtree 能通过 paint index 限制 rows，但无关 dock 的入口、状态解析和原生 pane/text 分支仍会被访问。因此 `ChromeCommandPatchCount=1` 只证明局部命令/像素提交，不能证明 CPU scene traversal 是局部的。
3. 当前最高优先级改为两项：ShellContent 提交改为 generation-checked 的目标 dock 原位 patch，先重绑 paint/hit index、失败回滚且不 materialize 整窗；绘制入口按稳定 host region 对 damage 做第一层路由，无关 top chrome/dock/splitter/floating window 不进入子绘制。两项都必须有 visit/snapshot counters 和当前 profiling EXE 证据，不能以源码结构推断完成。
4. `Arc::make_mut` 只在没有外部 presentation generation 快照时保证原位更新；若输入派发仍跨同步重算持有旧 generation，则为保持快照一致性会退化为完整 clone。产品 profile 必须同时记录 structure copy/fallback，若热路出现该退化，下一步应把 monolithic `Arc<HostWindowPresentationData>` 拆成 shell/layout/dock 级不可变 Arc，而不是放宽快照隔离。

### 15.6 当前算法裁决与实施切片

本轮复审不再把“有 dirty flag / 有 damage / 有 cache”视为完成。判断标准改为实际工作量是否与变化量相关，并明确允许的上界：

| 场景 | 允许工作 | 禁止工作 | 目标复杂度 |
| --- | --- | --- | --- |
| 稳定按钮 hover / click | pointer cell 查询、目标 action、目标 Dock presentation patch、目标 damage command | 整窗 presentation materialize、三侧 Dock 遍历、业务 model/descriptor/text/SVG 重建 | `O(hit cell + changed controls + damaged region)` |
| 同一区域 Drawer 切换 | 新目标 pane payload、目标 Dock rail/header/tab/paint models、对应 hit/paint index rebind | 完整 Workbench snapshot、非目标 pane payload、整窗 Arc 替换 | `O(target dock models)` |
| native resize | root constraint propagation、必要的 arranged/hit geometry 更新、已有 command snapshot 投影复用 | 业务状态、SVG decode/raster、文本 shaping、静态 GPU image identity 更新 | layout 可 `O(N)`；其他阶段应为 `O(1)` 或 0 |
| 内容不变资源事件 | 合并事件、受影响源指纹比较 | raster/tree 驱逐、atlas generation 推进、GPU upload | `O(changed source bytes)`，下一帧上传为 0 |
| 单个 SVG 内容变化 | 单源 tree/raster 更新、单个内容键的新 atlas slot/page | 全 raster cache / 全 icon atlas 清空、无关图标上传 | `O(source + raster pixels + affected atlas page)` |

已经落入当前工作树的实现切片：

1. `ShellContent` 不再通过 `get_host_presentation().materialize()` 构造整窗值。提交先借用 presentation generation 完成结构/模型校验，释放快照后以 expected structure generation 执行目标 Dock patch；retained hit/paint reverse index 先验证并重绑，再用 `Arc::make_mut` 更新 shell/layout/单 Dock。`ui.shell_content.structure_in_place_count` 与 `structure_copy_count` 用来证明热路是否真正原地。
2. presentation 的其他生产读取点改为 generation borrow：Workbench node projection 只借用当前 nodes；完整 apply 只复制必须跨 replacement 保留的 close-prompt/node state，不再为了读取几个字段 materialize 整窗。当前生产源码中的 `get_host_presentation()` materialize 调用已归零，测试 helper 除外。
3. command record 入口增加第一层 damage routing：top/status chrome、left/right/bottom dock、splitter和 floating window 在 region 不相交时不进入子绘制；Dock 路由记录 `ui.paint.dock_route_visit_count`。template subtree 仍使用已有 paint index/effective clip，full damage 保持原顺序与行为。
4. visual raster cache 的路径事件增加内容指纹门。缓存逻辑资源时记录候选源的 BLAKE3 指纹；合并后的 watcher 事件只驱逐字节实际变化、出现或消失的 base key，稳定事件记录 `ui.visual_asset_cache.unchanged_path_event_count` 并保留 raster。SVG tree cache 使用同样规则，避免无变化事件导致下一帧重新 parse。
5. 路径级 raster 失效不再调用 `invalidate_editor_icon_atlas()`。SVG raster 的既有 `retained-image:{size}:{pixel hash}` 已是最终 RGBA 内容寻址；相同内容重新光栅后自然命中原 `IconSourceKey`、atlas slot 和 GPU generation，真正不同内容只分配新 slot。
6. 复查还发现原 atlas 的产品入口与单测不一致：测试直接构造 `icon:*`，但真实 SVG raster 输出 `retained-image:*`，`is_editor_icon_key()` 因而会拒绝产品 SVG，所谓图集只在人工测试键上成立。加载器现在仅对 Icon / TemplateIcon 语义把最终内容键包装为 `icon-raster:{retained-image-key}`，atlas 接受该前缀；普通图片/preview 保持 standalone。身份仍由最终像素决定，不会因路径事件改变。
7. editor icon atlas 增加 64 页 / 64 MiB 双预算及 page LRU。活跃 command 先 touch 页；新内容只使用新页，达到预算时淘汰当前 command 未引用的最旧页、删除该页 slots、复用稳定 page id，并为新页推进 generation。发布过的页仍不可变，旧 draw list 持有的 `Arc<[u8]>` 与旧 GPU generation 不被就地改写。

仍需动态证明的门：

- focused Rust 回归必须覆盖目标 Dock 无 materialize 原地提交、持有旧 generation 时 copy-on-write 隔离、damage 只访问目标 region、内容不变事件保留 raster/tree、单 SVG 变化不重置无关 atlas slot、atlas 达预算后的 LRU 淘汰；
- 新 profiling EXE 中稳定点击的 full model/presentation/command 数必须为 0，`structure_copy_count=0`，非目标 Dock route visit 为 0；
- 200 次 resize 中 command snapshot build 必须保持每 transaction 1 次并复用，其间 SVG parse/raster/shared upload 为 0；
- 内容不变 watcher 场景的 SVG parse/raster/GPU upload 为 0；单 SVG 修改后 shared upload 只允许受影响 atlas 页一次，并且其他页 generation 不变；
- 双窗口共享、64 MiB 压力和 10 分钟 soak 的 shared/local resident bytes 必须达到平台或按 LRU 回落。静态源码结构不替代上述证据。

### 15.7 Native resize 快照支路的生产可达性修复

复审继续沿真实事件循环追踪后，确认此前的 resize 结论仍遗漏了一个直接的状态机错误。GPU presenter 已实现 `present_during_native_resize()`：第一次构造 retained draw-list snapshot，后续只 retarget surface size，并有 build/reuse counter；但生产 `redraw_requested_impl()` 在 `pending_presenter_resize.is_some()` 时直接返回。更严重的是它在返回前已经取走 pending redraw，而 `SurfaceResized` handler 又明确不 queue redraw。结果是这条快照复用优化在连续 resize 的正常生产路径中不可达：窗口拖拽期间显示冻结或依赖平台拉伸旧帧，直到最后一个事件后 80 ms 才配置 surface、提交真实 reflow 并重画。

修复后的状态机将两个不同频率的工作拆开：

1. 每个 `SurfaceResized` 只覆盖保存最新 physical size，继续把 retained reflow deadline 推迟 80 ms；同一时刻 queue 一个 `WindowResize` full present，但 `frame_update=false`，因此不会同步运行 layout/presentation transaction。
2. resize redraw 先消费并配置最新 pending surface size，再进入已经存在的 `present_during_native_resize()`。GPU 路径只在 transaction 第一帧构造 command snapshot；后续帧复用 draw list、纹理、文本与 batch，只改变目标 surface size。
3. deadline 到期时仅补配置尚未来得及 present 的最后尺寸，然后结束 native-resize pending 状态并 queue 一次 `frame_update=true` 的真实 WindowMetrics reflow。业务 model、chrome snapshot 继续由 committed shell stage cache 复用。
4. 普通输入/维护 redraw 若在 resize transaction 内合并，仍只做 snapshot present，不允许绕过 debounce 触发布局；presentation 错误、surface resize 错误和无 presenter 启动阶段维持现有保守退出/无操作语义。

无 presenter 的启动边界不能保留旧 `pending_presenter_resize`：presenter 创建本身通过 `UiSurfaceDescriptor::from_winit_window()` 读取当前 surface size，晚到地重放旧 pending 值反而可能覆盖新尺寸。因此 helper 先消费 pending slot，再判断 presenter 是否存在；这个分支只丢弃冗余事件，不丢失真实尺寸状态。

资源事件路径也进一步限制了文件工作量：同一个 SVG/图像源即使对应多个 tint、DPI 或语义 variant base key，一批 watcher event 中也只读取并计算一次 BLAKE3 指纹。受影响 base key 共享该批次的 `PathBuf -> SourceFingerprint` 临时表，事件成本从最坏 `O(variant_count * source_bytes)` 收敛为 `O(source_bytes + affected_variant_count)`；稳定内容事件不删除 CPU raster，也不推进 GPU atlas generation。

这项修复只关闭“拖拽中没有交互帧”的错误，不把停止后的完整几何投影冒充为已优化。动态 gate 必须同时满足：

- 连续 200 次 resize 中，每个 transaction 的 `ui.window_resize.command_snapshot_build_count = 1`，`command_snapshot_reuse_count > 0`；拖拽段 model/chrome/presentation/text-shape/SVG parse/raster/static GPU upload 均为 0；
- surface reconfigure 次数不得高于实际已提交 resize 帧数，重复尺寸事件由 pending slot 合并；resize present 的 p95 需落入单帧预算；
- 停止后 `WINDOW_METRICS` 只提交一次真实 reflow，`WorkbenchModelBuildCount = 0`、`ChromeSnapshotCount = 0`；若完整 presentation/hit/pointer/native presenter 投影仍主导最终卡顿，再实施 geometry-only presentation patch，而不是降低 debounce 或放宽 correctness fallback；
- 软件 presenter 可保守完整 present，但 GPU 产品路径必须命中 snapshot reuse。只有当前源码 profiling EXE 的 counter 与 ETW/Tracy 时间线共同证明上述条件，才可关闭 resize gate。

### 15.8 当前源码 profiling 证据合同与 2026-08-13 验证状态

为避免继续用旧 EXE 的数字推断当前源码，profiling capture 增加 `window_resize` 产品场景与 source-bound gate：

- 原生窗口交互由独立 `tools/ui-profile-native-resize.ps1` 生成 24 个确定性尺寸步进，40 ms/步，最后恢复原始 extent；证据记录完成步数、耗时、working set/private bytes 起止与峰值。
- timeline gate 要求 `ui.window_resize.command_snapshot_build_count == 1`、`command_snapshot_reuse_count > 0`、`surface_reconfigure_count > 0 && <= completed_steps`，并限制 `workbench_model_build_count <= 1`、`chrome_snapshot_count <= 1`。这可以区分“窗口在动”与“真的复用 GPU command snapshot”。
- capture manifest 现在绑定 24 个关键源码，包括 layout/presentation、pointer dispatch、Workbench/Pane hit index、resize event loop、GPU native-resize present、资源刷新策略、raster/SVG cache、像素加载、icon atlas、batching、WGPU image cache，以及 Runtime profiling 接口和 hotspot 聚合器；editor EXE 与 runtime DLL 任一个早于最新关键源码修改时间都会 fail closed，不能启动 capture。
- 本轮扩展为 24 个 manifest source path 后已重新实测：`ui-profile-native-resize.Tests.ps1` 2/2、`ui-profile-capture-output-contract.Tests.ps1` 14/14，PowerShell 解析通过。它们只证明 capture/source-bound 合同本身有效，不能替代当前产品 EXE 的帧时间与缓存命中率证据。
- 产品 profiling build 第一次申请被协调器以 `admission_checkpoint_stale` 拒绝，未进入 Cargo，不计为编译结果。第二次于 2026-08-13 获准执行，89.7 秒后在基础依赖阶段失败：协调器分配的 `D:\cargo-targets\zircon-engine\pool\db56...` 整个目录在多个 rustc 并行写 `.d` 文件时被回收，`cfg-if`、`autocfg`、`unicode-ident`、`windows-link` 等均报告 `os error 3`。失败后目标目录确认不存在；尚未编译到 Zircon 项目 crate，因此不能归因为本次 Rust 源码。
- 协调器随后进入 circuit-open，窗口截至 `2026-08-13T07:10:31Z`。在熔断恢复并成功生成晚于关键源码的 EXE/DLL 前，`window_resize`、`click`、SVG/resource refresh 的动态性能 gate 均保持 open；静态合同不能替代当前产品帧时间。
## 15.9 全量 fallback 的隐藏平方复杂度与下一步实施门槛

2026-08-13 的再次静态复核确认，现有“增量布局、局部 arranged patch、局部 hit patch”还不能证明交互热路已经收敛。只要任一 dirty 域或保守校验令路径回退到全量构建，两个生产算法仍会把节点 ID 反复交给 `UiArrangedTree::get`；该接口底层是对 `Vec<UiArrangedNode>` 的线性 `find`。

当前成本不是报告计数所表达的简单 `O(N)`：

- `build_hit_grid` 对 draw order 中每个节点做一次线性 owner lookup，并为可交互节点沿祖先链重复线性 lookup，用于可见性、effective input policy 和 bubble route。最坏成本接近 `O(N^2 * D)`，其中 `D` 为祖先深度。
- 全量 render extract 至少在 owner text prewarm、overlap admission 和 command collection 三个 pass 中重复 owner lookup/祖先可见性 lookup。文本缓存可以避免重复 shaping，但不能抵消这些结构查找，成本仍可接近 `O(P * N * D)`，其中 `P` 为 pass 数。
- `UiSurfaceRebuildReport.hit_grid_outer_node_visit_count` 和 `render_outer_node_visit_count` 只记录 draw-order 外层访问数，没有计入上述内部 `Vec::find`，因此“访问 N 个节点”并不代表线性复杂度。
- `state_flags.dirty` 会被扩张为 `hit_test + render + input`，但生产 pointer transient 路径已核实通过 component-state store 将 hover/pressed/focus 标成 render-only；enabled/clickable/visibility/input-policy 这类改变命中资格的状态才使用该标记。因而不能无条件删除这项扩张，动态 profiling 应继续确认按钮慢帧究竟来自 render patch fallback、编辑器 presentation/command rebuild，还是其他 invalidation 来源。

这解释了为何单独补局部 patch 后，按钮和窗体缩放仍可能“卡得不可交互”：局部快路只覆盖了部分 invalidation 组合，而 fallback 的基础算法本身仍不合格。

实施顺序调整如下：

1. **M4.1 全量索引化**：为一次 arranged 快照建立稳定 `node_id -> node slot` 索引；full hit build、full render extract、owner text prewarm 和可见性/祖先查询必须使用该索引。目标复杂度为 `O(N log N + pointer_nodes * D log N + cell_membership)`，禁止在 draw-order 循环内部调用线性 `UiArrangedTree::get`。
2. **M4.2 dirty 域闭合验证**：保留现有“交互视觉状态 render-only、pointer eligibility/input policy/visibility 变化才 hit/input”的生产语义；补动态计数和回归，防止其他 mutation/dispatch 路径把 hover/pressed/focus 再扩张成 hit rebuild。
3. **M4.3 cell 局部成本可见化**：局部 hit patch 的验收必须同时报告受影响 cell 数和 cell occupancy；在所有节点重叠于一个 64px cell 的压力样本中，不得用 outer-node count 掩盖 `retain/insert` 的 `O(cell occupancy)` 成本。
4. **M5.1 resize 双速率验收**：连续原生 resize 事件只更新 surface/projection 并复用 retained draw list；模型布局和 chrome command 重建在稳定 deadline 后最多各一次。
5. **M6.1 SVG 端到端暖缓存验收**：同一内容 fingerprint 的 SVG 在重复 frame/hover/click 中，parse 和 raster miss 不再增长；GPU shared image upload 只在新 `(content key, generation)` 出现时发生。路径别名、尺寸变体和 watcher 批量失效分别统计，不得把 CPU raster cache hit 误当作 GPU cache hit。

验收证据必须包含：

- 至少一个规模化 full fallback fixture，证明 full hit/render 不再执行按总节点数的内部线性查找；
- 重复按钮交互的 layout/arranged/hit/render 各域 rebuild/patch 次数；
- 24 步原生窗口 resize 的 surface reconfigure、模型布局、draw-list build/reuse 数；
- 同一 SVG 图标冷帧与暖帧的 parse、raster hit/miss、GPU upload/cache-hit 和显存驻留字节；
- CPU p50/p95/max、working set/private bytes 峰值与测试前后差值。

### 15.10 布局事务 slot 索引与 full fallback 线性化进展

继续沿 native resize 的完整 reflow 路径检查后，发现上一节只覆盖 arranged snapshot 之后的隐藏查找，measure / arrange 本身仍有同类问题。`slot_for_container_child()` 原来对 `tree.slots` 做线性 `find`，并被 measure、Zircon arrange、Taffy admission、axis constraint、Grid、Masonry、Wrap 和 incremental-root arrange 共 17 个调用点反复调用。节点/slot 数同阶时，完整窗口 reflow 的 slot 查询部分最坏为 `O(N * S)`；Grid/Wrap 等多 pass 容器还会重复这项成本。外层 layout visited-node counter 同样没有记录内部 slot 扫描，因此会把平方级实际工作误报成 N 次访问。

当前工作树实施了事务级 `UiLayoutSlotIndex`：

1. full layout 在 responsive style resolution 后建立 `(parent_id, child_id) -> [slot index]`，Surface 随后持久保留该派生索引；incremental measure/arrange、responsive Grid 与正式 slot mutation API 共用它。常规局部布局不再每次为全部 slots 重建索引。
2. lookup 仍按原始 Vec index 顺序检查候选，并按 container 对应的 `UiSlotKind` 选择第一个匹配项。因此重复 parent/child edge 或不同 kind 共存时，不会因索引化改变原来 first-match 语义；新增低层回归锁住该契约。
3. arranged build 同样在单次事务内建立 slot/node 索引；full hit grid 与 full render extract 复用 surface 已有 `node_id -> arranged index`，owner、祖先 visibility/input policy/bubble route 和三个 render extraction pass 不再调用线性 `UiArrangedTree::get`。canvas layering 的 parent/child slot 集合也只预聚合一次。
4. 索引为每条树 parent-child edge 预填“无 slot”负缓存；否则复杂 Surface 中普通 child 第一次 measure/arrange 仍会扫描全部 slots。热命中不克隆候选 Vec、不分配，只在缓存 edge 身份失配时扫描并修复该边；slot 总数变化时整体重建。
5. 同 cardinality detach/attach 不再只靠长度判断：查询新 edge 时若缺失，或旧 index 的 `(parent_id, child_id)` 身份不匹配，按当前 Vec 修复该 edge。新增测试覆盖单 slot 从旧 child 替换为新 child；重复 edge 仍保留 Vec 顺序的 first-match 语义。
6. 复杂度目标修订为：首次/full slot index build `O(N + S log E)`，稳定 edge lookup `O(log E + edge duplicates)`；full arranged/hit/render 的结构 owner lookup为 `O(log N)`，总体不再含显式 `N * Vec::find(N)`。公开 `UiTree.slots` 仍是 Vec；生产模板构建、node-pool detach、virtual rows 直接修改时都会改变 slot 数量并触发整体失效。同数量原地制造“旧 edge 仍合法、另增重复 edge”的非正式写法无法无扫描感知，必须迁移到 Surface mutation API，而不是重新在每帧支付全量哈希成本。

当前静态证据：layout pass 中 `slots.iter().find` 已归零；相关文件 `rustfmt` 与 scoped `git diff --check` 通过，只有仓库 CRLF 提示。受管 Windows `zircon_runtime` check 仍未进入 Cargo：2026-08-13 最新申请被 coordinator 以 `unmanaged_artifacts_detected` 拒绝，明确列出 `D:\ZirconBuilds\mvp-test-fixtures-34392` 等其他会话未登记构建产物。当前任务不拥有这些目录，不能擅自删除。因此本节实现状态是“源码完成、动态未验收”，不能写成性能 gate 通过。

下一步关闭顺序：

1. 编译并执行 slot first-match、布局等价、full hit/render 等价回归；任何 borrow/signature/行为错误先在低层修复。
2. 为 responsive candidate 增加持久 topology generation 仍需动态 profile 证明收益；当前 full root resize 已将三次 node collect 收敛为一次候选扫描，incremental responsive 只访问 dirty nodes，slot edge identity 已覆盖同 cardinality detach/attach correctness。
3. 扩展 profiler 记录 layout slot lookup、responsive visited nodes/slots、hit affected cells/entry visits/sort work；规模门不得只断言 outer node count。
4. 当前源码 EXE 上执行 1000 次按钮/鼠标与 200 次窗口 resize，确认结构查找成本下降后再判断剩余主导项是布局计算、presentation clone、command traversal 还是 GPU present。

### 15.11 Responsive 候选索引与按钮伪状态传播复审

继续追踪 `pointer -> component state -> runtime style -> dirty index -> render patch` 的真实产品调用链后，确认按钮慢响应还有一个此前未计入的算法放大器。旧 `apply_pointer_component_state()` 对 route 的每个 entered/left/focus/pressed 节点立即调用 `apply_runtime_state_style_subtree()`；后者无条件 DFS 该节点全部后代，并对每个节点匹配全部 runtime pseudo rules。鼠标从窗口外进入一个深层按钮时，route 中祖先路径会形成大量互相重叠的子树扫描；press 状态随后又通过 `state_flags.pressed` 同步重复执行一次。其成本不是 `O(changed controls)`，而可能接近 `O(sum(changed ancestor subtree sizes) * runtime rules)`。

本轮实现将伪状态传播拆为选择器依赖驱动的两条路径：

1. runtime style index 在编译 stylesheet 时提取“非最终 segment 中含 pseudo state”的祖先 segment。状态变化节点若不匹配任何这类 segment，使用单节点 cascade，只重算目标控件；典型 `Button:hover` 不再遍历按钮后代。
2. 只有节点类型/class/id/host 确实可能匹配祖先伪状态 segment 时，才保守遍历其子树，以保留 `.toolbar:hover Text.label` 和 child combinator 的正确语义。同一 pointer route 先完成全部 hovered/pressed/focused 状态更新，再求最小覆盖 roots，避免祖先和后代重复扫描。
3. transient `state_flags.pressed` 若 component state 已在上一阶段完成，只登记 render dirty，不再重复 runtime style cascade。
4. runtime style apply 现在返回实际发生属性变化的 `(node_id, dirty flags)`；surface 将每个后代通过 `mark_node_dirty()` 登记到 tracked dirty/invalidation index。此前只有 `node.dirty` 被直接写入而 tracked set 未闭合，局部 rebuild 可能看不到祖先伪状态真正改变的后代。
5. 增加两类回归合同：self pseudo state 后只允许目标 root 出现在 `pending_rebuild_node_ids()`；512 深度的祖先 hover 仍必须改变 leaf foreground，且 leaf 必须进入 pending rebuild set。

布局 responsive 阶段也已从三次全树 node collect/遍历收敛为一次事务级 `MuiResponsiveCandidates`：分别记录 media-query、visibility、responsive container 和 implicit-grid parent 候选；各 pass 只访问对应列表。slot pass 仍扫描 slots，但在 expensive metadata 检查前按 implicit-grid parent set 过滤。该索引在 responsive apply 前按当前 tree 重建，因此不会仅凭 slot 数量判断 topology freshness，也不会在同 cardinality detach/attach 后复用旧 child identity。

当前动态状态仍为 open：`zircon_runtime` 受管 Windows check 在进入 Cargo 前被 coordinator 以 `cargo_cpu_lane_reserved` 拒绝，reservation `98d24e507b7442279d2de721743f00d7` 归属 `coordinator01-benchmark-identity-review-maintenance-20260811`。上述 Rust 文件已格式化，但 focused tests 尚未执行，不能声明按钮延迟 gate 通过。

下一步实施和验收顺序：

1. 受管 lane 可用后先编译 runtime，并执行 self/ancestor pseudo、pointer click、dirty-domain、incremental layout/hit/render 契约；编译/行为错误优先在低层修复。
2. 给 runtime style apply 增加 visited-node/rule-match 计数；1000 次 hover/click 要求 self-only button 每次 style visited nodes 与 changed route nodes 同阶，ancestor-dependent subtree 扫描必须有明确 reason。
3. template action 的 `ControlPropRef` 已改为 Surface 派生 control-id index。首次查询全量建立，之后按 `UiTreeNodes::pending_mutation_node_ids()` 同步；同 ID 以 `BTreeSet` 最小 node id 保留原 BTreeMap first-node 语义，dirty clear 前同步 metadata rename/remove。候选身份失配时保守重建，避免陈旧 node 返回；仍需执行 metadata change、duplicate insert、whole-tree replacement 与 dirty-clear 回归。
4. 为 `last_layout_root_size` 反序列化后首次不同尺寸补 round-trip 回归。当前源码已在 cache 为 `None` 且 arranged tree 非空时强制 root layout invalidation，不再存在先前复审所述 no-op；仍需用不同 root size 的行为测试锁住该条件。layout-engine report 的 selection count 也已由 `replace_selection_at()` 增量维护 backend、support、fallback reason 和 Taffy build aggregates，patch 热路不再调用 `recompute_counts()`；后续只需执行和规模验证，不重复实现。
5. 为 hit cell occupancy、runtime style visited rules、responsive candidates/slots、presentation region visits和 SVG/GPU upload 建立同一时间线；只有当前源码 profiling EXE 的 input-to-present p95、CPU/RSS/GPU 驻留同时达标后，才关闭按钮、resize、SVG 三个 gate。

### 15.12 本轮持久索引收口与构建阻塞

本轮在不删除其他会话产物的前提下继续关闭了两条确定性热路：

1. `UiLayoutSlotIndex` 从布局函数内的临时索引提升为 `UiSurface` 的 serde-skip 派生缓存。full layout 完成后刷新；incremental layout 只做长度/访问 edge 身份校验，正常按钮样式和局部几何更新不再扫描全部 slots。无 slot edge 使用负缓存，避免 measure/arrange 多 pass 重复扫描。
2. `UiSurfaceControlIndex` 让 template action 的 control property 引用从每次 `tree.nodes.iter().find_map` 收敛为 `O(log controls)` 查找，并利用现有 mutation-node 集合增量维护。该缓存和 layout slot cache 均实现自定义 `PartialEq`，不把 serde-skip 派生状态纳入 Surface 值身份。
3. pointer hover/press/focus 已确认使用 component-state store 和 render-only dirty；只有 runtime pseudo rule 的实际 style delta 才附加 text/style/layout dirty。`state_flags.dirty` 继续保留给 visibility、enabled、clickable、input policy 等会改变命中资格的状态，不能为性能无条件删除。
4. 最新受管 Windows 验证不是 Rust 编译失败。协调器在 admission 阶段因其他会话遗留未登记 D/E/F 产物拒绝任务，源码 crate 尚未执行。报告继续把 compile/test/profile gate 标为 open，不用 scoped diff check 冒充编译通过。

下一切片审查 hit grid 的 cell-local 算法：局部 geometry patch 目前虽然以 changed node 为外层单位，但 cell membership 变化时仍可能克隆全部 entries 并对受影响 cell 全排序。必须记录 `affected cells / entry visits / max occupancy / sort work`，并为跨 64px cell 移动、单 cell 10k 重叠和零半径命中补等价/无重复回归，才能判断是否需要从 Vec 重排升级为位置索引或分层 bucket。

### 15.13 SVG / GPU 图像缓存端到端复审

本轮沿编辑器真实产品链路重新检查了 `visual asset candidates -> SVG tree -> raster pixels -> chrome icon atlas -> UiSurfaceDrawList -> WGPU shared image registry`。用户看到的“SVG 反复加载”必须拆成四种成本，不能因某一层存在缓存就认定全链路完成：

1. SVG tree cache 已按稳定路径别名保存 `Arc<usvg::Tree>`，暖命中不再 `canonicalize/metadata/read/parse`；raster cache 也按逻辑资源、目标尺寸和 tint 保存最终 RGBA。窗口 resize 本身不会清空这两层缓存。
2. raster cache 查询原本发生在候选路径列表生成之后。暖帧虽不读文件、不 parse、不 raster，仍重复分配候选 `PathBuf`、别名与查询字符串。正确边界是先按逻辑资源键查询 raster cache，只有 miss 才构造候选并访问文件系统。
3. 编辑器 raster 与 icon atlas 已使用 `Arc<[u8]>`，但转换到 `UiSurfaceImageResource` 时调用 `to_vec()`，把整张独立图片或 atlas 页再次深拷贝。新 presenter、跨窗口或本地 image cache miss 时，即便 device-shared registry 随后命中，这次 CPU 复制已经发生。
4. device-shared registry 按 `(resource_key, generation)` 持有真实 WGPU texture，present 阶段会在上传前 resolve，命中时不会 `queue.write_texture`。不能把一次非原子的前置 `is_resident` 查询直接当作省略源像素的保证：查询后资源可能被另一窗口的 64 MiB LRU 淘汰。正确修复是让 draw-list resource 与 WGPU staging 共享 `Arc<[u8]>`，保留 present 阶段的原子 resolve/prepare。
5. icon atlas 只在新内容键出现时推进新 page generation；已发布 page immutable。若 hover/resize profile 中 generation 继续增长，必须按 `candidate builds / tree parse / raster / atlas generation / shared resolve / shared upload writes` 分层定位。

实施顺序：

1. 将候选列表改为 cache-miss 后调用的延迟生成器，以“第二次请求候选闭包调用数仍为 1”锁住暖缓存契约。
2. 将 `UiSurfaceImageResource` 与 WGPU image cache 的 CPU source 改为 `Arc<[u8]>`；owned/borrowed chrome stream 转换只克隆 Arc。首次真正需要上传时直接借用 slice，shared registry 命中时不产生整页 memcpy。
3. 保留 presenter-local resident 快速省略与 present 阶段 device-shared resolve 双保险；不增加有 LRU TOCTOU 竞态的全局 resident 承诺。
4. 当前源码 profiling EXE 验收要求：200 次 resize 与 1000 次 hover/click 后 SVG tree parse/raster/candidate-build 不增长，atlas generation 不增长，shared upload write 为 0；新窗口首次消费同一代图集只允许 shared resolve。单 SVG 内容变化只允许新内容页一次 upload。

当前状态：延迟候选生成已经落地，chrome image resource 到 WGPU canonical CPU source 已改为 `Arc<[u8]>` 共享；公共 inline command payload 仍保留 `Vec<u8>` 兼容，只在没有 resource-table 的 fallback 路径复制。受管 Windows `zr_rhi` build/test 于 2026-08-13 获准进入 Cargo，但协调器在编译中途回收了自己分配的 `D:\cargo-targets\zircon-engine\pool\f9fef...`：`serde_core`、`syn`、`zr_rhi` fingerprint/dep-info 均报 `os error 3`，link 随后缺失已编译 `.rlib`。日志未出现本次源码诊断，但验证结果仍计为环境失败而非通过，动态 gate 保持 open。

### 15.14 Pointer 命中语义修复与非布局输入局部更新

继续沿 `pointer event -> dirty domains -> arranged snapshot -> hit grid -> render extract` 复核后，确认按钮慢响应还有一条独立于伪状态样式的确定性全量路径：只要 dirty summary 含 `hit_test` 或 `input`，旧 `rebuild_dirty()` 就无条件重新构造全部 arranged nodes 和整个 hit grid；若同一帧还含 `render`，又因为 hit/input 标志存在而禁止 `patch_render_nodes()`，最终退化为完整 render extract。`UiInvalidationReason::Interaction` 因此同时触发三个全量阶段。即使 hover/press 主路径已改为 render-only，enabled/clickable/focusable/input-policy/pointer-events 等正常交互属性变化仍会命中该退化。

复核还发现必须先修复的 correctness 缺口：`UiTreeNode::supports_pointer()` 会检查 `UiPointerEvents`，但 `UiArrangedNode` 原来没有保存该字段。full hit build 只依据 arranged visibility/enabled/clickable/hoverable/focusable，因此 `UiPointerEvents::None` 和 `SelfNone` 在 arranged/hit 阶段可能失效。局部优化若沿用这一快照会把错误语义缓存得更久。

当前工作树的修复与算法边界：

1. `UiArrangedNode` 增加 serde-default 的 `pointer_events` 快照；自身命中同时检查 visibility 与 `allows_self_hit_test()`，祖先路径同时检查 visibility 与 `allows_child_hit_test()`。`SelfNone` 只排除父节点自身但保留可命中 child，`None` 排除整棵 pointer subtree；新增低层回归锁住两种语义。
2. 非布局 hit/input rebuild 先调用 `patch_arranged_tree_input()`。它只访问 tracked dirty nodes，并要求 tree id、roots、node/slot 数、node index、拓扑、frame、clip、z/paint order、visibility、clip ownership 和 slot summary 与已发布 snapshot 完全一致。任何结构或可见性变化都回退 full arranged build，避免局部路径发布不等价 canvas/render ancestry。
3. `input_policy` 与 `pointer_events` 会影响后代的 effective policy/path，因此该两项变化把受影响集合扩展为 dirty root 的实际后代；普通 enabled/clickable/hoverable/focusable/control-id 更新只访问目标节点。受影响节点按现有 arranged index 原位替换，复杂度与 changed subtree 同阶。
4. hit grid 复用已有 node/entry/cell 反向索引。资格保持不变时只更新受影响 entry、bubble route、policy 和所在 cells；资格新增/删除、零面积变化、越出既有 grid bounds、缺失索引或 cell 不合法时返回错误并完整重建 hit grid。优化不删除 correctness fallback。
5. arranged 局部 patch 成功后，即使该帧同时含 hit/input，render 仍允许对同一受影响集合执行 `patch_render_nodes()`。典型单按钮 Interaction 帧的阶段 outer visit 目标由 `N/N/N` 收敛为 `1/1/1`；父级 policy 变化为 `O(changed subtree)`；资格增删允许 hit 阶段回退 `O(pointer nodes)`，但 arranged/render 不再被迫同时全量。
6. 新增 dirty-domain 门：单按钮 Interaction 必须报告 arranged/hit/render 各访问 1 个节点；将目标 `pointer_events` 改为 `None` 时 arranged 仍访问 1 个节点，但 hit grid 必须 full fallback 并真实移除该 entry，未变化 sibling entry 必须保留。

验证状态：所有改动 Rust 文件已通过 `rustfmt`，相关 scoped `git diff --check` 仅有仓库既有 CRLF 提示。最窄 managed Windows `zircon_runtime` test 于 Cargo admission 前再次被 coordinator 以 `unmanaged_artifacts_detected` 拒绝，本次明确指向其他会话的 `D:\ZirconBuilds\mvp-test-fixtures-28608`；当前任务不拥有该目录，未删除。源码没有进入 rustc，因此上述用例仍是“已编写、待执行”，按钮 latency gate 继续保持 open。

后续动态验收必须同时记录 full fallback reason，不能只看局部计数：1000 次稳定 hover/click 中 Interaction 帧不应出现 full arranged/render build；enabled/pointer eligibility 切换允许 hit fallback，但结果必须与强制 full rebuild 等价；父级 `None -> Auto`/`Auto -> None` 在深树中必须按 subtree work 计数并验证所有后代命中结果。

### 15.15 Native pointer 热路径与 Pane 弹层命中索引复审

本轮继续从真实 `CursorMoved -> native host dispatch -> Runtime route -> interaction mutation -> damage -> present` 链路向下追踪。此前的 dirty-domain、arranged/hit/render 局部 patch 只解决了状态变化后的重建范围，没有覆盖状态变化前重复执行的空间查询、route 复制和 Pane 弹层预检。因而“局部 rebuild 已存在”仍不能推出按钮可交互；输入前半段如果每次 move 都扫描完整模板节点，同样会在进入 rebuild 前耗尽单帧预算。

当前工作树中的结构性改动与边界如下：

1. Surface 已发布的 hit path 现在可以直接作为可信 routing path 交给 Runtime dispatcher；公开 `dispatch()` 仍对外部 route 的候选祖先路径执行 defensive lookup，避免外部构造的 route 绕过 tree authority。无 handler 的默认 dispatcher 不再遍历 route，也不构造每节点 context；handler context 借用 event-lifetime route，dispatcher 结束后把同一 route 移入 result，不再保留与 route depth 同阶的 context/result ownership clone。
2. `UiHitPath` 只保存一条 root-to-leaf 规范序列；bubble/tunnel 分别反向/正向迭代，不再同时拥有两条反向 Vec。未 capture 且最终 target 等于 hit target 时，`UiPointerRoutingPath::HitPath` 直接复用该序列；capture/redirect 才拥有第二条 `ExplicitRootToLeaf`，因此物理命中与实际 dispatch 语义仍分离。稳定 hover 先借用比较，只有集合实际变化时才替换 owned state；serde 仍输出旧 `root_to_leaf`、`bubble_route` 与 `bubbled` 数组形状，但不把线形诊断副本保留在运行时 route 中。
3. Editor body move routing 将 Workbench template 命中从 popup 分支和普通分支的两次查询收敛为一次，查询结果按 `menu > workbench popup > pane > normal workbench > clear` 复用，未改变弹层优先级。`ui.pointer.workbench_hit_index_query_count` 直接记录产品每次 move 的空间查询次数，稳定 Workbench body 目标是 `<= 1`。
4. Pane template 的普通 control 查询已经使用 committed `UiSurfaceFrame.hit_grid`，但旧逻辑在每次 move 前仍逆序扫描全部 template nodes 查找 open popup；Console 的 scrolled/static 特殊分支也重复这项扫描。新 `HostPaneTemplateHitIndex` 在 Pane presentation 发布时一次性保存 open、enabled、具有 control identity 的 popup row；无 popup 的正常 Pane move 从 `O(total template nodes)` 收敛为 `O(1)`，有 popup 时为 `O(open popup rows)`。
5. Pane frame 与 popup index 由同一 `ModelRc` generation 同步构建。查询前必须验证 index 所属 node model 与 Pane 当前 model 同 identity；任何 mismatch 都保守回退旧全扫描，避免用性能优化发布陈旧 hit。现有无索引测试/调用方继续走兼容 fallback，不把缺索引解释成“没有 popup”。
6. `ui.pointer.pane_popup_index_query_count` 与 `ui.pointer.pane_popup_index_candidate_count` 分别记录预检次数和实际候选规模；10k template-node fixture 在无 popup 时要求一次 query、零 candidate visits，同时命中最后一个普通 control。该测试只证明算法边界，尚未获得当前源码 Rust 动态 green。
7. 再次核实原生 resize 事件循环：pending redraw 只在 empty -> pending 时请求一次 native redraw，后续请求合并；拖拽期间只配置最新 surface 并 present retained snapshot，80 ms 静止后才 commit 一次真实 reflow。这里没有“每个 resize event 都同步 rebuild 全 UI”的当前源码证据。剩余门是当前产品 EXE 中 snapshot build/reuse、surface reconfigure 和停止后单次 reflow 的实际 p95/p99，而不是继续盲目减少 `request_redraw()`。
8. 先前复审提出的三个 correctness 风险在 current source 已有防线：公开 `UiTreeNodes` 的 mutation 路径登记 pending node ids；反序列化后 `last_layout_root_size=None` 且已有 arranged tree 会强制 root invalidation；responsive slot 局部路径会核对 parent/child identity，并在同 cardinality detach/attach 后回退完整 resolution。它们仍需编译执行既有回归，但不应重复实现另一套 dirty/topology 机制。

与参考实现的裁决保持一致：Unreal `SlateInvalidationRoot`、`SlateInvalidationWidgetList/WidgetProxy` 和 `HittestGrid` 以稳定 widget index、依赖顺序 dirty queue、cached elements 和 cell-local reverse mapping为主；Slint 的 item tree / partial renderer 用 property dependency 与 damage 驱动重绘；Fyrox/Godot用于验证 editor/runtime ownership 和资源/控件 invalidation 边界。Zircon 的正确方向是让已发布 Surface/Panes generation 同时拥有 geometry、hit、render 与 popup/control 派生索引，而不是在每个输入事件重新解释 `Vec<TemplateNode>`，也不是把 popup/runtime authority下放到 Editor 特判。

当前复杂度和剩余成本应按以下口径验收：

- Runtime empty-handler route：普通命中仅拥有一条规范路径并直接 move 到 result；capture 同时拥有物理 hit path 与必要的 dispatch path。反向传播是零分配迭代，序列化时才按旧 wire contract 生成反向数组。
- Workbench move：每个 body event 最多一次 Workbench spatial query；popup 与 normal dispatch 复用同一 hit。
- Pane popup precheck：无 popup 为 `O(1)`；存在 popup 时与 open popup 数同阶；普通 control 继续由已发布 hit grid 局部查询。
- identity mismatch、无 committed index、popup model mutation：允许 `O(N)` 保守 fallback，并必须计数；产品稳定帧不允许反复命中该 fallback。
- 输入到 damage、damage 到 frame-update、frame-update 到 submit/present 必须位于同一 scenario timeline；只记录某个 helper 的微秒数不能证明按钮响应。

本轮静态验证状态：相关 Rust 文件已执行 `rustfmt`，scoped `git diff --check` 仅报告仓库 CRLF 提示。托管 Windows 测试三次均未形成 Zircon 行为结果：一次在 coordinator database lock 前退出；一次显式 `E:\ZirconBuilds` target 在依赖编译中途消失；一次 coordinator ephemeral target 同样在基础依赖编译中途被删除并报 `os error 3`。这些是构建基础设施失败，目标测试执行数为 0，不能作为 source green 或 UI red。

2026-08-31 canonical pointer-path 静态补充：结构合同与确定性模型 10/10 通过。模型绑定 HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784` 和当前 worktree；100 万事件、depth=100 时，普通命中的 owned path 从每事件 3 条降为 1 条，node identity 写入下界从 3 亿降为 1 亿，路径 Vec 分配下界从 300 万降为 100 万；有物理命中的 capture 从 3 条降为 2 条，写入下界从 3 亿降为 2 亿。artifact 为 `E:\zircon-profiles\runtime-ui-pointer-path-authority-pressure-20260831-r1.json`，SHA-256 `C485C250A5B5F4826FF12A9B417F86811FEBBA03AE43EEF21F5D9166ACAE47A9`，source manifest `93B2F73D10367F386D16C3893DDBD1436AB16A8DF122D6B9F5F75B3C33799489`。该模型不含 allocator metadata、CPU、RSS、cache locality 或产品输入时延；受管 Rust parity/counter 与产品 trace 仍为 open gate。

同轮继续追踪到 `UiInputDispatchResult` 后确认，route 层的单一路径仍会被完整诊断重新物化：pointer trace 同时构造 preview、bubble、focus、popup stack 和 route steps；`UiInputManager` 又从 `diagnostics.route_target`、`route_trace.capture_target` 与 `route_trace.bubble_path` 反推 press、capture 和 hovered state。因此现在不能直接清空或截断 trace，否则会改变输入行为；当前压力模型也明确没有把这些诊断分配计入 candidate 数字。更重要的是，capture 时 dispatch path 与 physical hit path 不同，hover/under-cursor 状态不应继续从 capture route 推导。Unreal 的 `FEventRouter` 借用同一 `FWidgetPath` 做 tunnel/bubble，而 capture/direct policy 仍把 routing path 与 widgets-under-cursor 分开保存；Zircon 后续必须保持相同所有权边界。

诊断收敛按以下依赖顺序实施，禁止先加一个重复 Vec 的“receipt”掩盖问题：

1. 在现有 `UiInputDispatchResult` 内增加 compact routing receipt，分别保存 route target、capture target、physical root-to-leaf hit path 与仅在 capture/redirect 时存在的 dispatch path；它是 manager、dynamic surface routing 和 Editor bridge 的行为权威，不新增并行 result 类型。
2. pointer dispatch 在消费 `UiPointerRoute` 时把既有 canonical path 移入 receipt；普通命中不复制，capture 最多保留 physical + dispatch 两条路径。active-pointer hover 使用 physical path，事件传播使用 dispatch path。
3. `UiInputManager`、dynamic multi-surface capture 和 Editor shell fallback 全部迁移到 receipt 后，`UiInputRouteTrace` 才降为诊断投影。默认只保留固定大小摘要和低基数字段；显式 full 模式必须同时限制 path nodes、steps、notes、popup rows 与总字节，并报告 dropped/truncated 计数。
4. full trace 从 receipt/dispatch observation 生成，只在显式采集时支付反向序列与字符串成本；shipping 默认路径不得构造 `route_steps`、pointer-source `String` 或 popup-id `Vec<String>`。旧 wire 字段只在 full capture/兼容序列化边界生成，不在每个输入事件常驻。
5. 下层回归先证明 diagnostics off/summary/full 三种模式的 reply、effects、capture、hover、focus 与 route target 完全等价，再执行 allocation counter 与产品 pointer p95；只看到 trace 为空不能作为性能或正确性通过。

2026-08-31 current-source candidate 已完成上述行为/诊断解耦的第一阶段，但尚未通过 managed Rust 或产品验收。`UiInputDispatchResult` 现在携带 serde-compatible `UiPointerRoutingReceipt`；普通命中把既有 physical root-to-leaf path 直接移入 receipt，并以 `HitPath` 视图复用为 dispatch path，只有 capture/redirect 才拥有第二条路径。`UiInputManager` 的 hovered/press/capture、dynamic multi-surface capture 和 Editor shell fallback 已不再读取 diagnostics；hover 明确消费 physical path，事件传播消费 dispatch path。产品 dynamic surface manager 以及 Editor drag/resize direct dispatch 选择 `Summary`，该路径在 receipt 发布后跳过 preview/bubble/focus/popup trace、route steps 和 pointer-source 字符串；兼容 API 与测试默认仍为 `Full`。

本阶段 source contract/ownership/pressure suites 为 59/59，另有 14 个相关 Rust 行为回归已写入 interface/runtime 下层，覆盖 receipt serde/legacy default、普通路径复用、summary/full reply/receipt parity、capture 下 physical hover 与 dispatch path 分离、窗口/原始鼠标/analog-navigation/drag-drop Summary/Full 字符串物化边界、drag payload shared authority与wire shape、Summary 的 O(1) 动态诊断预算判定、canonical analog control 借用，以及 adversarial path/popup/UTF-8/steps/notes budget；由于 Cargo lane 未获本切片授权，这些 Rust 用例尚未执行。Full pointer trace 在构造期限制为每条路径 128 nodes、256 steps、32 notes、16 popup rows、8 KiB combined string bytes；Full 与已有可变长 payload 的 Summary result 执行同一输出上限，并通过 serde-default `UiInputDiagnosticsTruncationReceipt` 报告 dropped nodes/steps/notes/popup/bytes，普通 pointer、raw mouse motion 与 analog/navigation/drag-drop Summary 在 route/steps/phase/notes 构造前退出，再以固定字段空检查跳过 finalizer。source-bound 压力 artifact 为 `E:\zircon-profiles\runtime-ui-pointer-diagnostics-summary-pressure-20260831-r4.json`，SHA-256 `664A1061BCE94AA6D217AB34F1BFBF9CFE4E1AEC0C53E66C7E1B46E9A44B02B8`。模型分别包含 100,000 个窗口归一化 pointer-move 与 100,000 个 raw mouse motion；前者 bubble/focus depth 各 12、popup depth 3。产品 Summary 避免旧 eager baseline 的 2,800,000 次诊断 identity copy、400,000 次诊断向量分配、300,000 次窗口泵诊断字符串分配和 100,000 次 raw-motion 诊断字符串分配，同时在两个事件路径合计保留 1,600,000 次固定字段 presence checks；显式 Full capture 单独估算为 4,000,000 次 identity copy / 500,000 次向量分配。该 artifact 是绑定当前 Zircon/Unreal source SHA 的确定性模型，不是 allocator、CPU 或产品时延证据。其它 non-pointer 分支仍可能在 finalizer 前短暂构造较大的诊断集合；managed behavior test、allocation counter 和产品 input p95 也仍为 open gate。

同轮 analog ownership 复核以 Unreal `FNavigationConfig::GetNavigationDirectionFromAnalog` 的 typed `FAnalogNavigationKey(FKey, direction)` 为参考：Zircon 不再为了读取 retained value 克隆整份 `UiAnalogInputEvent`/control `String`，而是复制一个 `f32` 后继续消费原事件；已经是 lowercase ASCII alnum 的 canonical control 通过 `Cow::Borrowed` 直接分类，只有带大小写或分隔符的兼容 alias 才生成 normalized owned string，repeat-state key 语义保持不变。analog 与其派生 navigation dispatch 复用产品 `Summary`，route trace、steps、handled phase 和 analog notes 只在 `Full` 构造；旧 owner helper 继续默认 Full，避免改变兼容调用。`E:\zircon-profiles\runtime-ui-analog-ownership-pressure-20260831-r2.json`（SHA-256 `D5B65F8951D3AECDD7BD59F26DF3A969191508988A32D25175020EC535ED03E7`）在 100,000 个 12-byte canonical control 事件上记录避免 200,000 次 transient String allocation 和至少 2,400,000 control bytes copy。该 artifact 同样只是 source-bound 下界；active navigation 的 repeat-state key formatting、map lookup、CPU/allocator timing和产品 gamepad latency明确未计入。

Drag-drop 侧以 Unreal `FDragDropEvent` 持有 `TSharedPtr<FDragDropOperation>`、Slate user在整段路由中保留同一operation为主参考：Zircon 在platform normalization边界把payload一次提升为`Arc<UiDragPayload>`，input event、reply effect、applied effect和retained drag state只共享不可变authority，serde仍输出原来的payload对象。dispatch不再克隆整份drag event/reply/applied effect；产品`Summary`在route-policy、steps、phase和notes投影前退出。事务层仅对同session、同pointer、同target的单个`Update`跳过全surface mutation snapshot；target transition、Begin/Accept/Drop/Complete、popup/dismiss和multi-effect仍保留atomic rollback。`E:\zircon-profiles\runtime-ui-drag-drop-ownership-pressure-20260831-r1.json`（SHA-256 `B467E04C9334EB1C7601B87D8BD6A70DD6926F3AD338970F13DA3A6F10A410ED`）在100,000 updates、100 target transitions、4 KiB/8-string payload、50,000 retained-entry units模型中把旧6次/event深clone下界记录为600,000次、4,800,000次字符串分配和2,457,600,000 payload bytes copy，candidate为0次深clone/300,000次`Arc`引用clone；全snapshot由100,000降为100，模型消除99,900次和4,995,000,000 retained-entry clone units。该artifact不含Arc原子指令、allocator/CPU/RSS、实际类型尺寸、hit/style成本或产品drag latency，不能替代managed Rust与Editor trace。

下一动态 gate 顺序固定为：

1. target 生命周期稳定后，批量执行 Runtime pointer/dirty-domain/arranged-hit-render 回归与 Editor Workbench/Pane hit-index 回归；先解决编译和最低层 correctness，再进入 profiling。
2. 构建与 source manifest 绑定的 profiling Editor EXE，执行至少 1000 次稳定 hover/click；记录 input-to-damage、damage-to-frame-update、frame-update-to-submit/present 的 p50/p95/p99/max，以及 Workbench query、Pane popup candidate、style visited、full fallback reason。
3. 执行 200 次 native resize；拖拽段 model/chrome/text/SVG rebuild 必须为 0，command snapshot 每 transaction build 1 次并产生 reuse，停止后真实 reflow 至多一次。
4. 执行 SVG 冷/暖、内容不变 watcher、单文件变更、双窗口共享和 64 MiB 压力；暖 hover/resize 的 candidate/tree parse/raster/atlas generation/shared upload 必须不增长，并同时报告 CPU/RSS/GPU resident bytes。
5. 只有当前 EXE 的产品 p95/p99、fallback 分布、CPU/RSS/GPU 驻留达到门槛后，才能把按钮、resize、SVG gate 从“源码实现/待验收”改为完成；此前不得提交“性能已修复”的里程碑结论。

### 15.16 SVG/GPU 缓存证据链与动态门槛修订

再次从计数产生点追到 `UiScenarioHotspot`、timeline JSON 和 capture gate 后，确认“代码里有缓存”与“产品交互实际命中缓存”之间原来仍断了一层：编辑器和 WGPU 已产生 raster、SVG tree、GPU upload/shared registry 等计数，但 Runtime hotspot 聚合器没有完整保存这些字段；部分 SVG/raster 计数只使用全局 `ui.visual_asset_cache.*` 名称，无法归属到当前 `idle_hover`、`click` 或 `window_resize` 场景。这样即使 resize 中发生重复解析/上传，旧 timeline 也可能看不见，不能作为缓存有效的证据。

本轮完成以下证据链修正：

1. `UiScenarioHotspot` 增加 serde-default 的 Workbench/Pane hit-index、visual asset cache、SVG tree cache、GPU image upload/shared registry/cache admission/resident 等字段；旧 trace 反序列化时新字段为 0，避免破坏历史产物读取。计数型字段求和，resident bytes 取场景峰值。
2. raster warm hit/miss/candidate-build 与 SVG tree memory hit/miss 现在同时写入当前 UI scenario；聚合器不再只看到脱离交互上下文的全局计数。新增聚合回归锁住 hit-index 与 GPU/视觉缓存字段不会再次在 export 前丢失。
3. `window_resize` gate 要求完整交互序列完成并恢复原尺寸，snapshot build 为 1 且有 reuse；拖拽期间 raster miss、SVG tree miss、cache admission reject、invalid image payload、视觉缓存 full invalidation 必须为 0，GPU upload write 与 cache allocation 各最多允许一次预热噪声。该上限只用于当前自动场景，最终 200 次暖 resize 的正式门槛仍是静态图片 upload 不增长。
4. 自动 `asset_refresh` fixture 修改的是 `.zmaterial`，不是已驻留 SVG。因此它的正确合同是“记录到资源变化且视觉缓存 full invalidation 为 0”，不能强制 targeted SVG invalidation 大于 0。targeted 计数仍被导出供真正的单 SVG 变化场景使用；单 SVG 动态用例后续必须先预热该资源，再修改内容并要求只失效对应 tree/raster/atlas generation。
5. 静态复核没有发现窗口 resize、hover 或普通 redraw 主动清空 SVG tree、raster、icon atlas 或 device-shared image registry。复核发现 watcher backlog 的 `Lagged` 分支原来会直接清空 SVG tree、raster 和 icon atlas，可能形成“全清 -> 重解析/光栅化/上传 -> 主线程更慢 -> watcher 更易积压”的反馈回路。当前实现改为对有界驻留源逐项计算内容指纹，只淘汰真实变化的 logical asset/tree；未缓存资源下次自然读取最新内容。明确 sprite-atlas source/product仍保守 full invalidation；无 locator Texture无法映射file-backed source，不再清空视觉缓存，详见15.53。`visual_asset/svg_tree_reconcile_source_visit_count` 与 reconciled invalidation count 用于 10k backlog 量化一次性扫描成本与实际淘汰规模。
6. 继续沿 `MUI module -> JS path extraction -> usvg tree -> raster variant -> icon atlas -> WGPU` 复核后，发现普通 `.svg` 已复用 `Arc<usvg::Tree>`，但 MUI 模块图标仍在每个尺寸/tint raster miss 上重新读取模块文件、提取 path 并构造 usvg tree。当前 MUI 路径已并入同一个按规范化源路径、有界 LRU、watcher/reconcile 可失效的 tree cache；尺寸和 tint 只影响上层 raster key，不再触发 DOM 重解析。回归要求同一模块源连续请求返回同一 `Arc` tree。source-bound profile manifest 同时纳入该入口。
7. WGPU 端复核确认纹理与 bind group 以 `(resource_key, generation)` 驻留：首次 prepare 才创建纹理/写入 GPU，后续 present 命中只更新 touch；device-scoped shared registry 允许多个 native presenter 复用同一纹理。SVG/普通图片的最终 key 来自尺寸与 RGBA 内容哈希，图标 atlas page 以稳定 page key + generation 发布。因此这轮新增修复针对真实存在的 CPU 解析漏口，不把已经存在的 GPU 驻留层误报成缺失；动态 gate 仍必须用 upload/cache-hit/resident counters 证明产品路径确实命中。

当前验证状态：profiling PowerShell 文件解析通过；当时 critical source manifest 为 26/26，且无重复、无缺失，profiling output contract 15/15、native resize contract 2/2 通过，scoped `git diff --check` 无空白错误，只有仓库行尾转换提示。后续 native submission、retained projection、image-resource owner 拆分与 retryable-present 调度已将 manifest 扩为 33 个路径，必须重新执行该合同后才能更新为新通过数字。相关 Rust 文件已执行 `rustfmt`。前一次 managed Windows `zircon_editor` focused test 已通过 admission 并进入 Cargo，但 coordinator 分配的 `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d` 在基础依赖编译期间被外部删除，Cargo 以 fingerprint/dep-info `os error 3` 和缺失 build-script `os error 2` 退出；没有编译到 Zircon crate。本轮 presenter focused test 的最新申请则在 admission 阶段由 coordinator 返回 `database is locked`，同样没有进入 Cargo。两次都属于托管构建基础设施故障，不计为源码通过或失败。因此 Rust 聚合/MUI tree/presenter 回归、Editor EXE 构建与产品 profile 仍未执行，缓存/按钮/resize 三个动态 gate 继续保持 open。

### 15.17 GPU 局部投影代次契约复审

继续从 Editor command stream 追到 WGPU compiled batch/vertex cache 后，确认此前把两个不同的对象误用了同一个 generation。Editor 的 region `ChromeCommandStream` 只包含 damage 内的命令，是补丁而不是完整 UI 投影；WGPU 的 versioned damage cache 则明确假设 draw list 仍携带完整投影，只借助 scissor 选择局部写入。旧 presenter 把累计 `slow_path_rebuild_count` 当作 producer generation，同一次慢路径计数之间的多次 hover/click patch 会共享 key，WGPU 因而可能复用第一次 patch 的 batch 和 vertex geometry，后续 damage 命令即使已经变化也不会重新编译。其结果不只是统计失真，而是悬停、按下或局部文本反馈可能停留在旧位置或旧内容。

当前修复遵循以下边界：

1. 日常 full 与 damage command stream 都改为 unversioned。它们继续使用 retained surface texture 做局部写入，但 compiled batch/vertex cache 不再把不完整 patch 当作完整投影复用；每个真实 patch 只编译该 patch 的可见 draw items。
2. native resize 的冻结 command snapshot 是完整且不可变的投影，因此仍可 versioned。presenter 只在新 resize transaction 构造快照时推进独立 generation；同一 transaction 后续尺寸只改变 target surface size，继续复用相同 generation、batch plan 和 vertex buffers。普通 present 结束 transaction 后清除快照，下一次 resize 必须获得新 generation。
3. `slow_path_rebuild_count` 只保留诊断含义，不再承担内容 revision。未来如果要让普通 full frame 也复用 compiled projection，必须由 command producer 提供能覆盖 geometry、style、text、clip、image identity 和 ordering 的完整内容 generation，并保证 versioned damage 输入仍是完整投影；不能重新借用布局/慢路径计数。
4. retained surface texture 到 swapchain 的整面 copy 暂不视为可直接删除的错误。交换链新获取的 texture 不保证保留上一帧内容，region patch 之后仍需要把完整 retained texture 提交到当前 swapchain image；只有引入平台明确支持的 preserved backbuffer 或独立合成策略，才可安全缩小这一步。
5. `UiScenarioHotspot` 现在保留 compiled draw items、batch plan build/cache hit、vertex buffer create/upload 和 retained cache copy bytes。采集输出必须把这些计数与 draw items、damage、frame latency 放在同一场景解释，不能只用 draw-call 数推断 CPU/GPU 缓存命中。

动态验收门更新为：

- 1000 次稳定 hover/click 的 live patch 不得出现 compiled plan cache hit；`gpu_compiled_draw_items` 应与实际 damage-visible items 同阶，不能随完整窗口命令数增长。出现 cache hit 说明 partial/full projection 契约再次混淆。
- 200 次 native resize 的每个 transaction 应只有一次 batch plan build和首次 vertex upload，随后产生 cache hit；command snapshot generation 在 transaction 内稳定、跨 transaction 改变。
- SVG 暖路径的 raster/tree miss 与 device image upload 必须为 0；batch/vertex 重编译与 image texture 重上传是两类不同指标，不能把 unversioned patch 的 vertex upload 误报为 SVG GPU texture cache 失效。
- 当前源码的 presenter/MUI/hotspot 回归、profiling EXE 与产品数据仍待托管构建成功后执行。本节记录的是已确认的契约错误、源码修复和验收方法，不代表卡顿 gate 已关闭。

### 15.18 Native resize 固定投影的 GPU 栅格缓存

继续沿 native resize snapshot 追到 WGPU surface submit 后，确认此前的优化只复用了 CPU command/batch/vertex/text 对象，却通过 `bypass_retained_surface_cache` 在每一个 resize present 上直接把完整 UI 重绘到新 swapchain texture。交换链每帧仍需 reconfigure/acquire，但让数百个 UI draw、glyph atlas prepare 和 image prepare 随鼠标缩放事件频率重复，正是“命令快照已复用但窗口仍卡”的结构性缺口。

当前算法改为固定投影纹理：

1. resize transaction 第一帧以独立 generation 构造完整 command snapshot，在 projection size 的 retained texture 上完整栅格化一次；只有该提交完成记录后，缓存才进入 `ResizeProjection(generation)` 状态。
2. 同 transaction 后续 target size 变化要求 generation 精确相同，只 reconfigure/acquire surface，并复制 `min(projection, target)` 的纹理交集。target 更小时自然裁剪；任一维更大时先把新 surface 清为 opaque black，再复制投影，禁止保留未初始化 swapchain 像素。
3. copy-only 帧跳过 image prepare、text prepare、compiled vertex buffer resolve 和所有 UI draw calls。batch plan key 仍做常数级 generation 命中，用于证明快照身份稳定；实际 draw call、vertex upload、SVG/GPU image upload 必须为 0，只允许 raw copy bytes 和必要的 target-clear pass。
4. retained cache 状态不再用一个 `initialized` 布尔值混淆普通 damage baseline 与 resize projection，而是互斥的 `Uninitialized / OrdinaryBaseline / ResizeProjection(generation)`。普通 damage 只接受 `OrdinaryBaseline`。
5. Editor 同样不把 resize snapshot present 标记成 ordinary surface cache ready。缩放事务后的第一个普通交互即使携带 damage hint，也必须构造完整 command stream并覆盖 current-size baseline；否则 projection size 与最终 target 不同且下一帧是局部按钮更新时，cache resize 会清空纹理后只绘 damage 区域，未损伤区域会错误变黑。
6. `zr_rhi_wgpu/src/ui_surface.rs` 已将 native acquire/submit/render-mode 协调拆到 `ui_surface/presentation.rs`；image cache policy 与具体 GPU texture/bind-group resource 也拆为 `image_cache.rs` 和 `image_cache/resource.rs`。root 保留 presenter lifecycle、公开入口和资源 lifetime；各生产 owner 均低于 800 行。profile source manifest 同步绑定 RHI contract、presentation、retained cache、batch/image/resource/shared registry 与 Editor retryable-present 调度，总数扩为 33。
7. retained cache 的 ready 状态只在 `queue.submit` 之后提交。surface acquire、render encode 或 readback copy encode 任一步失败时都不得把尚未提交的 texture 标记为可复用；后续 resize 必须继续走 full redraw，而不是复制未定义内容。
8. copy-only 帧不再用零值伪装 image cache 指标。presenter 通过增量维护的 CPU/GPU resident byte counters 和 shared registry atomic snapshot 以 `O(1)` 读取真实驻留量，同时将 draw-list stats 的 `surface_size` 修正为当前 target size，避免 resize profile 继续报告首次 projection 尺寸。
9. image cache 的 GPU resource owner 已拆入 `image_cache/resource.rs`，cache policy 增量维护 local GPU bytes 与 retained CPU source bytes；shared registry 同步维护 device-scoped resident byte snapshot。resize 高频采样不再为了读取驻留指标遍历全部本地/共享纹理 map。

新增静态/行为合同覆盖：generation 未就绪时 full redraw、就绪时 copy-only、copy-only 忽略 stale damage、扩大 target 的黑边像素与 intersection copy bytes、ready 状态必须晚于 queue submission，以及 resize 后首个普通 damaged present被强制提升为 full command stream。此前静态验证为 source manifest 32/32 且无缺失/重复、profiling output contract 15/15、native resize contract 2/2（合计 Pester 17/17）、PowerShell parser 5/5、`rustfmt --check` 与 scoped `git diff --check` 通过；retryable-present 调度进入清单后当前目标变为 33/33，并新增提交结果导出合同，需以下文最新复验数字为准。三次最新 managed Windows Rust 申请都未进入 Cargo：一次 coordinator database lock，两次 admission 分别检出当前任务不拥有的 `D:\ZirconBuilds\mvp-test-fixtures-24756` 与 `D:\cargo-targets\zircon-engine\ephemeral` 并返回 `unmanaged_artifacts_detected`；未删除这些外部产物，也不能把 admission failure 记为源码 green。

后续提交链切片已关闭该源码级 correctness 风险：WGPU `get_current_texture()` 返回 `Lost / Outdated / Timeout / Occluded` 时显式发布 `UiSurfacePresentOutcome::RetryableNoSubmit`，不再用零工作量成功 stats 推进 `presented_frame_count`；`Lost / Outdated` 同时 reconfigure surface。Editor 在 cache、paint diagnostics、first-present capture 与 latency completion 之前把该 outcome 转成 typed retry，由 event loop 以原 damage/full request 重新排队 present，且不触发 model frame update；validation/device 错误继续 fatal。源码与三层回归已写入，managed compile/behavior test 和产品 acquire-retry profile 仍待执行，因此这里关闭的是协议实现缺口，不是动态性能 gate。

动态 gate 尚未关闭：当前源码 EXE 必须证明 200 次 resize 每 transaction 仅一次 UI full raster/vertex upload，后续 `gpu_draw_calls=0`、`gpu_image_upload_writes=0`、`gpu_batch_plan_cache_hit_count>0`，同时给出 resize present CPU/GPU p50/p95/p99、working set/private bytes 峰值；现阶段仍不能宣称窗口缩放已达到目标。

### 15.19 当前 UI owner 审计同步与静态契约复验

实现后的全链静态复验发现，Runtime 09 架构审计仍锁定旧目录快照：UI 根 owner 19、surface owner 23、全树精确小写 `legacy` 54。当前源码已经增加 `editable_text_composition.rs`，并将 retained control index、focus 子 owner 与 invalidation transaction 分离为 `control_index.rs`、`focus/`、`invalidation.rs`；把生产模块回退以迎合旧计数会破坏本轮性能架构。因此审计脚本、Rust mirror guard、当前 UI 架构文档及四份 current-source mirror 已同步为 UI 20、surface 26、全树 `legacy` 70，且 70 个命中全部在测试/fixture，生产命中仍为 0。历史日期段中的旧快照保留，不重写为当前事实。

复验结果：相关 17 项 Python UI 合同全部通过，其中覆盖 Workbench O(1) hit index、单 generation present、paint-selected transient hover、asset watcher bounded generation 与 Runtime 09 owner map；结构审计直接返回 `missing_doc_anchors=[]`、`risks=[]`，`legacy_production_hits=0`、`taffy_production_hits=175`。这关闭了架构守卫漂移，不关闭 Rust/EXE 动态 gate。下一步仍必须经受管 Windows validator 编译 `zr_rhi_wgpu` focused native submission tests，再构建当前 Editor EXE，并以真实按钮、连续鼠标和 200 次 resize profile 证明提交与缓存计数。

### 15.20 `zr_rhi_wgpu` 首次当前源码编译与共享验证阻塞

2026-08-14 受管 Windows validator 首次成功越过 admission 并实际编译当前 `zr_rhi_wgpu --lib`。第一轮 rustc 暴露 10 个具体错误：owner 拆分后缺失 `WgpuUiExternalImage`、`WgpuUiImageResourceStats`、`UiSurfaceImageResource` 与 `UI_IMAGE_TEXTURE_FORMAT` import；geometry 测试缺本地 `quad` helper，`SolidVertex` 无值比较语义，且 `Vec<&[SolidVertex]>` 使用了错误的双引用 `flatten`。修复采用显式 owner import、测试本地构造器、`PartialEq` 顶点值语义和 `flat_map(|triangle| triangle.iter())`，没有修改生产几何、缓存或渲染语义。第二轮 rustc 成功完成并产出 209-test binary，证明本轮生产 `zr_rhi_wgpu` 源码可以编译；只剩共享 interface 与既有 RHI 测试的 unused/dead-code warning。

宽过滤随后以 exit 101 退出。直接执行同一受管 binary 的 `--list` 成功枚举 209 tests，按 `native_submission::` 执行得到 8 passed / 1 failed；失败不是 WGPU 设备或缓存行为，而是 source contract 用同名外层 presenter wrapper 切片，未稳定锚定 `impl WgpuUiSurfaceRenderer`。合同已改为先选 renderer impl，再验证 shared-context constructor 包含 `create_surface(&context.instance, target)` 且不含 `request_device`；等价源码切片自检为 `create_surface=true / request_device=false`。修改后的 binary 重编尚未完成：后续 admission 先被其他 session 的 CPU reservation 阻塞，再被持续生成的 `D:\ZirconBuilds\mvp-test-fixtures-*` 标记为 `unmanaged_artifacts_detected`；上一受管 pool target 也在测试后被外部清理，不能借旧 binary 补跑。当前准确状态是“生产 crate 已通过 rustc；最新测试合同待重编，不能宣称 9/9”。

非 Cargo gate 已在修复后完整复验：profiling/source-manifest Pester 17/17，PowerShell parser 5/5，UI Python contracts 17/17，scoped `git diff --check` 通过且仅有 CRLF 提示。cross-session 查询进一步确认 fixture 由 `coordinator01-benchmark-identity-review-maintenance-20260811` 的 coordinator regression 生成；当前任务未删除、终止或接管外部任务。Runtime 09 session 注册又因既有 stale primary `codex-runtime-ui-m3-pointer-closeout-r4-20260808` 返回 `plan_wip_limit_reached`，因此协调事实只保留在 coordinator 和本报告中，不创建第二套 session note。Editor profiling build、当前 EXE 和产品 profile 仍保持 open。

### 15.21 保留式提交链自审与 2026-08-14 动态门状态

本轮继续逐段复核 `CompiledUiBatchPlanCache -> text/image prepare -> retained cache -> swapchain copy -> Editor baseline commit`，结论如下：

1. 稳定 `(generation, projection_size)` 的完整投影会复用批处理拓扑和不可变几何；damage present 复用同一完整投影计划并只用 scissor 裁剪。无 generation 的 region-only command stream 不进入完整投影缓存，避免把局部命令错误发布成后续全屏基线。该行为是 correctness 边界，不是遗漏缓存。
2. native resize 首帧把冻结投影完整 raster 到 projection texture；同 generation 的后续 target-size 变化只 reconfigure/acquire surface，必要时清黑扩展区，再复制缓存与目标的交集。copy-only 模式不 resolve draw buffers、不 prepare text、不 prepare image，也不发出 UI draw call。resize projection 与 ordinary damage baseline 使用互斥状态，Editor 离开 resize 后强制下一次普通 present 重新建立全屏基线。
3. retained texture 只在 surface capability 含 `COPY_DST` 时启用；否则走直接 render 路径，不会向不支持复制的 swapchain texture 发出非法 copy。缓存状态只在 `queue.submit` 后提交，避免 encoder 构建失败提前发布 ready。
4. SVG tree 暖命中先查内存 alias；候选路径、`canonicalize`、metadata、文件读取和 parse 只发生在 miss 或 watcher/reconcile。raster 暖命中发生在候选闭包之前。GPU image registry 按同一 device 的 `(resource_key, generation)` 共享 texture；presenter-local 层只创建 bind group，不重复 `write_texture`。两层都受 entry/byte budget 约束，统计 resident bytes 使用增量计数而非每帧全表求和。
5. 提交结果协议已在源码中显式区分 `Submitted / RetryableNoSubmit / Fatal`。只有 Submitted 推进 frame/baseline；RetryableNoSubmit 不替换 `last_present_stats()`，Editor 不提交 cache/diagnostics，event loop 保留同一场景与 damage 重新 present，并记录 `ui.surface.retryable_no_submit_count`。这项改动已覆盖 `zr_rhi` 默认 outcome、WGPU 四类 retryable acquisition/frame count、Editor baseline 与 event-loop no-frame-update retry 回归；managed Rust gate 未恢复前仍不能声明行为测试通过。

2026-08-14 最新 managed focused test 仍未进入 Cargo。`validate-matrix.ps1 -Package zr_rhi_wgpu -LibTests -TestFilter native_submission:: -SkipBuild` 在 admission 阶段被 coordinator 以 `unmanaged_artifacts_detected` 拒绝，当前列出的阻断根为 `D:\cargo-targets\zircon-engine\ephemeral`；它来自其他 coordinator 任务，当前会话不拥有且不得删除。此前生产 `zr_rhi_wgpu` 已在同一 managed 通道通过 rustc，但该编译早于本节新增的 RHI/WGPU/Editor 提交结果生产协议；当前生产源码与后续扩展到 11 项的 native submission 合同都待重编执行，不能沿用旧的 compile green 结论。

构建产物位置也存在明确治理冲突：本机只有 C/D/E/F 四个本地盘；`tools/build-editor.ps1` 和 validator 要求 `ArtifactOutputDirectory` 位于 coordinator 管理的 D/E/F 之外，而本里程碑又禁止把产物放到 C。可行的动态路径是让 validator 在 coordinator 管理的 D 盘 Cargo target 内生成 profiling EXE，并直接从受管 target 运行，profile 数据写入合同允许的 `E:\zircon-profiles`；最终独立 bundle 发布在增加第五个本地盘、放宽 artifact publish 规则或用户允许 C 盘之前保持 blocked。

下一步严格按证据顺序执行：

1. coordinator admission 恢复后，先执行 `native_submission::`、retained pixel parity、batch cache、image residency 和 geometry focused tests；任何低层失败先修低层，不启动产品 profile。
2. managed profiling build 生成当前源码绑定的 Editor EXE，直接使用受管 target，禁止手工把 Cargo target 复制到 D/E/F 非登记目录。
3. 执行按钮 hover/click、连续 resize 和 SVG 冷/暖三组产品 profile，记录 input-to-present p50/p95/max、layout/arranged/hit/render rebuild/patch、batch-plan/vertex/text/image cache、GPU time、RSS/private bytes 和 image resident bytes。
4. 产品 profile 必须导出 `ui.surface.retryable_no_submit_count`，并证明该值增长时 submitted frame、baseline、first-present 和 damage-to-submit 只在最终 Submitted 时推进；不能把 retry 尝试记成低延迟成功。

同日阻断后的非 Cargo 复验结果：UI hit/presentation/watcher/architecture Python contracts 17/17，profile capture 与 native resize Pester contracts 19/19，5 个相关 PowerShell 文件 parser 5/5，33 个 profile critical source 路径存在且无重复，相关 Rust owner 文件 `rustfmt --check` 通过，scoped `git diff --check` 无 whitespace error（只有仓库既有 LF/CRLF 转换提示）。这些结果只关闭结构和采集合同，不替代上面的 managed Rust 与产品 profile gate。

### 15.22 Surface acquire 早置与零准备重试

继续复核 native submission 的实际执行顺序时发现，初版 outcome 协议虽然阻止了 baseline/frame-count 误提交，但 `get_current_texture()` 仍位于 compiled batch resolve、image prepare 与 text prepare 之后。窗口缩放或合成器压力下，一次 `Lost / Outdated / Timeout / Occluded` 会先完成绝大多数 CPU/GPU 准备，再返回 retry；event loop 重排后又执行同一套准备。SVG raster/GPU texture 暖缓存能避免重复解码和上传，却无法消除 batch traversal、image residency touch、text prepare 与可能的 buffer work，因此该顺序仍会放大按钮和 resize 卡顿。

实现已调整为 `resize/configure -> acquire -> acquired present_index -> retained-cache mode -> compiled batch -> image/text prepare -> encode/submit`。retryable acquire 现在在 generation、retained texture resize、batch、image、text、vertex buffer 和 encoder 之前返回一个零工作量 `WgpuUiSurfacePresentation`；`Lost / Outdated` 只执行必要的 surface reconfigure。成功 acquire 后才推进内部 `present_index`，使 image LRU 与 GPU timestamp generation 不受 retryable 尝试污染。新增 `native_submission::wgpu_ui_surface_acquires_before_advancing_or_preparing_the_frame` 锁定 acquire 必须早于 generation/batch/image/text；源码索引自检为 acquire 206、generation 343、batch 1425、image 2137、text 2694。

同一复审还发现一个 post-submit 状态分叉：GPU timing `begin_map` 发生在 `queue.submit`、retained-cache commit 之后，旧代码在 map 失败时仍返回 fatal RHI error，尽管 UI 帧已经提交且随后会 `present()`。实现现将该失败降级为 abort 当前 readback sample 并继续返回 Submitted；提交前的 readback-copy encode 错误仍保持 fatal。新增第 11 项 native-submission 合同锁定 queue submission 后可以 abort timing frame，但不得出现 `return Err`。

产品采集同时新增 `ui.surface.submitted_count`、`ui.surface.retryable_no_submit_count` 与 `ui_surface_present_outcomes.json`，文件显式汇总 submitted、retryable-no-submit 和 damage-to-submit 样本数。profile source manifest 已把 `window/event_loop/redraw/present.rs` 纳入第 33 个关键路径；profile output contract 17/17（含构造 trace 的 outcome 汇总行为测试）、native resize contract 2/2、Python UI contracts 17/17、PowerShell parser 5/5、source manifest 33/33、`rustfmt --check` 和 scoped `git diff --check` 均通过。

当前 managed Rust 仍无新结果：最低层 `zr_rhi` outcome focused test 再次在 admission 阶段被 `unmanaged_artifacts_detected` 拒绝，这次阻断是其他任务持续生成的 7 个 `D:\ZirconBuilds\mvp-test-fixtures-*` 目录；未进入 rustc，当前任务未删除或接管这些目录。因此 `zr_rhi -> zr_rhi_wgpu -> zircon_editor` 行为测试、当前 11 项 native-submission 合同、profiling EXE 与产品 profile 继续保持 open。

### 15.23 Retryable surface present 的事件循环退避

继续从 outcome 分支追到 Winit 调度后，确认 outcome 和 acquire 早置仍没有完全关闭卡顿：`redraw_requested_impl` 在 present 前先取走 `pending_redraw`，旧 retry 分支再把同一请求放回普通队列并立即调用 `window.request_redraw()`。因此普通 redraw 的 empty-to-pending 合并无法限流；合成器连续返回 `Timeout / Occluded / Lost / Outdated` 时，事件线程会形成 acquire-retry 忙循环。即使每次 retry 已经是零 batch/image/text 准备，循环本身仍会占用 UI 线程并与鼠标、窗口消息竞争。

当前事件循环增加独立的 deferred surface-present slot、deadline 与 attempt 状态。第一次失败延迟 8 ms，连续失败依次为 16/32/64/128 ms，之后封顶 250 ms；deadline 进入现有 `ControlFlow::WaitUntil` 最早唤醒计算，到期后才合并回普通 redraw 队列并请求一次 native redraw。真实 pointer、resize、background/external redraw 不受该 deadline 阻塞：它们仍立即进入普通队列，下一次 `RedrawRequested` 会把 deferred retry 与真实请求合并，保持 full/region damage 与 frame-update 升级语义，并清除旧 deadline。只有实际 Submitted 才把 attempt 归零。

这项状态分离同时解决两个相反风险：retry 不再自旋；retry 也不会占住普通 pending slot，使后续按钮或 resize 请求因为“队列非空”而失去 native redraw 通知。每次失败还记录实际选择的 `ui.surface.retry_backoff_ms`；`ui_surface_present_outcomes.json` schema 2 导出 backoff sample count/min/max/average。构造 trace 行为测试得到 2 个样本、min 8 ms、max 16 ms。新增 `test_editor_surface_present_retry_performance_contract.py` 3/3 锁定 retry 分支不得直接 `queue_redraw/request_redraw`、状态必须有界并参与 wait policy、真实 redraw 必须消费 deferred retry；既有 UI 核心 Python 合同仍为 17/17，profile capture 17/17、native resize Pester 2/2，相关 Rust 单元测试覆盖具体 delay、到期前不可见、damage 保留、真实 redraw 合并与 submit reset，`rustfmt --check` 和 scoped `git diff --check` 通过。

动态 gate 仍未关闭。后续受管 Cargo admission 再次检出其他任务产物，最新错误明示的阻断已收缩为 `D:\ZirconBuilds\mvp-test-fixtures-6212`、`-6592`、`-860` 三个目录；仍未进入 rustc，本任务没有删除这些目录。恢复后必须先执行 Editor event-loop retry tests，再在产品 profile 中故意制造遮挡/缩放 surface retry，验证 retry 间隔有界、UI 线程不再持续满载、真实按钮输入仍可触发立即尝试，并确认最终 Submitted 才完成 damage-to-submit。

### 15.24 重复 native resize 的三层去重

继续复核 resize 事件到 `surface.configure` 的链路时发现，同尺寸 `SurfaceResized` 仍会穿透三层：事件层重置 80 ms reflow debounce 并排队 snapshot present；Editor GPU presenter 调用 RHI resize 且清除 ordinary damage baseline；WGPU native renderer 再次执行 `surface.configure`。Winit 的正常 redraw 合并只能合并尚未消费的请求，不能阻止跨多个 event-loop turn 的重复尺寸通知，因此该链会放大窗口拖拽抖动和平台重复事件。

当前实现建立三层同值 no-op。事件层比较 retained physical size，并同时保留 translated metrics 中真实 scale-factor 变化；物理尺寸与 scale 都未变化时在任何 scale/size 状态写入之前直接计数 `ui.window_resize.duplicate_size_suppressed_count` 并返回。独立的 `ScaleFactorChanged` 也以 f32 bit identity 去重并记录 `duplicate_scale_suppressed_count`，不会反复推迟最终 reflow。真实 scale-only 变化只安排 retained reflow，不再把同一物理尺寸塞给 presenter，因此不会把 GPU no-op 误计为 `surface_reconfigure_count`。Editor GPU presenter 在 clamped extent 等于 current size 时保留 surface 和 damage cache。WGPU `UiSurfacePresenter::resize` 在 clamped extent 相同时不进入 native renderer；surface acquisition 的 `Lost/Outdated` 分支仍直接强制 configure，不受该优化影响。

新增 `test_editor_native_window_resize_performance_contract.py` 4/4 锁定 size/scale 事件必须在 mutation/reflow 前去重、Editor 必须在 RHI resize/cache invalidation 前返回、WGPU 必须在 native resize 前返回；Editor Rust 回归额外检查 same-size resize 的底层调用数为 0 且 baseline 保持 ready。GPU lifecycle 已加入 source-bound profile manifest，关键路径总数由 33 增为 34。

修复后的非 Cargo 门已完整复验：resize 去重合同 4/4、surface retry 退避合同 3/3、核心 UI hit/presentation/hover/watcher/architecture 合同 17/17、profile capture 合同 17/17、native resize 合同 2/2、PowerShell parser 5/5；source-bound manifest 为 34/34，且 missing 0、duplicate 0。相关 Rust owner 的 `rustfmt --check` 与 scoped `git diff --check` 通过。该证据关闭源码结构与采集协议，不替代仍被外部 fixture admission 阻断的 managed Rust test、当前 Editor EXE 和产品交互 profile。

### 15.25 当前 RHI/WGPU 动态验证与 Editor 支持层阻断

coordinator 回收外部 `mvp-test-fixtures-*` 后，受管 Windows Cargo lane 首次完整验证了本轮提交结果与保留式 GPU 路径。`zr_rhi` 当前源码重新编译，默认 present outcome focused test 通过。`zr_rhi_wgpu` 首轮 rustc 暴露 retry presentation helper 仍为子模块私有、父级测试不可见；helper 只提升为 `pub(super)` 并显式 test import 后，生产和测试 crate 均重新编译。

11 项 native-submission 合同随后先得到 10/11：唯一失败是旧合同仍要求提交后的 readback map 错误继续传播，与新协议“已提交帧必须继续 present，只 abort timing sample”相冲突。合同已改为锁定 `queue.submit -> retained cache commit -> surface.present -> success`；受管复验为 11/11。扩展执行当前 managed test binary 的全部 `ui_surface::` 合同时，首轮 110/111；唯一失败发生在离屏 full-baseline pixel test 的 draw-call 断言，两个互不重叠的 solid quad 已由实例化批处理合并为一次 draw，而测试仍期望两次。断言改为 `visible_draw_item_count=2 / draw_calls=1` 后，受管 focused pixel test 通过，随后同一当前二进制 `ui_surface::` 111/111 通过；逐像素检查证明 damage 外区域保持绿色，没有被首次 baseline 清黑。该组同时覆盖 batch/vertex cache、retained damage/copy、target-only resize、image residency、shared GPU image registry、text cache、geometry 与 retry outcome。

上层 `zircon_editor` focused same-size resize test 随后启动完整当前源码编译，但未到达 Editor test binary：共享 `zircon_runtime` 在 mesh geometry seed、transient texture identity、ECS cached query、font path closure、resource streamer lifetime 等多个其他 owner 中累计约 40 个编译错误。本轮 UI owner 中同时暴露一处 `UiLayoutSlotIndex` fallback closure 会移动 `predicate`，已改为按引用调用并通过 Rust 2021 `rustfmt --check`；其余跨子系统错误不在本任务所有权内，未擅自修改。按共享 coordinator recovery 通知，当前 Job 正常结束后暂停新的 Cargo，等待 Tooling rollover 与 UI12 reservation。准确动态状态是：RHI/WGPU 低层 green；Editor presenter/event-loop Rust tests、profiling EXE、按钮/鼠标/resize/SVG 产品 profile 仍 open。

### 15.26 CPU/RSS/GPU 产品证据参数化与最新编译阻断

UI12 随后的受管检查从保留的 `zircon_runtime` fingerprint 中恢复了 37 条真实 current-source error 和一条 abort summary，并已按 Runtime、WGPU/Render、PBR/Mesh-SDF、Text 的既有 owner 路由。本轮已经修复的 `slot.rs:102:62` E0507 不在该集合中；同样消失的 resource-streamer E0716 不再重复建立 Failure。由于当前源码仍不能产出 `profiling/zircon_editor.exe`，不能把旧二进制或低层 WGPU test binary 冒充产品 profile，Editor 动态 gate 继续保持 open。

采集工具原先只记录 pointer/resize 的 wall time、working set 和 private bytes，CPU 只能依赖可选 WPR ETL；点击也只固定执行少量样本，native resize 次数固定为 24，无法直接执行本计划的 1000 次点击/指针和 200 次缩放压力门。本轮将证据合同补齐为：

1. `ui_interaction_evidence.json` 对 click storm、pointer storm 和 native resize 同时记录 start/end processor time、processor-time delta、`cpu_core_utilization_percent`（100% 表示一个逻辑核）和按逻辑处理器数归一化的 `cpu_system_utilization_percent`，并继续记录 start/end/peak working set 与 private bytes。
2. `-AutoClickCount`、`-AutoClickDelayMs`、`-AutoResizeStepCount` 与 `-AutoResizeDelayMs` 从命令行传到对应 native 执行器，并写入 source-bound manifest；resize step count 保持 2–240 的有界范围，1000 次点击和 200 次缩放无需修改脚本源码。
3. GPU draw/upload/shared-image residency、SVG tree/raster/atlas generation、damage-to-submit、retry outcome 继续来自同一 session 的 `timeline.zrtrace.json` 与 `ui_hotspots.json`，CPU/RSS 证据不另建无法关联源码和二进制的旁路采样。
4. 交互压力执行逻辑留在独立的 `ui-profile-native-resize.ps1`，避免继续扩大接近 2000 行的 profile 编排脚本；行为测试明确证明 999/1000 次点击会拒绝证据，而 1000/1000 且包含 CPU 字段才通过。
5. Pester 先以缺失 helper/参数的 3 条失败证明合同有效；实现后 `ui-profile-capture-output-contract.Tests.ps1` 19/19、`ui-profile-native-resize.Tests.ps1` 3/3，共 22/22 通过。

共享 Runtime 错误归零并生成当前源码 profiling binary 后，产品验收命令固定为：

```powershell
$env:CARGO_TARGET_DIR = '<coordinator-managed-target>'
.\tools\ui-profile-capture.ps1 `
  -ScenarioList material_lab_click,idle_hover,window_resize `
  -SkipBuild -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 45 `
  -AutoClickCount 1000 -AutoClickDelayMs 4 `
  -AutoPointerMoveCount 1000 -AutoPointerMoveDelayMs 2 `
  -AutoResizeStepCount 200 -AutoResizeDelayMs 16
```

验收必须同时读取 source manifest、interaction evidence、timeline 和 hotspot 报告：click/pointer/resize 的 processor-time、wall-time 与 RSS/private 增量可量化 CPU/内存成本；GPU image upload、SVG parse/raster/candidate build 和 atlas generation 在暖交互阶段不得增长；retryable no-submit 不得计入成功 frame 或提交延迟。任何一项缺证据都不能关闭“按钮/鼠标/窗体缩放不可交互”结论。

### 15.27 当前源码全链路算法复审与 profile-first 裁决

2026-08-14 在不启动新 Cargo 的前提下重新读取 Editor host transaction、Runtime invalidation/layout/hit/render、文本缓存、SVG/raster/icon atlas、WGPU damage/text/image 以及 Unreal/Slint 对应实现。复审纠正了早期报告中“Editor 只有一个全局 dirty bit”“GPU damage 只是统计”“SVG/GPU 没有缓存”的过时判断。当前事实、复杂度和剩余风险如下：

| 阶段 | 当前源码权威与算法 | 稳态/局部复杂度 | 尚未关闭的产品风险 |
| --- | --- | --- | --- |
| Editor invalidation | `HostInvalidationTransaction` 按 `All/View/ShellContent` scope 合并 reason；纯 view 或单 shell scope 有 fast target | `O(scope count log scope count)` 合并；命中 fast path 后与目标 view/pane 数同阶 | 多 shell scope、混合 reason、legacy flags 会扩大到 full；没有 node/resource/window changed set |
| Runtime invalidation | `UiInvalidationTransaction` 按 `UiNodeId` 合并 typed dirty/reason，commit 发布 changed nodes 和 domain generations | `O(k log k)` | Editor host 尚未把它作为唯一产品 authority，跨层粒度可能丢失 |
| Layout | propagated roots、ancestor collapse、`UiLayoutSlotIndex`、responsive candidates、局部 measure/arrange | 约 `O(k*h + V log V)`；root resize 一次 `O(N)` | `BTreeSet/BTreeMap` 常数、重复 transaction layout、fallback 分布尚未由 current EXE 量化 |
| Hit/popup | cell grid + reverse index；frame publication 时构建 projected popup grid；frame/instance 共用 authority | 普通 query 只访问命中 cell；patch 为 `O(k + P + affected cells)`，`P` 为已投影 popup entries | 需动态证明 rebuild 在输入前完成、stable event 不触发 fallback；popup correctness tests 尚待 Cargo |
| Render command | per-node buckets、固定 command ranges、geometry patch；full extract 仅在显式 fallback | `O(k + changed command ranges)` | 上游 full presentation 若推进 generation，仍可能使局部 cache 不可达 |
| GPU damage | Editor 提交一个 old/new bounds union damage rect；WGPU retained mode 使用 `Load + scissor` 并剔除不相交 draw work | 与 scissor 内 command/batch 同阶 | 单 union rect 可能把远离区域合并成大 overdraw；先测 damage/surface area，再决定是否升级 bounded multi-rect |
| Runtime text CPU | 4096 项 measure cache、2048 项 layout cache、frame dedup、shape-run cache、16 文档/32 MiB retained plain-text cache、viewport partial layout、8 项 chunk 并行 prewarm | 暖 lookup 与 hash bucket candidates 同阶；shape 只在 miss | resize/generation 稳定性和缓存 eviction work 尚需 profile；长文本 partial resolve 是有意 uncached geometry |
| Editor text CPU | `PaintTextLayout` 以文本、rect、font generation、smoothing、wrap 为 key，容量 2048 | 暖 lookup 均摊 `O(1)` | 满容量时整表 `clear()`，可能形成周期性尖峰；必须用 RSS/eviction 时间线证明是否值得改为增量 LRU |
| WGPU text | glyphon atlas/swash cache；整批 renderer 以 `(draw generation, projection size)` 复用，target-only resize 保留 projection key | stable generation 直接返回 cached batches | projection/generation 过宽时会重建所有 text buffers/renderer；需以 `gpu_text_shapes/builds/cache_hits` 验证 |
| SVG/raster/atlas | 1024 项 SVG tree cache；4096 项/64 MiB raster cache；正负结果、alias/reverse index、targeted invalidation；immutable icon-atlas pages，最多 64 页/64 MiB | 暖 raster hit 不构造 candidates、不读文件；暖 SVG alias hit 不 canonicalize/stat/parse | 冷 miss 仍同步；容量淘汰扫描只应出现在冷插入；需 current EXE 证明 watcher 不错误扩大 invalidation |
| GPU image | presenter-local `(resource_key,generation)` bind/cache + device-shared registry；两层均 256 项/64 MiB；shared hit 不 `queue.write_texture` | stable local hit 不取 global lock、不上传 | 多窗口/64 MiB eviction 后可能合法重新上传；必须报告 shared resolve/upload/resident bytes 和 RSS |

参考实现给出的不是“每帧完全没有线性工作”，而是线性工作必须在明确的 publication/rebuild 阶段发生，并由稳定 identity 和 changed set 限定：

1. Unreal `FSlateInvalidationRoot` 用稳定 widget proxy、phase heap/queue 和 ancestor/range coverage 生成最终 update list；Zircon 当前 Runtime 已有 changed-node transaction，但 Editor host scope 仍不足以表达同等局部性。
2. Unreal `FHittestGrid` 在 paint publication 时记录 paint-space geometry、render bounds、cell range 和 sort key，事件时只查询 cell；这直接支持 Zircon projected popup grid 必须在 frame publication 时生成，而不是每次鼠标事件解释 arranged/render 数据。
3. Unreal vector cache 以 brush identity 和 pixel size 缓存 SVG/raster，miss 进入 pending requests 并并行 raster；Zircon 的稳定 source/content/size key 与 atlas 分层一致，差异是冷 miss 仍同步，是否异步化必须由冷启动 profile 决定。
4. Slint partial renderer保存 per-item cached geometry/property tracker，以旧/新 bounds 形成 dirty region，最多保留 3 个矩形并按最小额外面积合并；这为 Zircon 单 union rect 的后续升级提供候选算法，但不是无数据情况下立即替换的理由。

本轮 Runtime popup repair 的静态验收已更新：projected index 在 base hit-grid full rebuild 时强制 rebuild；input policy/pointer-events 的实际 descendant affected set 会同步到 projected cache；局部 `UiProjectedHitTestIndex::patch` 只遍历 `changed_node_ids ∪ projected_node_ids`，不再执行 `base_index.grid.entries.iter().any(...)` 全表检查。`incremental_patch_source_does_not_scan_all_base_entries` 源码守卫和 z baseline crossing fallback regression 已存在。四个相关模块的 Rust 2021 `rustfmt --check --config skip_children=true`、七个候选路径的 `git diff --check` 和独立 no-global-scan 断言均通过；UI12 尚未释放 Cargo lane，因此这些结果仍是 static GREEN，不得写成 Rust test GREEN。

下一阶段不再凭静态阅读继续扩展缓存。必须先用 current-source Editor EXE 完成固定压力矩阵并按以下规则裁决：

1. 1000 次 click、1000 次 pointer move、200 次 native resize 必须来自同一 source manifest，并同时给出 wall time、processor time、单核/系统 CPU、working set/private bytes 的 start/end/peak。
2. 每次交互必须关联 host fast/full target、scope cardinality、Runtime changed nodes、layout/arranged/hit/render visited count、full fallback reason、Workbench/Pane spatial-query candidate count。
3. 暖交互阶段 SVG candidate build/tree parse/raster、atlas generation、GPU image upload write、GPU text shape/renderer build必须为 0 或由明确内容/generation 变化解释；cache hit 和 resident bytes 必须同时存在，不能只看 miss 计数。
4. GPU 必须报告 submitted/retry outcome、draw/visible command/vertex/text/image work、retained copy 和 damage。若现有 `PaintedPixels/RedrawRegion` 不能从 surface extent推导 `damage area / surface area`，先补这个观测缺口，再决定 bounded multi-rect。
5. 只有 profile 证明某个结构仍主导 p95/p99/max 或 RSS/GPU residency，才进入下一轮实现：host transaction granularity、layout data structure、multi-rect damage、Editor text-cache LRU 或异步 SVG miss 分别由对应证据触发，禁止把所有候选改动同时混入一轮。

### 15.28 Profile 裁决所需的 invalidation 与 damage 分母

执行 15.27 的 profile-first gate 前再次检查采集链，发现现有数据仍不足以区分“局部算法没有命中”和“局部算法命中但 damage union 太大”：`PaintedPixels` 只有每帧实际绘制面积，没有同一次 present 的完整 surface 面积分母；Editor host 也只导出 slow-path 数量，没有 transaction 数、scope 数以及最终 fast/full target。因此即使得到高 CPU 或高 GPU 工作量，也无法从 artifact 判断放大发生在 host invalidation、Runtime patch 还是 damage 合并阶段。

本轮只补观测，不改变 invalidation、布局、渲染或缓存算法：

1. GPU 和 softbuffer presenter 在每次绘制提交路径记录 `presented_surface_pixels = width * height`，与同场景累计 `painted_pixels` 形成 `damage_coverage_percent = painted / presented * 100`。resize 场景逐帧使用当时 surface extent，避免用固定首帧尺寸估算分母。
2. `begin_recompute_invalidation_phase` 记录 host transaction 数、scope 总数、legacy-dirty transaction 数，以及 `Full / ShellContent / WorkbenchProjection / ViewPresentation / WindowMetrics / PaintOnly` 六类最终 target。每个进入决策函数的 transaction 只归入一个 target，后续 profile 可直接检查 target 总数与 transaction 数是否一致，并定位 full widening 是否由 legacy dirty 伴随发生。
3. `UiScenarioHotspot` 为这些字段增加向后兼容的 `serde(default)`，Runtime counter 聚合与 Markdown summary、`ui-profile-capture.ps1` 场景证据同步导出原始分子、分母、coverage 和 host target 分布；不把派生百分比作为另一份可漂移的持久 authority。
4. TDD source contract 先在缺失 `presented_surface_pixels` 时得到 1 条预期失败；场景行为合同再证明旧 gate 会错误接受 target/transaction 不一致。实现后的自动验收先检查六类 target 总数等于 transaction 数，并要求 `painted_pixels == 0` 或存在正的 surface 分母且 `painted_pixels <= presented_surface_pixels`；计数链漂移、缺分母和 coverage 超过 100% 都 fail closed。
5. source-bound manifest 将 host 决策生产点、softbuffer 分母生产点和 Runtime Markdown 导出 owner 纳入 freshness authority，关键源码从 34 扩为 37；当前检查为 37 个路径全部存在、重复 0、缺失 0。纯采集 PowerShell 不参与 Editor/Runtime 二进制时间戳比较，避免脚本修改错误地强制重编产品二进制。
6. 修复后 profile output contract 21/21、native resize contract 3/3，合计 Pester 24/24。相关 PowerShell 解析、11 个 Rust 文件的 Rust 2021 `rustfmt --check --config skip_children=true`、scoped `git diff --check` 均通过。
7. Runtime projected hit index 同批静态复验确认：局部 `patch` 的 base lookup 只针对 `changed_node_ids` 与既有 projected subtree，不含 `base_index.grid.entries.iter()`；全表遍历只在显式 rebuild、order-plan 和 cell/index 重建路径。`incremental_patch_source_does_not_scan_all_base_entries` source guard 保留这一复杂度合同。

UI12 仍未发布 Cargo lane，因此新增 Rust 聚合/兼容性/计数器名称测试尚未动态执行，当前 Editor EXE 和 1000 click、1000 pointer、200 resize 产品 profile 也尚未生成。本节关闭的是“profile 缺少裁决分母与 host 决策证据”的源码/采集合同缺口，不关闭鼠标、按钮、缩放或 SVG/GPU 缓存的动态性能 gate。

### 15.29 既有 WGPU 时间戳的场景化 GPU 证据

继续检查 15.28 的采集权威时确认，产品并不缺 GPU 计时器：`zr_rhi_wgpu` 已在 UI pass 上使用 timestamp query，并通过异步 readback 发布 `gpu_timestamp_supported`、`gpu_time_us` 和 `gpu_profile_latency_frames`；Editor presenter factory 在 profile capture 启用时调用 `with_gpu_timing()`，GPU stats owner 也已记录时间样本和回读延迟。真正缺失的是这些既有计数器没有进入 `UiScenarioHotspot`，因此 source-bound artifact 无法给出 GPU p50/p95/max，也无法判断“设备支持但尚在异步 warm-up”和“计时从未启用”。

本轮保持计时算法和渲染路径不变，只闭合已有数据链：

1. Editor 对每个声明 timestamp support 的 submitted present 记录 `gpu_timestamp_supported_present_count`；实际 `gpu_time_us` 仍只在异步样本到达时记录，二者分离，避免用 frame 数伪造 GPU 样本数。
2. Runtime accumulator 保留每个场景的 GPU 时间样本，排序后导出 `gpu_time_sample_count`、p50、p95、max，并对 `gpu_profile_latency_frames` 取最大值；新字段均使用 `serde(default)` 保持旧 artifact 可读。
3. Markdown summary 和 `ui-profile-capture.ps1` 同时显示支持 present 数、样本数、GPU 分位数及最大回读延迟。场景确实生成 GPU batch 时，自动验收要求支持计数和至少一个时间样本都大于 0；没有 redraw batch 的 idle-hover 仍可作为纯事件路径证据，但不能冒充 GPU patch 证据。
4. source-bound manifest 新增启用计时的 presenter factory 和发布计数器的 GPU stats owner，从 37 扩为 39 个关键源码；当前为存在 39、缺失 0、重复 0。
5. TDD RED 精确得到 3 条预期失败：脚本没有 GPU 时间字段、缺样本仍被接受、manifest 仍为 37。实现后 profile output contract 22/22、native resize contract 3/3，合计 Pester 25/25；PowerShell parser 0 error，相关 Rust 文件 `rustfmt --edition 2021 --check --config skip_children=true`、scoped `git diff --check` 均通过。
6. 同轮再次直接切片 `UiProjectedHitTestIndex::patch`，确认不存在 `base_index.grid.entries.iter()`；overlay base 稳定性只检查实际 affected entry，已有 source guard 禁止回归为每次增量 patch 的 `O(N)` 扫描。全 base hit-grid rebuild 由 caller 显式转入 projected rebuild。

Cargo lane 仍未获 UI12 明确释放，因此 Rust 聚合测试和产品 GPU timestamp 行为尚未执行。当前结论仅是“已有 GPU timer 已完整接入 source-bound profile 合同”；只有 current-source Editor EXE 跑完 1000 click、1000 pointer、200 resize 并产出非零 GPU 样本后，才能用 p50/p95/max 裁决下一轮布局、damage 或缓存优化。

### 15.30 Projected hit-grid 的 source-bound 动态验证阻断

UI12 明确释放 Cargo lane 后，本轮按下层优先顺序提交四张 Windows managed validation ticket，分别覆盖 projected-grid 模块的仿射 frame/clip、popup 内部 z 与 stack topmost、base full rebuild、no-global-scan source guard，frame/instance authority 与 physical virtual pointer，parent input policy 的 descendant incremental patch，以及 missing lookup/reindex。四张 ticket 绑定同一 exact7 source manifest `57575239bcce7bd3ad119c6fe6f74f80e152e459494c81793b3132607af5bca4`；外部漂移 `surface/render/cache.rs` 明确未进入 manifest。

动态结果不是 Rust RED。ticket `69750716903a4254bea2401665adb421`、`0f9c7e3efa1348a58710d58280a9360c`、`f4373d46564747e6a4528c0873b74f62`、`184ea8155d5e4136a1accf95d5c19283` 分别对应 copy job `4e6b2d53fa8d4276bf295198ec2a7256`、`bc4600f60d5446cb932a883a7bec57ab`、`874d33160c5e4b858bbaec7603a9cd85`、`7a4bee882de4409f8038fcfb56521c4f`；四者均在 `closure_planning` 以 `validation_copy_compile_time_resource_missing` 终止，Cargo/rustc 未启动，实际用例数为 0，Cargo exit code 不适用。

复用 materializer 自身 Rust include 词法解析器的只读扫描还原了被 copy status 丢失的 detail：`tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/host_adapter.rs` 仍 `include_str!` 已被 Plugins01 硬切删除的 `plugin/native_plugin_loader/host_api_adapter/tests.rs`。当前 adapter 测试已拆入 `abi_decode/bridge_scope/context_handles/ecs_registration/registration_policy` 子 owner，因此这是 Runtime15/Plugins01 的 stale review guard 和共享 source-closure blocker，不是 popup exact7 的编译或行为失败。本轮没有修改该外部 owner，也没有继续启动重复 Cargo；Runtime 下层动态 gate、Editor 产品层验证、profiling EXE 与 1000 click/1000 pointer/200 resize 继续保持 open。

### 15.31 Editor presentation 目标身份丢失：当前最高优先级结构问题

2026-08-15 继续从事件 producer、record、retained-host bridge、host transaction 一直追到 recompute target，确认当前最可能让按钮和局部编辑退化为全量工作的不是 Runtime 缺少缓存，而是 Editor 在进入这些缓存之前丢失了变更目标。`EditorEventEffect::PresentationChanged` 不携带 view、pane、shell mount 或 invalidation reason；`apply_record_effects` 因而只能调用全局 `request_presentation()`。`apply_dispatch_effects` 只有在 effect 集合恰好保留一个 `HostShellContentScope` 时才走 shell fast path，否则写入 `All` scope；recompute decision 在 legacy dirty、非纯 view、非单 shell scope时落到 `Full`。一旦进入 Full，后面的 changed-node、per-node command bucket、hit grid、text/SVG/GPU cache 即使算法正确，也会因为上游重新发布整套 shell projection 而失去大部分收益。

当前源码量化如下：

1. `editor_event_execution/**` 加失败路径共有 25 个生产 `EditorEventEffect::PresentationChanged` 构造点，分布在 11 个文件；这些构造点都不表达目标。成功记录无论 `ExecutionOutcome.changed()` 是否为 false，都会原样保存并应用 `execution.effects()`。动画命令的 missing-target/no-change 分支明确返回 `changed=false`，同时仍携带 `PresentationChanged + ReflectionChanged`；其中状态行确实需要刷新，但只需要状态栏或所属 view，不应据此全局重建。
2. retained host 另有 24 个生产 `.mark_presentation_dirty()` 调用，分布在 23 个文件；`.mark_presentation_dirty_for_view(...)` 只有 1 个生产调用。多个 UI asset detail handler 已经持有稳定 `instance_id`，包括 collection、component adapter、palette drag、source cursor/text、binding、preview、structure 和 style 操作，却在成功后丢弃该 identity 并调用全局 dirty。这是可直接验证的协议放大，不是推测。
3. `HostInvalidationTransaction` 本身已经能用 `BTreeMap<HostInvalidationScope, HostInvalidationMask>` 合并多个 scope，且纯 `View + PRESENTATION_DATA` 可以形成 `ViewPresentation(Vec<ViewInstanceId>)`；实际 producer 很少把目标送到这里。单 shell fast path还要求恰好一个 scope，合并任意 unscoped presentation 会主动清除已有 shell scope。也就是说，本地 patch machinery 已存在，缺失的是从 producer 到 transaction 的连续 target authority。
4. Editor 内已经存在第二套更完整的目标协议：`EditorViewInvalidationMask` 包含 `LAYOUT / TREE_STRUCTURE / PRESENTATION_DATA / PAINT_ONLY / POINTER_HOVER / VIEWPORT_IMAGE / HIT_TEST / WINDOW_METRICS / RENDER`，`ViewDirtySet` 按 `ViewInstanceId` 合并 mask，message bus/watch map 也能携带 view dirty。retained host 同时维护形状近似的 `HostInvalidationMask`。继续增加第三套 reason enum 会扩大漂移；正确方向是收敛现有两套协议并保持 view identity，而不是再在 bridge 中猜测事件语义。

参考实现支持这一裁决。Unreal 的 `EInvalidateWidgetReason` 把 Layout、Paint、ChildOrder、RenderTransform、Visibility 等原因附着到稳定 `FWidgetProxy`，`FSlateInvalidationRoot::InvalidateWidget` 只把该 proxy 放入对应 phase heap，最终 update list 和必要的 reindex range 都由受影响 proxy 推导；不会把一个 targetless “presentation changed” 转换成整个窗口的 slow path。Fyrox 分离 `invalidate_visual / invalidate_measure / invalidate_arrange`，并在 measure/arrange 的输入和 valid flag未变时直接返回。Slint 由 per-item geometry/property tracker形成 dirty region，旧/新 bounds只扩大对应区域。这些实现共同要求“目标 + 原因”在 publication 前保持可追踪，而不是仅在渲染末端增加更多缓存。

结构方案固定为以下顺序，profile 未证明前不混入多 rect、text LRU 或异步 SVG 等其他候选优化：

1. **收敛合同而非新增平行类型。** core/editor message 层继续以 `ViewInstanceId + EditorViewInvalidationMask` 作为业务无关 authority。Editor event outcome 应携带一个可合并的 `ViewDirtySet` 或等价 target map；`EditorEventEffect` 保留项目开关、导入、toast 等命令副作用，但不再用 targetless `PresentationChanged/LayoutChanged/RenderChanged` 代表 UI invalidation。core contract 不得依赖 retained-host 的 `ActivityDrawerSlot`；host bridge根据当前稳定 mount/generation把 view target投影成 `View` 或 `ShellContent` scope。
2. **先修明确持有 identity 的 producer。** UI asset detail handlers、table row、console filter/clear、局部 popup/selection control先使用已有 view target。真正跨 view 的 selection、hierarchy、draft和 asset state 变化由 message bus/watch map 的订阅关系生成 target set，不在 executor 中硬编码 Inspector/Hierarchy 等 pane id。Open/close project、layout topology和无法解析稳定 owner 的变化才允许显式 `All`。
3. **保持多目标，不因合并自动扩大。** transaction 合并继续是 `target -> reason mask` 的 map union；paint-only 与 scoped view合并必须保留 scope。多 view presentation 应继续进入现有 `ViewPresentation(Vec)`；多 shell mount或带 layout/tree reason 的 target set需要独立 patch计划或带明确 widening reason 的保守 Full，不能静默清空 scope。widening 必须记录 producer、原 target count、reason 和 fallback cause。
4. **把 no-op、状态反馈和数据变化拆开。** `changed=false` 不能简单删除所有 UI 更新，因为 status/toast可能变化；应分别发布 document/view dirty、status-bar shell dirty和toast dirty。没有任何可见反馈的 no-op不得产生 presentation transaction。失败路径同样只更新实际错误反馈 owner，除非失败回滚确实改变了业务 view。
5. **先做下层合同，再迁移调用面。** 下层测试首先证明两个不同 view target合并后仍产生 `ViewPresentation` 而非 Full、scoped + paint保持 scope、unscoped structural event才 widening、no-op status只刷新状态 owner。随后迁移 UI asset detail和console等明确局部 producer，最后再处理 selection/hierarchy等多订阅域；每批必须有 source-bound profile对照，禁止一次替换全部 25 + 24 个调用点。

验收复杂度和产品门槛：target merge 保持 `O(k log k)`，其中 `k` 是本次变更 view 数；projection/patch成本与目标 view的可见节点和 changed-node 数同阶。1000 次 UI asset局部操作、console filter/clear和非结构按钮交互中，`HostInvalidationFullTargetCount`、shell snapshot/model/chrome full build和 slow-path command rebuild目标为 0；host target总数必须等于 transaction总数，view/shell target identity不得在 bridge丢失。若局部 producer迁移后 full仍占主导，再依据 widening reason选择修 host multi-scope patch；若 full已归零而 p95/p99仍高，才继续裁决 layout container、damage region、text LRU或同步 SVG miss。上述结论必须由当前 EXE 的 CPU、RSS、GPU timestamp、damage coverage和cache counters共同证明。

### 15.32 Projected-grid managed validation 的 Windows 命令长度阻断

Runtime15 的 native host adapter hard cut 已在 HEAD `36b8e5ef3b93e929bd64c8311799e0fff03c885b` copy-complete 后，本轮只重提一张 `projected_grid_tests` ticket `9d4b750c134b4a7abcb6b7f0a32ebf19`，copy job `b40cbf04ebad41788caaf03e788244a5`，exact7 manifest仍为 `57575239bcce7bd3ad119c6fe6f74f80e152e459494c81793b3132607af5bca4`。该票仍在 `closure_planning` 失败，没有 copy run、Cargo job或 Cargo run，实际用例数为 0，exit code不适用；其结果不能解释为 popup source RED。

使用生产 `CargoInputClosurePlanner` 的同路径只读复现已确定真实根因：相同 10 个 workspace members产生 14,542 个 tracked paths和 4,288 个可解析 include expressions，所有 compile-time resource均存在；随后 `validation_copies.py` 将 2,349 个 resource roots一次性传给 `git ls-files -- ...`，命令约 166,863 字符，Windows在进程创建时抛出 `FileNotFoundError [WinError 206]`。旧 `_fail_materialization` 又把非 `CoordinatorError` 压成 `validation_copy_materialization_failed + {}`，因此 ticket最初没有 actionable detail。Tooling 已以 maintenance commit `9b9e03755` 将 pathspec 命令限制为 24,000 字符的有界批次，并增加 2,400 roots 与 WinError 206 actionable detail 回归；但运行中的 coordinator instance `c0795e5710d946b7837284a1119eaf54` 启动于该提交之前，仍必须受控 rollover 才能使用新代码。

Tooling 提交后的 immutable-copy 预审还发现三个归档 Runtime09 owner 的 worktree-only 支持路径。ownership transfer preview 对三者均返回 eligible，随后完整当前内容转移到活跃 Runtime09 session并以独立 maintenance commit `c279c19f41c28882bf13a1858af63592739ea757` 进入 HEAD：`dynamic_api/session/hud.rs` 与 `menu.rs` 补齐 `UiRenderExtract.raster_scale`，`asset/project/paths.rs` 移除 HEAD 中 4 个对 `&u16` 调用的非法 ASCII 方法并保留该 owner 的完整 resolver 收敛。commit tree 精确 3 路径，popup exact7未纳入；Rust 2021 rustfmt、scoped diff-check和 HEAD/current source guards通过，没有运行 Cargo。

再次重试前仍必须同时满足：daemon完成受控重启并证明加载 `9b9e03755` 或后续 HEAD；Runtime15 路由中的 extension-registry API 及其 host-adapter consumer以 copy-complete commit进入 HEAD；managed lane、git mutex、OS Cargo/rustc为空。满足后重新计算 exact7 manifest，仍只先提交 `projected_grid_tests`，等待 materialization和终态，再按 authority、parent-input patch、missing-reindex顺序逐张验证；popup exact7和外部 `surface/render/cache.rs` 在此期间保持不变。

2026-08-15 后续状态已收敛。Tooling post-commit rollover action `dbf5a77221634bd8890afa5a5a101d8d` 成功，新 daemon instance `451ba7e16d084a389e20686557d41f53`、schema 62，从 committed HEAD `eb52a1aad` 启动，supervision failure count归零；Windows argv batching与 artifact receipt修复均已加载。Runtime08 callback-factory 支持以 `aced9293efcf9d99454d025245497f716734f3a1` 进入 HEAD，commit tree精确 11 路径，三条退役 `scene_hook.rs` 已由 `git cat-file -e HEAD:<path>` 证明不存在，其余八条 owner路径存在。当前剩余 copy-stability 条件只有 Render17 在 `wgpu_render_framework_construction/construct.rs` 中的 `viewport_products: Default::default()` 单字段支持提交；该文件仍是 worktree-only，不能纳入 Runtime09 overlay。coordinator明确要求两项支持都到齐后再发唯一 Cargo authorization，因此即使 daemon、Runtime08、OS Cargo/rustc=0、`.git/index.lock` absent 已满足，仍不得提前重算或提交 ticket。

### 15.33 点击 profile 的目标身份与采集器版本权威

继续复核自动交互而不是只看计数器字段后，发现旧 `material_lab_click` 仍不足以证明按钮路径可交互。Editor 导出的 `UiProfileNamedFrame` 已包含稳定 `id / kind / surface / frame / clip`，模板控件还以 `template.{surface}.{control_id}` 编码 control identity；但 PowerShell 的 `Get-LiveGeometryClickPoints` 立即把条目降成裸 `X/Y`。`Invoke-PointerClickStorm` 因而只记录请求数、完成数、CPU 和 RSS，严格 gate 只校验 1000 次是否发出了 1000 次 mouse down/up。geometry 不存在时的三个 ratio fallback 同样可通过。这个合同无法区分“命中 dispatchable control”、“反复点到同一个 no-op 区域”和“控件已移动但旧坐标仍发出输入”，也无法把 full invalidation 归因到实际 target。

本轮只修采集 authority，不修改产品交互或 Runtime09 popup 源码：

1. `Get-LiveGeometryInteractionTargets` 保留每个点击或 hover 点的 `target_id / target_kind / target_surface / source`；click/pointer storm artifact 同时记录输入 target 集合、坐标、`target_count` 和 `used_geometry`。ratio fallback 仍可用于非严格人工排障，但标记为 `ratio_fallback`，在 `RequireScenarioEvidence` 下 fail closed。
2. click-storm gate 要求所有 target 来自 `ui_profile_geometry.json` 且三段 identity 非空。`material_lab_click` 进一步要求 target 均为 `template_control`，ID 以 `template.` 开头。geometry 生产端的 `is_dispatchable_template_node` 已排除非交互模板节点，因此这组坐标至少绑定到当帧真实可分派控件，不再只是视觉猜点。
3. Material Lab binding 是明确的 paint-only 特例：retained-host callback 直接返回 `HostInvalidationMask::PAINT_ONLY`，不走 targetless `EditorEventEffect::PresentationChanged`。场景 gate 现在要求 transaction 和 paint-only target 均非零，同时 `host_invalidation_full_target_count == 0`、`host_invalidation_legacy_dirty_transaction_count == 0`；target 总数与 transaction 总数、damage 分母、GPU timestamp 等既有门禁继续同时成立。
4. `source_manifest.json` 原先只哈希 39 个产品关键源码和两个二进制，未哈希实际执行的采集脚本；dirty-tree 状态不能唯一标识脚本内容。现在 `capture.tool_files` 独立记录 `ui-profile-capture.ps1`、`ui-profile-native-resize.ps1`、`profile-capture-paths.ps1`、`profile-capture-manifest.ps1` 的路径、SHA-256、字节数和修改时间，任一缺失即 fail closed。这 4 个工具指纹不参与“产品二进制必须晚于产品源码”的 freshness 比较，避免脚本更新错误强制重编 EXE。
5. pointer storm 原本用窗口尺寸和两个步长做伪随机扫点，空白区域也能完成 1000 次并通过。现在它循环实时 geometry 中最多 8 个可交互控件中心；`material_lab_hover` 只接受 dispatchable `template_control`，完成数、CPU/RSS 和 target provenance 必须同时存在。
6. TDD 第一组先得到 3 条预期 RED：目标提取函数仍丢身份、Material Lab 错误接受 Full、完整 1000 click 但无 provenance 仍通过；第二组再得到 2 条预期 RED：manifest 工具指纹数为 0、缺 native helper 仍成功；pointer 组得到 1 条预期 RED：1000 move 无 geometry identity 仍通过。实现后 profile output 合同 24/24、native interaction 合同 3/3，共 27/27；4 个 PowerShell 文件 AST parser 0 error，scoped `git diff --check` 通过。

这组修复只让未来的 1000-click 证据可归因。Material Lab 是稳定的“dispatchable button + paint-only host transaction + GPU/damage/cache”微基准，却不能验证 15.31 的 Editor event 目标身份丢失，因为它刻意绕过该协议。正式优化前还必须增加至少一个持有 `ViewInstanceId` 的产品操作场景，要求 `ViewPresentation` 或明确 shell target 非零、Full 和 legacy 为 0，并记录 producer/target identity；否则不能用 Material Lab 绿色推断 25 个 targetless `PresentationChanged` 构造点已解决。当前仍没有新的 profiling EXE 或产品动态样本，1000 click、1000 pointer 和 200 resize gate 保持 open。

### 15.34 Resize 场景的 SVG 与 GPU 图像缓存正证据

继续审查 `Test-WindowResizeCounterGate` 后确认，旧门禁只能证明 resize 期间没有观察到 visual/SVG miss、GPU image admission reject、invalid payload 或超过一次的 upload/allocation。这个条件存在空场景假绿：完全没有 SVG 或 GPU 图像命令的轨迹同样会得到全部零值并通过，因而不能回答“SVG 是否真的走缓存、GPU 是否真的复用图像”这一问题。

本轮不增加 Rust 计数器，直接使用产品已经发布到 raw timeline 的现有 authority，将 resize 门禁改为同时要求正、负证据：

1. `gpu_image_vertices > 0` 证明 resize 帧实际提交了图像几何；`gpu_image_prepare_cache_hits > 0` 证明 presenter-local `(resource_key, generation)` 缓存被命中。`gpu_image_prepare_command_visits` 同时打印用于区分缓存快路和重新检查 image command。15.89进一步确认同进程warmup结束时profiler会重启而GPU cache继续驻留，因此当前measured resize门已收紧为0；warmup本身不能再为measured命令回扫提供豁免。
2. `visual_asset_cache_hit_count > 0` 与 `svg_tree_cache_memory_hit_count > 0` 证明恰好一次的 resize snapshot rebuild 确实经过 visual/SVG 内存缓存；两类 miss仍必须为 0。这样不会把“场景根本没有 SVG”误写成 SVG 缓存绿色，也不要求 200 个 native size step重复解析或查找 SVG。
3. 既有约束保持不变：snapshot build必须为 1且 reuse 为正，surface reconfigure 有界，model/chrome build不超过 1，GPU upload/cache-key allocation不超过 1，admission reject、invalid payload、visual full invalidation全部为 0，并恢复原始窗口尺寸。
4. 动态 Pester 先得到两次精确 RED：完整 `24/24` resize但图像顶点/缓存命中为 0时旧门禁错误返回 true；补入 `image_vertices=144` 与 GPU cache hits `24` 后，visual/SVG正命中仍为 0时旧门禁再次错误返回 true。实现后正例只有在 visual hits `8`、SVG tree hits `4`、所有 miss 为 0时通过。
5. 最终 profile output contract 为 25/25，native resize contract 为 3/3，合计 28/28；4 个采集 PowerShell 文件 AST parser 均为 0 error，scoped `git diff --check` 仅报告仓库既有 LF/CRLF 转换提示。
6. 同轮还发现 CPU/RSS artifact 的严格门禁只检查 `processor_time_delta_ms` 存在。TDD 得到 2 条 RED：完整且 geometry/source-bound 的 1000 click 与 1000 pointer在缺失 wall time、两个 CPU 比例及全部 RSS/private采样时仍被接受。新增统一 `Test-InteractionProcessEvidence`，要求 elapsed/processor time、core/system CPU、logical processor count、working set 与 private bytes 的 start/end/peak全部存在、有限、非负，并要求 peak不小于 start/end；click、pointer 和 native resize共用同一合同。

这项修改关闭的是 resize 采集器的空证据漏洞，不是产品性能结论。只有 Tooling 修复 validation-copy 的 Windows argv batching、Runtime popup 下层验证返回、并生成绑定当前源码与四个工具 SHA-256 的 Editor profiling binary 后，200-step native resize 才能提供真实 CPU/RSS/GPU/SVG 数值。真实轨迹若没有 visual/SVG正命中，门禁将 fail closed，必须先确认场景是否确实包含 SVG；若有正命中但出现 miss、upload或 allocation随 step数增长，才进入 SVG invalidation或 GPU residency owner修复。

### 15.35 交互延迟边界与历史卡顿证据

继续复核“鼠标和按钮仍然卡”所对应的测量边界后，确认当前采集合同仍可能给出误导性绿色。`Invoke-PointerClickStorm` 与 `Invoke-PointerMoveStorm` 的 `elapsed_ms` 包含脚本主动插入的 click/move delay，只适合量化压力期间的进程 CPU/RSS，不能解释为单次输入响应时间。`ui_hotspots.json` 的 frame 指标来自 UI 场景计数器，而历史 `timeline.zrtrace.json` 中的 `retained_host_tick` 覆盖另一段 host 工作；两者没有同一个起点，因此不能拿较小的一项代替用户感知延迟。

当前产品源码并非完全没有延迟观测。Winit `WindowEvent` 进入 Editor event loop 并完成 platform-event translation 后，pointer/keyboard/IME handler 调用 `begin_input_latency_sample()`；dispatch 产生 redraw 时记录 `input_to_damage_us`，并从第一份待提交 damage 开始记录 `damage_to_submit_us`。成功 present 才结束后一段；`RetryableSurfacePresent` 会保留 damage 起点并延迟重试，不会把 no-submit 写成成功提交。这个设计已经能把 handler内的输入路由/回调/失效与 frame update/present 两段分开，但不包含设备事件到 Winit callback 的 OS 队列等待，也不是 device-to-photon 测量。仍有三个合同缺口：

1. 本轮复核开始时，`ui-profile-capture.ps1` 只把 `damage_to_submit_us` 导出为 sample count，没有输出 `input_to_damage_us`，也没有两段的 p50/p95/p99/max；当时 artifact 无法判断卡顿主导段。本轮已把 `ui_surface_present_outcomes.json` 升为 schema 3，用统一 nearest-rank 汇总同时导出两段的 sample count、p50/p95/p99/max，并继续把 retryable no-submit 与 submitted count 分离。
2. 两段 counter 没有携带 `UiInputSequence` 或同等 correlation id。事件合并时，多个 input-to-damage 样本可以共享第一份 pending damage，而单个 damage-to-submit 样本又可能跨 surface retry；分位数可以用于阶段定位，但不能相加后冒充逐事件 end-to-end latency。
3. native resize 走独立 debounce/snapshot present 路径，没有调用 pointer/keyboard 的 input sample。resize 的真实响应必须单独记录 native size event ingest、presenter resize/snapshot submit 和最终 debounced retained reflow，不能用外部 16 ms step delay或普通 pointer counter替代。

Unreal Slate 的本地参考源码支持这个分层边界。`FSlateApplication::PollGameDeviceState` 先排空 platform input；`ProcessMouseButtonDownEvent` 用独立 cycle stat 和 `FScopeProcessInputEvent` 包围预处理、hit path定位及 dispatch，`FEventRouter::Route` 再用 `FScopeRouteInputEvent` 包围沿 widget path 的路由。另一侧 `FSlateTrace` 分别记录 widget invalidation 的 target/investigator/reason、widget update 的 start/end/affected count、paint pass以及整帧 tick/repaint/paint/invalidate 数量。Slate 同样没有用 paint duration 单独代表 input latency，而是让输入路由、失效来源、update/paint 和 frame publication 保持可区分。

现有历史 artifact 只能作为结构定位，不是 current-source acceptance。对 `E:\zircon-profiles\runtime09-svg-m7\shell-content-presentation-patch` 中时间最新的 14 组 `click-dock-patch-spaced` timeline 做统一只读聚合：capture 间 `retained_host_tick` p50 中位数为 117,912 us，13/14 的 capture p50 超过 16.67 ms，14/14 的 capture p95 超过 50 ms；`visual_assets_render_svg_parse` 共 391 次、累计 1,794,855 us，而 `paint_command_image` 共 5,193 次、累计 4,768 us，前者耗时约为后者 376.44 倍。最新一组 `20260811-201002-click-dock-patch-spaced` 中 retained-host tick p50 为 136,279 us、p95 为 236,384 us、max 为 2,359,875 us；同组 `ui_hotspots` 却给出 click frame p95 5,705 us。这个冲突证明两份指标覆盖不同工作边界，也把历史卡顿定位到 presentation/scene conversion/同步 SVG parse 一侧，而不是 GPU image draw 本身。由于这些目录没有 `source_manifest.json`，不能用于声明当前源码已修复或仍以相同比例回归。

同一批历史 timeline 中，12/14 组已经含两段 latency counter，另外 2 组为零样本。对 12 组做同一 nearest-rank 回算，共得到 767 个 input-to-damage 样本和 632 个 damage-to-submit 样本：各 capture 的 input-to-damage p50 中位数为 37.9 us、p95 中位数为 2,398.5 us、全局最大样本为 291,299.4 us；damage-to-submit p50 中位数为 5,788.8 us、p95 中位数为 198,323 us、全局最大样本为 2,412,531.5 us。旧样本因此更具体地指向 damage 产生后的 redraw scheduling、frame update、presentation、render/present阶段，而不是常态 hit route；input 段仍有少数 200 ms级离群值，不能据此把输入路径整体判为已达标。两组零样本也证明旧采集可在缺失关键证据时继续产出报告，新 schema 3 gate 会直接拒绝这种 artifact。

下一轮 current-source profile 的延迟门固定为：

1. 每个 click/pointer 场景同时导出 input-to-damage 与 damage-to-submit 的 sample count、p50/p95/p99/max；CPU/RSS 注入压力数据继续单独报告，不与延迟分位数混算。
2. current-source binary 若还没有 input sequence 到 successful present 的关联，就只报告两段延迟并明确“不可组成逐事件 end-to-end”；不得把 frame counter、脚本 wall time或二者之和命名为 input latency。
3. resize 需要独立的 event-to-snapshot-submit 与 last-event-to-debounced-reflow 指标；200-step gate 同时检查 snapshot reuse、surface reconfigure、SVG/GPU image cache、CPU/RSS 和这两段 resize latency。
4. 先用分段 p95/p99/max 选择 owner：input-to-damage 主导才处理 hit route/callback/invalidation；damage-to-submit 主导才处理 projection/layout/render/present；两者都低但显示仍迟缓时再检查 compositor/present feedback 与 GPU timestamp。没有该裁决证据，不继续扩大缓存或重写布局算法。

证据导出采用 TDD 收口：第一组实现前 profile output suite 为 23/25，两个 RED 分别证明源码没有 input-to-damage 字段、synthetic timeline 的分段统计为空；实现后同套件 25/25。第二组先得到 25/26，唯一 RED 证明 `RequireScenarioEvidence` 尚未消费 latency artifact；新增 gate 后 26/26，click/pointer storm 只有在两段样本数均大于零、数值有限非负且 `p50 <= p95 <= p99 <= max` 时才通过。native resize suite 仍为 3/3，四个 PowerShell 文件 AST parser均为 0 error，scoped `git diff --check` 仅有既存 LF/CRLF 提示。没有运行 Cargo，也没有触碰 popup exact7 或产品算法。尚未关闭的是第 2、3 项 correlation/resize 边界；它们必须与 current-source产品 profile一起裁决，不能由合成测试代替。

### 15.36 全 UI 范围清点与审查边界

本轮在不进入 popup/Cargo 阻断链的前提下，把审查范围从 layout、damage、SVG/GPU cache 和 Editor presentation 扩大到窗口调度、原生输入、焦点、计时器、虚拟化、无障碍与多 surface owner。静态清点覆盖 `zircon_runtime/src/ui`、`zircon_editor/src/ui` 和 `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface`，合计约 5,100 个 Rust 文件、55 万行源码；其中生产代码约 41 万行、测试约 14 万行。共享 worktree 正在持续 hard cut，精确文件数只能作为审查规模元数据，不能替代 source-bound manifest。审查采用“入口 -> authority -> cache/index -> invalidation -> publication -> input consumer”的数据流取样，并对每个可能的热路径确认是否存在全树扫描、重复分配、跨 generation 重建或并行 authority。

窗口事件循环本身没有发现“空闲持续全速刷新”的证据。Editor 的 `event_loop/lifecycle.rs` 在存在 runtime、maintenance、resize 或 retry deadline 时使用 `WaitUntil`，否则使用 `Wait`；`event_loop/redraw.rs` 只在 pending redraw 从空变为非空时请求 OS redraw，native resize 期间优先复用 snapshot 并抑制 retained frame update；`tick.rs` 也只在明确请求 frame 时推进 polling/recompute。该层应列为已建立的正确性护栏，而不是当前重写对象。任何后续修改都必须保持：idle CPU 不随窗口数量增长、相同 pending damage 不重复 `request_redraw`、resize debounce 之前不做 retained 全量 reflow。

Runtime window pump 的保守边界也基本合理：resize/create/scale-factor 变化会把各 root 标记为 layout + hit + render，保证下一输入使用新几何；普通 redraw request 只标 render。窗口几何变化确实是 root-wide barrier，不能为了局部化而允许 stale hit geometry。需要单独观测的是 root 数、受影响节点数和相同 native resize burst 内的实际 rebuild 次数，而不是删除 barrier。当前生产源码没有稳定使用 `UiWindowRedrawReason::Animation` 的证据，因此不能把持续重绘先归因给 animation reason；动画属性的 dirty-domain 放大由既有 `PERF-MVP-315` 继续负责。

### 15.37 非渲染域的复杂度与风险分级

| 域 | 当前 authority / 算法 | 单次复杂度与分配 | 对当前鼠标卡顿的解释力 | 决策 |
|---|---|---|---|---|
| 原生 pointer move | presentation generation 为 `Arc` 快照；menu frame 小集合；floating window 逆序；workbench template 用 cell bucket；pane 为固定分支后进入局部 bridge | 常态约 `O(M + W + C + route depth)`，`C` 为命中 cell 候选；未发现每 move 全 UI 树 materialize 或扫描 | 中低；历史常态 input-to-damage 很小，但有离群值 | 先补分段 counter，不重写 hit algorithm |
| window redraw / resize | pending reason 合并；resize snapshot reuse；几何事件是 root barrier | idle `O(1)`；resize fallback 与 root/node 数相关 | damage 后长尾的可能入口，但不是重复 OS redraw 的已证实来源 | 保留算法，量化 snapshot/reflow/present 各段 |
| Runtime timer | typeahead/submenu/tooltip/toast 四个 `BTreeMap`；每 tick 分别找 due 项并临时收集 | 无到期也为 `O(T)` 四轮扫描；到期项产生临时 `Vec` | 低于当前鼠标主路径，但会使多 surface idle 成本线性增长 | 复用 `PERF-MVP-255`，不建立第二 owner |
| input batch | window pump batch 是无上限 `Vec`，高频 move/resize 缺 typed coalescing barrier | burst 为 `O(E)` route/result，可能重复 dirty/rebuild | 高压输入时较高，单事件常态未证实 | 复用 `PERF-MVP-314`；resize 后 pointer 必须保留 geometry barrier |
| 焦点导航 | 每次构造全树 focus candidate，modal scope 再扫 `tree.nodes`，候选每次排序 | `O(N + F log F)`，group/spatial 路径可重复构造 | 对 mouse move 低，对大树 keyboard/gamepad 导航高 | 规划 generation-stamped focus/navigation index，不能再造独立几何树 |
| 虚拟列表 | visible-window 数学为 `O(1)`，但 Runtime 总数取 `children.len()`，measure/arrange 仍访问全部 retained child；Editor `virtual_rows` 又把全部逻辑行克隆成真实 `UiTreeNode` | 完整同步至少 `O(S + L log L)` 与 `O(L)` 节点/属性存储，Runtime layout 仍为 `O(S)`；`L` 为逻辑行、`S` 为surface节点 | 高；hierarchy paint/input 已分别为 `O(V)`/`O(1)`，隐藏模板镜像仍可独立放大全 shell fallback | hierarchy 删除逐行模板镜像；确需可编辑控件的 inspector 使用有界 slot pool；逻辑总数与materialized count必须解耦 |
| 无障碍 | snapshot extraction 至少三次全树遍历并构造有序 map/set；action 路径每次获取完整 snapshot | 通常 `O(N log N + R log N)`；隐藏 relation target 修补存在退化为 `O(H*N)` 的形态 | 当前 Editor 产品接线弱，不能解释普通 mouse 主路径 | 作为正确性与未来规模项；frame publication 缓存 snapshot/delta，action 不得重建全树 |
| pane interaction state | 每个 setter clone 小型 state 后比较并交换 `Arc`，含若干 frame/shared string | `O(S)`，目前 `S` 固定但每 move 有 refcount/复制流量 | 可能形成微小常数热点，不能凭静态形态定罪 | profile clone/Arc RMW/changed=false 次数，达到占比再拆字段 |

焦点路径与 Unreal 的差异最明确。Slate 的 directional navigation 复用窗口 `FHittestGrid::FindNextFocusableWidget`，next/previous 则沿既有 widget path；Zircon 当前 `navigation_candidates` 每次从 root 递归并排序。正确方向不是另建一棵 focus tree，而是在 authoritative arranged/projected entry 上维护 focusability、tab order、navigation group 和 generation，以 changed-node patch 更新索引；弹窗/modal stack 必须与 projected hit authority 使用同一 frame publication 顺序。这样查询可趋近命中 cell/当前 group 候选规模，而不是全 surface 节点数。

虚拟化也不能只用 `visible_range` 为 `O(1)` 就判定已经完成。Runtime 的可见范围计算是常数工作，但 `UiVirtualListConfig` 只有 `item_extent/overscan`，总数来自已物化的 `children.len()`；measure先递归全部child，arrange再生成全部position并逐个清空窗口外subtree。Editor `virtual_rows.rs` 还通过全树查找control/existing/stale row并按逻辑总数克隆真实节点。结构方案必须先区分数据模型、逻辑extent和有界实例：无需模板交互的hierarchy删除逐行镜像；确需输入/焦点的列表以data generation、visible range和prototype generation驱动稳定slot pool，range平移只重绑 `entered + exited`，不能再把“隐藏全量child”称为虚拟化。

无障碍目前更像尚未闭合的产品边界：Runtime 能生成完整 snapshot，但 Editor 产品路径没有稳定的 access provider 消费，dynamic preview 仍发布“runtime UI surface accessibility extraction unavailable”占位结果。优化前先明确接线合同；之后 snapshot 应由 frame publication generation 缓存，结构/语义 dirty 生成增量或受影响 subtree rebuild，accessibility action 读取同 generation snapshot。不能为了降低 action 延迟跳过 relation、focus 或 hidden target 校验。

### 15.38 Editor 输入 authority 分裂与鼠标热路径

生产源码静态计数显示，Editor UI 仍有 15 个直接 `dispatch_pointer_event` / `dispatch_input_event` 调用、27 个 `UiPointerDispatcher::default()`、33 个 `UiSurface::new(...)`、14 类 pointer bridge 和 12 个局部 `rebuild_surface`；没有产品代码稳定通过 `UiInputManager` 汇聚这些 surface。`UiInputManager` 的完整窗口输入生命周期目前主要由 Runtime 测试 fixture 使用，产品侧则在 menu、workbench、pane、template 和各 tool bridge 中分别持有 dispatcher、hover/capture 或 rebuild 入口。这是输入序列、double-click、IME、timer、capture 和 dirty merge 的 authority 分裂风险，但静态计数本身不能证明它是当前 pointer p95 的最大耗时。

直接追踪 native pointer move 后没有发现 generation 深拷贝或全树扫描：`HostPresentationGeneration` 只是克隆内部 `Arc`；menu 只遍历当前 menu frames；workbench hit 先查 spatial bucket；floating window 按 topmost 逆序，之后是固定 pane 分支。当前缺失的是这些阶段的独立计时与规模计数。下一份 current-source profile 在改算法前必须增加：presentation generation acquire、overlay/menu check、workbench cell query 的 candidate count/time、floating/pane route、bridge dispatch/reducer、interaction-state changed/no-op、dirty merge 和 redraw request。只有某段对 input-to-damage p95 或 CPU 样本形成主要占比，才进入对应实现。

长期结构目标是“每个 native window 一个输入上下文，管理多个 Runtime surface”，不是强制把所有 pane 合成一棵巨树。该上下文统一拥有 `UiInputSequence`、pointer capture、focus/IME、double-click、timer deadline、popup/modal stack 和 surface z-order；每个 pane/template surface 仍可保留独立 tree/layout/render cache，通过稳定 `surface_id + generation` 注册。route 先在窗口级 z-order/index 选择 surface，再委托该 surface 的已发布 frame authority；Editor bridge 只提交业务 callback 和 target dirty，不再自行维护另一套 capture/hit 生命周期。

迁移必须分三步：先用 lower contract 证明多 surface topmost、capture、popup/modal、resize geometry barrier 和 physical pointer diagnostics；再让一个低风险 tool pane 通过窗口输入上下文，与旧路径做 route/result/damage parity；最后按 pane 域迁移并删除对应私有 dispatcher。迁移期间禁止双 dispatch 或同时写两份 hover/capture state，禁止以“兼容 fallback”长期保留两套 authority。

### 15.39 Unreal、Slint 与 Fyrox 的增量刷新对照

| 参考实现 | 本地源码证据 | 可吸收的原则 | 不直接照搬的部分 |
|---|---|---|---|
| Unreal Slate | `FHittestGrid::AddWidget` 在 paint 时缓存 geometry/render bounds/cell coverage；`GetBubblePath` 从单 cell 恢复 ancestor path；`FSlateInvalidationRoot` 以 widget proxy、reason 和 update heap区分 fast/slow path；application 统一 route、focus 和 active timer | frame publication 时建立 authoritative spatial/input index；目标 + reason 不丢失；窗口级输入 owner；slow path必须有显式原因和计数 | Slate 的宏观 widget/window对象模型、全套 prepass/paint 机制不能直接移植 |
| Slint | `DirtyRegion` 保持最多 3 个 rect，超限按最小面积代价合并；partial renderer 缓存 item geometry/dependency，属性变化用 old/new bounds 扩大 damage | damage 是有预算的小集合；几何/property dependency 与 generation绑定；full refresh只作显式 fallback | 固定 3 rect 是 Slint 的权衡，不应未经 Zircon damage coverage/profile直接采用 |
| Fyrox | widget 保留 `measure_valid`、`arrange_valid` 和上次 constraint；输入未变时早退；visual/measure/arrange invalidation 分离 | layout cache key必须包含约束与依赖 generation；dirty domain分离；重复测量应为零工作 | Fyrox 的全局 UI/message 组织不是 Zircon Editor 多 surface 输入边界的现成答案 |
| Godot Control | 已有审查证明 queue/redraw 与 control invalidation按 owner聚合 | redraw请求合并，避免 event-path直接重绘 | 不用 immediate-style redraw替换 retained frame authority |

这些参考实现没有支持“每次交互重建全部文本/全部命令/全部 hit entry”的结论，也没有支持“无界缓存一切”。共同模式是：稳定 owner、明确 generation、目标化 dirty reason、publication 时维护索引、event hot path只查询缓存、缓存 miss/full fallback可观测。Zircon 已经具备 arranged tree、hit grid、per-node render command bucket、text/SVG/GPU cache 和 damage region等基础件；首要工作是消除上游 identity widening、让多个 cache 共享同一 publication generation，并用 counter证明 fallback没有随事件数增长，而不是新增第四套缓存层。

### 15.40 非阻断优化顺序、压力矩阵与实现门

阻断中的 popup projected-grid 动态验证继续冻结；本节只定义可以独立完善的裁决顺序，不把静态 review 写成动态 GREEN。

1. **先建立 current-source mouse 分段证据。** 运行 source-bound 1000 click、1000 pointer 和 200 resize；补齐 input sequence correlation、resize event-to-snapshot-submit 与 last-event-to-debounced-reflow。先裁决 input-to-damage 或 damage-to-submit 哪一段主导，再选 owner。
2. **第一实现候选仍是 Editor target identity。** 15.31 已证明 targetless presentation 会在缓存前 widening。先迁移明确持有 `ViewInstanceId` 的 producer；要求相同局部操作的 Full/legacy 为 0，并对比 projection visits、command rebuild、damage coverage、CPU/RSS/GPU。
3. **只有 current-source 再现同步 SVG 或 presentation 长尾，才处理该层。** warmed run 中 SVG parse/miss 应随唯一 `(asset, generation, raster scale)` 数量而非事件数增长；GPU upload/allocation应随唯一 image generation增长而非 frame数增长。若 miss不增长而 CPU仍高，不能继续扩大 cache，应检查 scene conversion、text/layout、damage 或 present retry。
4. **窗口输入上下文是结构收敛，不是无证据的性能捷径。** 只有分段计数显示 bridge/dispatcher/dirty merge明显占比，或产品功能要求 capture/IME/timer统一时，才按 15.38 分 pane迁移。该工作不得与 layout/render算法重写同批进行。
5. **规模型问题按独立基准推进。** focus index、virtual row table、accessibility generation cache分别用专门 synthetic + product场景证明，不借用 click profile；timer和input batch继续由 `PERF-MVP-255`、`PERF-MVP-314` 管理，Runtime09不复制计划或实现 owner。

压力矩阵固定如下，所有结果都要记录 source manifest、硬件/窗口/DPI、warm-up、CPU、RSS、GPU timestamp、node/candidate/command/cache计数和 p50/p95/p99/max：

| 场景 | 规模 | 必须观察的复杂度合同 |
|---|---|---|
| 相同控件 idle hover | 1,000 moves | changed=false时 dirty transaction、damage、rebuild、present均为 0；event route不随 surface总节点数增长 |
| 控件边界与 pane 穿越 | 1,000 moves | 每次只更新 old/new hover owner；cell candidates、route depth与实际局部结构同阶 |
| 局部按钮/asset edit | 1,000 clicks | target identity连续；Full/legacy为 0；每次最多一个有界 view/shell transaction；SVG/GPU miss不随 click数增长 |
| native resize | 200 steps | snapshot build有界且 reuse为正；retained reflow按 debounce而非 step逐次发生；SVG parse/upload/allocation有界 |
| focus navigation | 1/100/10,000 focusable nodes | 记录 tree visits、candidate count、sort/alloc；目标查询不再稳定等于全节点数 |
| virtual list scroll | 1/100/10,000/100,000 rows | materialized rows与 viewport/overscan有界；range平移工作量接近 entered + exited，不扫描逻辑总行数 |
| timer idle/due | 0/10/1,000/10,000 armed | 无 due visited/alloc为 `O(1)`；due工作与弹出的 `K` 同阶并受预算约束 |
| accessibility action | 1/100/10,000 nodes | 同 frame generation不重复 extraction；单 action不构造完整 snapshot；语义/focus/relation parity保持 |

绝对延迟目标只作为同机工程预算，不伪称 Unreal/Slint 的实测值：60 Hz前台交互应以 successful submit 的 16.67 ms帧预算为上界，路由/invalidating 的 input-to-damage p95优先压到 1 ms量级，CPU侧 damage-to-submit p95预留在 8 ms以内；120 Hz模式按 8.33 ms总预算重新裁决。若 compositor/vsync使 successful submit跨帧，必须同时报告 CPU frame ready 与 present feedback，不能把二者混为一项。更重要的硬门是增长率：no-op工作为零，局部工作随 changed/candidate/visible 集合增长，cache miss随唯一 generation增长，任何 `O(N)` fallback都有明确 reason/counter且不会在1000次稳定交互中重复出现。

在这些证据出来前，明确不执行四类动作：不重写整个布局系统；不把所有 pane强并为一棵 surface tree；不新增平行 invalidation/input authority；不使用离屏整窗位图缓存掩盖 presentation、SVG 或 hit/layout 重建。下一轮实现只选择 profile 中占主导的一项，先写下层回归和复杂度 counter，再做单一结构修改并用同一场景前后对比。

### 15.41 滚动路径的确定性 O(N) 重建

滚动路径已经找到一个不依赖历史 profile 就能成立的结构问题。Runtime `UiTree::set_scroll_offset` 在 offset 真正变化时无条件写入 `layout + hit_test + render + input` dirty，只有 `visible_range` 会按窗口是否变化决定（`zircon_runtime/src/ui/tree/node/scroll.rs:40-53`）。即使保留现有 surface，增量布局也会从 propagated root 收集整个 subtree、为全部 visited node 复制旧 geometry、递归 measure/arrange，再逐项比较新 geometry（`zircon_runtime/src/ui/layout/pass/incremental.rs:55-101,140-161,243-262`）。因此“offset 变了但 virtual window 没变”目前仍可能把一个 viewport 的全部 materialized children带入布局工作。

Editor 又在这个基础上增加了更大的放大。`HierarchyPointerBridge::handle_scroll` 从 Runtime surface 读回 offset 后立即 `self.rebuild_surface()`（`hierarchy_pointer/handle_scroll.rs:21-26`）；该函数重新创建 `UiSurface`、`UiPointerDispatcher` 与 route map，逐个遍历 `layout.node_ids` 插入全部 row，最后执行 `surface.rebuild()` 并替换旧 authority（`hierarchy_pointer/rebuild_surface.rs:24-110`）。viewport 的 `virtualization` 还是 `None`，所以一次 wheel offset 变化的工作和逻辑 row 数 `N` 同阶，并伴随 node/path/string/map allocation、完整 layout/hit publication和 dispatcher状态替换。Asset content/tree/reference、menu和 welcome recent等 bridge也存在“scroll state变化 -> rebuild_surface”的同类形态；它们必须逐 owner迁移，但不能再复制一套临时 scroll cache。

这一结论也解释了为何“Runtime 已有增量 layout/hit grid”仍不足以让 Editor 可交互：Editor 在调用增量 machinery 前先销毁了稳定 node identity和上一帧 index。正确算法固定为：

1. `UiSurface`、dispatcher、route intent和 row node identity在 pane 生命周期内保持稳定。wheel只提交 scroll property transaction，不重新构造 surface。
2. visible window不变时跳过 measure；只更新 scroll transform、clip、visible entry frame以及 old/new damage。`input` domain只在 eligibility/route语义变化时 dirty，位置变化归 hit geometry patch，不应重建 callback/route表。
3. visible window变化时由一个 generation-stamped row table复用交集，只移除 exited rows、创建 entered rows；工作量与 `entered + exited + visible` 同阶，不与逻辑总行数同阶。Hierarchy 当前 `virtualization: None` 必须改为 Runtime authoritative virtual window或等价的稳定 materialized-row合同。
4. hit index在 frame publication 中一次性应用 scroll transform/clip；event hot path只查已发布 cell，不解释 layout tree。capture、hover、focus和IME继续引用稳定 node id，滚动不能通过替换 dispatcher隐式清空输入序列。
5. layout specialization首先区分 measure-invalid 与 placement-invalid。scroll offset通常只需要 arrange/transform；只有 content/constraint/font generation变化才重新 measure。若某容器确实需要 root-wide layout，必须记录 widening root、visited node count和原因。

Unreal `SScrollBox::OnMouseWheel` 只把 wheel交给现存 `ScrollBy`，根据 offset是否真正变化返回 handled；惯性 active timer只在滚动期间注册并在停止时注销（`dev/UnrealEngine/.../SScrollBox.cpp:845-890,1165-1181,1221-1249`）。它仍会发出 Layout/Volatility invalidation，但保留 widget tree、cached geometry和 invalidation proxy，由 `FSlateInvalidationRoot` 的 fast path处理 dirty proxy，不会每个 wheel重建 widget、dispatcher和 route table。可吸收的是稳定 identity + typed invalidation + 有界 active timer，不是照搬 Slate 的全部 prepass实现。

### 15.42 文本缓存的 authority 分裂与位置相关失效

Runtime 文本层并非无缓存：每个 `UiSurface` 有 measure/layout cache，shape run和 retained document还有独立 entry/byte预算及命中、冲突、淘汰工作计数。这里的剩余内存风险是 measure/layout主要按 entry数限制而不是 resident bytes，并且 cache随 surface实例存在；在 Editor 仍创建大量短命 surface时，RSS上界会随 surface数和单项文本体积放大。该风险必须由 per-cache resident bytes、surface count和 eviction timeline量化，不应凭 entry count直接改实现。

更明确的问题位于 Editor painter 的第二套 `PaintTextLayout` cache（`paint_text/draw/layout/cache.rs:11-70`）：

1. key拥有一份新的 `String`，每次 lookup都执行 `text.to_string()`；还包含绝对 `rect.x/y`。artifact定位又把 `rect.x/y` 加入 glyph origin，因此同一文本、字体、宽度和 shaping结果只要随滚动或 pane移动，就生成新 key并重新构建绝对坐标 glyph。
2. cache是 process-global `OnceLock<Mutex<HashMap<...>>>`。所有窗口和 painter在 lookup/store上共享一把锁；miss在锁外 build是合理的，但 hit、second lookup和insert仍串行。
3. 容量固定 2048，达到上限直接 `cache.clear()`，没有 LRU、byte budget、hit/miss/clear/resident-byte counter。稳定工作集略大于 2048时会形成“整表清空 -> 连续 miss/重建 -> 再清空”的锯齿抖动。
4. key同时包含 Runtime font generation和 host font cache key，说明 Runtime 已经拥有 shaping/font代际 authority；Editor 再按完整 text + absolute rect缓存最终 layout，形成平行且粒度不一致的 authority。

目标设计不是删除所有缓存，而是拆开“内容布局”和“屏幕放置”。Runtime继续拥有 text/style/constraint/font-generation 对应的 shaped/layout artifact；Editor painter只消费稳定 artifact id，并缓存确有必要的 host raster/placement派生物。glyph位置以局部 origin存储，draw时应用平移。为了保持现有 subpixel smoothing正确性，key保留量化后的 raster phase和 width/height constraint，而不是绝对 x/y。淘汰改为 indexed LRU + entry/byte双预算，不允许 capacity reached时全清；计数至少包括 lookup/hit/miss/build、resident entries/bytes、eviction visits/bytes、clear-all、lock wait以及因 position-only变化造成的 miss。只有 `hierarchy_scroll` / text-heavy pane profile证明该层主导 damage-to-submit或RSS，才实施这次 authority收敛。

参考实现支持这一分层。Unreal `FCachedShapedTextKey` 由 text range、scale、shaping context和 font info组成，不包含屏幕绝对坐标；measure和subsequence优先复用整段 shaped run（`dev/UnrealEngine/.../ShapedTextCache.h:13-50,53-98`，`ShapedTextCache.cpp:119-132,239-260`）。Slint 的文本缓存回归明确要求 unchanged second render为 0 miss，文本或字体属性变化才 miss（`dev/slint/api/rs/slint/tests/text_layout_cache.rs:32-98`）。Zircon 的验收也必须表达“位置变化复用内容布局，内容/约束/字体代际变化失效”，而不只是检查 cache长度。

### 15.43 SVG/GPU 驻留、饱和复杂度与多窗口 DPI 边界

本轮再次确认“SVG反复加载是因为完全没有 cache”和“GPU图像没有驻留”都不是当前源码事实。SVG query-path alias命中会在 canonicalize/stat/parse之前返回并记录 memory hit；raster cache有 4096项/64 MiB双预算和 source reverse index；WGPU presenter-local image cache与 device-shared registry均以 `(resource_key, generation)` 保留独立代际，容量256项/64 MiB，并导出 prepare hit、shared resolve/upload、GPU/CPU/shared resident bytes、allocation、upload write、admission reject和prune visits。暖交互若 generation稳定，不应重复解析、raster或 `queue.write_texture`。

仍需 profile裁决的资源风险是饱和路径而非基本命中路径：

| owner | 暖命中 | 饱和/失效成本 | 决策门 |
|---|---|---|---|
| visual raster | `BTreeMap` lookup后 clone shared pixels | 每次淘汰在全表 `min_by_key(last_used)`，最多扫描4096项且持有全局 mutex | 记录 lock wait、eviction scan和resident bytes；只有压力场景命中上限才换 indexed LRU |
| SVG tree | alias命中不读文件；miss有明确 parse counter | 1024项、无 byte budget；淘汰全表 min scan；冷 miss同步 parse | 暖 parse必须为0；若大 SVG导致RSS或同步 miss主导，再加 byte budget/后台 parse |
| local GPU image | stable local generation直接 hit，不访问 shared registry | 超出 entry/byte预算时扫描所有 entries、binary-search active sources并排序候选，约 `O(R log R)`，`R <= 256` | 现有 prune visits/reject/resident/upload计数足够先裁决，不先重写 |
| shared GPU image | 跨 presenter复用 device texture | registry mutex内做 admission/eviction；多窗口同代际可共享，代际并存会合法占用预算 | 多窗口 profile同时报告 shared resolve/upload/resident和每窗口local residency |

多窗口还有一个独立 correctness/scaling边界。`HostPaintThemeSnapshot` 把 `scale_factor` 放在 process-global `ArcSwap` authority中；任一 shell effective scale变化都会调用 `apply_host_paint_scale_factor` 并递增全局 generation（`paint_theme.rs:13-25,134-151,171-182`，`shell_metrics.rs:37-63`）。thread-local active snapshot能保证一次 paint读取一致 generation，但不能证明两个不同 DPI native window可以同时持有不同稳定 scale；后更新窗口可能使全局 generation和 raster/text key在窗口间抖动。当前产品若只有单 native window，这不是鼠标卡顿主因；多窗口里程碑前应把 design tokens/palette保留为进程共享，把 scale/window metrics移到 window/presentation generation，使移动一个窗口跨显示器只失效该窗口。该项必须用双窗口 100%/150% DPI交叉移动测试验证，不与当前 scroll/text优化同批实现。

Slint partial renderer提供了资源和几何代际的边界范例：per-item cached geometry/property tracker跨帧保留，geometry变化同时标记 old/new bounds；没有移动和property dirty时只注册 dependency，full screen refresh被标为 last resort（`dev/slint/internal/core/partial_renderer.rs:408-525,755-807`）。这再次说明 cache应由稳定 owner和generation驱动，full clear只能是显式、可计数的生命周期事件。

### 15.44 新增 hierarchy_scroll 证据合同与下一轮裁决

为避免在没有滚动数据时直接修改上述算法，本轮先在 `tools/ui-profile-capture.ps1` 增加 source-bound `hierarchy_scroll` 场景，在 `tools/ui-profile-native-resize.ps1` 增加原生 wheel storm。严格模式的合同如下：

1. `-AutoWheelCount 1000 -AutoWheelDelayMs 2` 在 `ui_profile_geometry.json` 发布的 `layout.left_region` 中发送滚轮；target identity固定为 `layout.left_region / pane_region / left`。fallback坐标只允许人工排障，`RequireScenarioEvidence` 必须拒绝。
2. wheel每32次切换方向，避免很快到达列表端点后把大量 no-op误当作滚动性能；artifact记录 requested/completed、delta、direction batch、elapsed、processor time、单核/系统 CPU以及 working set/private的 start/end/peak。
3. 1000/1000、geometry provenance和CPU/RSS自洽只是输入压力门；同 session仍必须有非零且单调的 input-to-damage、damage-to-submit p50/p95/p99/max，并消费 UI hotspot、GPU timestamp、damage和cache counters。
4. `hierarchy_scroll` 映射到产品现有的 `idle_hover` profiling namespace，但 wheel/pointer storm 的类型判定使用原始请求场景，避免同一批命令同时设置 pointer 与 wheel 数量时交叉误判。source manifest 当前绑定56个关键产品源码，除 hierarchy handler/rebuild、native scroll dispatch、`ui_perf`、Editor paint-text cache、Runtime scroll和incremental layout外，还覆盖 hierarchy算术路由与稳定authority、shell drag geometry patch及其dirty recompute入口；采集工具本身继续以四个独立 SHA-256指纹绑定。旧 binary若早于这些产品源码会 fail closed。
5. timeline新增五项低开销证据：`hierarchy_scroll_dispatch_count` 是埋点执行哨兵；`hierarchy_surface_rebuild_count`、`hierarchy_row_insert_count`、`hierarchy_dispatcher_rebuild_count`、`hierarchy_route_map_rebuild_count` 量化实际重建工作。严格门要求 dispatch与completed wheel完全相等、requested/completed wheel一致、surface/dispatcher/route-map重建计数一致，并允许优化后四项重建工作为0；因此“0重建”不能由埋点未执行或输入未送达伪造。
6. 新增观测合同经历三次有效RED，分别暴露缺失计数门、缺失manifest路径和多场景pointer/wheel误分类；最终 profile output suite `28/28`、native interaction suite `3/3`，合计31/31。三个Rust文件 `rustfmt --check`通过，三个PowerShell文件 AST均为0 error，56路清单无重复/缺失；本轮没有运行 Cargo，也没有触碰 popup projected-hit实现。

后续 current-source 产品命令在原 click/pointer/resize矩阵外增加：

```powershell
$env:CARGO_TARGET_DIR = '<coordinator-managed-target>'
.\tools\ui-profile-capture.ps1 `
  -ScenarioList material_lab_click,idle_hover,hierarchy_scroll,window_resize `
  -SkipBuild -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 60 `
  -AutoClickCount 1000 -AutoClickDelayMs 4 `
  -AutoPointerMoveCount 1000 -AutoPointerMoveDelayMs 2 `
  -AutoWheelCount 1000 -AutoWheelDelayMs 2 `
  -AutoResizeStepCount 200 -AutoResizeDelayMs 16
```

当前源码静态路径已经证明 changed wheel 会进入全量 `rebuild_surface`，但动态RED仍须由上述五项counter给出：`hierarchy_surface_rebuild_count` 应随changed wheel增长，`hierarchy_row_insert_count` 应按全部逻辑row累加，dispatcher/route-map重建应与surface重建相等，layout visited应接近materialized row数。2026-08-15对九个允许的受管Windows目标根目录做精确探测，没有找到任何 `zircon_editor.exe`，因此本轮没有运行旧二进制或把历史profile冒充current-source证据。

修复后的硬门是暖态 `hierarchy_surface_rebuild_count == 0`、dispatcher/route-map recreation为0、position-only text layout miss为0、SVG parse/raster和GPU image upload为0；layout measure为0或仅由真实 constraint/content变化解释，arrange/hit/damage工作与 visible/entered/exited rows同阶。交互预算沿用15.40：input-to-damage p95优先到1 ms量级，CPU damage-to-submit p95在8 ms以内，同时报告p99/max和RSS/GPU residency。若滚动profile证明 surface重建已归零而文本 cache miss仍主导，再实施15.42；若文本命中稳定而present长尾仍高，再裁决 damage/present，不把三个算法改动混在同一批。

### 15.45 Editor 私有 pointer surface 全量重建清单与迁移顺序

对 `zircon_editor/src/ui/retained_host` 中生产 `UiSurface::new`、`rebuild_surface`、`surface.rebuild()`、scroll state和virtualization路径的完整静态盘点表明，Editor当前同时存在“按逻辑集合全量重建”和“固定小集合增量重建”两类实现。二者不能只按函数名归为同一风险；真正需要优先迁移的是高频输入或resize触发且工作量随集合规模增长的owner。

| 优先级 | owner / 触发 | 当前工作量与证据 | 迁移边界 |
|---|---|---|---|
| P0 | shell drag authority / 每次完整dirty recompute，包含连续window resize | `sync_recompute_viewport_and_pointer_layouts` 每次无条件调用 `update_layout_with_workbench_layout_frames`，后者总是重新执行 `build_drag_surface`并替换surface、dispatcher、route map（`app/host_lifecycle/recompute_viewport.rs:60-79`，`shell_pointer/bridge.rs:142-164`）。基础为root + 8个drop/edge节点，每个浮动窗再增加5个节点，最后完整 `surface.rebuild()`（`shell_pointer/drag_surface.rs:35-123,316-389`），复杂度为 `O(1 + F)`，且即使输入authority未变也不跳过。 | 保留一个稳定surface与dispatcher；geometry-only resize只patch root/target frames并发布hit geometry，浮动窗拓扑只有增删或ID变化时才patch node/route。先加resize计数证明 rebuild/node insert归零，再评估Runtime layout widening。 |
| P0 | hierarchy / changed wheel | 每次offset变化重建 `2 + N` 节点、dispatcher和route map，`virtualization: None`；见15.41和15.44。 | 首个列表owner：稳定node identity + visible row window + scroll transform/hit patch。 |
| P0 | asset content/tree/reference / changed wheel | 三个bridge均在offset变化后调用 `rebuild_surface`；content重建 `2 + folders + items`（thumbnail为 `2 + items`），tree/reference各重建 `2 + N`，三者均 `virtualization: None`（`asset_pointer/content/bridge.rs:132-145,245-405`，`asset_pointer/tree/bridge.rs:103-116,176-258`，`asset_pointer/reference/bridge.rs:106-119,179-272`）。move本身已有算术row lookup，不需要全表hit rebuild。 | 复用hierarchy的同一Runtime scroll/virtual-window合同；不要为三个asset pane再造私有缓存。列表和thumbnail grid分别提供visible-range projector。 |
| P0 | menu popup / submenu hover、popup wheel、menu切换 | submenu path变化的mouse move会重建整棵menu pointer surface（`menu_pointer/host_menu_pointer_bridge_handle_move.rs:12-74`）；一次popup wheel可能先因offset重建，再因hover parent变化第二次重建（`host_menu_pointer_bridge_handle_scroll.rs:14-96`）。重建遍历全部menu buttons及每个打开层的visible items，并重建dispatcher/route map；可滚动root popup仍 `virtualization: None`（`host_menu_pointer_bridge_rebuild_surface.rs:31-160,177-290`）。 | popup open/close与submenu topology变化只patch受影响layer；hover只改state/paint，不改input topology；wheel只改root popup transform与已发布hit geometry。加入“单次事件最多一次publication”门。 |
| P1 | welcome recent / changed wheel | 每个offset变化重建surface/dispatcher/route map；每个recent item创建row、open和remove三个命中节点，合计 `2 + 3N`，且 `virtualization: None`（`welcome_recent_pointer_bridge_handle_scroll.rs:12-27`，`welcome_recent_pointer_bridge_rebuild_surface.rs:25-165`）。 | 在共享列表迁移完成后接入同一visible-row合同；每个materialized row保留三个稳定child identity。 |
| P1 | viewport toolbar template / 每次toolbar click | click入口在命中前无条件调用 `BuiltinViewportToolbarTemplateBridge::recompute_layout(surface_size)`（`app/viewport/toolbar_pointer/click.rs:19-31`）；该调用会重建template surface并重新投影host controls（`callback_dispatch/template_bridge/viewport_toolbar/bridge.rs:66-76`）。后续pointer bridge虽有 `(generation, surface_origin)` guard，但无法抵消前面的同尺寸重算。 | template bridge缓存last constraint/layout generation；同尺寸且document/style/font generation未变时直接复用已发布surface frame，click hot path只做hit lookup和dispatch。 |
| P2 | document tabs、drawer headers、host pages / layout sync与首次measured-frame click | 三者都按tab数重建surface/dispatcher/route map。sync有layout equality guard；click测量也只在frame真正变化时重建，所以不是每次move/wheel热路径，但首次点击某tab仍可能先做 `O(T)` 重建（`document_tab_pointer/*rebuild_surface.rs:21-136`，`drawer_header_pointer/rebuild_surface.rs:18-101`，`host_page_pointer/rebuild_surface.rs:20-130`）。 | measured frame直接patch对应tab/close geometry；集合变化才增删node/route。等P0 profile后迁移。 |
| P3 | activity rail、shared viewport、resize surface、detail scroll | activity rail只在layout变化重建小规模tab集合；shared viewport为2个节点；resize surface为root + 3个splitter节点且已原位改frame、仅changed时调用Runtime rebuild；detail scroll为root + viewport两个节点，wheel只读回offset而不重建（`detail_pointer/handle_scroll.rs:10-34`）。 | 保持现有guard；后续只把geometry-only `surface.rebuild()`收敛到明确的layout/hit patch。不得与P0列表或shell drag同批扩大变更。 |

据此，下一轮不应从SVG或文本cache先猜测主因。实现顺序固定为：`shell drag resize authority -> hierarchy scroll -> asset list family -> menu popup -> welcome recent -> viewport toolbar click -> tab/header测量`。每个owner先补“事件数、surface rebuild、node insert、dispatcher/route rebuild、layout visited、hit patch、publication count”证据，再实施一项迁移并用同一场景比较；不允许一次提交同时重写scroll、text、GPU image和popup authority。当前没有current-source Editor可执行文件，因此该表是可复核的静态复杂度与触发频率结论，不是动态耗时排名。

### 15.46 shell drag resize authority 静态候选

本轮已按15.45的首项形成独立候选，范围限定在 shell drag pointer authority，不包含popup、文本或GPU资源实现：

1. `HostShellPointerBridge` 现在以ordered floating-window IDs判定拓扑。ID数量与顺序不变时调用 `patch_drag_surface`，原位更新root、8个基础target和每窗5个浮动target的frame/input state；只有拓扑变化、稳定节点缺失时才重新创建surface、dispatcher和route map。测试专用authority generation证明连续resize与同ID浮动窗移动不替换authority。
2. drag callback不再捕获构建时的陈旧 `DragTargetFrames`/floating frame；它们共享一个 `ArcSwap<DragHitGeometry>`，publication完成后原子替换不可变几何快照。side/document edge以及floating edge回调均读取同一代几何；同ID浮动窗回归同时要求旧位置拒绝、新attach位置命中、新edge位置命中。
3. build会为当前拓扑中的不可见浮动窗保留 `Ignore` 节点，使“projection暂缺 -> geometry恢复”仍能走稳定patch，不因可见性切换重建callback/route authority。frame快照使用固定字段加每窗单一 `Vec<Option<UiFrame>>`，避免每次resize为13类node构造 `BTreeMap`。
4. `update_target_node` 改为先通过只读 `tree.node` 比较；只有frame/clip/input/state实际变化才调用 `node_mut`。这消除了原实现“函数返回false但已经把节点登记为mutation”的隐性dirty放大，resize surface也同步受益。
5. TDD先建立两条Rust回归的源码RED（测试引用5次、生产定义0次），实现后源码门为测试引用5次、唯一生产定义1次。8个Rust路径 `rustfmt --check`为0，scoped `git diff --check`为0；profile output contract `28/28`、native resize contract `3/3`，56路source manifest无重复/缺失。
6. `window_resize` timeline新增 `shell_drag_authority_rebuild_count`、`shell_drag_node_insert_count`、`shell_drag_geometry_patch_count`、`shell_drag_node_patch_count`、`shell_drag_dispatcher_rebuild_count`、`shell_drag_route_map_rebuild_count`。严格门要求geometry patch为正、node patch不少于geometry patch，且authority/node insert/dispatcher/route-map rebuild全部为0；输出合同已分别证明authority rebuild非零或geometry patch缺失时会fail closed。首次拓扑realization不计作rebuild，后续稳定拓扑fallback才进入重建计数。

该候选尚未运行Cargo，也没有current-source Editor动态profile，不能宣称resize p95已经达标。受管lane释放后，先跑两条focused Rust回归和既有retained drawer/tab drag集合，再构建Editor并执行200-step native resize；硬门是稳定topology期间authority generation/recreation为0、旧/新geometry路由正确、input-to-damage与damage-to-submit采样完整，最后才比较CPU/RSS与p95/p99/max。

### 15.47 hierarchy pointer 常数authority与算术行路由候选

15.41确认的滚轮 `O(N)` 不只是Runtime layout问题：hierarchy绘制端已经通过 `visible_hierarchy_row_range` 把paint约束到可见行，但私有pointer surface仍为每个逻辑行创建一个UiTree节点、一个dispatcher callback和两个HashMap条目；每次有效wheel又重新创建整套authority。10,000行时，命中侧为10,002个节点，单次滚动重做 `O(N)` 节点/arranged/hit/route工作，而绘制侧只处理约 `O(viewport_height / row_pitch)` 行，两端算法不一致。

本轮候选按“私有命中authority不复制可算术集合”收敛：

1. hierarchy pointer surface固定为root + viewport两个节点；逻辑行不再进入UiTree、dispatcher或route map。pane尺寸变化只原位patch这两个frame并重建常数规模surface，node ID列表、hover、scroll和主题row metrics变化均不替换surface/dispatcher/route authority。
2. move/down/scroll先由viewport完成边界与输入路由，再用 `floor((point_y - viewport_y + scroll_offset - row_y) / row_pitch)` 求候选index，显式拒绝横向inset、行间gap、非有限坐标和越界index，最终只做一次 `node_ids.get(item_index)`。因此单事件命中复杂度与逻辑行数无关，为 `O(1)`；paint继续保持现有 `O(V)` visible-range。
3. viewport注册Scroll handler，阻止Runtime默认scroll把无行节点的私有surface标记为全窗口layout/hit/render dirty；bridge直接累加并clamp唯一scroll state，随后用新offset投影鼠标下方行。scroll后不需要额外sync即可点击同一远端行。
4. Rust回归使用10,000个逻辑行，要求surface节点数恒为2、pane geometry和大幅scroll均不推进authority generation、scroll后立即命中超过第1,000行、click与scroll route一致、row inset仍返回ListSurface。源码RED为两个观测接口定义0且存在一个全量行插入循环；实现后两个接口各唯一、行插入循环/行节点路径/scroll rebuild均为0，算术公式与单次indexed lookup各唯一。
5. `hierarchy_scroll` profile门从“重建计数彼此一致即可”提升为目标算法合同：dispatch必须等于已完成wheel，`surface_rebuild/row_insert/dispatcher_rebuild/route_map_rebuild`四项必须全为0。有效RED证明旧fixture的1000次authority重建和50,000行插入仍会假绿；GREEN output contract为 `28/28`。source manifest增加click/move/Scroll guard/route projector/sync五个owner文件，总计56路且无重复/缺失。

该候选目前只有TDD源码RED、静态复杂度门和PowerShell合同证据，尚未得到Rust编译或current-source产品profile。受管lane授权后必须先运行新增10,000行回归与既有hierarchy selection/scroll集合，再用1000次交替wheel验证四项重建为0、input-to-damage与damage-to-submit样本完整，并比较优化前后的CPU、RSS、p50/p95/p99/max；在此之前不得把 `O(1)` 源码结构推断写成动态达标结论。

### 15.48 asset pointer family 常数authority候选

继续审查 Asset Browser tree、content list/thumbnail 和 reference list 后，确认绘制侧与命中侧原先也存在算法分裂。绘制元数据已经通过visible row range及sorted scroll group的 `partition_point` 把列表paint限制为 `O(log G + V)`，thumbnail geometry也可由列数和scroll offset直接求item index；pointer move路径已经具备同类算术投影。但down/click仍为每个逻辑条目创建UiTree node、dispatcher callback和 `BTreeMap<UiNodeId, Target>`，每次有效wheel又重建完整surface。10,000条数据时，滚轮前后会分别承担 `O(N)` 节点构造、路由表分配、arrange和hit-grid构建，而屏幕实际只显示少量条目。

本轮候选把三类私有pointer surface统一到一个 `AssetPointerSurfaceAuthority`：

1. 每个surface只持有root + viewport两个节点以及一个固定dispatcher。内容集合、hover、selection和scroll offset变化不再改变命中拓扑；pane/profile导致的root或viewport geometry变化只原位patch两个frame，再做常数规模surface rebuild。
2. viewport显式处理Move、Down和Scroll，避免Runtime默认ScrollableBox在wheel热路径写入私有surface dirty state。bridge只对有限delta累加并clamp唯一scroll offset，随后使用新offset投影当前鼠标下的条目；滚动后不需要再次sync。
3. list/tree/reference共享一个带finite、正stride、gap和bounds检查的row-index函数。content thumbnail继续复用 `AssetThumbnailGridMetrics::item_index_at_point`；reference在算出index后仍检查 `known_project_asset`，未知条目保持ListSurface语义。
4. click/press仍先经过viewport的UiSurface边界、clip和dispatcher合同，再做一次 `Vec::get`/indexed lookup生成业务route；因此没有绕过窗口命中边界，也不需要为可算术集合复制第二份节点authority。逻辑集合变化为 `O(1)` surface工作，单次move/down/wheel命中为 `O(1)`，内存从 `O(N)` pointer nodes/callbacks/map entries降为固定两节点，业务字符串集合仍由既有layout snapshot持有。

参考实现边界与本地 Unreal Slate 一致但更适合当前固定行高数据。`STableViewBase::Tick` 只在refresh/scroll/geometry条件成立时调用 `ReGenerateItems`，`GetNumGeneratedChildren`返回的是ItemsPanel中已生成child数量，而不是数据源总量；`STileView::ScrollBy`按 `GetNumItemsPerLine` 和tile scroll-axis算术更新offset。Zircon当前Asset绘制已经虚拟化，因此本轮没有再建立一套可见widget池，而是消除仅为命中复制全量逻辑条目的私有树。

TDD先记录源码RED：新增测试引用固定authority观测接口9次，而生产定义为0；三桥接器仍有5个逐项循环、12个BTreeMap引用和6个运行期 `rebuild_surface` 调用。实现后的静态门为：三桥接器逐项循环0、BTreeMap引用0、row-node ID引用0、scroll/full rebuild入口0；10,000项回归要求三个surface节点数均为2，大幅wheel后不再次sync即可命中第5,000项并保持click route一致，unknown reference仍命中ListSurface，且三类authority generation均不推进。相关Rust文件 `rustfmt --check` 与scoped `git diff --check`均为0。

source-bound profile manifest新增shared authority和三个bridge路径，从56路增至60路；output contract显式验证四个路径均存在，防止旧Editor二进制冒充该候选的动态结果。当前仍未运行Cargo，也没有current-source Editor，所以本节只确认复杂度结构和静态合同，不能声称CPU、RSS或交互p95已经下降。lane授权后的顺序固定为：focused 10,000项回归 -> 既有tree/content/thumbnail/reference产品路由集合 -> current-source Editor asset wheel/click场景；动态门必须同时报告pointer surface rebuild、node insert、dispatcher/map rebuild、input-to-damage、damage-to-submit、CPU和RSS，并要求稳定数据集wheel期间前四项全为0。

### 15.49 menu popup 分层authority与缓存投影候选

menu popup 的旧路径比15.45的初步判断多一个隐藏的线性成本。打开Window菜单后，Runtime ScrollableBox先改私有popup node的scroll state；bridge读回offset后调用 `rebuild_surface`，为所有可见逻辑菜单项重新创建node、dispatcher callback和route-map条目，再额外派发Move。若新hover使 `open_submenu_path` 改变，同一wheel结尾还会第二次重建。与此同时，`popup_grid_layout`/scroll clamp通过 `menu_items_for_layout`解析动态Window菜单；当layout没有显式menus时，每次调用都会重新分配含全部preset action string的Vec。因此10,000项菜单单次wheel不只是 `O(N)` UiTree重建，还含重复模型构造和字符串分配。

本轮候选把authority拆成稳定边界节点与缓存数据投影：

1. UiSurface继续保留真实top-level menu button、dismiss overlay以及每个已打开popup layer节点，因而clip、层级、dismiss和submenu topmost仍由Runtime命中权威决定；popup layer内部不再为每个逻辑item复制节点。surface节点数从 `root + buttons + dismiss + layers + items` 降为 `root + visible buttons + dismiss + open layers`，与item count无关。
2. 打开菜单或layout generation变化时，一次性缓存解析后的 `popup_items`，同时构建 `HashMap<Vec<usize>, flattened_index>`。pointer事件不再调用 `menu_items_for_layout`，也不再为动态preset actions重新分配字符串；扁平索引查找为按path哈希的摊销 `O(depth)`，避免旧 `menu_item_route_index` 对前置兄弟子树的线性扫描。
3. popup node显式处理Scroll，阻止Runtime默认ScrollableBox写入surface dirty。root popup wheel只累加并clamp bridge中的唯一offset，再用 `column=floor(local_x/column_width)`、`row=floor((local_y+scroll_offset)/row_step)`求item，显式拒绝padding、row gap、非有限坐标和越界index。scroll不会重建surface，也不需要第二次UiSurface Move dispatch。
4. projector先递归检查当前open submenu path的最深popup，确保重叠层仍按topmost命中；root popup只在point不属于任何child popup时接受wheel。branch hover改变popup层级时仍重建，但工作量为 `O(button_count + open_depth)`；menu-bar横向滚动仍因clipped button集合可能增删而重建小规模button authority，未与大popup优化混淆。
5. disabled item继续投影为PopupSurface；enabled branch保留SubmenuBranch path；leaf从缓存模型克隆action id并保持历史preorder flattened index。Down/Move仍先通过UiSurface layer边界再投影，没有引入绕过clip/dismiss的产品调用特例。

TDD源码RED为：大菜单测试引用authority观测接口4次而生产定义0，rebuild文件仍有1个逐item插入循环和2个item-node引用，scroll函数有3个full rebuild入口。实现后item node/route ID引用0、逐item插入循环0、事件期动态menu build调用0；10,000 preset回归要求surface节点数仅为 `button_count + 3`，一次150,000px wheel后authority generation不变、hover直接到flattened item 5,002，并在不sync的情况下点击得到 `alpha-5000` action。popup model和path-index cache只在open menu/layout变化时构建。

manifest再加入8个menu authority/projector路径，从60路增至68路。当前候选仍只具备源码TDD、复杂度门和PowerShell采集合同，不具备Rust编译或current-source动态数据。lane授权后先跑pointer_bridge现有的single-column、multi-column、right-edge clamp、nested flip、disabled/dismiss和新增10,000项集合；产品profile必须记录单wheel publication count、surface rebuild、popup item node insert、model cache build/hit、input-to-damage、damage-to-submit、CPU和RSS。稳定flat popup wheel的硬门为surface rebuild/item insert/model rebuild均为0；submenu path真实变化允许一次layer publication，不允许同一事件两次。

### 15.50 Welcome Recent 固定authority与产品滚轮证据候选

Welcome Recent原实现为每个recent project建立row、Open与Remove三个pointer node，并在有效wheel后重建完整surface、dispatcher和route map。该路径位于Editor首屏，即使常见列表较短，也会在恢复大量历史项目或压力数据时把一次输入放大到 `O(N)` 节点、字符串route和hit-grid工作；更重要的是，它与已经按visible range绘制的列表形成两套不一致的规模模型。

本轮候选把私有surface固定为root + viewport两个节点。Move、Down、Scroll先经过Runtime viewport边界和clip，再以两行规范几何得到row pitch，使用一次 `floor` 与一次indexed project-path lookup投影row/Open/Remove route；横向边界、row gap、非有限坐标、零尺寸frame和越界index均显式拒绝。scroll只更新并clamp bridge中的唯一offset，随后用新offset投影当前指针，不调用sync或重建authority。pane尺寸或metrics变化只patch两个固定节点的geometry；项目集合、hover和scroll变化不再复制逻辑条目到UiTree。

10,000项目回归的源码合同要求surface节点数恒为2，大幅wheel后不再次sync即可命中第1,000项之后的Open/Remove，旧位置与row gap拒绝，frame-path与业务route保持一致，authority generation不推进。埋点同时纠正了一个旧归属错误：`hierarchy_scroll_dispatch_count`不再在全局native scroll入口递增，而只在真实hierarchy owner记录；Welcome owner独立记录dispatch、surface/authority rebuild、row insert、geometry patch、dispatcher与route-map rebuild。严格 `welcome_recent_scroll` 场景只接受 `welcome.recent.viewport / welcome_recent_viewport` 的发布geometry，要求dispatch与完成wheel完全相等，并要求六类retained-authority工作全部为0；比例坐标只能排障，不能通过evidence gate。

profile geometry schema升至2并发布实际Welcome viewport frame，critical-source manifest从68扩到80；PowerShell AST、scoped rustfmt/diff检查通过，output contract最终 `29/29`。该节仍没有Rust编译和current-source产品profile，不能声称CPU/RSS或p95已经下降。lane授权后先跑10,000项目下层回归与既有Welcome点击/移除/滚动集合，再执行1000次交替wheel，报告input-to-damage、damage-to-submit、CPU、working set/private bytes与上述七项counter。

### 15.51 Viewport toolbar 发布时布局与点击时缓存绑定候选

15.45记录的无条件click-time `recompute_layout(surface_size)`不仅重复做template surface dirty rebuild和host projection，还掩盖了一个多pane authority错误：单个 `BuiltinViewportToolbarTemplateBridge`依次为document、side dock和floating pane生成不同尺寸的toolbar frame，其 `host_projection`最终只保留最后处理尺寸；旧点击路径必须重排到当前surface尺寸，才能让命令绑定再次与被点击pane一致。这使布局状态成为命令分发的隐式前置条件，也让同尺寸重复点击承担与输入本身无关的layout/project成本。

修复后，命令绑定在bridge构建时按 `(control_id, UiEventKind)` 建立两级 `BTreeMap`，其生命周期与authored projection一致，不再依赖最后一次responsive layout。每个pane的geometry仍只在presentation publication阶段由对应尺寸的host projection生成 `Arc<UiSurfaceFrame>`；click入口读取已提交presentation中的frame，pointer bridge按 `(surface generation, surface origin)`跳过重复control投影，随后只做hit lookup和route dispatch。click hot path中的template `recompute_layout`调用清零，跨pane尺寸不会再通过临时改写共享host projection修复绑定。

TDD源码RED为重复点击测试引用 `layout_recompute_count`两次而生产定义为0，且click入口仍有1次无条件recompute；实现后计数接口唯一、click recompute调用0、稳定binding cache引用唯一归属。新增 `viewport_toolbar_click` profile场景只选择 `ui_profile_geometry.json` 中kind为 `viewport_toolbar_control` 的控件，错误kind或ratio fallback均fail closed；9个toolbar click/pointer/publication owner加入critical-source manifest，总量89、0缺失、0重复，output contract `29/29`通过。Rust scoped `rustfmt --check`和`git diff --check`通过，但当前没有Cargo或真实Editor动态结果。

受管lane恢复后先跑bridge binding-cache与root repeated-click回归，再跑现有projection fallback/surface-generation集合，最后构建current-source Editor执行 `viewport_toolbar_click` click storm。验收必须同时证明事件全部完成、target provenance正确、重复click不推进template layout recompute、input-to-damage与damage-to-submit percentile完整，并报告CPU/RSS；如果点击后的业务状态真实改变布局，允许后续publication产生一次新frame，但输入命中前不得先重排。

### 15.52 Tab/Header 测量几何的局部发布候选

Document Tab、Drawer Header与Host Page此前虽然会在测量frame完全相同时短路，但第一次收到paint/callback提供的真实 `tab_x/tab_width` 时都会重新创建整个 `UiSurface`、dispatcher和route map。Document/Drawer的重建工作量为所有surface tab总数 `O(T)`；Host Page除首次变化全量重建外，每次点击还通过 `.position(|tab| tab.page_index == item_index)`线性查找目标。该行为把一次已知目标的点击变成输入前布局权威重建，也是“相同点击暖态较快、窗体缩放或首次交互仍卡”的确定性来源之一。

本轮候选先补Runtime下层 `UiSurface::rebuild_authored_frames(root_size)`合同。它一次发布节点中已有的显式frame，同时记录root-size基线、建立dirty索引、提交初始invalidation并清空dirty；否则这些轻量pointer surface从 `rebuild()`切到 `rebuild_dirty()`后，首次事件仍会因未知root size与残留structure/layout dirty退化为全树扫描。下层回归要求后续移动一个leaf时layout/arranged/hit/render访问计数均为1，旧位置拒绝、新位置命中。

三组桥接的完整layout/topology变化仍重建authority，但发布入口统一使用authored-frame基线，并把tab父节点标记为 `ParentDirected`，使leaf layout invalidation不向父级扩散。单个测量变化只写目标节点的fixed constraints与parent-local position，再调用一次 `rebuild_dirty`；不得提前覆盖 `layout_cache.frame`，否则Runtime无法从重建前后frame差异生成hit geometry patch。Document/Drawer对变化tab之后连续的未实测fallback区间做局部后缀更新，遇到下一个已实测tab立即停止，保持旧 `next_x`语义但不重建其它surface、dispatcher或route map；节点缺失才显式回退完整重建。Host Page新增 `item_index -> tab_position`索引与独立measured-frame cache，目标查找和几何patch均为 `O(1)`，后续layout相等比较不再被点击时写回的frame污染。

TDD静态RED为三条回归引用authority generation 9次、生产定义0次，且三个measurement入口各有一次无条件full rebuild；Runtime authored-frame测试引用新API一次、定义0次。实现后的源码门为三个generation accessor唯一、三个dirty rebuild入口唯一、提前frame写入0、Host Page线性 `.position()` 0、三处authored publication与三处parent boundary齐备。19个Rust路径 `rustfmt --check`与scoped `git diff --check`均为0。当前尚未运行Cargo或current-source Editor，因此不能宣称这些回归已编译，也不能把静态复杂度变化写成CPU/RSS或p95改善；lane授权后必须先跑Runtime authored-frame下层回归，再跑三组产品桥接与大tab集合压力测试，最后用首次点击、重复点击和resize后首次点击分别比较authority rebuild、visited nodes、input-to-damage及damage-to-submit。

### 15.53 无 locator 运行时纹理错误清空 SVG/GPU 缓存

15.43确认的三层图像缓存虽然存在，但 `refresh_project_assets` 还有一条会让暖缓存周期性失效的错误放大路径。`visual_asset_cache_refresh` 原先把任意 `ResourceKind::Texture` 且新旧 locator 都为空的资源事件直接分类为 `All`；应用该分类时会同时清空sprite-atlas解析、SVG tree、visual raster以及editor icon atlas。无 locator 的运行时纹理没有路径，无法与这些file-backed cache中的任何source dependency建立对应关系；因此viewport或其它运行时纹理换代一旦进入该事件流，就可能把不相关的稳定SVG重新推入candidate discovery、parse、raster、atlas publication与GPU upload。该问题不是“没有GPU cache”，而是上游失效域把缓存持续清空。

本轮修复只删除这条无依赖依据的全失效条件，并保留三个必要边界：显式命中 `editor-sprite-atlases` 的source/product仍执行 `All`；带locator的SVG/bitmap事件仍经reverse source index做 `Paths`定向失效；资源流出现generation lag时仍执行content-fingerprint `Reconcile`，只删除内容确实变化的resident source。普通无 locator 运行时纹理事件因此为 `None`，但它仍继续参与既有asset backend refresh planning，本修复没有吞掉资源事件或改变运行时纹理生命周期。

TDD先把旧行为固定为源码RED：单独无 locator Texture要求 `VisualAssetCacheRefresh::None`，同一事件叠加 `resource_generation_lagged`要求 `Reconcile`，而生产谓词仍优先返回 `All`。实现后 `events_contain_unlocated_texture`和组合全失效条件均为0处，两条回归仍在；`refresh.rs`的scoped `rustfmt --check`与`git diff --check`均通过。当前没有Cargo或current-source Editor动态验证，因此只确认失效算法已收窄。lane恢复后的产品门是在持续无 locator runtime texture churn与1000次hover/click下，`visual_asset_full_invalidation_count == 0`、暖态SVG parse/raster与GPU image upload均为0，同时cache hit、resource event处理和viewport更新继续前进；显式修改一个SVG时必须只推进该source的targeted invalidation与新generation upload，修改sprite atlas时仍允许一次完整atlas失效。

### 15.54 事件循环错误轮询原生窗口状态

继续沿鼠标热路径审查时确认，pointer dispatch、redraw coalescing和pane命中本身都已避免无变化全量工作，但 `UiHostWindowEventLoop::about_to_wait_impl` 原先在每个事件批次结束前无条件调用 `sync_host_window_state`。该函数依次调用原生窗口的 `surface_size()`、`scale_factor()`、`is_maximized()`与`outer_position()`；高频鼠标移动会因此把一次纯缓存命中额外放大为四类winit/平台边界查询。这里没有随UI节点数增长，却会为每个事件批次增加固定的原生调用延迟，并且发生在所有按钮、hover和drag路径之外，容易造成“各组件都已局部更新但整窗仍迟钝”的共同底噪。

当前状态权威已经具备事件驱动闭环：窗口创建成功后仍执行一次 `sync_host_window_state` 初始化；之后 `SurfaceResized`更新physical size，`ScaleFactorChanged`更新DPI，`Moved`更新position，close路径更新visibility。`window_maximized`当前没有产品读取方。因此本轮删除 `about_to_wait`中的重复同步，不改创建时初始化和各WindowEvent更新。新增源码合同明确禁止该热路径重新引入 `sync_host_window_state`或直接调用上述原生查询；TDD等价静态断言在修改前命中RED，修改后为GREEN。

该修复只能静态证明每个事件批次从4次原生状态查询降为0次，尚不能宣称输入p95改善。lane释放后的动态验证应在1000次无状态pointer move、按钮hover和drawer drag中比较事件处理CPU、input-to-damage p50/p95/p99/max，并确认窗口move/resize/DPI变更后缓存metrics与实际WindowEvent一致；创建后的初始size/scale/position也必须保持正确。若平台存在不发送对应WindowEvent的状态变化，再为那个明确状态补低频或事件专用同步，不得恢复所有输入批次的全量轮询。

### 15.55 Runtime presenter pending 期间反复销毁 GPU surface

`RetainedViewportPresenterFactory::create`依赖异步解析的RenderFramework：第一次调用 `render_framework()`会提交resolve job并返回“still starting”。旧启动流程随后创建standalone GPU presenter作为fallback；但 `about_to_wait`每个事件批次都执行升级函数，而升级函数在判断factory是否ready之前先 `drop(self.presenter.take())`。pending job因此导致当前可用GPU surface被销毁，升级返回None后又重新创建standalone surface并强制redraw；下一批次继续重复。该行为与UI节点数无关，却能把任意pointer/keyboard事件放大为GPU device/surface资源生命周期、窗口surface重建和全帧present，是本轮确认的P1公共卡顿根因。

修复把依赖解析与destructive handoff分离。`RuntimeUiSurfacePresenterFactory`新增无surface副作用的 `poll_ready`，viewport实现只轮询/启动既有RenderFramework resolve job；event loop在pending时保留当前presenter，只设置50ms poll deadline并把它并入 `ControlFlow::WaitUntil`。只有ready后才释放standalone surface并尝试一次runtime presenter创建；成功进入shared active，失败记录诊断并通过明确的standalone factory恢复可用presenter，同时以attempted状态阻止后续输入批次重试。初始factory路径也先poll readiness，不再用一次必然失败的surface create充当ready probe。

源码TDD先证明旧实现缺少readiness poll、once-only gate和bounded deadline；实现后静态合同要求 `factory.poll_ready()`严格位于drop之前，旧无条件helper为0，唯一trait实现完整，pending/attempt/success/fallback均有profile counter。source-bound manifest加入event-loop state、handoff helper、factory trait与viewport实现四个owner，从89路增至93路。scoped rustfmt/diff与PowerShell output合同通过后仍只能确认状态机结构；lane释放后的故障注入必须让RenderFramework保持pending至少1000个pointer events，要求presenter create/drop/upgrade attempt不随事件增长、pending poll不超过约20Hz、fallback持续present。随后验证ready只产生一次handoff，create failure只产生一次fallback恢复，并比较CPU/RSS与input p95/p99/max。

### 15.56 后台事件状态合并未合并原生 wake

`HostEventLoopWake`原先只在消费侧用 `AtomicBool::swap(false)`合并pending状态；发布侧却在每次callback中无条件 `store(true)`、获取proxy mutex并调用 `EventLoopProxy::wake_up()`。因此同一批10,000条资产或任务事件虽然最终只产生一次maintenance frame update，仍可能向winit/原生事件队列提交约10,000个proxy wake；首个事件消费pending后，其余事件大多为空操作，但依然与pointer、keyboard和resize事件竞争队列与调度时间。这解释了为什么上层frame/redraw已经合并时，后台事件风暴仍可能让按钮和鼠标响应出现长尾。

修复把发布侧改为原子边沿门：只有 `requested` 从false变为true的调用才获取proxy并发送native wake；pending尚未消费时的后续callback直接返回。消费后下一次发布仍会建立新的边沿并正常唤醒，proxy尚未安装时也继续由 `install_proxy`检查pending并补发，因此没有丢失启动期通知。单个pending epoch的原生wake、proxy lock与队列写入从 `O(P)` 收敛为 `O(1)`，其中 `P`为该epoch内发布次数；消息mailbox与maintenance frame语义不变。

TDD先加入纯原子合同，要求第一次发布返回signal、重复发布不signal、消费后再次signal；实现前源码RED证明生产边沿helper缺失，随后唯一生产定义与调用通过rustfmt、scoped diff和source guard。source-bound profile manifest加入 `event_wake.rs`，关键源从93增至94且0重复；PowerShell profile输出合同 `29/29`通过，临时输出限定在E盘。本节没有运行Cargo，也没有current-source产品风暴数据。受管lane释放后应增加10,000次同epoch后台publish压力，要求native proxy wake数接近1而非publish数、maintenance frame仍完成、消息全部被后续tick消费，并同时报告pointer事件延迟、CPU与队列长尾。

### 15.57 damage paint 索引重复分配与排序

继续审查普通GPU present后，需要先修正一个容易误判的结论：damage frame虽然仍从 `record_host_frame_commands` 进入workbench根 painter，但当前实现已经在顶层按dock/top chrome/floating/extension区域短路，在template-node入口用 `HostWorkbenchHitIndex` 选取候选row，并在node clone、命令生成、文本布局和图像preview raster之前拒绝区域外节点。正常present也只借用 `HostPresentationGeneration` 的 `Arc` structure并进入paint scope，不会每帧 `materialize()`完整presentation。因此当前damage CPU路径不是简单的“全UI重新生成后再裁剪”。

真正仍位于高频路径的是 `HostTemplateNodePaintIndex::rows_for_clip`。旧实现对每次局部查询都创建 `HashSet`与 `Vec`，收集cell候选后按 `(z_index, row)`重新排序；即使damage完全落在单个64px cell内，也承担 `O(K)`去重和 `O(K log K)`排序。clip覆盖完整索引边界时还会重新构造 `0..N` row并执行 `O(N log N)`排序。这里的paint order只在geometry/z membership变化时改变，却在每次hover/click damage中重复计算，和Unreal invalidation fast path保留稳定widget sort order、只处理dirty proxy的原则不一致。

本轮候选把排序移到 `HostTemplateNodePaintIndex::new`：一次生成完整 `paint_order_rows`，并把各cell已有bucket按同一 `(z_index, row)`原地排序。单cell query现在直接克隆已排序bucket，查询复杂度降为 `O(K)`且不创建 `HashSet`、不排序；完整clip直接克隆预排序row，降为 `O(N)`且不排序。跨cell query仍保留原有去重和全局排序，避免同一row跨cell重复并保持严格paint order。索引构建增加一份 `N * usize` 的row-order数组，并把原本每次查询发生的排序前移到generation构建；现有semantic-only rebind只有在frame、clip、z和dispatch membership不变时才复用索引，所以不会让缓存顺序过期。

TDD回归使用同cell内乱序z-index节点，要求单cell与完整clip都返回 `[front, middle, back]` 对应row顺序，并证明查询期sort计数保持0；生产路径新增 `ui.paint_index.full_order_reuse_count`、`single_cell_order_reuse_count` 与 `multi_cell_sort_count`，而 `WorkbenchPaintIndexCandidateCount`、`TemplateNodeVisitCount/CloneCount/DamageRejectCount`继续量化实际候选。源码RED先证明build-time reuse probe缺失，随后rustfmt、scoped diff与no-query-sort source guard通过。`index.rs`原本已属于94路critical-source manifest，不需要扩大清单。本节未运行Cargo，也没有current-source CPU/p95数据；lane授权后必须先跑该底层回归，再在1000次稳定hover/click和小damage滚动中要求single-cell reuse推进、multi-cell sort只随真实跨cell damage推进，并报告candidate count、damage-to-submit、CPU及RSS，不能把静态 `O(K log K) -> O(K)`写成已测收益。

### 15.58 GPU present profiling 逐计数器锁竞争与观测扰动

继续检查性能证据链本身后，确认 `record_present_stats` 每次成功GPU present含50个 `record_current_ui_perf_counter` 调用点；其中patch/full二选一，因此单帧最多实际提交49项。profiling capture开启时，每项都会独立进入Runtime全局 `ProfileRecorder` mutex，并分别生成counter snapshot字符串。普通非profiling构建会编译掉这些调用，所以该问题不能解释普通Editor的基础卡顿；但它会直接抬高我们准备作为验收依据的 `damage_to_submit` 与CPU样本，并在并行profile producer存在时放大锁竞争。

Runtime recorder本来已有 `record_counter_batch`，可在一次锁内为整批counter分配同一timestamp。本轮将该现成能力作为隐藏跨crate API开放；Editor新增独立 `ui_perf/counter_batch.rs`，只在普通profiling capture active时构建批次，`profiling-tracy`则保留旧的持续tracing语义。GPU stats先保留全部49项的条件、名称和值，再一次提交；default stats回归要求46项、region patch唯一且三项GPU timing缺席，timestamp-supported stats要求49项、full rebuild唯一且三项timing各出现一次。

源码RED为GPU stats中50个独立record调用点且batch入口为0；实现后独立入口为0、batch入口为1，scoped rustfmt与diff检查通过。source-bound critical manifest加入Editor batch模块和Runtime profiling owner，从94路增至96路，防止旧二进制绕过观测开销修复。当前仍没有运行Rust测试或current-source产品profile，不能声称p95已经下降；动态验收必须先证明counter数量/名称/值与旧合同一致，再比较同一场景的recorder lock acquisition、capture CPU和damage-to-submit，普通非profiling产品数据继续单独报告。

### 15.59 WGPU surface、device loss与多窗口恢复边界

沿Editor presenter继续向下读到 `zr_rhi`、WGPU UI surface和Runtime RenderFramework后，确认当前surface稳态路径并不存在“每次present重新创建device或重复上传稳定SVG”的源码证据。`WgpuUiSurfacePresenter::resize`对相同extent直接返回；native presenter可复用Runtime已经协商的instance/adapter/device/queue；图片以 `(resource_key, generation)`先查presenter local cache，再查device-scoped shared registry，只有两级miss才创建texture和写入GPU。`presentation.rs:134-157`也已把 `Outdated/Lost/Timeout/Occluded`视为 `RetryableNoSubmit`，其中前两项只重新configure当前surface。这些机制支持继续用产品counter验证实际命中率，而不是再建一套SVG/GPU cache。

真正缺失的是surface瞬态错误之上的device生命周期。`zr_rhi/src/ui_surface.rs:578-590`公开结果只有 `Submitted/RetryableNoSubmit`，`zr_rhi/src/device.rs:17-43`只有通用 `SurfaceUnavailable`；`RenderBackend`拥有device、queue和device-scoped shared image registry并把克隆context交给每个presenter，但生产图形链中没有 `device.lost`、`on_uncaptured_error`、device generation或recovery owner。于是一个窗口的surface acquire失败、共享device真正丢失、validation错误和不可恢复terminal failure不能在跨层合同中稳定区分；多窗口presenter也无法在device失效时合并为一次全局重建。该缺口影响故障隔离和长时间Editor可靠性，但它不解释无故障稳态下的按钮或resize卡顿，不能替代当前CPU/p95 profile。

Unreal源码提供的可迁移边界是分层owner而不是API照抄。`SlateRHIRenderer.cpp:236-340`用每窗口 `FSlateViewportInfo`保存viewport、extent和projection，`749-765`只为缺失窗口创建viewport，`2127-2136`只resize目标窗口，`842-859`在native window销毁前同步释放对应viewport并等待in-flight present。设备移除则在更低的D3D12 RHI queue/fence层检测：`D3D12Submission.cpp:1094-1103,1119-1123,1274-1282`查询removed reason并进入统一GPU crash处理，`D3D12Util.cpp:876-901`集中输出每个device的reason；DRED、breadcrumb和page-fault设置位于adapter初始化。当前参考版本选择明确终止而非进程内自动恢复，因此这里只能证明“window surface与device failure必须分层、失败必须可诊断”，不能把Unreal误写成自动恢复先例。

Zircon目标状态机应由最低共享owner `RenderFramework/RenderBackend`唯一持有device generation：

1. `Active(device_generation)`下每个native窗口各自拥有 `SurfaceActive / SurfaceRetry(reason, deadline) / Closed`；`Outdated/Lost`只重配该窗口，`Timeout/Occluded`等待下一次有界重试，均不得推进device generation或影响其它健康窗口。
2. device-lost observer把任意数量的并发通知合并成一次 `DeviceUnavailable(old_generation, reason) -> Recreating(attempt)`；停止新submit并保存可诊断reason。recreate失败使用有界backoff或进入明确terminal unavailable，禁止每个输入事件重试。
3. 成功后原子发布 `Active(new_generation)`。所有presenter观察generation变化后各重建一次surface configuration、pipelines、retained surface、text atlas、draw buffers、GPU timer/readback和presenter-local bind groups；device-scoped shared image registry随新context重建。CPU draw-list、资源identity/generation和可重新提供的像素源继续保留，随后每个存活窗口只请求一次full redraw。
4. 单窗口close可以吸收其pending surface retry/recreation callback；单个surface lost不得重建其它窗口。真正device loss才按presenter数量执行一次 `O(P)` fan-out，恢复工作不得落到pointer/event hot path，也不得由Editor建立第二个device/cache authority。

测试必须先在 `zr_rhi`/RenderFramework使用可注入状态源证明transition和coalescing，再到WGPU presenter验证资源代次，最后做Editor双窗口产品故障注入。矩阵至少覆盖四种surface结果、重复device-loss通知、recreate失败与backoff、恢复中关闭一个窗口、双窗口单surface lost、device generation切换后两窗口各一次full redraw，以及稳定SVG在同generation上传为0、恢复到新generation后每个live资源只重新上传一次。动态证据增加surface outcome reason、device generation、recovery count/duration、presenter/cache recreation、full-redraw-after-recovery和shared-image reupload bytes，并同时报告CPU、RSS与交互延迟。当前结论达到E3设计审查，未修改Render/RHI生产源码、未运行Cargo，也不构成device recovery已实现或性能已达标。

### 15.60 IME与无障碍产品桥接不能复制第三套UI状态

IME下层合同比Editor产品路径完整。`winit_translation.rs:181-220`把平台事件保留为 `UiInputEvent::Ime`，覆盖Preedit、Commit、Cancel和DeleteSurrounding，cursor range使用UTF-8 byte range；Runtime TextInput已有composition、selection、delete-surrounding、focus lifecycle和candidate cursor geometry回归。Editor retained window却在 `events/keyboard.rs` 的IME入口调用 `platform_text_input`，而 `platform_input.rs:86-90`只接受 `UiInputEvent::Text`。因此所有由 `WindowEvent::Ime`产生的 `UiInputEvent::Ime`都返回None并取消latency sample。与此同时，`HostTextInputFocusData`只保存字符串与edit frame，字符输入永远追加到末尾，Backspace只pop最后一个Unicode scalar；paint conversion把selection/caret固定为None，窗口只调用 `set_ime_allowed`，没有候选窗cursor-area同步。这不是单个match写错，而是Editor绕开Runtime TextInput后形成的第二套append-only编辑权威。

Unreal的 `ITextInputMethodContext`明确说明平台IME需要完整editable context：composition状态、代码点长度、selection/caret、range读取与替换、text bounds与screen bounds；context随焦点Register/Activate/Deactivate，并通过notifier上报layout/selection/text变化。Zircon不应只把Commit提取成普通字符串来“修”中文输入。目标是每个focused editable control只拥有一个session authority：完整 `UiImeInputEvent`进入同一编辑状态机；Preedit只更新composition，不提前提交业务value；Commit/Cancel/DeleteSurrounding保持UTF-8边界与selection语义；selection、caret、preedit decoration和cursor geometry从同一published state生成；Enable/Disable/UpdateCursor host request再映射到winit window。Editor template text input可以发布真实Runtime TextInput node，或使用一个薄adapter复用同一neutral edit state，但旧append-only逻辑与新session不得并存。

无障碍同样是“下层有实现，产品没有闭环”。Runtime `accessibility_snapshot()`可生成role/name/relation/state/action/bounds/focus并验证diagnostics，`accesskit.rs`可双向映射TreeUpdate与action request，相关Runtime测试覆盖文本、range、popup、scroll与focus。但生产搜索表明 `snapshot_to_accesskit_tree_update`只在该模块定义和测试调用；workspace只有Runtime可选 `accessibility-accesskit` feature，没有Editor/App的AccessKit OS adapter。`runtime_event_adapter`能从自定义ABI JSON还原accessibility action，不等于Windows屏幕阅读器已经接入。

接线时不能在每帧直接调用当前snapshot。`extract.rs`先遍历全部tree node收集relation target，再遍历全部node构建DTO，并在children filtering中第三次遍历整树；每个node的effective hidden还可能向上遍历祖先，之后继续执行name/description解析和整树diagnostics，最坏为 `O(N*h)`并产生完整nodes、maps、sets和strings。Unreal Slate在这点上的可迁移经验是稳定accessible widget/id cache、显式dirty gate、默认每tick最多处理100个widget，并用children buffer保证重建完成前OS查询仍看到上一代一致树；它也承认全树生成需要抑制性能峰值。

Zircon更合适的目标是把无障碍作为 `UiSurfaceFrame` 的另一项不可变publication product。structure、layout bounds、visibility、focus、semantic value/text/action各自进入typed dirty set；更新集合扩展到必要祖先、children order与label/description relation dependents，只在commit时生成新accessibility generation。OS adapter每native window创建一次，只在generation变化时发布TreeUpdate/delta，平台action携带目标generation回到同一个Runtime dispatcher；构建期间仍保留上一代完整frame，弹层和多窗口各有明确root/close生命周期。正常未启用assistive technology时不构建平台payload，启用后稳定pointer/paint也不得推进a11y generation。

验证顺序必须是Runtime text/a11y下层合同 -> Editor focused text/窗口adapter -> Windows产品。IME覆盖中文/日文多阶段preedit、候选range、commit/cancel、delete-surrounding、选择替换、DPI/移动后的candidate geometry、焦点切换和关窗；无障碍覆盖screen reader读取、focus/activate/value/text selection/scroll/popup、双窗口和关闭窗口。10,000-node压力必须报告dirty/full build、node/relation visit、published node/delta bytes、adapter update、action latency、CPU与RSS；稳定无语义变化的1000次pointer事件要求a11y build/publication为0。当前只完成E3审查，相关Editor/platform与Runtime accessibility源存在外部改动，本轮保持只读且未运行Cargo。

### 15.61 焦点导航仍在事件期重建全树候选

Runtime焦点导航目前没有消费已发布的frame authority。`next_navigation_target`的Tab路径先由 `active_modal_scope_for` 调用 `topmost_active_declared_modal_scope` 全扫 `tree.nodes`，再由 `navigation_candidates` 从每个root递归整树、为每个focus candidate克隆group id，过滤后执行 `O(F log F)` 排序，最后线性寻找当前项。方向路径重复modal扫描和候选构建/排序，再对全部保留候选读取 `tree.layout_cache.frame` 做线性斜率与距离评分。显式 `DirectionalNavigationTarget::Group` 又会单独重建、过滤并排序全部候选。因此一次查询通常为 `O(N + F log F)`，手工group与部分modal路径可在同一事件中重复该成本；临时 `Vec` 和group `String` clone也随事件数增长。`UiTreeNodes`已有 `pending_mutation_node_ids`，但该信息没有用于导航查询。

这里同时存在正确性分裂。pointer命中已转向 `UiSurfaceFrame` 中的projected hit grid，control-anchored popup可从arranged placeholder经过双轴仿射变换到最终render geometry；导航却继续读取未投影的tree layout frame并按tree `z_index/paint_order`选modal topmost。弹层在旧placeholder与最终位置不同时，鼠标命中和方向导航可能选择不同空间中的目标。仅把当前候选 `Vec` 缓存在 `UiTree` 会固化这个错误，而且会把可序列化tree变成第二个frame authority。

目标是增加一个surface-owned、不可序列化的 `UiFocusNavigationIndex`，由rebuild/frame publication与arranged/projected hit authority在同一代次维护。它保存稳定node lookup、最终投影frame、focus/tab/group/modal属性、显式方向边以及按scope的tab order；方向导航复用最终frame的空间cell，descendant/scope成员关系使用同一发布树的稳定 ancestry 信息。full build可以为 `O(N log N + cell membership)`；Next/Previous通过scope order与node-to-position查找趋近 `O(log F)` 或 `O(1)`，manual node/group为索引查找，方向查询与经过的cell及候选数相关，而不是无条件与surface全部节点相关。不得另算一套popup transform、另建几何树或让event hot path触发lazy full build。

增量更新必须按语义依赖扩张，而不是只patch直接dirty node。frame/z/clip变化更新对应spatial membership；enabled/visible/focusable/tab-index/manual-edge更新目标entry；parent、navigation group、modal open/root或popup stack变化扩张到受影响subtree、scope order与relation dependents。结构插入/删除/reparent、lookup缺失或scope拓扑不明时允许显式full fallback并计数。纯hover/pressed等render-only变化不得更新导航索引或推进其generation。首版可以在相关input/layout/structure commit时整批重建以先移除事件期 `O(N)`，但必须记录build reason和visited nodes；只有profile证明publication成本显著后才做changed-node patch，不能以复杂增量实现阻塞正确authority收敛。

下层回归顺序固定为：现有tab/group/manual/modal行为与新索引等价；control-anchored popup使用最终投影frame、旧placeholder不参与方向选择；parent enabled/visibility/focusability变化正确更新descendant资格；insert/remove/reparent和missing lookup触发可观测full fallback；frame-path与instance-path读取同一generation；1000次render-only pointer变化的navigation build/patch均为0。10,000节点压力分别执行1000次Next/Previous与四向导航，报告build/patch/full fallback、visited cell/candidate、query allocation、CPU和RSS，并要求Tab查询不再访问N个tree node。该实现依赖冻结中的projected-hit exact-7先完成下层验证，因为它会触及相同的 `surface.rs`、`rebuild.rs` 和 `frame_publication.rs`；当前只达到E3设计审查，未改生产源码、未运行Cargo，也不声称键盘或手柄延迟已经改善。

### 15.62 `virtual_rows` 是全量模板复制，不是数据级虚拟化

本轮沿 hierarchy 的paint、pointer、template sync与Runtime layout纵向复核后，确认同一场景列表存在不对称的三段实现。原生hierarchy painter通过 `visible_hierarchy_row_range` 只遍历裁剪viewport内的行，10,000行回归已限制绘制数为 `O(V)`；`HierarchyPointerBridge` 的surface固定为root + viewport两个节点，命中先过该常数authority，再以 `(content_y / row_pitch).floor()`和一次indexed lookup得到逻辑行，因此输入为 `O(1)`。但是 `WorkbenchSceneTree` 仍是普通 `VerticalBox`，其 `repeat = { kind = "virtual_rows" }` 只被Editor模板桥解释；`TemplateBridgeVirtualRowSequence::reconcile(total_row_count)` 对 `total - authored` 的每一行克隆一个真实 `UiTreeNode`和slot。换言之，绘制与命中已不随逻辑行增长，隐藏的template projection仍复制全部逻辑行。

这条复制链不是一次性启动成本。`prepare_chrome_state_for_layout` 每次进入完整shell snapshot都会调用 `sync_scene_entries(scene_entries.len())`，且发生在shell geometry/layout reuse裁决之前；window-metrics committed-stage也会重新计算template layout frames。一次full reflow先让repeat执行多轮surface扫描：找parent/prototype、收集existing/stale rows、找max node id、枚举全部required rows，随后 `scene_tree_control_ids`再次扫描并排序。接着每行写visibility、text、depth、indent、entity、parent、subtree hash、expanded和selected，最后 `SceneHierarchyProjectionState::replace`再构建两棵包含克隆control string的 `BTreeMap`。因此完整同步至少为 `O(S + L log L)` 时间、`O(L)` retained节点/metadata/route存储和大量字符串分配；`S` 本身又包含 `L`。稀疏fragment已借助entity/control索引做到 `O(Delta log L)`，但它不能消除full reflow与常驻内存的全量镜像。

Runtime现有 `UiVirtualListConfig` 也不能直接解决此问题。其逻辑总数由已存在的 `children.len()`给出，没有独立的model count；`measure_node_with_profile`在计算container前递归测量全部child，`arrange_scrollable_children`先用 `child_positions`为全部child分配并生成position，再逐个调用 `hide_subtree_layout`或arrange可见项。它目前是“全量retained tree上的paint/hit裁剪”，不是Unreal/Slint意义上的实例虚拟化。Unreal `SListView::ReGenerateItems`从 `CurrentScrollOffset`开始生成并在viewport填满时停止，已有widget按item复用；Slint repeater显式保存model offset、cached item height和可见instances，只创建窗口所需实例并清理离窗项。本地实现应吸收“逻辑集合与实例集合分离”，而不是再给全量child遍历加缓存。

目标按业务需要分两条hard cut，避免一刀切产生第三套authority：

1. hierarchy的产品paint、scroll和pointer已经以 `SceneEntries`/runtime row为权威，应删除逐逻辑行的template节点镜像。模板只保留面板、标题、静态按钮和viewport结构；选择、展开、重命名与拖放继续使用entity/logical index路由。`SceneHierarchyProjectionState`收敛为generation、selection和必要的entity/index映射，不再保存每个逻辑实体对应的template control string。未来无障碍使用逻辑collection semantic provider，不以10,000个隐藏widget换取可访问性。
2. Inspector属性确实需要稳定的编辑、焦点、IME和提交binding，不能简单删除实例。它应使用 `visible range + overscan` 的有界slot pool，slot node ID稳定，另有 `slot <-> property key/index`映射；range移动复用交集，只重绑进入/离开的slot。active edit/focus/capture必须按property key固定或在回收前提交/取消，selection不按slot identity保存。逻辑content extent由 `total_count * row_pitch`发布，不能从materialized child数反推。当前 `set_inspector_scroll_px`为空实现，也必须在同一切片接通scroll publication，不能只减少节点却让长Inspector仍不可滚动。
3. 共享Runtime合同随后收敛为model count、logical extent、published visible range和bounded materializer四项；`UiSurfaceFrame`只发布当代materialized entries，事件期不触发实例补齐。固定行高的hierarchy/Inspector先走算术offset；只有产品确实需要variable height时才引入measured prefix/Fenwick索引，并要求高度变化按受影响后缀或分块更新。首版不得同时发明通用recycler、variable-height tree和accessibility adapter。

实现顺序为：先加source-bound计数并建立现状RED（logical rows、materialized template nodes、repeat full scan、property writes、layout visited、pool entered/exited/rebound）；再删除hierarchy镜像并证明native paint/pointer/selection/rename/drag行为不变；最后为Inspector建立bounded pool与scroll闭环。下层回归必须覆盖10,000 hierarchy行时template row节点为0、surface总节点与逻辑行无关，1000次scroll不触发template reconcile/property write；10,000 Inspector属性时materialized rows不超过 `ceil(viewport/row_pitch) + 2*overscan`，远端滚动后编辑提交到原property key，focused row回收规则确定，插入/删除/reorder与DPI/viewport变化保持正确。产品profile报告full reflow与稳定scroll的CPU/RSS、materialized node峰值、layout visited、字符串分配代理计数和input-to-damage p50/p95/p99/max。当前相关Editor文件已有外部改动，本轮只完成E3静态设计并更新计划，没有修改生产源码或运行Cargo。

### 15.63 真实按钮的局部damage仍被失效语义和publication断开

本轮重新从物理窗口事件追到最终present，先排除了一个容易误导优化方向的假设。生产 `WindowEvent` 在 `window/event_loop/events/pointer.rs:60` 直接进入 `dispatch_native_pointer_button`，使用当代不可变 `HostPresentationGeneration` 的命中索引；Workbench模板命中随后在 `button_dispatch/workbench/primary/activation.rs:16-17` 调用真实callback并返回目标frame的 `region_with_frame_update`。`dispatch_componentized_workbench_pointer_event` 的生产调用者为0，只有测试直接调用。它会在Runtime模板surface内维护hover/pressed/focus并执行 `refresh_after_state_change`，属于尚未收敛的平行测试桥，不能用它的局部单测证明真实窗口按钮延迟，也不应让产品事件重新经过它。生产hover已经通过 `HostPaneInteractionStateData` 的独立interaction generation覆盖绘制，稳定目标内move可以无状态变化、无结构publication，这是应保留的正确快路。

真实click的问题发生在callback之后。`EditorWorkbenchTemplateSurface::refresh_after_state_change` 已能把Runtime changed-node集合转换成 `pending_host_projection_patch_indices`，`apply_workbench_projection_presentation` 也能按changed rows重绑host nodes与hit index并只请求旧/新frame damage；但该快路只有pending reason精确等于 `WORKBENCH_PROJECTION` 时才被 `begin_recompute_invalidation_phase` 选中。相反，workbench control先在bridge里改少量selected/checked/visible节点，再把Editor event effect转换为 `UiHostEventEffects`。current source对viewport chrome action已有意跳过泛化 `PresentationChanged`，并设置 `sync_viewport_chrome + PAINT_ONLY + RENDER`；然而 `sync_viewport_chrome` 只有写入与merge，没有生产consumer，已经存在的 `patch_scene_viewport_chrome` 也没有调用者。`PAINT_ONLY`和`RENDER`按合同不进入host recompute transaction，`recompute_if_dirty`的入口又不因它们单独运行，所以bridge中已经准备好的pending host projection不会被提交。现有interaction回归仍断言工具切换结果 `presentation_dirty=true`，与新的viewport effect合同相互矛盾；如果旧泛化presentation仍出现，decision会直接选择 `Full`并在局部damage present前重建完整shell。两种状态分别是“局部视觉未发布”和“为了发布一个按钮而full recompute”，都不满足局部交互合同。

`complete_paint_only_recompute` 不能填补这条断链。`HostInvalidationRoot::invalidate_scoped` 只把 `requires_host_recompute()` 的mask写入pending transaction，而PAINT_ONLY/POINTER_HOVER/VIEWPORT_IMAGE/RENDER都被明确排除；函数内部记录 `ChromeCommandPatchCount`后只清legacy flags和发布diagnostics，并没有执行command或projection patch。该计数名称会让profile误报“已经patch”。真实的 `WORKBENCH_PROJECTION` path有独立patch/noop/fallback/node/damage计数，应成为模板结构变化的唯一publication路径；interaction overlay与viewport image继续作为独立generation直接更新，不需要经过host结构重算。

damage侧也仍需收敛。目标Workbench节点可直接返回hit frame，但generic pane callback会把pane、两份center band和三份status frame合并成一个保守区域；close prompt、viewport toolbar、chrome、resize与tab-drag在frame解析失败时都会退化为 `NativePointerDispatchResult::full_frame()`。当前只有 `RedrawFullFrame`总量，无法区分pointer geometry miss、runtime continuous wake、surface retry或正确的显式全帧动作。因此静态审查只能证明fallback存在，不能证明它在用户复现场景中的命中率。正确性兜底先保留，但必须有typed reason、来源generation和目标identity，动态profile后再逐个消除可达原因。

目标实现不新增平行authority，而是把现有三段合同接通：

1. 给 `UiHostEventEffects` 增加明确的Workbench projection commit语义。凡bridge的 `refresh_after_state_change`产生pending nodes，callback outcome必须携带该语义；decision允许它与RENDER/PAINT_ONLY等不会改变shell topology的域合并，先执行 `apply_workbench_projection_presentation`，再保留render submission。LAYOUT/TREE_STRUCTURE/WINDOW_METRICS、多个不兼容scope或projection validation失败才进入有原因的full fallback。
2. viewport chrome action同时消费 `sync_viewport_chrome`：从同一runtime chrome snapshot构造 `SceneViewportChromeData`，调用已有结构patch，并把Workbench selected/checked/status node changed rows与hit index放进同一结构generation提交。若统一由Workbench projection patch覆盖这些节点，则删除未使用的独立flag/helper；禁止两条patch对同一字段形成竞争authority。
3. PAINT_ONLY只表示“结构无需重算，相关interaction/image/theme generation已经在事件处理期间原子发布”。删除或更名没有实际patch的 `ChromeCommandPatchCount`记账；真正的projection/command patch只在提交成功后计数。测试专用Runtime pointer bridge在产品覆盖具备后hard cut，避免继续维护第二套hover/press publication语义。
4. `HostRedrawRequest::Full`携带typed reason或在构造点记录等价reason counter。pointer fallback至少区分missing/invalid target frame、pane semantic overdraw、resize/tab-drag geometry miss；runtime wake、surface retry和显式全窗口变化使用不同类别。局部callback返回的damage优先由实际mutation outcome给出旧/新frame集合，generic pane union只保留为未迁移action的可观测fallback。
5. callback、host projection patch与redraw必须绑定同一source/target generation。输入命中的旧generation允许完成当前route；mutation commit发布新结构generation和对应hit index后才请求damage present，不能先present旧structure再等待下一次无关presentation纠正按钮状态。

下层RED先证明：一个Workbench tool click只patch实际changed rows，structure generation推进一次，hit index与新nodes同代，`HostInvalidationFullTargetCount`、slow path、full presentation和full redraw均为0；RENDER仍提交一次viewport frame。`PAINT_ONLY`必须满足“已有直接generation变化”或“零pending bridge projection”，否则测试失败。再覆盖projection missing/hit-index rebind失败，各只触发一次带reason full fallback；generic pane动作报告实际damage区域与fallback reason。产品层用1000次同目标hover、1000次tool/button press-release和混合status/viewport动作报告input-to-damage、callback、recompute、patch、damage-to-submit p50/p95/p99/max，要求稳定hover结构generation与present为0，局部click的full target/full redraw/fallback为0，painted pixels受damage约束，CPU/RSS不随事件数增长。Unreal对照边界是 `SlateApplication.cpp:5371` 将输入route独立计时，`SlateInvalidationRoot.cpp:356-423`只在明确slow-path条件下清cache并全量paint，否则执行 `PaintFastPath`；Zircon应迁移的是“输入route、失效原因、增量publication和fallback可观测”四项分离，而不是复制Slate类型。当前结论达到E3静态设计，未改上述Rust生产源码、未运行Cargo，也不声称按钮响应已经改善。

### 15.64 auto layout把局部dirty预先扩张到大容器根

Runtime已经有 `UiIncrementalLayoutStats`、dirty node集合和子树layout入口，但当前root选择在真正measure之前就做了保守扩张。`mark_layout_dirty` 从目标向上遍历，只要父节点是默认 `ContentDriven` 或任何auto-layout container就继续把祖先标成layout/hit/render dirty；`incremental_layout_roots` 随后对这些dirty节点再次执行相同的 `propagated_layout_root`。选出最高root后，算法先递归收集整个subtree、复制全部旧geometry，再从该root递归measure每个child并arrange整棵子树。其成本不是稳定的 `O(changed nodes)`，而是 `O(size of topmost contiguous auto-layout ancestor subtree)`；祖先链越深，局部文本、visibility或约束变化越容易重新访问整个Workbench模板。

现有 `LayoutBoundary` 不能在产品树中形成有效隔离。其默认是 `ContentDriven`，只有这个枚举值声明 child invalidation应向上传播；但两处生产判断都额外使用 `|| parent.container.is_auto_layout_container()`，所以 `ParentDirected` 的Vertical/Horizontal/Scrollable/Grid/Masonry父级仍然传播。下层测试 `surface_dirty_layout_revisits_auto_parent_when_child_size_changes` 正式固化了这一行为：即使VerticalBox标为ParentDirected，单child高度改变也访问root和两个children共3个节点。这个用例证明了auto parent需要重排siblings，却没有证明该parent的outer desired size必须继续影响更高祖先，两种依赖目前被混成一个布尔条件。

静态资产统计说明这不是边缘情况。受跟踪 `.zui` 一共有1,213处container声明，其中1,080处属于auto-layout family；仅 `zircon_editor/assets/ui` 就有951处container，885处为auto layout（440 HorizontalBox、430 VerticalBox、9 GridBox、5 ScrollableBox、1 Masonry），显式 `boundary` 声明为0。也就是说几乎整个Editor模板依赖默认ContentDriven，而当前auto override会让未来补写ParentDirected也无法截断传播。Overlay/Free等非auto容器可以命中现有局部测试，但它们不代表真实Workbench主干。

正确算法应把“父容器需要重新排列其children”和“父容器的desired size发生变化、因此还要通知祖先”分开。Unreal Slate的 `FWidgetProxy::ProcessLayoutInvalidation` 在局部prepass后比较 `NewDesiredSize` 与 `CurrentDesiredSize`，只有desired size或visibility语义改变才把parent加入layout update heap；prepass heap再用父覆盖子range避免重复处理。这不是要求照抄Slate heap，而是证明向上传播可以由measure结果驱动，不必在计算前假定所有auto祖先都会变化。

Zircon目标算法如下：

1. dirty transaction先保存原始layout原因、可能受影响轴和起始节点，不再由 `mark_layout_dirty` 预先把整条auto祖先链全部标脏。结构插入/删除/reparent、未知custom layout backend可以显式进入保守fallback；普通text/constraint/visibility变化进入增量measure队列。
2. 自底向上重算起始节点的cached `desired_size/content_size`。child desired size或collapse occupancy未变化时停止向父传播；变化时只把直接layout owner入队。auto owner可以使用未dirty child的cached desired size重算自身，不再递归measure全部children；同一parent的多child变化在transaction内合并一次。
3. 传播按轴计算。父约束、child slot规则和container axis共同决定width/height dependency：Fixed/ParentDirected外部尺寸或Stretch分配可截断对应轴，Auto/StretchContent/intrinsic content继续上推。首版可用二维bitset和保守unknown fallback，不需要同时发明通用constraint solver。
4. arrange只从实际需要重新分配children的最小layout owners开始。未变化的child desired cache直接复用；只有frame/clip变化的child subtree继续更新绝对geometry，未移动且clip不变的sibling不得递归arrange。arranged/hit/render继续消费真实geometry-changed集合，不以“曾参与measure”代替“几何真的变化”。
5. `LayoutBoundary`需收敛为可验证的containment合同，而不是装饰字段。默认推导应结合constraints与slot，不要求人工标注全部885个auto容器；对large panel/viewport/scroll island允许显式override并由compiler诊断矛盾声明。root size/DPI变化仍执行一次完整responsive/layout `O(N)`，native resize transaction负责把连续事件合并到稳定尺寸，不能用局部算法跳过最终reflow。

下层RED至少覆盖四层嵌套auto树：leaf内容变化但desired size不变时parent enqueue为0；child desired height变化时只重排直接VerticalBox及其siblings，并在固定外框停止；Auto/StretchContent slot确实逐级传播；width-only变化不污染独立height dependency；collapse/uncollapse、insert/remove/reparent与custom backend fallback保持正确；root size变化仍访问全树一次。规模测试在10,000-node、多层固定panel中执行1000次局部文本/约束变化，记录raw dirty nodes、measure visits、desired-size changes、parent enqueues、arrange owners、geometry subtree visits、fallback reason、CPU与RSS，要求无desired变化的transaction工作量与dirty count同阶，固定layout island外visited为0。当前layout、tree与Workbench ZUI路径均有外部改动，本轮保持只读、未运行Cargo；该项是E3设计结论，不是已实现的性能收益。

### 15.65 Surface frame与render publication把retained cache重新摊平

当前Runtime已经有一份接近正确的Surface权威，但其publication仍是粗粒度owned snapshot，render链也没有真正消费它。`UiSurfaceFramePublication::surface_frame`会复用同一`Arc<UiSurfaceFrame>`直到dirty；一旦任一域dirty，它却同时克隆完整`arranged_tree`、`UiRenderExtract`和projected hit grid。`transient_state_changed`甚至会在只有window state或focus变化时执行同一重建，因此一次按钮聚焦可以把小状态提交放大为`O(N + C + H)`深拷贝。更关键的是，产品render路径没有传递该frame handle：`RenderFramework`、`ViewportRenderFrame`和one-slot pipelined queue继续逐层拥有`Option<UiRenderExtract>`，其核心仍是`UiRenderList { commands: Vec<UiRenderCommand> }`。当前外部未跟踪的`RuntimeUiSurfaceSet::render_extract`候选也在每次capture中为所有surface新建command Vec、克隆每条command并重写node id；该文件不属于本轮修改，但它证明仅把`.zui`加载成retained `UiSurface`仍不能消除提交期复制。

粗粒度generation还会把成本继续传给Editor consumer。`ViewportToolbarPointerBridge::sync_surface_frame`只缓存`(surface_frame.generation, surface_origin)`；任何顶层generation变化都会重新扫描全部`arranged_tree.nodes`、为匹配control克隆action key、collect新controls Vec，比较后还可能重建私有pointer surface。它无法表达“focus/window/render变了但toolbar geometry/hit没有变”。因此只把frame大字段换成Arc仍不够：每个子快照必须有可比较的domain generation，consumer按真实依赖订阅；顶层generation只表示原子commit边界，不能继续被解释为所有域都失效。

command在下游还会被重新编译。Runtime renderer对每条command调用`to_paint_elements(0)`；Editor retained host也调用同一入口。一次转换会创建临时element Vec，克隆text、style、brush、font/atlas resource key和resolved text payload，创建debug label，并对完整command执行一次流式JSON hash以生成cache generation。2026-07-22子叶已经把“一条command产生四个element时重复hash四次”和临时JSON byte Vec消除，但稳定帧仍对全部commands重新hash、展开与深拷贝。`UiSurfaceRenderCache::update`也会遍历新extract的全部commands、逐项等值比较并为changed/new entry再clone；它缓存的是命令副本和damage，不是可直接交给presenter的generation-owned typed element/batch range。因此当前成本可分为三次独立的`O(C + payload bytes)`：surface frame发布clone、multi-surface flatten/global-id rewrite、presenter command-to-element编译，随后才进入真正draw planning。

这与Unreal Slate的关键边界不同。`FSlateInvalidationRoot::PaintInvalidationRoot`只在显式slow path清除并重建cached element data，否则执行`PaintFastPath`；`FSlateCachedElementData`按invalidation root保存per-widget element list、`ListsWithNewData`和persistent cached batches；`FSlateElementBatcher::AddCachedElements`只重新处理`ListsWithNewData`，随后把已有cached batches加入当帧render batch list。Unreal仍会按batch数量做每帧提交遍历，不能据此承诺稳定帧literal `O(1)` GPU编码，但它不会重新生成每个widget的draw payload与batch topology。Slint的`CachedRenderingData`同样用cache index + generation关联backend cache，dirty pass按geometry和property tracker决定局部重绘。Zircon应吸收的是“生成代拥有typed render artifact、稳定帧只传递句柄、renderer复用batch”，不是增加一个与`UiSurfaceFrame`竞争的全局cache。

目标合同收敛为一条publication链：

1. `UiSurfaceFrame`继续是layout/hit/focus/accessibility/render同代权威，但内部按dirty domain引用不可变generation-owned子快照：layout/arranged、projected hit/navigation、render、accessibility分别持有共享handle和domain generation，focus/window等小状态保留在顶层。顶层frame generation原子提交所有域；只变focus或window state时，新frame复用其余子快照identity，不能深拷贝整树。consumer按所依赖domain generation决定patch，不能因顶层commit变化重扫无关域。兼容的owned DTO只在序列化、debug dump或显式冷路径导出时生成，不能继续作为产品提交ABI。dirty rebuild在发布前完成，事件或present读取不能触发lazy全树clone。
2. 每个Surface publication保存稳定`surface namespace + local UiNodeId`、content/geometry/resource generation、ordered render segments、typed paint element range、damage和renderer cache key。changed nodes只替换所属segment；未变segment的`Arc` identity、element payload与resource handles保持不变。多Surface合成只发布有序segment-handle表，禁止每帧flatten commands或重写全部node id；global identity在route/accessibility/diagnostic边界由namespace组合。
3. `UiRenderCommand`保留为authoring/extraction DTO，command-to-element编译移动到dirty publication。content/style/text/image变化重建typed payload与cache generation；纯geometry变化若payload/topology不变，只patch geometry/clip/order metadata。renderer按segment generation维护batch与GPU resource binding，只有changed segments进入element batching；稳定帧允许`O(B)`遍历/编码cached batches，不允许`O(C + payload bytes)`hash、String/Vec clone或element重建。
4. `RenderFramework`、runtime frame、Editor viewport和异步队列传递共享publication handle。one-slot queue只增加一个Arc引用；producer和worker都不能为生命周期安全深拷贝commands。renderer/device generation变化可以显式失效GPU-side batch/resource cache，但不得倒推重建未变的layout、hit或text payload。
5. menu/HUD旧fallback不建立第三套special cache。它们先用`dynamic_component_generation`和type-scoped `dynamic_component_rows`替代`node_records()`全World扫描，再投影为同一Runtime UI Surface segment；稳定component generation的world visits、String/Vec build和publication变化均为0。viewport resize属于geometry generation，只在native resize transaction提交的最终size上每Surface重建一次。

复杂度门必须区分工作域。稳定单Surface frame read与queue handoff为`O(1)` Arc clone、深拷贝字节0；focus/window-only publication同样为`O(1)`未变子快照引用复用。稳定多Surface composite read复用既有composite handle，不按surface数重建表。单节点content变化为`O(DeltaCommand + DeltaElement + affected batch)`，单Surface变化只替换该segment及有序handle索引，不访问其它surface command payload；稳定present为`O(cached batch submit)`，而非重新执行command conversion。显式full tree/style/backend/device fallback可以为`O(N + C)`，但必须有typed reason和generation，连续resize只在coalesced最终尺寸发生一次。

下层RED先锁定identity与counter：连续两次stable publication/submit返回同generation和同layout/hit/render/segment/element Arc identity，surface frame clone bytes、command clone bytes、element Vec build、generation hash、payload clone、composite rebuild和World visits均为0；focus/window-only变化只推进顶层generation，layout/hit/render子快照保持`Arc::ptr_eq`且viewport-toolbar arranged scan/control Vec/pointer-surface rebuild均为0；一个节点变化只替换其segment，未变segment `Arc::ptr_eq`为true；两个Surface中只变一个时另一个generation/handle不变，render order、clip、hit、focus与accessibility仍绑定同一composite generation；geometry-only变化不得重建text/image payload；renderer/device generation失效只重建允许的GPU artifact。再以1/100/10,000 commands、1/1,000/100,000 World nodes及1/60/240Hz执行stable、focus切换、单节点变化、单Surface变化、resize和device recovery，报告domain publication/clone bytes、consumer domain scans、extract visits、command/element/hash/payload clone bytes、changed segments、recached batches、submitted cached batches、CPU/RSS和Runtime/Editor pixel parity。当前两份failure handoff仍为open，`runtime_ui.rs`为外部未跟踪源码，本轮只写E3设计与验收，不改生产实现、不运行Cargo，也不声称render提交已经变快。

### 15.66 Asset Browser preview cache会过度失效、失效不足并发生LRU循环抖动

用户报告的“SVG会反复加载”不能只用主visual cache已有命中来否定。Workbench模板图标路径确实已有SVG tree、target raster、GPU local/shared registry三层cache，但Asset Browser和若干pane projection还走另一条`layouts/views/preview_images.rs`路径。该路径有最终`Image` cache，miss时却同步执行candidate文件探测、`fs::read`、canonicalize、`usvg::Tree::from_data`和intrinsic-size raster；它不复用主SVG tree/raster service，也没有目标像素尺寸参数。一个大viewBox SVG即使只显示为小thumbnail，也会先生成并驻留完整intrinsic RGBA。

当前cache identity同时存在两种相反错误。`thumbnail_nodes.rs`和selected preview把`catalog_revision ^ asset.resource_revision`作为单个u64 generation：任一资产导致catalog revision变化时，所有未变资产的generation都会改变，旧preview全部miss；XOR本身还允许不同输入对产生同值，不能表达结构化identity。另一方面，preview job完成时`EditorAssetCatalogGeneration::updated_asset`只推进`publish_epoch`并保留`catalog_revision`，而`AssetWorkspaceSnapshot`/`AssetItemSnapshot`没有携带per-asset preview epoch或artifact content identity；若artifact仍写入同一路径且resource revision未变，cache可能继续返回旧Image。现有回归只证明相同borrowed key命中、手工不同u64替换和LRU淘汰，没有覆盖这两个生产代次合同。

规模行为会把错误放大。`AssetBrowserProjectionInput`把selected asset和全局catalog revision都纳入整pane cache key；miss后`append_asset_browser_thumbnail_nodes`遍历全部`snapshot.visible_assets`，这里的visible表示filter结果，不是viewport materialized range。每项同步调用preview loader并物化约8个template nodes。最终Image cache只限制128个source bucket，且容量按条目而不是RGBA字节。对超过128个source按固定顺序全扫后，cache只保留尾部128项；下一次selection或catalog projection从头扫描时会逐步淘汰这些尾项，到达它们之前已经全部丢失，形成接近100% miss的经典循环LRU抖动。于是一次选中变化可能为`O(A * file/decode/raster) + O(A)`节点构建，并且RSS上限与单图尺寸无关。这条链足以解释局部交互期间重复SVG parse/raster；稳定GPU是否重复upload仍需看内容哈希命中counter，不能仅凭CPU miss推断。

Unreal Content Browser提供了更合适的边界。`SAssetView::UpdateThumbnails`从真实`VisibleItems`求最小/最大index，只保留可见项和有界`NumOffscreenThumbnails`，复用`RelevantThumbnails`，新进入范围才创建thumbnail并优先可见项；`FAssetThumbnailPool`以`FThumbId(ObjectPath, Width, Height)`管理texture/refcount/free list，只对具体dirty object执行refresh，并以`MaxFrameTimeAllowance`限制每tick生成成本。Zircon不需要复制UObject或实时thumbnail renderer，但必须吸收per-asset identity、visible relevancy、资源复用和有界后台工作四项。

目标实现如下：

1. 发布typed `AssetPreviewIdentity`，至少包含project namespace、asset UUID、preview artifact key/content generation、requested pixel extent和raster scale；源资源revision与preview publication generation分字段比较，禁止XOR折叠。preview job即使复用同一路径，也必须只推进目标asset的preview identity；无关catalog delta不得改变其它asset identity。
2. Asset Browser逻辑filter结果与materialized thumbnail集合分离。grid根据scroll offset、viewport和overscan只维持bounded stable slots；selection只patch旧/新卡片语义，catalog delta只patch受影响record，不能重新遍历/复制全部逻辑资产或触碰其它preview handles。
3. preview miss不得在pointer/callback/recompute热路同步parse/raster。materializer发布placeholder并向有界thumbnail worker提交去重请求；完成后以目标asset identity原子替换一个slot并请求其damage。离开relevancy window的低优先任务可取消或降级，visible任务优先。
4. editor icon、raw SVG media和asset preview统一消费同一visual resource service中的source tree与target-raster cache；domain可以保留thumbnail policy，但不能再维护独立SVG parser/cache authority。raster key包含内容identity、target physical pixels、tint/color space和scale；CPU image cache按实际RGBA bytes预算与LRU管理，不只按128个source计数。
5. GPU继续使用现有device-scoped `(resource_key, generation)` shared registry和presenter bind-group cache。CPU thumbnail完成只发布内容寻址resource identity；稳定handle不得因selection/catalog/top-level frame generation改变。单asset内容变化最多产生一次新CPU raster和每device一次upload，不新增Asset Browser专用GPU cache。

下层RED覆盖identity与错误失效：A资产catalog/resource变化后B的preview handle保持`Arc::ptr_eq`且lookup hit；同路径B preview publish epoch变化只让B miss一次；构造两组旧XOR碰撞输入必须得到不同typed identity；project切换中相同`res://`路径不串图；96px@1x/2x target得到独立且有界raster。产品回归用1,000和10,000 filtered assets、128旧容量反例、连续selection、单asset preview完成、单asset source修改、快速scroll与返回执行：materialized nodes/handles不超过visible + overscan，selection的preview lookup/parse/raster为0，单asset变化影响1项，scroll decode与entered slots同阶，同一identity无重复in-flight request，稳定1000次hover/click的SVG parse/raster/GPU upload均为0；记录cache hit/miss/eviction、RGBA resident bytes、sync/async decode、queue age/cancel、CPU/RSS和input-to-damage。当前相关Editor文件存在共享改动，本轮只读并更新E3报告，不运行Cargo，也不把历史29次parse或单窗口GPU命中外推为当前Asset Browser已达标。

### 15.67 preview cache命中后paint仍深拷贝并哈希整幅RGBA

`preview_images.rs`返回的`Image`本身已经用`Arc<[u8]>`共享像素，cache hit和节点模型中的`Image::clone`因此是`O(1)`；但paint边界立即丢失了这项优势。`template_image_pixels`在优先preview时调用`retained_image_pixels`，后者先通过`Image::to_rgba8()`把整幅Arc像素复制到新的`SharedPixelBuffer<Vec<u8>>`，随后又对`buffer.as_bytes()`执行一次`to_vec()`，再让`retained_image_resource_key`流式哈希全部RGBA，最后才把第二份Vec转回`Arc<[u8]>`。即使没有tint、文件cache命中、SVG没有重新parse且GPU texture已驻留，每次相关paint command重建仍会发生两次全图字节复制、一次全图hash、一次resource-key String分配和一次临时buffer释放。

这条成本还绕过了调用点已经算出的target size。thumbnail和普通template image都会从最终frame得到`target_width/target_height`，但preview分支只用它选择调用顺序，`retained_image_pixels`继续发布intrinsic width/height和全部intrinsic RGBA；缩放留到GPU采样。对于`P = width * height * 4`字节的preview，一次paint至少是`O(P)` hash与约`2P`瞬时CPU复制；一次全pane rebuild则是`O(sum(P_i))`。15.66中的大intrinsic SVG、全量filtered-asset materialization和selection触发整pane projection会共同放大这条链。WGPU已有`image_upload_write_count`、`image_shared_resolve_count`、`image_shared_upload_write_count`和resident-byte计数，可以证明稳定GPU上传是否为0，但GPU cache命中不能消除发生在它之前的CPU复制与hash。

正确边界不是再给`retained_image_pixels`套一层按路径cache，而是让visual generation直接拥有不可变raster product：

1. `Image`/共享visual service发布`Arc<VisualRasterProduct>`，至少包含content identity、physical extent、color space、`Arc<[u8]>`和resource key；content key在decode/raster完成时计算一次，paint不得再从字节反推identity。兼容的`to_rgba8`只保留在显式导出、截图或冷路径，不能被command builder调用。
2. preview materializer按目标physical pixels产生或引用正确尺寸的product。无tint路径只Arc clone同一product；tint变体按`(source content identity, target extent, tint, color space, raster scale)`生成一次并由共享visual cache按字节预算管理，不能在每次paint复制并改写像素。
3. `HostPaintCommand`、Runtime typed element segment与RHI image source传递同一product handle和generation。稳定paint/publication只重用handle；geometry、selection或无关catalog变化不得复制RGBA、重算content hash或重建GPU identity。设备恢复可以从同一受预算管理的CPU product重新上传，不建立Editor、Runtime和presenter三份独立owned payload。
4. 若兼容owned DTO仍需要字节数组，只在序列化或明确cold export边界物化并计数；产品render queue、multi-surface composite与异步present queue只能传共享handle。

下层RED以能直接观察的字节工作量验收：构造1、100和1,000个preview，以及32px目标对应4K intrinsic源；首次decode/raster允许一次content hash和一次product分配，连续1000次stable paint、hover、selection与frame publication要求`preview_rgba_clone_bytes=0`、`preview_content_hash_bytes=0`、`preview_product_build_count=0`且product/pixel `Arc::ptr_eq`保持；纯geometry和opacity变化同样复用product；单个tint或content变化只重建一个目标变体。产品profile同时报告command rebuild、CPU/RSS、input-to-damage、GPU shared resolve/upload/write与resident bytes，避免把GPU命中误当成整条visual链已命中。当前相关Editor paint文件存在共享改动，本轮只读并更新E3设计，不改生产实现。

### 15.68 sparse `ModelRc`只局部化写入，顺序读取仍可能退化为`O(N log N)`

当前共享工作区正在把Workbench projection从整Vec替换推进到changed-row overlay，这个方向本身正确，不能因为候选尚未动态验收就退回全量模型。`build_host_contract_workbench_window_node_patch_at_mount_and_scale`只构造pending projection nodes，记录精确`changed_rows`并调用`previous_nodes.with_row_patches`；`PersistentRowPatchMap`用按row bit分支的持久trie，单行写入只path-copy `depth = ceil(log2(row_count))`个节点。下游`HostWorkbenchHitIndex::rebind_workbench_nodes`也只检查changed rows的index membership，并复用buckets、popup rows、parent rows和未变paint index。因而单行projection、damage和hit rebind的目标复杂度可以保持`O(Delta * log N + affected cells)`，这部分不是全量重建。

但候选容器目前只优化了随机写入和随机读取，没有守住顺序读取合同。`ModelIter::next/next_back`对overlay的每一行都重新调用`patches.get(row)`；binary trie lookup为`O(log N)`，所以任何完整遍历从原连续Vec的`O(N)`退化为`O(N log N)`。`ModelRc::PartialEq`直接执行`self.iter().eq(other.iter())`，`map_preserving_metadata`也用完整iterator重新物化连续Vec；full paint/hit fallback、冷重建、debug/export或错误使用value equality时都会支付该成本。当前只有一处产品`with_row_patches`入口，且增量paint/hit会消费changed rows，因此不能把这个最坏界直接当作现有点击p95主因；但恢复、fallback和累积更新是工程合同的一部分，不能用“热路通常不迭代全部行”掩盖算法退化。

持久trie还缺少空间与密度边界。最新版本不会线性保留所有旧root，但一个拥有`P`个patch leaf的binary trie仍会常驻leaf和共享branch；旧`HostPresentationGeneration`被异步present、diagnostic或测试持有时，旧root也会继续存活。当前没有trie node count、patch density、old-generation retention或compaction计数，重复1万次单行更新的测试只验证值与depth，不验证分配、RSS和全遍历时间。`Arc::make_mut(host_presentation)`在旧presentation仍被持有时还会克隆顶层presentation结构；其中`ModelRc` clone很轻，但其它owned shell/String字段并不免费，必须与model storage计数分开。

Unreal Slate没有通过“每次比较整窗行模型”发现变化。`FSlateCachedElementData`按invalidation root保存per-widget element lists和persistent cached batches，invalidated widget只进入`ListsWithNewData`；`FSlateElementBatcher::AddCachedElements`只rebatch这些list，再线性提交已有cached batches。Zircon不需要复制Slate的数据结构，但必须保留同一原则：generation发布exact changed set，稳定/局部提交不靠value equality发现变化，完整顺序读取不因局部写优化而提高复杂度阶。

目标容器与publication合同如下：

1. `ModelRc`发布不可变model generation、row count、storage handle和exact changed-row delta。相邻generation的增量consumer只读delta；`PartialEq`不得出现在input、recompute、paint、hit或publication热路，稳定相等先由generation/handle证明。显式深值比较仅限测试、debug或cold validation并计数。
2. 保留当前持久trie时，增加按row有序的patch cursor，与base iterator归并；顺序遍历不得对每个base row重新从root lookup。维护实际trie node count，并在node count或patch density超过与`N`成比例的阈值时把当前值线性compact成新的shared-row base。这样随机`get`保持`O(log N)`、Delta写入保持`O(Delta log N)`，顺序遍历为`O(N + trie nodes)`且由compaction约束为摊销`O(N)`。
3. compaction只能发生在publication/cold maintenance边界，不能由pointer query或paint iterator lazy触发；一次generation至多compact一次，发布后storage immutable。旧generation继续读旧root，直到引用释放，不允许原位改写破坏异步present。
4. 若后续选择成熟persistent-vector库替代binary trie，必须先通过license/dependency、`Rc`/`Arc`线程边界、random get/update、linear iteration、version retention和RSS基准；当前workspace没有现成persistent collection依赖，不能未经证据新增库或再手写一套并行model authority。
5. paint/hit继续消费同一个model generation与changed rows。增量index rebind只访问changed rows，full fallback线性访问全部rows；render segment与15.65的typed publication绑定同一model generation，不能在Editor overlay、host presentation和Runtime element cache分别推导变化。

下层RED必须覆盖双复杂度而不仅是功能值：对N=1/100/10,000/100,000，分别执行Delta=1/10/1,000与连续10,000次单行替换，记录patch trie depth/node allocation、resident nodes/bytes、old generations、compaction count/time、random lookup steps、sequential base rows/trie nodes、value-equality fallback、paint/hit changed-row visits和presentation structure copy。要求稳定clone为`O(1)`且0 row clone，单行patch分配与`log N`同阶，增量paint/hit与Delta同阶，完整iterator只返回N行且总访问受线性上界约束；compaction前后像素、hit、popup、focus、route和row identity等价。当前`primitives.rs`、`persistent_row_patch_map.rs`及其projection/host提交链属于共享外部候选，本轮只读审查并更新E3计划，不吸收或修改这些源码。

### 15.69 current-source profile工具已有采集骨架，但验收仍缺样本完整性和预算判定

继续只读复核`tools/ui-profile-capture.ps1`、`profile-capture-manifest.ps1`和Runtime profiler ring后，确认不需要另建第二套profile工具。当前候选已经强制使用coordinator-managed `CARGO_TARGET_DIR`、非C盘输出、current-source Editor/Runtime二进制指纹、关键源码哈希、dirty-tree哈希和二进制freshness；自动交互覆盖click/pointer/wheel/native resize，产物也已把CPU time、working set/private bytes、input-to-damage、damage-to-submit、GPU timestamp、damage分母、cache/residency与截图/hit一致性放在同一session中。这些都是应保留的正确底座。profiler entries/bytes/drop与sealed snapshot底层预算继续由既有`PERF-MVP-566`及`PERF-MVP-324/326`拥有；本节只把它们接成UI性能验收前置条件，不建立重复owner。相关工具和interface文件存在外部未提交改动，本轮不编辑它们。

当前`-RequireScenarioEvidence`仍不能作为性能通过判据，原因是：

1. `Test-UiSurfaceLatencyEvidenceGate`只要求sample count大于0、数值非负且`p50 <= p95 <= p99 <= max`，没有检查interaction outcome覆盖率，也没有执行本计划的1 ms/8 ms p95预算。一次1000-event storm只留下一个合法延迟样本也可能通过。
2. Runtime profiler用固定容量ring覆盖旧frame/span/counter，而`ProfileSnapshot`没有written/dropped/oldest/newest sequence。压力事件超过`MaxFrames/MaxSpans/MaxCounters`时，报告可能只代表任意尾段；source manifest虽然记录容量，却不能证明measured window无丢样。
3. CPU/RSS gate只验证字段存在、非负和peak不小于start/end，没有预算、稳态回落或规模斜率。高CPU、持续private-byte增长仍会被当成“证据完整”。
4. `material_lab_*`、`viewport_toolbar_click`、`hierarchy_scroll`和`welcome_recent_scroll`会归并到通用`click/idle_hover` hotspot场景。独立interaction artifact保留目标身份，但跨artifact没有统一interaction sequence，难以证明某个latency/counter样本对应哪次产品操作。
5. 现有脚本能够执行1000 click、1000 pointer和200 resize，却还没有可配置的`N=1/100/10k/100k`产品fixture入口；因此它能测时间，不能单独证明布局、模型、导航、虚拟化或thumbnail工作量与`Delta/V`同阶。

验收协议必须在不新建并行authority的前提下扩展现有session：

1. 每个场景分为`warmup -> measured -> quiescence`三段。interaction获得单调sequence和typed outcome（damaged、intentionally-no-damage、rejected/failed）；span/counter/frame带同一capture epoch或可关联区间。`completed = damaged + no-damage + rejected`，验收要求rejected为0，且需要damage的click/resize不能记为no-damage。
2. recorder发布每类样本的written、overwritten/dropped、oldest sequence和newest sequence。measured window内任一覆盖都fail closed；扩大ring只能作为临时采集参数，不能代替容量证据。
3. 无WPR的产品运行负责预算裁决，WPR/Tracy只做失败后的热点归因，避免采集器扰动改变p95。每个场景至少独立运行3次，报告每次source manifest、完整样本数和分位数，不只挑最好一次。
4. 暖态目标固定为input-to-damage p95不高于1 ms、CPU damage-to-submit p95不高于8 ms；同时报告p99/max而不以平均值掩盖离群。稳定pointer move允许typed no-damage，且不得请求present。局部click要求Full target、full redraw和无typed reason的fallback均为0。
5. RSS验收同时报告warmup末端、measured peak/end和quiescence末端；缓存resident bytes必须能解释保留量。10倍逻辑规模时，单Delta路径的visited/alloc/presentation-copy不得随`N`同比增长；完整iterator允许线性增长，但不允许`N log N`或重复全树pass。
6. 在当前三个基础storm之外增加source-bound规模fixture：model `N=1/100/10k/100k`与`Delta=1/10/1k`、10,000 hierarchy/assets/focus/layout nodes、1/100/1,000 preview及32 px target/4K intrinsic反例。fixture规模、visible/overscan、logical/materialized count必须进入manifest和artifact，不能由文件名或人工说明推断。

因此，下一次lane开放后的顺序不是直接跑全矩阵：先由工具owner补齐sequence/capacity/outcome与预算gate的低层合同，再用一份current-source profiling build执行3次基础storm；只有样本完整且预算判定有效，才运行规模fixture和WPR归因。任何旧profile、尾段ring样本或“字段齐全但无阈值”的GREEN都不得触发下一轮算法改写。

### 15.70 profile样本留存与资源预算已进入静态候选，产品裁决仍未成立

15.69列出的采集可信度缺口已经先在最低共享层补齐，不需要建立第二套profile authority。`ProfileSnapshot`现在以向后兼容的默认空数组发布每个recorder各自的frame/span/counter留存快照；每类快照包含capacity、written、overwritten、retained以及oldest/newest sequence。recorder在写入ring时直接累计覆盖次数，start/reset同步清零；Editor合并多个snapshot时保留每个recorder的独立留存行，不能把两个相同容量的recorder折叠成一个汇总结果。旧JSON payload没有该字段时仍可反序列化，这保证诊断协议是additive change。

现有`ui-profile-capture.ps1`继续作为唯一产品采集入口，但证据校验已拆成两个小模块。latency export升级到schema 4，逐recorder验证`written = overwritten + retained`、容量和sequence边界；缺少留存证据、数据不一致或任一overwritten均fail closed。latency gate现已执行input-to-damage p95不高于1,000 us、damage-to-submit p95不高于8,000 us。process gate会交叉验证`processor_time_ms / elapsed_ms`与单核/整机CPU百分比，要求平均不超过一个逻辑核；working set/private end增长不超过64 MiB、peak增长不超过96 MiB，并按场景限制CPU工作量：click 0.5 ms/op、pointer与wheel 0.25 ms/op、resize 35 ms/step。

runner默认对每个场景执行1次fresh-process cache-prime和3次独立fresh-process measured run，run id、phase与ordinal进入manifest，run之间保留2秒quiescence；只有measured run参与强制证据与截图/WPR产物。manifest显式记录`run_process_scope=fresh_process`和`within_process_warmup=false`，禁止把前一个已经退出的Editor进程解释成对后一个进程内layout、SVG raster、GPU texture或retained publication cache的暖态预热。source manifest把recorder、Editor snapshot merge及两个证据模块纳入关键哈希，避免工具或数据合同改变后继续复用旧二进制/旧报告。当前静态证据为：latency Pester 5/5、process Pester 5/5、capture output contract 29/29，PowerShell parser通过；三个Rust候选文件的`rustfmt --check --config skip_children=true`通过。以上只证明gate会正确拒绝丢样、超预算和不一致的进程数据，不证明Editor已经达到预算。

仍有四个明确缺口。第一，interaction尚未发布15.69要求的typed action outcome与统一sequence，因此稳定pointer的intentionally-no-damage、click/resize必须damage以及rejected=0还不能由工具自动证明。第二，尚无同一Editor进程内的`warmup -> recorder reset/start -> measured -> quiescence`控制边界；现有fresh-process数据只能作为冷进程复现和缓存prime证据，不能用于“暖态p95”结论。第三，Interface窗口/IME与旧snapshot兼容合同已取得managed Cargo结果，Runtime `core-min`也已编译通过；但recorder/merge、Runtime HUD/menu和真实Editor仍未全部闭环，不得用raw Cargo或旧target替代。第四，真实Editor的1000 click、1000 pointer、200 resize三轮CPU/RSS/p95及SVG parse/raster/GPU upload仍未运行。后续顺序固定为recorder与merge下层测试、HUD/menu focused回归、fresh-process profiling Editor基础storm；同进程控制合同完成后再裁决暖态预算。基础storm满足留存和预算门后，才增加规模fixture并据动态热点选择下一项retained/invalidation算法。popup projected-hit 7路径保持独立冻结，不纳入本候选或其source manifest。

### 15.71 Interface current-source 已闭环，recorder与产品性能仍分层验收

2026-08-16的Windows受管结果先关闭了UI12指出的current-source接口编译阻断。`zircon_runtime_interface` job `74e4b39d6c8c4d42816c560cc8f94f0f`以`window_`为filter，受管build和test均exit 0，同一test binary直接枚举19个窗口/IME/adapter合同；它覆盖`ime.rs`中已经normalize为`UiInputEvent`的匹配，以及adapter四处用绑定加`is_empty()` guard表达空`preedit_clauses`的合法模式。job `2ae6125f6f9f46058cfb812c736953c8`单独运行`profile_snapshot_deserializes_pre_retention_payload`，exit 0，证明旧payload在无retention字段时仍以默认空值反序列化。共享`zircon_runtime core-min` job `c555b198299445ffb2ecc2a0b3b38595`编译exit 0，未再出现`event_routing.rs:273/519`的move-after-dispatch E0382。这三项只是编译/下层合同证据，不是Editor性能GREEN。

Runtime recorder focused job `fb40d0ac53364356b0b6cb0e543549fd`使用`recorder_`过滤器启动，目标是同时验证ring最新值留存与reset清空sequence authority。前台等待器在10分钟窗口超时后退出，底层受管`cargo/rustc`进程树继续自然终止；coordinator因supervisor断开将job记为`orphaned`、`exit_code=null`，不能冒充GREEN。该job的最新`output-test-lib-zircon_runtime`包含370条共享current-source编译错误（E0433 259、E0422 49、E0425 30，其余为少量其它错误），第一条可行动阻断是`graphics/scene/.../render/render.rs:704` E0583缺少`mod tests`文件。结构化error-span筛选中`event_routing.rs`、profiling recorder、IME和window adapter四个Runtime09-owned路径均为0。因此本轮只能证明已指定的7条编译错误不再出现，recorder测试未执行也未被证明失败。按用户要求不处理这些foreign blocker，不启动Editor job，也不重试同一filter；待共享Runtime test harness清零后再继续下层验收。

interaction完整性不能通过强制“latency样本数等于输入数”实现。现有redraw合并允许多个输入合法地共享第一个pending damage和一次present，所1:1门会把正确coalescing判为丢样。下一层合同应复用event loop已分配的单调`UiInputSequence`：每个physical input只发布一个typed outcome（damaged、intentionally-no-damage或rejected），damaged sequence以有界batch/range关联到成功present，retry保留该关联。验收要求每个请求/翻译输入恰有一个outcome、rejected为0、click/resize必须damaged，而稳定pointer允许no-damage；任何pending关联都必须有容量上限，不得在surface retry下无界保留sequence。同进程warmup/reset/start/measured/quiescence在该合同完成前仍是显式缺口，现有fresh-process cache-prime不能裁决暖态p95。

上述interaction与warmup缺口现已形成source-only静态候选。`PlatformInputTranslation`无论是否产生可分发事件都保留翻译前分配的`UiInputSequence`；event loop使用一个active input和一个pending present batch记录damaged、intentionally-no-damage、rejected outcome。连续damaged input可以跨no-damage input合并为`first_sequence/last_sequence/damaged_count`，只有成功present消费batch，retry保留；实现没有`Vec`/`VecDeque`等随事件增长的容器，状态空间为`O(1)`。outcome与input-to-damage使用一次counter batch提交，present range与damage-to-submit也使用一次batch提交，避免同一逻辑事务在ring覆盖边界被拆开。schema 5 evidence拒绝重复outcome sequence、孤立batch字段、错误range顺序、damaged membership不等和latency计数不匹配；它要求input-to-damage样本数等于damaged outcome数、damage-to-submit样本数等于present batch数，而不是等于全部输入数。

同进程测量不再用已经退出的cache-prime进程模拟暖态。每个交互measured进程先完成可配置数量的successful warmup presents；最后一次warmup present只推进`Warmup -> RestartPending`，不在present调用栈内reset recorder。事件/present中的`ProfileScope`和`ProfileFrameScope`全部drop后，`about_to_wait`才执行一次reset、清空input outcome tracker并从环境重新start capture；成功进入`Measuring`，失败进入`RestartFailed`且不把无capture状态误标为measurement。source-bound geometry/screenshot在warmup present先行提交，供自动交互定位，随后其counter/CPU开销被reset清除。重启成功后在既有非C盘session目录通过临时文件+rename发布schema 1 readiness marker，记录当前process id；runner启动前清除同session旧marker，自动交互在`SetForegroundWindow`或鼠标事件前必须等到PID匹配，否则fail closed。interaction完成后继续在同一PID内执行默认2秒quiescence，每100 ms刷新working set/private bytes并扩展峰值，末点增长使用64 MiB门、包括静默窗口的峰值使用96 MiB门；缺字段、PID不等、采样未完成或elapsed短于requested均拒绝。startup场景仍为fresh-process，交互场景manifest记录`run_process_scope=within_process_warm_measure`、warmup present count与同进程quiescence时长；多个measured进程之间另保留进程级间隔，不能混作RSS回落样本。readiness与输出目录authority加入critical-source fingerprint，总数为103。

当前非Cargo证据为latency 7/7、capture/output 30/30、process CPU/RSS 6/6、native resize 4/4，共47/47；PowerShell parser、相关Rust文件的`rustfmt --check --config skip_children=true`和scoped `git diff --check`均通过。共享coordinator已有另一受管Cargo作业，且Runtime test harness仍有foreign current-source blocker，因此本候选没有启动raw或额外managed Cargo；不能据此宣称Rust已编译、真实Editor暖态p95已达1 ms/8 ms、RSS已回落或SVG parse/raster/upload已归零。下一步仍是共享lane释放后的lowest-layer managed Rust回归，再运行source-bound Editor三轮click/pointer/resize产品profile。

### 15.72 hierarchy规模输入已绑定真实项目，逻辑Delta与产品观测仍未伪造

在共享Cargo lane继续占用期间，15.69的规模夹具缺口先完成了一个不依赖Rust编译的最小真实切片。`ui-profile-capture.ps1`为`hierarchy_scroll`增加可选`HierarchyLogicalNodeCount`，在measured Editor进程启动前调用独立`ui-profile-scale-fixture.ps1`：它只接受仓库外、非C盘且尚不存在的目标，复制当前canonical `templates/projects/renderable-empty`，以流式短记录覆盖`assets/scenes/main.scene.toml`并精确写入`N=1..100,000`个根实体。采集随后使用正常`--project`产品入口打开该项目；scale materialization先于`ZIRCON_PROFILE_CAPTURE=1`、source manifest和Editor启动，因此夹具生成时间不污染interaction epoch。

source manifest升级到schema 2并把scale工具加入第7个capture-tool fingerprint。非空`input_fixture`记录kind、project root、template owner、logical N、scene entity count，并分别记录复制后的`zircon-project.toml`与scene path/SHA-256/bytes/last-write；manifest写入前重新验证外部root、两个输入的精确project-relative位置、N与实体数一致且当前hash/length未变化。生成后修改任一输入都会以`UI profile input fixture changed after materialization`拒绝采集。实际wheel数量不再从`AutoWheelCount=0`误读为0：`Get-ScenarioRequestedWheelOperationCount`把默认值解析为24，并让交互发送和manifest使用同一数值。

这里主动删除了最初仅被记录、未实际改变产品模型的`HierarchyDeltaNodeCount`。保留该字段会制造“逻辑Delta已测”的假证据；当前完成的只有真实hierarchy N输入和真实wheel operation count。逻辑实体增删Delta、Editor实际观察到的logical/materialized/visible/overscan、asset/focus/layout/preview规模输入仍是open，不能由scene文件名或声明字段替代。

生成器上限实测使用`N=100,000`：优化前每实体7次`WriteLine`为10.987 s，改为每实体一次短记录流式写入后为4.741 s；最终scene 21,888,895 bytes，最后`entity = 100000`与名称存在。当前PowerShell进程终态working set约+25.77 MB、private约+18.93 MB、managed heap约+1.38 MB；这些是生成命令的起止样本，不冒充峰值RSS。两轮临时目录均在精确校验`E:\zircon-profiles\scale-generator-validation-runtime09`后删除，没有写入C盘。

最新非Cargo合同为latency 7/7、capture/output 31/31、process 6/6、native resize 4/4、scale fixture 4/4，共52/52；scale安全合同同时拒绝普通`C:\`和`\\?\C:\`/`\\.\C:\` device namespace别名。相关PowerShell parser为0 error，scoped diff check通过。该GREEN只证明输入生成、完整性和采集时序合同；它没有运行Editor、没有证明100,000实体成功materialize为预期UI行、没有给出CPU/RSS/p95或SVG/GPU命中，也不改变managed Rust与产品profile继续等待lane授权的事实。

### 15.73 Asset Browser规模输入已落地，下一步先修identity与虚拟化而不是扩大cache

`ui-profile-scale-fixture.ps1`现在提供第二种source-bound输入`asset_catalog_json`。它从canonical `templates/projects/renderable-empty`复制独立项目，在`assets`根目录精确生成1至10,000个`profile_catalog_asset_000001.json`至`profile_catalog_asset_010000.json`；每个文件都是UTF-8无BOM的合法最小JSON，使用Runtime内建`zircon.builtin.data.json` importer进入真实catalog，不伪造`.zmeta`或artifact。`ui-profile-capture.ps1 -Scenario asset_refresh -AssetCatalogItemCount N`在capture环境和Editor启动前生成该项目，通过正常`--project`入口打开，并在refresh交互中继续写入合法JSON。asset规模与hierarchy规模互斥，其他场景传入该参数会fail closed。

source manifest会在启动前重新验证project manifest、scene以及整个asset source set。asset set合同要求精确文件数、连续命名、总字节数和按`project-relative path + length + per-file SHA-256`聚合的digest全部保持一致；缺失、额外同前缀文件或任一文件内容变化都会拒绝采集。没有生成raw `.svg`规模集，因为当前project importer registry没有把独立`.svg`注册成可导入asset source；制造10,000个无法进入有效catalog/preview产品链的SVG只能形成虚假证据。JSON夹具只证明真实catalog规模输入，不能证明SVG parse/raster或GPU upload缓存命中。

10,000项生成器在E盘隔离目录的最新实测为278,894 source bytes；生成、首次完整性hash共39.835 s，随后独立完整性重算6.379 s。PowerShell进程起止working set约增加66,592,768 B、private bytes约增加56,311,808 B；这仍是起止采样，不冒充峰值RSS，也不属于Editor性能。早先实现的独立重算为10.072 s；改为按预期连续路径流式校验后重算下降36.7%，但文件创建阶段受小文件IO波动影响，本轮不宣称总生成时间改善。两轮临时目录均在验证后从`E:\zircon-profiles`精确删除，没有写入C盘。

最新非Cargo合同为latency 7/7、capture/output 32/32、process CPU/RSS 6/6、native resize 4/4、scale fixture 7/7，共56/56；其中asset夹具覆盖合法JSON、连续命名、全量source fingerprint变更检测、10,000上限以及超限前零写入。五个改动PowerShell/测试文件parser为0 error，scoped `git diff --check`通过。五套串行合并运行在180秒外层时限处进入scale第6项前被终止，因此没有把该批次冒充单次GREEN；最慢的capture/output随后单独32/32 exit 0，scale单独7/7 exit 0，前一批已完整结束的latency、process和resize分别为7/7、6/6、4/4。

对current source和`dev/UnrealEngine`的复核把后续优先级收敛为结构问题，而不是“GPU没有cache”：

1. Zircon thumbnail模式对全部`snapshot.visible_assets`循环，每项立即调用`load_preview_image_for_generation`并追加约8个节点；list模式先为全部资产构建cell字符串，再在每个逻辑row上用`nodes.iter().any`查找现有节点，当前同步阶段最坏为`O(N * M)`，当节点数随资产数增长时接近`O(N^2)`。当前源码没有把logical filtered assets与viewport内materialized rows/tiles分离。
2. preview miss同步执行文件候选探测、读取、`usvg` parse和intrinsic-size raster。`catalog_revision ^ resource_revision`会让无关catalog revision改变全部thumbnail generation，同时存在XOR碰撞；preview publish只推进`publish_epoch`而snapshot没有独立per-asset preview identity时又可能保留旧图。这一层同时制造过度失效和失效不足。
3. WGPU窗口局部cache和device-shared registry都以稳定`(resource_key, generation)`查询，各自上限256项/64 MiB；预算内稳定命中不扫描驻留集。新identity越界时，admission会扫描候选并按last-touched/key/generation排序淘汰。上层generation churn因此会把现有GPU cache持续推入共享miss、upload和`O(C log C)`淘汰路径，但不应通过再增加第三套Asset Browser GPU cache解决。
4. Unreal Slate `SListView::ReGenerateItems`从`floor(CurrentScrollOffset)`得到`StartIndex`，只循环生成到viewport被填满；`GenerateWidgetForItem`优先复用已生成row，generation pass结束时清理本帧未出现的row。逻辑items仍由模型持有，widget数量和昂贵资源请求则与viewport及少量边界项相关。这是Zircon Asset Browser应采用的authority分层，而不是照搬某个类名。

实现与验证顺序固定如下。第一阶段增加typed `AssetPreviewIdentity`，把project namespace、asset UUID、artifact/content generation、target physical extent和raster scale分字段比较，先用下层RED关闭全局revision扩散、XOR碰撞与同路径preview更新。第二阶段把filter结果保持为逻辑model，只为`visible + overscan`维护稳定slot，list和thumbnail共用scroll window authority；选择变化只patch旧/新slot，单asset delta只patch对应record。第三阶段把preview miss移出pointer/recompute热路，使用按identity去重、有界、可取消并按可见性排序的worker，完成后只发布一个slot damage。第四阶段让preview消费主visual service的SVG tree和target-raster product，paint直接使用预先计算的内容key与`Arc<[u8]>`，不得再次复制/哈希整幅RGBA。第五阶段才运行1,000/10,000 asset产品profile，覆盖快速scroll/返回、连续selection、单preview完成、单source修改和稳定hover/click。

第五阶段还有一个不能伪造的前置合同：当前`ui_profile_geometry.json`只发布通用layout regions、dispatchable template controls、Hierarchy使用的left region与Welcome recent viewport，没有发布Asset Browser content viewport或其logical/materialized/visible/overscan count。Asset content wheel由独立`AssetContentListPointerBridge`在browser viewport内路由；把Hierarchy的`layout.left_region`、任意可点击source-tree control或窗口比例坐标当作目标，可能命中错误surface。后续必须由geometry owner从`AssetContentPaintMetadata::viewport()`和同代次pane origin发布唯一`asset_browser.viewport` named frame及规模计数，runner只接受该source-bound target并拒绝ratio fallback。相关geometry schema/collector文件已有共享未提交改动，本轮不叠加编辑。

产品验收必须同时证明：`logical_count=N`且`materialized_count <= visible + overscan`；scroll新增decode与新进入window的slot同阶；selection的preview lookup/parse/raster/upload为0；单asset变化只改变一个preview identity；稳定1,000次hover/click的SVG parse、raster和GPU upload均为0；GPU admission prune visits在稳态为0；input-to-damage p95不高于1 ms、damage-to-submit p95不高于8 ms，CPU/RSS满足15.69预算。当前只完成了输入与静态合同，没有运行Editor、没有证明10,000 JSON已形成10,000个产品row，也没有声称上述动态指标GREEN。共享Runtime/Editor编译blocker继续由原owner处理，本节不修改对应Rust源码且不启动Cargo。

### 15.74 virtual scroll 的 `O(1)` 窗口数学被 alias mutation 与伪状态失效放大

继续从 Editor 集合投影追到 Runtime table/tree 默认滚轮后，发现一个比 `Vec` 切片更接近交互热路的结构问题。Editor `collection_window.rs` 确实只把 `visible + overscan` 交给宿主模型，但 `projected_collection_items` 会先克隆完整字符串数组，table row 也先转换并分配全部合法行再切片；这只能降低宿主模型常驻量，不能把投影 CPU 从 `O(N)` 变成 `O(V)`。不过 current-source 产品搜索表明该通路主要服务 component showcase、TemplateV2 host conversion和少量插件table，不是 Asset Browser/hierarchy的主数据authority，因此本轮没有为了一个容易通过的局部改动而抢先改它。真正的Runtime滚轮路径位于 `default_interactions/table/virtualization.rs` 与 `tree_view_virtualization.rs`。

table和tree的窗口计算本身是常数时间：从同一owner metadata读取total/start/count/row extent，按scroll delta得到next start，再计算visible/requested range。问题出在提交阶段。每次有效滚轮都会依次调用18次独立property mutation：5个total-count名称、`viewport_start`、`viewport_count`、2个visible-end名称、2个requested-start名称、2个requested-count名称、3个overscan名称以及2个scroll-offset名称。每次 `mutate_property` 都单独执行metadata `BTreeMap`更新、runtime-style baseline更新、component-state value克隆、binding report重算与dirty记录。由于 `UiSurfaceComponentStateStore::sync_from_property` 对任意属性都先写入 `values` 并以该写入结果返回changed，普通数值元数据也进入 `mark_component_state_render_dirty`；后者为每次调用重新构造单元素`BTreeSet`、检查ancestor pseudo selector、计算minimal style roots、应用node/subtree runtime style并记录render dirty。runtime selector从`UiComponentState.flags`收集hover/focus/pressed/checked/disabled/expanded/open/selected/drag/loading等retained pseudo state，也会把非retained的`bool=true` metadata作为自定义pseudo state；这些window属性是数值而不是布尔selector state，因此这里重复的style工作没有语义收益，但不能把所有非retained属性一概跳过。

因此一次滚轮在最终rebuild前虽然会被pending invalidation合并到同一个node，rebuild之前仍已支付18轮状态传播。令alias数为`A=18`、owner metadata属性数为`P`、一次style eligibility/node apply成本为`S`，当前提交前成本至少为`O(A log P + A*S)`，并产生18份binding report和18次component value clone；它不是随列表逻辑项数增长的复杂度错误，却是直接发生在每个wheel event上的大常数放大。它也解释了为何仅证明hit grid查询为局部、窗口数学为`O(1)`仍不足以证明按钮/滚轮可交互：任意非pseudo property mutation都可能走同一伪状态重算通道。当前table/tree回归只断言18个alias最终值和一个`SetVisibleRange`事件，没有限制binding report数量、style apply次数、dirty提交次数或每滚轮CPU。

Unreal与Slint的参考边界再次指向“一个typed状态转换，而不是一串字符串属性”。Unreal `SListView`的generation pass按current scroll offset取得逻辑item，只对进入viewport且无现有row的item生成widget，`OnItemSeen`保留复用映射，pass结束清理未出现row；scroll owner直接维护offset/visible widgets，不把一个窗口状态扩散成camel/snake alias mutation。Slint `update_visible_instances`在单个`RepeaterLayoutState`中维护offset、anchor和cached item height，复用已有instances、按需prepend/append并一次清理离窗项；`row_changed`也只更新当前offset覆盖的instance。Zircon可以继续在冷兼容边界发布Web/MUI alias，但热路authority必须是一份typed virtual-window transaction。

优化分三层执行，不能把最终目标缩成只减少几次函数调用：

1. 先拆开component value变化与runtime pseudo-state变化。`sync_from_property`发布结构化change classification；只有flags真实改变时才执行runtime pseudo-style node/subtree apply。普通值仍保留binding/value语义并按真实metadata dirty domain标记render/layout/hit/input，但不得重新匹配pseudo selector。下层RED要求numeric window mutation分类为`value_changed=true, pseudo_state_changed=false`，hover/focus/pressed等则只在flag转换时报告pseudo-state changed。
2. 再建立单node的typed `UiVirtualWindowState`/mutation transaction。窗口数学只生成一次canonical state；兼容alias在同一tree/component/style transaction内批量写入，合并dirty domain并只执行一次node invalidation。对外仍可在一份`UiBindingUpdateReport`中保留每个实际changed property update，但一个wheel不得发布18份独立report或重复style/dirty提交。table和tree必须共用该事务，不维护两份alias列表与提交算法。
3. typed transaction完成后才接真实data-source virtualization：model count/logical extent与materialized instances分离，scroll transaction驱动entered/exited stable slots、focus/a11y虚拟集合和异步数据请求。此层继续以15.62、15.73及`docs/plans/optimize` Runtime11a/Editor01的bounded materializer方案为authority，不能把alias批处理冒充列表已经虚拟化。

验收先锁定热路工作量，再测产品延迟。table/tree下层回归对一次有效scroll分别要求：一个typed window transition、一个component event、一个合并binding report、一个owner dirty transaction、numeric alias pseudo-style apply为0，最终全部兼容alias值与旧行为相同；第二次同方向scroll保持同一界，边界无移动时全部计数为0。再以1,000次wheel和N=1/100/10,000/100,000逻辑项测window math、property writes、style node/subtree applies、binding reports/updates、dirty commits、rebuild nodes、materialized slots、CPU与RSS；提交前工作不得随N增长，完成data virtualization后measure/arrange/render/hit与`visible + overscan`同阶。产品层继续使用source-bound `hierarchy_scroll`和未来`asset_browser.viewport`，执行三轮同进程warmup/measured/quiescence，要求input-to-damage p95不高于1 ms、damage-to-submit p95不高于8 ms且无样本覆盖。

下层动态证据入口已经加入table虚拟窗口回归，但结果仍保持pending。手动ignored基准固定为31个独立sample，每个sample从已prime的surface执行1,000次正负24 px交替wheel，并在每次dispatch后执行incremental `rebuild_dirty`；它对N=1/100/10,000/100,000分别输出storm与单event的p50/p95，同时累计typed transaction、`SetVisibleRange`事件、binding updates、dirty nodes、layout visited、arranged/hit/render outer visits以及render command rebuild/reuse。N=1必须是1,000次边界no-op且上述transaction/event为0；其余规模必须恰好每event一个transaction和一个component event，同一N的31轮工作计数完全一致，并且N=100/10,000/100,000之间的全部工作计数也必须相等，直接拒绝随逻辑行数增长的回归。该入口只裁决Runtime算法工作量和墙钟趋势，不能替代进程RSS、GPU/SVG缓存或input-to-damage证据；这些仍由既有`tools/ui-profile-capture.ps1`在非C盘执行三轮source-bound Editor产品采集。当前尚未取得managed test/benchmark或Editor profile终态，不能据此声称该项已经修复或已是唯一瓶颈。

生产候选已经按上述边界完成，但动态验收仍分层保持open。`UiComponentStatePropertyChange`将普通value与真实pseudo flag转换分开；通用property mutation仅在真实retained flag转换或runtime style索引确实依赖某个自定义布尔pseudo名称时进入runtime pseudo-style，其他value-only变化保留binding/component value并只走真实dirty domain。第二轮复审补充的`pseudo_state_names: BTreeSet<String>`在document编译时从selector state token构建，mutation热路是一次集合查询而不是扫描全部rules；self与ancestor-descendant自定义布尔pseudo各有一条直接回归。table/tree现共同构造一份`UiVirtualWindowState`，`mutate_tree_metadata_properties`在一个owner borrow中跳过未变alias、合并dirty并生成reflected updates，surface层再一次同步runtime-style baseline/component values、构造最终binding report并调用一次`mark_node_dirty`。静态source guard为typed window struct 1、共享surface transaction 1、lower batch 1、旧per-alias helper 0、intermediate property report 0、pseudo-style call 0、table/tree最终report push 2；当前12个Rust/报告路径的格式与diff检查继续作为静态门，功能和style过滤器必须在managed testing stage动态通过后才允许标成GREEN。

默认feature受管job `33329a15cc4d45528169cb059535b7c2`绑定current worktree运行`zircon_runtime --lib virtual_window`，在593秒后以exit 1规范release，测试数为0。最新fingerprint `zircon_runtime-0c255ba51a4c1888/output-test-lib-zircon_runtime`只有3条真实错误且primary span均不在本候选：`realtime_ibl_graph_plan/tests.rs:135` E0422缺`CubeMipRange`，`dynamic_api/session/extract_stats.rs:212` E0599对`&[u64]`调用`capacity()`，`realtime_ibl_time_slice/tests.rs:37` E0599引用不存在的`RealtimeIblOperation::CaptureCloud`。按用户要求本轮不修改这些graphics/dynamic foreign blocker。后续`--no-default-features --features ui`的同filter请求至今未启动Cargo：请求分别在admission checkpoint变化、UI12 reservation或兼容池busy阶段被拒绝，均没有Runtime09 job、测试数或Cargo exit code；最新兼容池owner为UI12 Editor构建job `7f2706f55bc14b97b91d47b9494ccc4a`。只有UI-only回归、ignored 1000-wheel基准和三轮Editor产品profile取得source-bound终态后，本节才允许发布性能收益结论。

### 15.75 `hierarchy_scroll`只能裁决Editor产品路径，不能归因Runtime virtual-window收益

继续核对产品采集器与当前Editor滚动实现后，必须把下层算法证据和产品交互证据明确拆开。`hierarchy_scroll`自动输入以`ui_profile_geometry.json`中的`layout.left_region`为唯一source-bound target；native scroll dispatch直接更新Editor私有`HierarchyPointerState.scroll_offset`，再把数值写入pane context。paint阶段由`visible_hierarchy_row_range`按viewport、clip、scroll和row metrics算出可见索引区间，只对该区间调用`row_data`与`draw_hierarchy_row`。全仓产品调用搜索没有把该路径接到Runtime `apply_default_table_virtual_scroll`、`apply_default_tree_view_virtual_scroll`或本轮`UiVirtualWindowState`事务。因此`hierarchy_scroll`是整体Editor hierarchy滚动是否可交互的正确产品门，却不能被用来声称Runtime alias batching直接降低了产品CPU或p95。

当前产品gate已经保留了有价值但不同层级的约束：每个wheel必须对应一个`hierarchy_scroll_dispatch_count`，并要求`hierarchy_surface_rebuild_count`、`hierarchy_row_insert_count`、`hierarchy_dispatcher_rebuild_count`和`hierarchy_route_map_rebuild_count`全部为0；interaction artifact还要求完整CPU/RSS证据、每wheel CPU不高于0.25 ms，以及15.70定义的input-to-damage和damage-to-submit预算。这能拒绝事件期重建整套pointer authority。它尚未发布`logical_row_visit_count`、`painted_row_count`、`visible_row_count`或overscan计数，所以即使N=100,000产品profile通过，也只能结合current-source的visible-range实现推断paint为`O(V)`，不能仅凭五个零重建计数证明所有hierarchy工作都与逻辑N无关。另一个冷边界仍为`sync_hierarchy_pointer_layout`：scene entries变化时会复制完整row slice并为全部entity分配字符串node id；wheel gate证明scroll不应触发它，但scene delta/full resync仍需单独规模验收。

后续证据顺序据此固定。第一，Runtime typed transaction只由UI-only `virtual_window`功能/样式回归和N=1/100/10,000/100,000的ignored 1000-wheel基准裁决。第二，Editor `hierarchy_scroll`以真实100,000实体项目运行三轮warmup/measured/quiescence，裁决私有hierarchy pointer、visible paint、damage、CPU/RSS与latency；在产品结论中必须明确它不经过Runtime transaction。第三，若要证明Runtime table/tree优化已进入产品，必须选择一个实际使用这些Runtime组件的产品surface或component showcase，发布唯一geometry target及`virtual_window_transaction_count`、pseudo-style apply、dirty commit和materialized-row计数；不得把Hierarchy左栏当作替代目标。第四，hierarchy结构里程碑补充visible/paint/logical-visit计数，并分别对stable scroll与scene delta/full resync给出`O(V)`和`O(Delta)`证据；在这些计数出现前，不把“零authority重建”等同于完整算法GREEN。

### 15.76 current-HEAD SVG/GPU资源身份复核：先裁决代际抖动，不增加第三套cache

2026-08-22以`HEAD bee4c707`重新沿`logical visual asset -> SVG tree -> target raster -> immutable icon-atlas page -> UiSurface image resource -> presenter-local bind group -> device-shared texture`读取当前源码。结论没有改变：Zircon并非缺少SVG或GPU缓存。SVG tree以`Arc<usvg::Tree>`和路径alias保留，raster cache为4096项/64 MiB，icon atlas为64页/64 MiB，WGPU local与device-shared层各为256项/64 MiB；稳定`(resource_key, generation)`命中不会重新parse、raster或`queue.write_texture`。当前需要裁决的是资源identity/generation是否稳定到达这些cache，以及跨层miss究竟停在哪一层。

当前command publication传递`(resource_key, generation)`并在需要时保留`Arc<[u8]>`兜底，但Editor的`is_image_resource_resident`有意只查询presenter-local cache。device-shared registry不能暴露成extract期的无条件resident bool，因为extract与present之间可能发生LRU淘汰；present阶段仍必须原子执行shared resolve/prepare。因而“local miss + shared hit”会避免GPU upload，却会为该presenter创建一次texture view/bind group并增加local cache allocation。它不是重复上传，也不是SVG重载。现有`gpu_image_prepare_cache_hits`、`gpu_image_shared_resolves`、`gpu_image_cache_key_allocations`与`gpu_image_shared_upload_writes/bytes`足以区分这四种状态，产品profile必须成组解释，不能只看upload或只看resident bytes。

icon atlas采用immutable sealed page：已见`IconSourceKey(resource_key, generation, width, height)`稳定命中原slot且不推进page generation；新identity只能写入新的未sealed page。持续产生新identity时会增加page generation，达到预算后按非active page的LRU淘汰并重用page index。这个策略保证已发布draw list不会观察到页内像素突变，但会把上游无关generation churn放大为新atlas页、local allocation、shared upload和淘汰工作。因此下一轮先量化`unique icon identity / atlas generation / local allocation / shared upload / prune visits`，只有证明identity稳定而immutable page策略本身仍造成主要p95/RSS压力，才评估增量页或更强的stable handle；不得以修改atlas策略掩盖错误的catalog/theme/frame generation传播。

参考实现支持同一裁决。Unreal Slate让`FSlateResourceHandle`成为可缓存且在资源销毁时失效的稳定句柄，vector cache以brush identity与pixel size聚合pending raster request，RHI resource manager再统一解析static/dynamic/vector resource；draw热路不重新解释源文件。Slint分别保持`ImageCacheKey -> ParsedSVG`与`TextureCacheKey(source, scalable target size, rendering, tiling)`，renderer只为具体target variant创建纹理并清理失去强引用的项。Zircon后续若引入更强句柄，也必须只统一现有identity和lifetime，不复制现有tree/raster/atlas/WGPU存储层，更不能删除shared LRU miss时保证正确显示所需的CPU source fallback。

动态顺序固定为：先用同一current-source profiling Editor执行SVG冷/暖、1000次稳定hover/click、200次resize、第二native window首次消费和64 MiB压力；分别报告candidate/tree parse、raster、atlas generation、local prepare hit/allocation、shared resolve/upload/prune以及CPU/RSS/GPU resident p50/p95/p99。暖单窗稳定交互要求parse/raster/generation/allocation/upload/prune均不增长；第二窗口同代资源允许shared resolve和一次local allocation但upload必须为0；单SVG内容变化只允许对应identity产生一次新raster/atlas generation和每device一次upload。当前`UiScenarioHotspot`已经保存shared upload、local allocation和prune visits，但`ui-profile-capture.ps1`只在`window_resize`专用gate读取local allocation，通用`click/idle_hover`证据展示尚未把这组字段成组纳入；产品矩阵前必须由现有工具owner补齐该读取/gate，不能把未展示字段当成0。证据显示generation churn来自上游时先修identity owner；只有identity稳定而跨窗口local allocation或atlas immutable-page churn仍成为主要热点，才进入stable resource handle/atlas policy实现。当前没有运行该产品矩阵，本节只收口算法判定与验证计划，不发布性能GREEN。

### 15.77 damage路径已有空间剪枝，下一步应证明changed segment而不是继续堆rect判断

2026-08-22对current HEAD的Editor command publication重新逐层核对后，需要修正“局部damage仍无条件全量遍历全部dock/全部template row”的过宽表述。`record_host_frame_commands`确实在每次有效publication中进入`draw_workbench_presentation_commands`，但componentized路径已经具备两级有效剪枝：`dock_damage_route`只访问与paint clip相交的left/document/right/bottom dock并发布`ui.paint.dock_route_visit_count`；template节点随后优先使用`HostWorkbenchHitIndex`返回的paint rows，并发布`workbench_paint_index_query/candidate`、`template_node_visit/clone/damage_reject`与`template_command_count`。因此，不能再把一次左栏局部damage描述成必然扫描四个dock和全部model row，也不能用增加更多叶子级`intersect`作为下一轮结构优化。

剩余未证明的成本边界位于空间候选之后。当前`build_chrome_command_stream_with_residency`仍为每次publication调用`extract_chrome_commands`，把本次所有`HostRecordedPaintCommand`逐项转换成新的`ChromeCommand`，再让`pack_editor_icons_into_atlas`至少三次遍历本次command slice：先找已驻留page、再收集pending icon、最后改写image payload。`ChromeCommandPatchCount=1`只说明最终stream带patch语义，不等于上一代command segment被原位复用。顶层componentized painter还会依次进入top chrome、componentized chrome、dock route、extension workspace、resize、floating、page overflow、menu/prompt与root overlay；其中部分函数会按damage早退，部分只有scope而没有候选/接受计数。故真正需要回答的是：对固定小damage和单control变化，进入多少layer、读取多少空间候选、克隆多少node、尝试/接受多少recorded command、转换多少command、扫描多少atlas command，以及最终替换多少稳定segment；而不是笼统问“是否局部刷新”。

参考源码给出两种成熟但不可直接混合的authority。Unreal `SlateInvalidationRoot.cpp`的`PaintInvalidationRoot`在slow path与`PaintFastPath`之间显式分流；fast path消费按widget sort order维护的`FinalUpdateList`，复用`FSlateCachedElementData`，只在layer/reindex依赖要求时把相邻widget加入更新范围。Slint `partial_renderer.rs`仍会在dirty-region计算阶段遍历item并比较缓存geometry/property tracker，但会缓存每项bounding transform、传播clip/opacity对子树的影响，并在实际render阶段以dirty region过滤绘制。Zircon当前已接近Slint的空间候选层，却尚未证明Unreal式“旧命令保留、只替换changed element list/segment”；因此目标不是复制两套系统，而是让既有generation、changed rows、damage index和command publication共享一个typed changed-segment合同。

在改生产结构前先补一组连续动态计数，并复用现有profile session而不是另建采集器：`paint_layer_entry_count{layer}`、`dock_route_visit_count`、paint-index query/candidates、template visit/clone/reject、`recorded_command_candidate/accepted`、`chrome_command_conversion_count`、`icon_atlas_command_visit_count`和`paint_segment_replace/reuse`。以N=1/100/10,000/100,000、Delta=1/10、固定visible range和固定damage运行stable hover、单按钮状态变化、单pane resize及full fallback；要求stable no-damage路径上述publication计数全为0，单control变化只访问相交layer且`candidate/record/convert/atlas/replace`受`O(overlap candidates + changed segment)`约束，10倍逻辑N不得同比放大，full fallback才允许线性访问N。每组同时记录CPU/RSS、input-to-damage与damage-to-submit p50/p95/p99，并保留像素、clip、z-order、popup和hit等价回归。

只有计数证明空间候选之后仍是主要p95热点，才实施15.65已定义的稳定`PaintSegment`：presentation generation发布精确changed segment，未变segment以共享handle复用；command extraction仅重建changed segment并按稳定z/clip顺序组合；icon atlas只检查changed image segment，GPU继续消费同一resource identity。若计数显示热点发生在候选生成、preview raster或GPU present，则回到对应owner修因，不以segment重构掩盖。当前Editor paint/profile路径均有其它会话活动，本轮只把current-source事实和RED门槛写入E3报告，不编辑生产源码，也不把静态剪枝存在解释为产品性能GREEN。

### 15.78 规模输入已存在，但现有resize/hierarchy gate仍可漏过全命令扫描

对HEAD中的`ui-profile-capture.ps1`、`ui-profile-scale-fixture.ps1`与manifest重新核对后，15.69第5项应按时间顺序理解为当时缺口，15.72/15.73已经补上真实规模输入：`HierarchyLogicalNodeCount`可生成并打开仓库外、非C盘的N<=100,000项目，asset catalog也可生成N<=10,000真实JSON source；manifest会绑定project/scene/source-set指纹，默认每场景运行3个measured进程，并保留同进程warmup与quiescence。后续不得再创建第二套fixture，也不能声称工具仍完全没有N入口。尚未落地的是产品实际观测到的logical/materialized/visible/overscan、逻辑Delta输入和SVG可导入规模fixture，这些不能从manifest的声明N反推。

`Test-HierarchyScrollCounterGate`当前要求requested/completed wheel一致、每个wheel恰有一次`hierarchy_scroll_dispatch_count`、平均CPU不高于0.25 ms，并要求surface/row insert/dispatcher/route-map五类authority rebuild全为0。这能拒绝事件期重建pointer authority，却没有读取`logical_count`、`visible_count`、`materialized_count`、paint-index candidates、template visits或painted rows。因而一个对100,000逻辑项每步执行O(N)只读扫描、但没有重建这五类结构且机器足够快的实现仍可能通过；当前gate证明的是固定预算与零重建，不是复杂度为O(V)。

`Test-WindowResizeCounterGate`更接近产品行为：它要求同一command snapshot只build一次并被复用，workbench model/chrome snapshot最多一次，SVG tree miss与visual miss为0，image upload/cache-key allocation最多一次，shell authority/dispatcher/route map不重建且geometry patch实际发生。缺口同样明确：它虽然读取`gpu_image_prepare_command_visits`，却不把该值纳入返回条件；也不读取shared upload bytes/writes、cache prune visits、paint-index candidates、recorded/converted command或atlas visits。CPU门是每resize step不高于35 ms的进程总量，不是15.69的input-to-damage <=1 ms和damage-to-submit <=8 ms p95。因此“snapshot reuse + SVG hit”仍可能掩盖每步O(C) prepare/atlas/command扫描，允许一次measured upload/allocation也弱于15.76定义的暖态零增长合同。

通用`Show-UiScenarioEvidence`目前展示local upload、shared resolve、prepare hit和SVG tree hit/miss，但没有把shared upload、local cache allocation与prune visits成组读取；可选字段缺失还会被显示为0。该行为适合向后兼容旧schema，不可作为“当前字段为零”的验收证据。工具owner下一轮应在不改变现有artifact authority的前提下：对current schema要求这些字段存在，把15.77的command-work计数接入resize/hierarchy明细，并把source-bound N与产品报告的实际规模做相等/上界校验。E盘现存8月9日至11日Runtime09历史profile不绑定当前`bee4c707`源码和上述新gate；本轮不复用旧数字，不宣称current-source动态GREEN。

### 15.79 projected popup authority已进入HEAD，动态验收仍须补齐

此前15.30/15.32记录的exact-7 worktree候选现已由commit `3019fae55 feat(runtime09): integrate popup frame authority`进入HEAD，后续整合提交没有删除它。current HEAD中的`frame_hit_test.rs`、rebuild、frame publication和tree hit-test都包含projected authority；旧事件期arranged/render扫描guard保持0，局部patch不扫描全部base entries。当前`surface.rs`、`widget_menu_behavior.rs`与default interactions的未提交差异属于Runtime74 compiled binding/component-event推进，不是popup repair，也未被本轮吸收。

集成状态不能代替current-source Rust GREEN。共享Cargo lane释放后只通过Windows managed validation依次运行：`frame_hit_test`下的non-uniform affine、z crossing fallback、base full rebuild、clip refresh、popup inner-z/stack order与no-global-scan source guard；tree hit-test的`missing_ephemeral_lookup_requires_explicit_reindex`；菜单产品合同的rendered target/old placeholder、frame/instance authority、physical virtual pointer和parent input descendant patch。每张结果必须绑定当前HEAD/source manifest并报告ticket/job/run、实际测试数与exit code；任何失败只返回第一条current-source actionable error，不并发扩散。该验证完成前，报告继续把popup修复标为“已集成、静态合同保持、动态open”。

本轮首次提交`frame_hit_test::tests`前的1秒客户端探针没有留下Cargo、ticket/job或运行记录；随后正式managed请求在admission前返回`cargo_reuse_pool_busy`，兼容池owner为job `abb01ecff3df4ab288496798b2560a3a`。2026-08-22稍后在coordinator `active=0`且OS `cargo/rustc=0`的释放窗口只重试同一filter，admission又被刚启动的UI12 Editor artifact check job `20d99349f86245a4aa5fd8d94a5111aa`抢先占用兼容池。待后续两份ephemeral Runtime build自然退出、OS再次双零后，同一filter终于进入coordinator治理门，但在Cargo acquisition前返回`unmanaged_artifacts_detected`：全局阻断路径为`F:\cargo-targets\zircon-engine`，details列出历史managed Cargo target与残留artifact。三次请求都没有Runtime09 ticket/job/run、测试数或Cargo exit code，不属于source失败。本计划不删除或接管Tooling治理的target目录；等待artifact audit/cleanup owner恢复admission后，仍从这一张下层filter继续。

### 15.80 UiAssetEditor detail event此前丢失view identity并扩大为全宿主presentation

对current HEAD的Editor invalidation链继续复核后，确认`UiAssetEditor`的detail-event子树不是缺少局部patch能力，而是在调用末端丢失了已经存在的`ViewInstanceId`。20个handler文件中的21个生产调用在成功修改当前文档的binding、collection、preview、source、structure、style、palette或promote草稿字段后统一调用`mark_presentation_dirty()`。该调用把`PRESENTATION_DATA`记到`HostInvalidationScope::All`并置legacy `presentation_dirty=true`；`begin_recompute_invalidation_phase`因此不能选择`presentation_only_view_ids()`，最终进入full target。相同事件若调用`mark_presentation_dirty_for_view`，则保持`HostInvalidationScope::View(instance_id)`且不置全局legacy bit，可直接进入`apply_scoped_ui_asset_presentation`，只重建对应pane patch并发布其damage。

本轮已把上述21个生产调用全部收窄到现有view identity；component adapter在Runtime借用dispatch返回后直接把envelope `target.subject`中的既有`String`移入`ViewInstanceId`，不为相同identity增加第二次字符串分配，发送给Runtime的component binding target与payload没有变化。真正具有跨视图语义的`save`、`keep_local_and_save`、widget/theme promote执行、source/reference open及asset browser emergency route仍由`ui_asset_editor/actions.rs`的显式allowlist保持全局presentation，未被局部化。新增合同枚举全部20个detail handler叶文件，要求生产段包含`mark_presentation_dirty_for_view`且禁止`mark_presentation_dirty()`；等价静态RED在修改前发现21个生产调用和1个旧测试断言，修改后20/20叶文件通过，21个Rust路径的rustfmt与scoped `git diff --check`均为GREEN。

多窗口同资产语义也按current source完成了反证。`EditorUiHost.ui_asset_sessions`的authority是`BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>`；restore为每个打开实例从source构造独立`UiAssetEditorSession`，所有detail edit先用传入`ViewInstanceId`取得唯一entry，`sync_ui_asset_editor_instance`也只更新同一`open_view_instances[instance_id]`的title/dirty。也就是说，尚未save的detail草稿不是asset-id共享presentation，局部化不会遗漏另一个打开窗口；真正写盘、promote或外部watcher/import改变共享资源时仍走原有跨视图刷新。合同测试因此进一步要求普通handler把原始`instance_id`交给scoped invalidation，component adapter则必须先借用同一envelope完成Runtime dispatch，再从该envelope的`target.subject`取回并失效同一identity，避免仅检查函数名却允许错误target。

这与Unreal Slate的边界一致，但不复制其类结构。`FWidgetProxyHandle::MarkWidgetDirty_NoCheck`把具体`FWidgetProxy`和`EInvalidateWidgetReason`交给`FSlateInvalidationRoot::InvalidateWidget`；fast path只消费按widget sort order维护的`FinalUpdateList`，root明确进入slow path时才重建更大范围。Zircon此处已有`ViewInstanceId`、scoped transaction和pane patch，因此正确动作是停止丢失target，而不是再建一套widget proxy或把所有`PresentationChanged`强行局部化。

沿scoped pane patch继续检查时还发现一个独立的复制放大：旧`patch_floating_windows`即使目标不存在，也会先为每个scene/native floating row克隆完整`FloatingWindowData`并构造新`Vec`，再判断pane identity。下层RED覆盖“2行均不命中时0 clone且共享原值存储”和“2行中1行命中时仅克隆目标行且另一行保持共享”；生产实现现在先借用行检查identity，只把命中行写入现有`ModelRc::with_row_patches`的稀疏overlay。静态GREEN确认旧`Vec::with_capacity`、全行clone计数和无条件`window.clone()`均已消失，rustfmt与scoped diff check通过。这个改动把行复制/新值存储从`O(F)`降为`O(M)`，`M`为匹配目标的行数；它没有增加新的索引，identity probe、native presenter ID收集和presenter store访问仍为`O(F + P)`。现有visited/cloned counter足以先判断该线性访问是否构成p95热点，未有证据前不再增加第二套窗口索引。

该切片当前只证明失效target不再无条件扩大、浮动行不再无条件复制，不发布动态收益。23个精确候选路径已取得无冲突lease；在Runtime与framework两份完整测试编译自然退出后，新的UI12 Editor artifact check立即占用兼容池，因此本轮没有为Editor切片提交竞争ticket。池释放后先运行`zircon_editor`中`instance_scoped_detail_events_do_not_widen_presentation_invalidation`、两条floating sparse patch回归及既有invalidation transaction/scoped presentation回归，再做同进程detail edit产品采集：要求`HostInvalidationFullTargetCount=0`、`HostInvalidationViewPresentationTargetCount=1`、`ScopedPresentationPatchCount=1`、projection/presenter coverage fallback为0，`floating_window_rows_cloned`不超过实际匹配presenter数，并同时报告`floating_window_rows_visited`、presenter visits、input-to-damage与damage-to-submit p50/p95/p99。现有profile报告已经导出这些target/scoped计数，但runner没有UiAssetEditor detail action场景；通用`click`只选择任意clickable frame，不能证明命中本修复。工具owner必须从current generation geometry发布稳定的detail control/action identity并拒绝ratio或任意click fallback，之后才能执行这项产品门。只有这些source-bound计数与像素/输入回归同时通过，才能把本项从静态候选提升为已验证优化。

### 15.81 template table selection存在幂等失效与pane target丢失两层放大

继续盘点remaining `mark_presentation_dirty()`后，生产路径只剩三类：可见Runtime Diagnostics随renderer snapshot变化、`ui_asset_editor/actions.rs`显式跨视图allowlist，以及`dispatch_template_table_row_selected`。第三类是高频表格/层级行交互的明确热点。callback已经携带`pane_id`、`control_id`、`source_index`和投影时发布的row identity，registry也只修改该pane/control的`selected_index`与`selected_row_identity`；但成功后调用者丢弃pane identity并调用全局presentation，使transaction同时得到`HostInvalidationScope::All`和legacy `presentation_dirty=true`，必然不能选择任何scoped target。

这里还存在更早且更便宜的幂等缺口。current `TemplateActionRegistry::select_table_row`验证identity后无条件`insert`两个属性并返回`true`，没有比较现有`selected_index`/`selected_row_identity`。因此重复点击已经选中的同一行仍被报告为state change，并稳定触发完整presentation recompute；这不是缓存容量问题，而是changed predicate错误。该registry路径当前有其它会话的未提交编辑，本轮仅记录finding，不接管或覆盖。

Unreal Slate的当前参考实现明确把这两层分开。`STableRow::OnMouseButtonDown`先通过`Private_IsItemSelected`读取`bIsSelected`；默认`bAllowPreselectedItemActivation=false`时，只有`!bIsSelected`才会clear旧selection、set新item并signal selection changed，重复点击已选行只维持focus/capture而不伪造selection mutation。行内`SelectorFocusedAttribute`与drop-zone attribute在`PrivateRegisterAttributes`中只注册`EInvalidateWidgetReason::Paint`。这支持Zircon先修changed predicate，再把真实换行收敛为目标pane/row paint，而不是让同一个点击穿透成host-wide presentation。

修复分为两个独立层次。第一层仍应在registry authority内比较规范化后的index与identity：完全相等返回`false`且不写map，changed case才原子更新两字段；下层回归覆盖same row no-op、index相同但identity代际变化、错误identity拒绝以及pane隔离。该层可让重复点击从host recompute降为既有registry lookup成本，但不能解决真正换行时的全局重算。该registry路径仍由Runtime74独占且带外部编辑，本轮不覆盖其worktree。

第二层已先完成active drawer的正确子集，而不是等待泛化pane transaction。`pane_id`由projection直接取自`PaneData.id`；current committed `WorkbenchViewModel.tool_windows`已经提供`ActivityDrawerSlot -> active_tab`权威映射。新增解析只在固定drawer集合中接受“唯一、visible、active identity完全相等”的pane，并把它转成既有`HostShellContentScope`；selection changed后命中该scope就发出`SHELL_CONTENT` transaction，复用已提交layout并只重建对应dock content/damage。原报告把该scope描述成只能表达tab layout transition并不准确：`CommittedShellState::patch_shell_content`同样接受layout未变化的同一active drawer content更新；它不能表达的是document/floating pane。后两者、committed state缺失、inactive/hidden identity及跨region歧义全部显式回退原`mark_presentation_dirty()`，没有把`pane_id`盲转成只支持UiAssetEditor的`ViewPresentation`，也没有伪造覆盖范围。

下层RED/GREEN覆盖active visible命中、inactive/hidden拒绝和重复active identity歧义拒绝；click合同要求scoped call先于且保留full fallback。解析复杂度为`O(D)`、空间`O(1)`，`D`是固定drawer slot数量，不扫描template nodes、逻辑rows、render commands或全宿主panes。新增`ui.table_selection.shell_content_scope_hit_count`与`ui.table_selection.full_presentation_fallback_count`用于产品采集区分真实命中和fallback；在registry no-op修复进入current source前，same-row仍可能被错误计为changed，但已不再迫使active drawer走全宿主presentation。

动态验收使用source-bound table control和row identity，分别执行1000次same-row click与1000次两行交替。active drawer交替选择要求每次`ui.table_selection.shell_content_scope_hit_count=1`、fallback为0、恰有一个shell-content target、`HostInvalidationFullTargetCount=0`，非目标pane projection generation/body surface storage保持共享且damage只覆盖目标dock；document/floating fixture在泛化target进入前必须证明fallback counter与full target各为1，不能静默漏刷。registry no-op修复进入后，same-row还必须满足selection changed count、dirty request、presentation transaction、projection rebuild和command publication全部为0。两组都报告input-to-damage、damage-to-submit与event CPU p50/p95/p99、RSS、pane lookup/clone/command计数，并以N=100/10,000/100,000固定visible range验证changed路径不随宿主pane总量或逻辑row总量线性增长。未完成这些source-bound动态门前，不把通用按钮/表格点击的响应问题归因于SVG或GPU upload。

### 15.82 可见诊断面板此前把每个视口提交扩大为全宿主presentation

继续审查最后一类根级presentation调用后，确认正常视口渲染本身不会重建Workbench；放大只发生在Runtime Diagnostics或Performance Timeline可见时。旧实现于每次成功`submit_extract_with_ui`后读取`runtime_diagnostics_visible`，只要任一诊断document、drawer或floating tab活动就调用`mark_presentation_dirty()`。因此即便仅右侧抽屉展示最新`RenderStats`，60 Hz视口也会持续发布`HostInvalidationScope::All`和legacy presentation dirty，而不是只更新该抽屉。SVG/GPU cache可以避免部分资源重建，却不能阻止这条Editor根级recompute链。

修复把布尔可见性提升为publication-time typed target：`None`、唯一活动抽屉的`ShellContent(HostShellContentScope)`、或`FullPresentation`。目标只在full shell或committed shell-content presentation发布时从Workbench model计算一次；唯一、visible、active且与`active_tab`一致的Runtime Diagnostics或Performance Timeline抽屉得到精确slot/instance scope。活动document/floating诊断、多个可见诊断实例、tab状态不一致均显式选择full fallback，避免局部刷新漏掉第二个消费者。payload收集复用同一缓存目标，不再为同一presentation重复扫描document/tool/floating模型。

render submission热路现在只用`std::mem::take`消费枚举，不扫描pane、template node或render command，也不克隆pane id。首个成功提交把`ShellContent/FullPresentation`转换为无scope负载的`Pending`并安排一次失效；下一次presentation前的连续成功提交看到`Pending`后只恢复同一状态，不重复失效。payload collector把`Pending`视为仍有可见诊断消费者，避免局部refresh在采集payload前因目标被take而错误返回空快照；presentation完成后才从新model重新发布下一目标。局部路径直接发出`SHELL_CONTENT`，复用committed layout并只patch对应dock content；保守路径保持原全量语义。新增`ui.runtime_diagnostics.shell_content_refresh_count`与`ui.runtime_diagnostics.full_presentation_fallback_count`区分命中和fallback。选择阶段复杂度为`O(Doc + Float + D * T)`且只在presentation publication发生，render提交热路为`O(1)`时间、`O(1)`空间；`D`为固定drawer slots，`T`为各drawer tabs。

下层合同覆盖唯一Runtime Diagnostics抽屉、Performance Timeline抽屉、document/floating fallback、多抽屉与不一致状态fallback、hidden/absent no-op，以及`Pending`仍采集payload；source guard要求热路消费已发布目标、在失效前写入`Pending`且不出现tool/document/floating遍历。静态`rustfmt --check --config skip_children=true`和精确`git diff --check`已通过，尚未把它标成动态GREEN。产品验收必须打开source-bound Runtime Diagnostics抽屉并执行至少1000个稳定viewport submissions：要求shell-content counter按presentation批次增长、full fallback为0、`HostInvalidationFullTargetCount=0`，非目标dock generation/commands保持复用，连续提交的dirty transaction数量不超过presentation批次数；document、floating和双诊断fixture则必须各自证明full fallback不漏刷。三轮同进程warmup/measured/quiescence同时报告input-to-damage、damage-to-submit、event/render CPU p50/p95/p99、RSS、pane payload build、command conversion以及SVG parse/raster和GPU upload计数，避免把根级recompute下降误报成整条资源链已经命中。

### 15.83 drawer resize已有committed shell stage，却仍在每个move请求full layout

抽屉拖动的changed predicate已经正确：相同preferred extent会在写入和dirty之前返回；但真实变化旧实现调用`mark_layout_dirty()`，使每个pointer move进入`HostInvalidationFullTargetCount`并重新读取runtime layout/descriptors、构建chrome和`WorkbenchViewModel`。这与现有stage cache能力不一致。`build_window_metrics_shell_snapshot()`不只处理native window尺寸，它从`committed_shell_state`移动已有layout、chrome、model和descriptors，读取同一个`transient_region_preferred`重新计算shell geometry，并刷新root/workbench template layout frames。因此拖动期不需要重新物化业务模型；只有鼠标松开后把preferred extent持久化到runtime layout的dispatch仍可能要求完整layout。

生产movement现把changed extent标为`HostInvalidationMask::WINDOW_METRICS`。该mask仍满足`requires_layout/presentation`，所以geometry、hit、pointer和render可见结果不会被降为paint-only；recompute decision会选择`WindowMetrics` target并优先调用committed shell stage，缺少committed state才显式fallback到原full shell builder。Rust与Python source合同同时要求unchanged extent先返回、changed extent使用`WINDOW_METRICS`且不得调用`mark_layout_dirty`。这把拖动期的model/chrome/descriptors工作从每move的`O(workbench model + chrome)`降为0次重建；geometry与template layout仍按当前shell规模重算，不能宣称整个resize已经是常量时间。

动态验收复用现有source-bound `drawer_resize`场景执行200步和三轮measured run。每个真实changed step要求`HostInvalidationWindowMetricsTargetCount=1`、`ui.window_metrics.committed_shell_stage_hit_count=1`、`WorkbenchModelBuildCount=0`、chrome snapshot与runtime descriptor/layout snapshot build为0、full target与committed-stage miss为0；final pointer-up允许一次runtime layout持久化和必要full target。报告同时保留geometry patch、pointer route、damage、command conversion、SVG parse/raster、GPU upload、CPU/RSS及input-to-damage/damage-to-submit p50/p95/p99。若p95仍高，再用15.77的layer/command计数定位geometry之后的command publication，不回退为跳过布局或降低resize采样率。

### 15.84 动态证据必须绑定本轮失效与publication热路径

对非C盘profile产物和采集器继续做只读审计后，确认现有数据不能验收15.80至15.83。`E:\zircon-profiles`中最近目录的时间停在2026-08-19，`runtime09-svg-m7`内可识别的`ui_hotspots.json`/timeline样本更早；它们只能作为历史诊断，不能证明当前HEAD与未提交候选。采集器自身不会调用Cargo：必须先由coordinator生成`CARGO_TARGET_DIR\profiling\zircon_editor.exe`和Runtime DLL，再以`-SkipBuild`运行；输出根被强制限制在`E:\zircon-profiles`。这满足非C盘和受管构建边界，但旧source manifest只用108个手工关键文件的修改时间约束二进制新鲜度，没有覆盖本轮局部presentation、诊断刷新和drawer movement，理论上可能让早于候选的二进制被错误接受。

本轮先用合同RED关闭这个证据漏洞，而不是复用旧exe制造性能结论。第一轮RED在完整`ui-profile-capture-output-contract.Tests.ps1`中得到36/37通过，唯一失败精确为critical source count仍是108而新合同要求116。生产清单先增加8个采样语义入口：retained host `app.rs`、committed shell scope、dirty marking、render submission、runtime diagnostics target、drawer resize movement、scoped presentation以及UiAsset detail event入口。随后增加“每个manifest路径必须在当前仓库存在”的合同，修正3个只指到缩短旧路径而未绑定真实owner的条目；再以本候选35个Rust文件反查清单，发现父`ui_asset_editor_detail_events.rs`不能绑定20个实际handler叶文件的mtime，table click与诊断payload/startup publication也缺少直接绑定。第二轮RED精确为37/38通过、critical source count仍是116而完整生产合同要求140；补齐24个叶级生产入口后，完整合同38/38、exit 0。当前精确静态验收同时为35个Rust owner的`rustfmt --check --config skip_children=true`通过、42条候选路径`git diff --check`通过、drawer resize Python合同1/1、2个PowerShell文件parser错误0、manifest路径140/140存在且重复0。每个文件都会记录SHA-256、长度和mtime，任一入口晚于editor/runtime二进制时采集直接失败。该修复只加强动态证据来源，没有改变Editor产品行为；上述静态结果不替代managed Rust编译或产品profile。

因此后续产品profile前置条件现在是明确且不可绕过的：新受管profiling build必须包含本轮候选；manifest记录当前Git revision、dirty entries、140个关键生产源码指纹和两份二进制指纹。`drawer_resize`和15.85新增的`runtime_diagnostics`已经具备注册场景、source-bound geometry、自动交互和专用counter gate；UiAsset detail action与table selection仍只有生产源码指纹，没有对应的自动产品交互权威，不能用通用click或旧目录替代。每个最终场景仍须执行三轮同进程warmup/measured/quiescence，并报告CPU/RSS、input-to-damage、damage-to-submit、host invalidation target、projection/command、damage pixels、SVG parse/raster和GPU image upload/cache计数。当前没有满足这些条件的新二进制，也没有启动新的Cargo或Editor采集，所以15.80至15.83继续是静态候选，不发布延迟改善数字。

### 15.85 source freshness不能替代产品交互覆盖

对现有profile runner、geometry schema、启动请求和Workbench默认布局做反向审计后，确认15.84此前把“已绑定生产源码”错误写成了“已建立产品场景”。Runtime Diagnostics可以在不增加测试专用产品入口的前提下补齐：用普通project启动保证viewport submission存在，从`ui_profile_geometry.json.drawer_tabs`精确选择`editor.runtime_diagnostics#1`且同时要求`kind=drawer_tab`、`surface=bottom`、frame可见并且全表唯一；激活后等待新geometry publication，再以稳定submission窗口要求`ui.runtime_diagnostics.shell_content_refresh_count>0`、`ui.runtime_diagnostics.full_presentation_fallback_count=0`，并要求`click`、`viewport_image`、`shell_content`三种相关场景的`host_invalidation_full_target_count`合计为0。这里明确排除startup基线，覆盖drawer激活与后续跨帧viewport刷新。该场景已在取得三份profile工具精确lease后进入runner和`AllUiScenarios`；选择失败、重复identity、错误surface或不可见frame都会直接失败，不允许ratio fallback或任意click替代。

实现遵循TDD。第一轮完整PowerShell合同为36通过/3失败：失败仅来自场景未注册、两个专用函数缺失；生产脚本加入唯一目标选择、一次真实drawer click、fresh geometry证据和专用增量门后，同一套件39/39、exit 0，三份脚本PowerShell parser错误0，`git diff --check`通过。`runtime_diagnostics`仅在报告聚合上复用既有`click`场景，自动交互和counter gate保持专用分支；即使全局设置`AutoClickCount`，也不会把单次诊断tab激活误判为click storm或要求不相关的storm latency证据。该工具GREEN只证明采集合同能够拒绝错误目标和full fallback；尚未有包含当前候选的受管profiling二进制，因此没有执行Editor产品采集，也不发布收益数字。

table selection暂时没有等价的现成产品fixture。`editor.ui_component_showcase`虽能发布`TableRowDemo` template control，但它位于exclusive document surface，按当前合同应走full fallback，不能证明活动drawer的shell-content scope。默认Build Export和Module Plugins抽屉虽然有可点击行，但它们是native pane nodes，不触发`dispatch_template_table_row_selected`；把这些行计作命中会产生假GREEN。验收前必须提供一个真实、visible、active且唯一的drawer-bound `WorkbenchTableRow` fixture，激活该drawer后从刷新geometry选择其行，并要求`ui.table_selection.shell_content_scope_hit_count>0`且`ui.table_selection.full_presentation_fallback_count=0`。

UiAsset detail同样缺少可由runner启动的资产绑定实例。当前GUI启动请求只支持project、builtin view和create project；直接打开`editor.ui_asset_editor_window`不能替代带route/source/document identity的真实UiAsset editor session。验收需要先增加正式的“打开项目内指定UI资产”启动/fixture合同，随后只从该窗口发布的template controls选择detail action，并要求目标view presentation计数增长、full target与非目标view generation变化为0。在这两个fixture和Runtime Diagnostics专用场景进入runner前，140个critical-source指纹只证明采集二进制没有陈旧，不能证明三条优化路径被执行。

### 15.86 当前源码算法复审：局部失效已经存在，但四个结构层仍未闭环

本轮重新读取current source后，先纠正一个容易误导后续实现的二分法：Zircon现在既不是“每个输入都全量重建”，也还没有形成成熟引擎那种从属性失效到最终像素的完整增量链。`layout/pass/incremental.rs`会把dirty node传播到必要的layout boundary/auto-layout祖先，去重后只measure/arrange这些根的子树，并统计visited/skipped与geometry-changed集合；frame-authoritative hit grid候选也已经把popup投影工作移出事件查询。问题是局部authority之间仍存在重复派生、顺序丢失和错误像素恢复。下一轮优化必须按下表逐层收敛，不能再把“有cache”或“有dirty rect”当作链路已经正确。

| 层 | current-source实际复杂度与正确性 | 参考实现给出的边界 | 目标合同 |
|---|---|---|---|
| 布局失效 | dirty root选择与子树访问是`O(K log K)`量级，`K`为传播后访问节点；这是有效增量。但每个auto-layout容器在`taffy_bridge/compute.rs:69-139`重新创建一个`TaffyTree`、`children.len()`个leaf和一个parent，compute后立即丢弃。`UiLayoutEngineTaffyTreeBuildStats`只统计这棵局部父子图，不是跨帧持久整树。measure递归还会克隆每个node的children；previous geometry已从第二棵`BTreeMap`收敛为按visited顺序线性消费的紧凑`Vec`，该子阶段由`O(K log K)`降为`O(K)`，但visited/changed set和完整子树成本仍在。 | Fyrox `fyrox-ui/src/lib.rs:1745-1904`即使从root调用measure/arrange，也以`is_measure_valid + prev_measure`和`is_arrange_valid + prev_arrange`在节点入口跳过；它证明“根入口”不等于“根重算”。Unreal `SlateInvalidationRoot.cpp:1281-1404`把pre-update、attribute、prepass、post-update分队列，并在prepass中跳过已被祖先覆盖的范围。 | 保留当前dirty-root传播语义；cache validity必须包含layout input、available size/scale、visibility occupancy与child-order generation，再把Taffy node/style identity持久化到surface/layout generation，局部patch style/children/available-size。根尺寸变化允许`O(N)`布局，但稳定hover/click必须是0个Taffy node创建、0个layout subtree visit。 |
| slot/order派生 | `UiLayoutSlotIndex`完整建立为`O(N+S)`；只以slot数量判断整体过期，同数量替换靠具体edge查询时扫描全部slots修复。`ordered_children_for_container`每次分配`Vec`并执行`O(C log C)`排序；measure与arrange的多个容器路径会各自派生顺序。 | Unreal把child order作为独立失效原因；只有child-order mutation才重建/修订fast widget list，而普通paint/attribute invalidation不重排整棵树。Fyrox只对`z_index_update_set`中的parent排序children。 | 把`(parent, child)->slot`和每parent稳定order编译成mutation-owned generation cache；slot replace必须显式推进generation，不能靠same-count lazy repair。measure与arrange共享同一ordered child slice，普通geometry/text变化不得排序未变parent。 |
| 绘制与批处理 | game/screen-space UI在`render.rs:55-63`把solid、image、三类text和post-text分开保存，`render/record.rs:74-105`固定按solid→image→text→post-text提交。原始command painter order因此会在混合资源重叠时改变。无法解析的Image/Icon又在`render.rs:381-395`降级为实心矩形。顶点缓存每帧仍在`record.rs:129-155`对全部vertex bytes做BLAKE3后才决定是否上传。 | Unreal虽然按element type生成batch，但每个render batch携带layer；`ElementBatcher.cpp:171-276`先对batch index按layer稳定排序，只在同layer且状态兼容时合并。Zircon自己的native/editor WGPU路径已有单一有序`DrawOp`流，可作为更贴近本仓库的正确模式。 | game UI改为单一ordered paint-op stream；batching只能合并相邻兼容op，或使用显式证明不改变blend/clip/layer依赖的稳定规则。Image/Icon必须解析为正式visual resource或显式missing-resource诊断，禁止生产矩形替代。vertex/material/image以publication generation和dirty range复用，稳定帧不得重新序列化并哈希完整payload。 |
| damage与保留表面 | native WGPU `presentation.rs:278-311`在`DamagePatch`对已经合成的retained texture使用`TargetLoad::Load`，仅scissor重放相交ops，再整纹理copy。半透明内容会叠加在上一帧自身像素上；删除、移动或缩小command也没有先恢复旧覆盖区域的背景。现有opaque色块测试不能证明alpha、delete或move正确。 | Slint `partial_renderer.rs:406-555`缓存每item旧/新bounding box和transform，geometry变化同时把旧区域与新区域标dirty；clip/opacity变化向子树传播。它把dirty region限制为最多3个rect，复杂时合并扩大，并明确区分reused/swapped buffer。删除项无法恢复相对屏幕区域时宁可full refresh（`partial_renderer.rs:815-824`），而不是静默留下旧像素。 | damage generation同时携带old+new bounds、clip/opacity影响闭包和删除信息。提交前必须先从可信base/backdrop恢复dirty region，再按原painter order重放完整相交layer closure；无法证明背景可恢复时升级full redraw。先补半透明、删除、移动、clip变化四条像素RED，再谈copy带宽优化。 |
| 命中测试 | projected frame grid的方向正确：authority应在frame publication/rebuild时构建，event query只访问cell并做clip/frame精确判断。尚未managed验证的current blockers不改变该算法裁决。 | Unreal `HittestGrid.cpp:859-930`在paint geometry上计算render bounding rect并写入覆盖cells；event query在`986-1085`只取目标cell候选并检查clip/transform。它在`1111-1144`仍会稳定排序该cell候选，因此合理目标是`O(H log H)`的局部候选成本，而不是声称绝对`O(1)`；关键是不扫描全部arranged/render节点。 | frame与instance查询共用同一published grid；dirty rebuild发生在输入消费前。记录cell candidate count、clip tests与sort count；稳定pointer move不得触发layout/render scan或grid rebuild。popup affine投影必须替换placeholder entry并保留stack+subtree order。 |
| 图像/SVG驻留 | 上层generation churn会击穿已经存在的WGPU `(resource_key, generation)`cache；单纯扩大GPU cache不能修复identity。game UI的未解析visual矩形fallback还会隐藏资源管线缺口。 | Slint `CachedRenderingData`以cache index + cache generation防止stale slot复用（`partial_renderer.rs:38-75`），cache clear只推进generation；Unreal cached element data与invalidation root分别持有绘制缓存和更新队列。两者都把稳定identity与失效原因置于热路径payload比较之前。 | 统一`VisualResourceIdentity = namespace + resource id + content/artifact generation + physical extent + raster scale + variant`；parse tree、target raster和GPU texture分层有界驻留。stable hover/click/resize step的SVG parse/raster/upload必须为0，resize仅在目标physical bucket变化时产生一次新raster/upload。 |

#### 15.86.1 不应优化掉的工作与必须删除的工作

根窗口尺寸改变、字体度量改变或祖先约束改变时，后代frame确实可能全部变化；这种情况下`O(N)`布局是语义成本，不应通过跳过布局或降低resize事件频率掩盖。类似地，hit cell内多个重叠候选需要确定topmost，局部稳定排序不是错误。真正必须删除的是：稳定输入上的layout/style/order重新派生；同一容器在一个逻辑generation内重复分配和排序；按资源类别打乱painter order；在旧合成像素上重放半透明内容；用整payload哈希代替generation；以及事件时扫描全树重建命中映射。

因此后续计数必须同时报告`N`（逻辑节点）、`K`（本次layout访问节点）、`C`（dirty commands）、`H`（目标hit cell候选）、`D`（dirty pixels/rects）和`R`（新资源products）。目标不是把所有数字写成0，而是证明它们由真实变化决定：稳定按钮交互应为`K=0,C=affected paint closure,H=local,R=0`；窗口resize允许`K`接近`N`，但Taffy node/style应复用，SVG parse为0，raster/upload只跟跨bucket资源数相关；删除/移动必须让`D`覆盖old∪new并保持像素正确。

#### 15.86.2 实施顺序与RED门

1. **先补观测，不改算法语义。** 为layout pass增加persistent/local Taffy node create/update/remove、ordered-child sort、slot-index rebuild/repair、children clone与geometry snapshot计数；为game UI增加ordered command→batch映射、payload bytes hashed/uploaded；为damage增加old/new bounds、restored pixels、closure ops；为hit query增加cell candidates/clip tests/sorts。所有counter必须批量提交且capture关闭时近零成本。
2. **先修绘制正确性。** lower layer先建立mixed rect/image/text重叠的painter-order RED，以及Image/Icon不得落入solid rectangle的资源诊断RED；实现单一ordered paint-op stream后，才允许相邻兼容batch合并。这个P0优先于微调布局分配，因为当前结果可能“更快但画错”。
3. **再修damage恢复。** 用半透明overlay、删除、移动、clip/opacity变化四组offscreen像素回归证明old∪new和backdrop restore；不能恢复时full redraw必须可观测。不得继续用opaque patch通过冒充通用合成正确。
4. **再持久化布局派生。** 先让slot/order generation成为mutation authority并消除measure/arrange重复排序，再引入persistent Taffy arena和style fingerprint。TDD分别证明稳定click零布局工作、leaf fixed-size paint变化不访问兄弟、single child desired-size变化只传播到必要祖先、root resize仍与full layout结果逐节点相等。
5. **最后收敛visual identity与GPU residency。** 以同一SVG的稳定hover、重复窗口尺寸、跨physical bucket resize、内容generation变化和cache eviction为下层用例；parse/raster/upload次数必须分别对应identity层变化，不得用更大cache预算掩盖generation churn。

每个阶段都先跑lowest-layer managed focused tests，再跑Editor产品场景；当前共享Rust接口/事件路由编译错误只登记为current-source validation blocker，本节不接管对应owner文件，也不启动raw Cargo。产品门仍要求三轮同进程warmup/measured/quiescence，报告CPU/RSS、input-to-damage p50/p95/p99、damage-to-submit p50/p95/p99，以及上述`N/K/C/H/D/R`工作量。没有source-bound interaction、像素RED/GREEN和current-source managed binary时，只能标记静态候选，不能发布“UI已优化”或延迟收益。

### 15.87 SVG反复加载的current-source根因不是单一GPU cache缺失

沿Editor visual asset到WGPU texture重新追踪后，确认current source已经存在四层驻留，不能再以“没有缓存”描述现状：

1. `visual_assets/svg/cache.rs`持有最多1024项`Arc<usvg::Tree>`，包括missing/parse-failed负缓存。稳定query path先走alias→唯一canonical path内存命中，不读取metadata或文件；miss才读取、BLAKE3 fingerprint并parse。targeted asset event只在fingerprint真实变化时移除对应tree。
2. `visual_assets/loading/cache.rs`持有最多4096项、64 MiB的raster/missing cache。key为`base asset + exact target width/height + tint`；稳定命中发生在candidate path构建之前，因此正常hover若asset、尺寸和tint都不变，不应再次探测文件、parse或raster。
3. 小于等于64 px的icon raster可进入Editor icon atlas，最多64页/64 MiB；page以generation发布，command只携带page resource key、generation和UV。非atlas raster使用内容地址式`retained-image:{width}x{height}:{hash}`，chrome stream把重复`(key,generation)`像素压成一个resource，并在GPU已resident时移除RGBA传输。
4. WGPU窗口cache与device-shared registry都按`(resource_key,generation)`查询；本地上限256项/64 MiB。预算内命中是map lookup且upload为0，只有新identity或未resident resource才premultiply、admit和write texture。

Unreal的直接对照位于`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateVectorGraphicsCache.cpp:45-175`与对应header。`FVectorCacheKey`把`BrushName`和`LocalSize * DrawScale`得到的`PixelSize`作为稳定identity；paint侧miss只登记`PendingRequests`并立即在`ResourceMap`放置proxy，`UpdateCache`再统一`ParallelFor`栅格并写入atlas或独立texture。可迁移的是“稳定资源identity、miss请求队列、批量栅格、持久proxy/atlas”四项分离，而不是它的全量flush：`FlushCache`会清空resource map并使全部widget invalidation，这正是Zircon普通asset或pointer事件必须避免的放大路径。Unreal同样使用精确pixel size，因此它证明暖态不应重读资源，却不能替Zircon解决连续resize的相邻尺寸product爆炸；该问题仍需下面的physical bucket、hysteresis和最终像素误差合同裁决。

因此可复现的反复成本来自这些identity与失效边界，而不是再加第五层cache：

- `RasterTargetSize::from_frame`对每个逻辑frame宽高直接`ceil`为精确整数，没有DPI/physical bucket或resize hysteresis。连续窗口缩放会为同一SVG生成大量相邻尺寸key；每个新key都执行2x supersampled raster、downsample、可选tint和完整RGBA resource hash。即使parse tree命中，raster/CPU hash仍会重复，4096项cache最终进入LRU扫描淘汰。
- `retained_image_resource_key`每次对完整RGBA执行`DefaultHasher`。这保证相同像素得到稳定GPU identity，却让每个新raster先支付`O(W*H)`CPU hash；它也没有显式source/content generation，无法在raster前判定GPU product identity。
- icon atlas只向未sealed page分配；一次pack内的pending icons会共同落页，page随后sealed。后续帧首次出现的新icon会新建另一页。这个不可变页策略避免修改旧GPU page，但惰性暴露大量icon时会产生多页和多次page upload；达到页/字节预算后按非active LRU整页替换。
- sprite-atlas源事件会清空SVG tree、全部visual raster和Editor icon atlas。普通精确路径事件是targeted的；但`resource_generation_lagged`走reconcile，SVG tree cache会读取并hash全部resident tree sources，pixel cache也扫描已登记source fingerprints。它发生在asset refresh而非普通pointer hot path，却可能造成明显维护帧尖峰。
- 上层full presentation或不必要的command重建即使不产生parse/raster miss，仍会克隆`HostPaintImagePixels`的`Arc`、重建image commands、执行atlas pack/stream compaction和GPU residency probes。因此产品上看到“SVG相关scope很多”不能直接解释为文件被重复parse；必须同时看tree miss、raster scope、pixel cache miss、atlas generation和GPU upload。

优化算法分成三个authority，不混在一个string key中。`SvgSourceProduct`由canonical resource id与content generation持有唯一parse tree；`SvgRasterProduct`由source generation、quantized physical extent、raster scale和tint/variant持有`Arc<[u8]>`；`GpuImageProduct`只消费raster generation并在窗口/设备间共享texture。窗口resize时先计算physical target bucket：同bucket复用已有raster，跨bucket只生成最接近当前显示需求的一项，并取消已经被新resize事件淘汰的pending raster。不能用固定低分辨率拉伸所有尺寸，也不能让bucket误差超过像素/清晰度合同。

实施前先加下面的source-bound压力门：

| 场景 | 必须为0 | 允许增长 | 正确性门 |
|---|---|---|---|
| 1000次稳定hover/click | SVG tree miss/parse、raster、pixel miss、atlas page generation、GPU upload | tree/pixel/GPU hit与局部command reuse计数 | icon像素、tint、clip不变；input-to-damage p95≤1 ms |
| 同一窗口尺寸重复200帧 | parse、raster、full RGBA hash、atlas upload、GPU upload | resource residency probe至多每唯一identity一次/帧批 | command与texture identity稳定 |
| 200步连续resize | parse；同一physical bucket内raster/upload | raster/upload仅按跨越bucket的最终有效product增长 | 每步显示尺寸/裁剪正确，最终帧与精确reference像素差在阈值内 |
| 单SVG内容变化 | 非目标SVG parse/raster/upload | 目标source tree一次、当前可见target raster一次、GPU generation一次 | 旧generation不再命中，其他资源identity保持不变 |
| resource-generation lag | pointer/input路径中的同步reconcile | 维护帧可分批访问有界source集合 | UI保持可交互，queue age与每帧hash byte budget可观测 |

实现顺序固定为：先补tree parse、raster product、RGBA hash bytes、atlas page create/generation/upload及GPU identity计数；再把logical frame→physical bucket变成统一typed函数并加清晰度像素RED；然后把content generation贯穿parse/raster/GPU product，删除raster后的全payload hash作为常规identity来源；最后把generation-lag reconcile改成可恢复游标的有界维护任务。现有cache与targeted invalidation不能回退，稳定命中必须继续在candidate/path/filesystem之前返回。未运行上述产品压力门前，不能仅凭cache类型存在宣称用户观察到的SVG重复加载已经解决。

### 15.88 稳定hover的SVG/GPU churn现在会使产品证据失败

2026-08-22在current `main` HEAD `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`上复核采集器，确认15.76指出的部分观测缺口已经由其它current-source推进覆盖：`UiPerfCounter`和timeline现已发布visual raster cache hit/miss、candidate build、SVG tree memory hit/miss、GPU image prepare hit、本地与device-shared upload、cache-key allocation等字段；`window_resize`也已经要求visual/SVG miss为0、local upload/allocation最多1次。剩余真实缺口是`idle_hover`只有通用frame/alert门，重复SVG解析或GPU materialization即使被记录，仍可能通过`RequireScenarioEvidence`。

本轮只修改profile evidence authority，没有编辑Rust热路或current-source blocker。新增`Test-StableVisualAssetCacheCounterGate`只对`idle_hover`生效，并要求visual cache hit、SVG tree memory hit、GPU image prepare hit均大于0，借此拒绝“字段缺失被默认成0”的假GREEN；同时要求visual miss、candidate build、SVG tree miss、GPU image command visit、本地upload、device-shared upload和cache-key allocation全部为0。它已接入最终`RequireScenarioEvidence`合取，因此稳定hover中任一层重新解释路径、重新raster、回扫上传源command或重新materialize GPU identity都会使采集失败，而不是只在摘要中打印。

TDD证据保持分层。RED运行得到`39 passed / 1 failed`，唯一失败为缺少`Test-StableVisualAssetCacheCounterGate`。实现后合成timeline先证明稳定命中通过，再逐一把七个churn counter设为1并证明全部拒绝；完整`tools/tests/ui-profile-capture-output-contract.Tests.ps1`结果为`40 passed / 0 failed`、exit 0，PowerShell parser error为0，两个改动工具路径的`git diff --check`通过。第一次GREEN全套因共享机器负载在120秒超时，但在超时前新用例已经通过；随后300秒上限的完整重跑在58.47秒正常结束。该证据只证明门禁逻辑，不是Editor性能结果。

压力运行参数必须显式固定，不能把默认24次resize当成15.87要求的200步压力矩阵。待current-source managed profiling Editor可用且共享Cargo lane空闲后，按同一受管binary/source manifest执行：

```powershell
pwsh -NoProfile -File .\tools\ui-profile-capture.ps1 `
  -Scenario idle_hover -SkipBuild -AutoInteract `
  -AutoPointerMoveCount 1000 -AutoPointerMoveDelayMs 2 `
  -MeasuredRunCount 3 -RequireScenarioEvidence

pwsh -NoProfile -File .\tools\ui-profile-capture.ps1 `
  -Scenario window_resize -SkipBuild -AutoInteract `
  -AutoResizeStepCount 200 -AutoResizeDelayMs 40 `
  -MeasuredRunCount 3 -RequireScenarioEvidence
```

两组运行都必须保留同进程warmup、每进程source/binary manifest、CPU/RSS和latency artifacts；resize报告还必须明确requested/completed包含最终恢复原尺寸的一步。当前工作区已有多个其它owner的Cargo/rustc进程，且current-source Rust接口错误仍由对应owner处理，本轮没有启动Cargo、没有复用陈旧Editor binary、也没有发布动态数值。下一观测切片仍是15.87尚未存在的raster invocation、RGBA hash bytes和atlas page generation/upload计数；在这些字段与上述两组产品结果完成前，SVG优化状态保持“cache层已存在、稳定hover churn门已闭合、resize bucket/identity实现与产品验证open”。

### 15.89 measured resize不得回扫GPU图像命令

继续沿现有计数器的生产语义复核后，确认15.34原先允许一次`gpu_image_prepare_command_visits`并不成立。WGPU `image_cache.rs`先按`(resource_key, generation)`检查presenter-local cache，再检查device-shared registry；两条快路都在进入`command_indices`循环前返回，只有需要重新解析上传源时才增加command visit。`profile_capture.rs`在配置的一次warmup present完成后调用`reset_capture()`并重新启动measurement recorder，但不会销毁Presenter或其GPU cache；`WgpuUiSurface::resize()`也只更新surface config并调用`configure`，不会重建`image_cache`。因此warmup成本不应泄漏到measured timeline，而稳定资源在窗口缩放期间出现任意command visit都表示generation、residency或compiled upload-source快路失效。

本轮只收紧已有profile evidence authority，没有增加Rust字段或修改产品热路径。Pester回归在原有完整正例后注入`ui.window_resize.gpu_image_prepare_command_visits=1`：RED为`39 passed / 1 failed`，唯一失败精确证明旧门仍返回true；`Test-WindowResizeCounterGate`加入`imagePrepareCommandVisitCount -eq 0`后，同一完整文件为`40 passed / 0 failed`、exit 0、47.54秒。visit恢复为0后，shell drag authority rebuild与geometry patch的既有负例仍按原合同拒绝，说明新条件没有掩盖其它resize失效门。

同一语义也应用于`idle_hover`。第二条回归把稳定hover的command visit加入七项churn矩阵；旧门再次得到`39 passed / 1 failed`，因为visit=1仍被接受。`Test-StableVisualAssetCacheCounterGate`现在读取、打印并要求该值为0，最终完整文件为`40 passed / 0 failed`、exit 0、41.3秒。因此1000次稳定hover和200步resize都不能以“GPU cache hits为正”掩盖每帧仍在回扫image command的线性准备工作。

这仍是合成timeline合同，不是产品性能结论。下一次current-source managed profiling binary可用且共享Cargo lane释放后，`window_resize`必须以`AutoResizeStepCount=200`、一次同进程warmup和3次独立measured run执行；三轮都要求image vertices与prepare cache hits为正、prepare command visits为0，并同时保留visual/SVG miss、upload/allocation、CPU/RSS与input-to-damage/damage-to-submit证据。如果真实轨迹出现visit，先按resource key/generation、local/shared resolve、surface presenter lifetime和compiled upload-source identity归因，不能放宽门槛或再加一层平行cache。

### 15.90 稳定hover必须证明retained authority没有重建

继续检查`Show-UiScenarioEvidence`后发现另一个可直接解释“鼠标仍卡”的假GREEN：`idle_hover`分支只要求存在frame、无software fallback且hotspot alert为空。Rust hotspot analyzer会为presentation rebuild、chrome/model rebuild和full-frame redraw生成部分alert，但它没有把所有昂贵工作编码为稳定hover硬合同；合成或旧聚合报告也可提供空alert数组。于是full invalidation、legacy dirty、slow path、layout dirty、hit-index rebuild或chrome command full rebuild可能仍被门接受，甚至`redraw_region=0`时只打印warning而不失败。这无法证明按钮hover走局部命中和paint patch。

本轮没有新增独立门函数，而是在现有`Show-UiScenarioEvidence`职责内只对精确normalized `idle_hover`增加retained-authority合取。这样不会误伤统计时映射为`idle_hover`前缀的`hierarchy_scroll`、`welcome_recent_scroll`和`viewport_pointer`。产品正例现在必须满足：host invalidation transaction为正且全部落在paint-only target；full/shell-content/workbench-projection/view-presentation/window-metrics target、legacy dirty、slow path和layout dirty全部为0；presentation rebuild、chrome snapshot、workbench model、workbench hit-index build和chrome command full rebuild全部为0；同时workbench hit-index query、chrome command patch、dirty paint-only、region redraw和GPU batch均为正，full-frame redraw为0。

TDD先构造完整稳定hover报告，再逐一注入9种昂贵counter，并另外把hit query或command patch正证据清零。旧实现得到`40 passed / 1 failed`，唯一失败为新用例；实现后完整`tools/tests/ui-profile-capture-output-contract.Tests.ps1`为`41 passed / 0 failed`、exit 0、56.62秒。这个合同与15.88的SVG/GPU churn门互补：前者约束输入、失效和publication authority，后者约束visual resource与GPU residency；两者都通过才允许把1000次hover轨迹用于性能裁决。

主采集脚本当前2826行，继续新增场景门会违反大文件边界。本轮没有强行拆分，是因为新增逻辑属于已有`Show-UiScenarioEvidence`校验且`profile-capture-manifest.ps1`存在外部未归属改动；新模块必须同时进入工具指纹清单，不能只dot-source后遗漏source binding。最小后续拆分已经固定：取得manifest owner租约后，把当前1042-1500行的asset refresh、window resize、stable visual、hierarchy/welcome scroll与Runtime Diagnostics场景计数门整体迁到语义命名的`ui-profile-counter-evidence.ps1`，更新capture tool manifest与测试source aggregation，再删除主脚本中的原定义；不得用`part2`或`misc`式文件名。

### 15.91 现有command visibility计数不能证明局部绘制复杂度

继续核对自动交互目标后，不能把template paint-index的正计数直接加入通用`idle_hover`门。通用场景从`activity_rail_buttons`、`document_tabs`、`viewport_toolbar_controls`和`template_controls`的合集中最多选择8个目标，目标集合可能完全不经过template-node painter；只有`material_lab_hover`以`TemplateControlsOnly`生成目标，并在interaction evidence中拒绝任何非`template_control`或非`template.*` identity。故通用hover适合证明输入、失效、publication、visual cache和GPU residency没有退回昂贵路径，template候选复杂度必须由精确`material_lab_hover`场景证明，不能靠缺失字段按0解释或要求一条未必执行的路径产生正计数。

current live source已经发布`workbench_paint_index_query/candidate`、`template_node_visit/clone/damage_reject`和`fallback_sort`原始timeline counter。`draw_template_nodes_with_transform`在存在paint rows时只访问候选行，否则才遍历`0..row_count()`，这些计数可以表达template阶段的实际候选、访问和复制规模；但`ui_hotspot.rs`尚未把这些字段聚合进`UiScenarioHotspot`，所以当前`Show-UiScenarioEvidence`无法从hotspot artifact裁决它们。后续应扩展现有hotspot schema和同一采集authority，不得再建平行摘要文件；`material_lab_hover`门至少要求paint-index query为正、`template_node_visit <= workbench_paint_index_candidate`、`template_node_clone <= template_node_visit`且`fallback_sort=0`，并把缺少任一current-schema字段视为证据失败。

更关键的是，`UiSurfacePresentStats::command_visibility_scan_count`的公开注释写的是“deriving current visible statistics时访问的command rows”，实现语义却不是行数。`UiSurfacePresentStatsAccumulator::new`无条件把它初始化为1，`UiSurfaceDrawList::stats`和WGPU `draw_items_with_damage`随后遍历ordered commands并调用`record_visible`，但循环和`record_visible`都不再增加该字段；缓存full-projection stats时才把它改为0并令`command_stats_cache_hit_count=1`。因此当前`GpuCommandVisibilityScans`只能表示“执行过一次visibility stats pass”还是“复用了full stats”，无法区分扫描10条或100,000条command。用它作为O(candidates)或O(changed segment)证据会产生错误结论，`visible_command_count`和payload bytes也只能描述接受结果，不能代替被检查的候选行数。

下一非阻塞实现切片应由RHI owner在同一stats authority中拆开两个概念：保留或重命名pass counter，并新增逐条执行visibility predicate时累加的`command_row_visit_count`；full-stats cache hit时row visits必须为0，damage projection则报告真实候选/访问数。Editor perf schema、hotspot analyzer和profile gate随后一起接入该字段。规模验收必须使用source-bound `material_lab_hover`及N=100/1,000/10,000、固定可见范围和固定damage：逻辑N放大10倍时paint-index candidates、template visits和GPU command row visits不得同比增长，单control变化受`O(overlap candidates + changed segment)`约束，只有显式full fallback允许O(N)。当前RHI和template painter路径分别存在其它owner的在途工作，且Cargo lane未授权，本轮不编辑这些生产路径、不运行产品采集，也不把新增静态诊断写成性能GREEN。

该语义错误背后还存在一条真实产品热路。`CompiledUiBatchPlanCache::resolve`在generation与projection size命中时可以复用完整`BatchDrawPlan`，但只要`draw_list.damage.is_some()`就仍执行`Some(draw_list.stats())`；后者逐条读取全部`draw_list.commands`并计算damage可见统计，因此每次局部damage present都额外承担`O(C)` CPU扫描。结果随后无条件进入`WgpuUiSurfacePresentation`和`last_present_stats`，并不受profile recorder是否启用控制。render阶段的`record_draw_ops_to_view`又线性访问全部compiled `draw_ops`，对每个op做bounds/scissor判断后才提交，成本为`O(B)`。batch合并可能使`B << C`，但image/text residency命中都不能消除这两次遍历。

修复不能从关闭统计开始。generation-owned `BatchDrawPlan`应为每个稳定op保存可合并的command统计摘要，并建立按op bounds索引、按原始op顺序返回的damage candidate list；同一candidate list同时驱动局部统计累加和render recording，避免两套visibility authority。第一步删除cache-hit damage分支的`draw_list.stats()`，把额外统计成本从`O(C)`降到与候选op同阶；第二步让render pass从稳定有序candidate indices迭代，消除`O(B)`全op bounds扫描。跨layer依赖、clip/text batch union bounds与painter order必须保持；过大的merged batch若使candidate退化，应按可独立裁剪的稳定segment拆分，而不是回到逐command全扫。

下层RED应以同一generation、固定小damage和10/100/1,000/10,000 commands构造等价画面，要求第二次present的batch-plan build为0、command row visit为0、op candidate visits不随不相交command总数同比增长，并保持visible stats、clip、z-order、text/image与像素等价。产品规模验收再使用source-bound `material_lab_hover`，要求逻辑N放大10倍时paint-index candidates、template visits和GPU candidate visits不得同比增长；只有显式full fallback允许O(N)。当前WGPU UI surface目录被UI12整体租用，本轮不跨写该生产路径。

### 15.92 `UiSurfaceFrame`仍把小状态publication放大为全域复制与诊断重建

2026-08-23继续沿输入到present的current-source调用链复核，发现一个比15.91更靠上的确定性放大器。`UiSurface::surface_frame()`缓存的只是整个`Arc<UiSurfaceFrame>`；只要`publication.dirty`，或者缓存frame的`window_state`/`focus_state`与surface不同，就创建一份新的frame。新frame会深克隆`UiArrangedTree`、`UiRenderExtract`、authoritative `UiHitTestGrid`、layout report和完整focus state，重新生成focus path、pipeline report，并同步调用`ui_ecs_projection()`。`UiSurfaceFrame`内部这些大域仍是owned `Vec`/`String`图，而不是独立generation的不可变handle，所以最外层`Arc`没有消除publication时的复制成本。

这不是只在显式debug capture中发生。`focus_state`含changes、focused inputs、modal restore stack和hover route等可增长向量；hover、press、capture、keyboard navigation和窗口状态变化都可能使transient comparison成立。一次只改变focused node或window metadata的publication，仍会复制全部arranged nodes/draw order/canvas layers、全部render commands及其text/style/image payload、全部hit entries/cells及其control id/bubble route。`focus_path()`只需要沿既有arranged authority产生`O(depth)`小结果，但现在被绑在其它全域复制之后。

同步ECS投影进一步放大这条路径。`ui_ecs_projection()`先把所有render commands和hit entries分别计入`BTreeMap`，再遍历所有tree nodes并克隆path、children、component/control字段；每个node的effective-disabled检查会沿parent链向上走，hover状态又在`Vec`中执行`contains`。`UiEcsProjectionSnapshot::from_nodes`随后再次扫描nodes生成totals、schedule mask、schedule impacts和dirty-domain impacts。若`L/C/H/N/D/F`分别代表arranged payload、render command、hit entry/cell、tree node、平均祖先深度与当前hover-route长度，则一次focus/window-only publication的保守上界不是`O(1)`，而是至少`O(L + C log N + H log N + N*D + N*F)`，并分配一份新的layout/render/hit/ECS owned graph。深层tree使`N*D`最坏退化为`O(N^2)`。

ECS projection的产品消费也不支持把它放在每帧必经合同中。current source里frame携带projection后，普通debug snapshot只是把它clone到`UiSurfaceDebugSnapshot`，Editor debug reflector再读取schedule/dirty sections；显式`surface.ui_ecs_projection()`和delta API另有直接调用者。也就是说它是诊断/反射产品，而不是pointer routing、layout、paint extraction或GPU present的正确性依赖。把它强制塞入所有frame publication，会让关闭debug UI的产品仍支付全量诊断构建成本。

同一owned-publication问题继续跨层传播。runtime scene UI planning每帧为`PlannedScreenSpaceUi`新建vectors，遍历`UiRenderExtract.list.commands`，并让每条command经`to_paint_elements`生成临时元素再转vertices/draws/text/images；Editor retained host的template paint conversion也逐command执行同类flatten。pipelined queue虽使用有界slot，却在`FrameSubmissionKind::Extract`和`ViewportRenderFrame`中传递owned `UiRenderExtract`，Editor的`EditorRenderFrameSubmission`也每次拥有一份overlay extract；world-space UI merge还会新建command vector再extend。局部paint patch即使在Surface内保住了命中索引和damage，上层仍可能把稳定command graph重新flatten和搬运，形成`O(C)` CPU/分配成本。

Unreal的对应authority不是“每次复制一份完整frame”。`FSlateInvalidationRoot::PaintFastPath`消费invalid widget update list和reindex heap，只处理本轮失效widget；`FSlateCachedElementData`按root保留cached element lists与cached batches，`ListsWithNewData`只标记失效或新增的list。Zircon不必复制Slate类型，但必须复制这个责任边界：稳定域由generation-owned retained storage持有，publication只替换变更域，消费者持有不可变handle；诊断投影是订阅者或显式capture产品，而不是输入事件的隐式税。

目标数据结构应从“一个大owned frame”改为分域publication：

| publication域 | immutable authority | 允许推进generation的原因 | 不得触发 |
|---|---|---|---|
| layout | `Arc<UiLayoutFrame>`：arranged tree + layout report | structure/layout/text尺寸变化 | focus、hover、window activation |
| input | `Arc<UiInputFrame>`：projected hit grid + navigation/focus lookup indexes | geometry/input policy/popup stack变化 | paint-only tint、debug capture |
| render | `Arc<UiRenderFrame>`：stable paint segments + resource identities | paint/style/text/image product变化 | focus变化但无focus visual damage |
| transient | 小型window/focus/capture/pressed state + `UiFocusPath` | 对应小状态变化 | clone layout/input/render graphs |
| diagnostics | lazy或增量维护的`Arc<UiDiagnosticsFrame>` | 显式订阅/capture，或已有dirty set增量更新 | 普通`surface_frame()`、hit-test、present |

顶层`UiSurfaceFrame`仍可有单调generation，但未变化域必须保持`Arc::ptr_eq`，并携带各自domain generation供Editor、RHI和profile证据判断复用。publication只能读取rebuild阶段已经生成的authority；不能把大域构建延迟到`surface_frame()`后再由首个pointer event承担。hit-test继续只读已发布input frame，render queue传递render-frame handle或segment generations，不能退回事件时扫描arranged nodes/render commands，也不能靠call-site special case绕过统一authority。

render域内部也不能只把`Vec<UiRenderCommand>`包一层`Arc`后继续每帧flatten。rebuild/patch阶段应发布稳定`PaintSegment`或compiled-op handle，每个segment包含bounds、clip/layer/order、resource generations和可合并统计摘要；局部damage只返回保持painter order的candidate segment indices。Editor host、runtime scene planner和WGPU presenter消费同一个generation identity，只有新增/替换segment执行command-to-element/vertex转换。world-space与screen-space可有不同typed stream，但都应按generation组合handle，不能为了merge复制未变化command payload。

实施必须先有下层RED和可观测计数，不能以`Arc`类型存在作为GREEN：

1. 构造100,000 nodes、10,000 commands、10,000 hit entries的surface，完成一次warm publication后只改变focus；要求layout/input/render handle均`Arc::ptr_eq`，相应domain generation不变，ECS projection build/clone为0，新增allocated bytes与耗时只随focus depth变化。
2. 只改变window activation/size metadata但不触发布局尺寸变化；要求三大handle复用。真正的resize geometry rebuild只允许layout/input和受影响render segments推进，未变化资源与segment identity保持。
3. paint-only hover用固定一个control、N=100/1,000/10,000/100,000；要求published layout/input handle复用，render replaced segment count有界，frame publication cloned node/command/hit rows为0，曲线不能随N同比增长。
4. 显式请求debug snapshot时允许diagnostics generation推进，但第二次无变化capture必须命中；关闭debug订阅后1,000次pointer move要求ECS projection builds/ancestor visits/serialized bytes均为0。
5. scene/editor/RHI跨层测试记录`render_segment_rebuild_count`、`command_flatten_visit_count`、`submission_payload_clone_bytes`和damage candidate visits；warm稳定帧前三项为0，单control变化只访问重叠候选与changed segments，像素、clip、z-order、text/image及present stats与full reference一致。

建议的TDD落地顺序固定为：先在interface定义分域frame handle/generation合同和pointer-equality回归；再让Surface rebuild-owned state持有layout/input/render authority，并把ECS projection移到显式diagnostics publication；随后迁移Editor template hit frame与viewport toolbar bridge，删除“为hit-test临时构造完整`UiSurface`并rebuild”的产品路径；最后把scene/editor/RHI从owned extract + per-frame flatten切到stable segment handles。每一步都要保留旧全量reference作像素/route对照，但hard cutover后删除兼容双authority，不允许长期同时维护owned frame与handle frame。

本轮没有编辑上述Runtime/interface/RHI生产路径：Runtime74占用大范围UI/interface合同，UI12占用WGPU UI surface，且共享managed Cargo仍在运行。这里新增的是current-source静态证据和可执行迁移合同；在source-bound managed规模测试给出CPU、allocation、clone bytes与latency曲线前，不宣称这项架构瓶颈已经修复。

### 15.93 Editor viewport toolbar命中frame改为有界retained publication

15.92中“为hit-test临时构造完整`UiSurface`”有一条不依赖Runtime/interface/WGPU合同的Editor局部路径，可以先安全收口。`attach_viewport_toolbar_surface_frame_to_pane`原先对每个可见Scene/Game pane无条件执行`BuiltinViewportToolbarTemplateBridge::recompute_layout`，随后`surface_frame_for_projection_controls`再创建第二个`UiSurface`、插入root和全部route-bearing controls、执行完整`rebuild()`与`surface_frame()`。同一个presentation结构被重复附加，或者不影响toolbar尺寸/route identity的局部状态更新，也会重复支付template layout、host projection、hit surface layout/render/hit/ECS publication。

本轮在Editor bridge内新增单一`ViewportToolbarSurfaceFrameCache` authority，没有给call site增加平行hit-test。cache按`surface_key`保留最新frame，签名精确包含physical-independent surface width/height bits，以及每个eligible route control的`projection_control_id`、`Option<hit_control_id>`、component和arranged frame；节点顺序也属于签名。构建hit tree时只物化`Some(hit_control_id)`，但`None`仍保留在签名中，因此映射的增加与移除都会生成新frame。cache总量上限64项，使用单调access generation在插入超限时淘汰最旧项；稳定exact命中只做`O(P + log K)`映射验证并返回原`Arc<UiSurfaceFrame>`。同key、同size而route mapping变化时，cache复用已发布的component/frame重新投影小型hit surface；本surface缺少该size时还会在最多64项中查找其它surface的同尺寸layout signature，以有界`O(K + P)`构建本surface自己的tree/path/frame。只有整个cache都没有该size时才进入完整template layout，跨surface只共享布局几何，不共享surface identity。

产品attach现在先用cache中保存的projection-control列表验证surface size并一次计算当前viewport route映射。结果分成四态：exact mapping返回原frame；同surface、同size mapping变化从缓存geometry重新投影frame；其它surface已有同size时复用其layout signature并生成当前surface frame；所有retained entry都缺少该size时才执行`recompute_layout`并更新host projection。cache模块发布`prelayout_surface_frame_cache_hit/miss/reproject/shared_layout`和`surface_frame_cache_hit/miss`计数，后续产品profile可以分别量化稳定复用、route-only重新投影、跨pane布局复用与真实layout miss，不能只凭`Arc::ptr_eq`测试推断真实命中率。

TDD合同覆盖两层：bridge lower test要求相同key/size/mapping连续请求保持`Arc::ptr_eq`；单control映射变化必须发布新control id、保持`layout_recompute_count`不变，随后相同新签名再次复用；先前为`None`而未进入hit grid的映射变为`Some`时必须从缓存geometry重新投影出新增命中节点，同样不得relayout。pane product test要求第二次稳定attach的frame pointer不变，修改`mode`只替换route frame而两次路径的`layout_recompute_count`都不增长，随后全cache未见的新surface width必须替换frame并且只增加一次layout recompute；附加的1000次Select/Move/Rotate/Scale route-only状态风暴逐次验证最终hit control id，同时要求layout recompute总数保持首次attach后的常量。多pane审查进一步确认产品点击从pane持有的`Arc<UiSurfaceFrame>`命中，不读取bridge最后一次`host_projection`；双surface回归按1280/640不同宽度依次发布后回到首surface，必须复用首surface原Arc且不得再次relayout。第三个640 surface以及原1280 surface迁移到已有640 layout时都必须保持layout计数不变、得到独立Arc并与既有640 control frame几何一致，从而同时锁定surface identity和跨pane布局复用。旧代码没有cache，第一个pointer-equality断言必然为RED；第一版cache会在`None -> Some`、route-only零relayout和跨surface同size复用断言上RED。focused Rust tests仍未执行，当前状态只记为“实现完成、managed validation open”；本轮最终静态检查结果记录在后续验收回执中。

这个切片不能被描述为窗口resize整体修复。每个新的resize尺寸都会使签名变化并正确miss；主Workbench Surface的frame全域clone/ECS投影、RHI damage stats与draw-op全扫、SVG尺寸bucket/GPU product identity仍分别按15.87、15.91、15.92保持open。它消除的是稳定presentation重复附加和非几何状态更新中的确定性重复工作，并为后续分域frame publication提供一个已hard-cut到单一cache authority的Editor消费者样例。

首次exact-source managed validation ticket为`37e2347c44314619921f2191709700b1`，source manifest hash `b195deaee280b0637393a162a9096f797d36f5306016d307a9f253b666c9cab0`，copy job `918c9c02897c4f7480bfe140fb877b8f`，终态`failed`。它没有进入Cargo：materialization prepare以`validation_copy_baseline_drift`拒绝，漂移路径是Runtime74 owner下的`binding_targets.rs`、`binding_param_resolver.rs`、`control_scope.rs`及两条control-scope tests，首路径为`zircon_runtime/src/ui/surface/binding_targets.rs`。随后自审新增`None -> Some`失效回归并修正cache签名，旧manifest已经被current source取代；本轮不吸收这些外部改动、不重复提交，待对应support commit进入HEAD且baseline稳定后，必须重新计算Editor exact-5 manifest再提交同一`viewport_toolbar` focused batch。

与该产品候选并行收口的profile证据门已在current worktree完整重跑：`tools/tests/ui-profile-capture-output-contract.Tests.ps1`为`42/42`通过、0失败、0跳过，Pester记录`67.878s`、process exit 0；3个PowerShell路径的parser error为0。该GREEN只证明采集器会拒绝stable hover中的全量失效、SVG/GPU churn，以及resize中的GPU image command回扫；它不替代focused Rust tests或真实Editor profile。

### 15.94 Viewport toolbar retained frame cache进入source-bound动态门

15.93虽已为toolbar cache发布六个原始计数，但旧采集器没有消费它们，因而存在“源码里有cache、产品场景仍每次relayout却继续GREEN”的证据断层。本轮新增`Test-ViewportToolbarCacheCounterGate`，只对精确`viewport_toolbar_click`生效；warm measured run必须至少记录一次prelayout cache决策，允许exact hit、route-only reproject和跨pane同尺寸shared layout共存，但`prelayout_surface_frame_cache_miss_count`与`surface_frame_cache_miss_count`都必须为0。前者表示所有retained entry均没有当前尺寸，后者表示已进入完整projection/layout后的resolve路径；任一非零都证明测量窗口内发生了真实layout fallback。`surface_frame_cache_hit_count`只作为诊断输出，不要求为正，因为prelayout三条快路会在进入该resolve阶段前返回。

该门已加入最终`RequireScenarioEvidence`合取，不能由通用`click`或其它场景误触发。source-bound manifest同时加入`app/viewport_toolbar_projection/surface_frames/pane_frame.rs`与`callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs`，防止旧Editor二进制配合新PowerShell gate产生假证据。当前critical source清单为142项，142项唯一、0项缺失；两个新增实现路径都在清单中。

TDD先加入合成timeline正例，再分别构造零cache决策、prelayout miss和resolved miss。旧实现的完整合同运行精确失败于缺少新gate、最终gate组合和两条manifest source binding；实现后首次复跑只因清单条目从140增至142的旧精确计数失败，更新该合同后完整`tools/tests/ui-profile-capture-output-contract.Tests.ps1`为`43/43`通过、0失败、process exit 0、39.3秒。4个PowerShell文件AST解析均为0错误；toolbar exact-5 Rust路径`rustfmt --edition 2021 --check --config skip_children=true`通过，9路径scoped `git diff --check`无空白错误，只有仓库既有CRLF转换提示。

这仍不是产品性能GREEN。focused `viewport_toolbar` Rust tests和current-source Editor EXE尚未执行，真实计数也尚未证明1000次按钮交互在三次独立measured run内持续零miss。正式验收必须同时满足通用click的交互完成/CPU/RSS/input latency合同与本节cache合同，并报告hit、reproject、shared layout各自分布；若真实轨迹有miss，应先按surface size、route identity、pane publication generation和cache lifetime归因，不得把miss阈值放宽或在call site增加第二套frame cache。

本轮同时复核UI12上报的7条current-source编译指纹。当前HEAD已包含提交`7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`：pointer route以借用方式分发，navigation route在消费前预取target/kind；IME测试直接匹配`UiInputEvent::Ime`，四处preedit clause使用字段绑定加`is_empty()` guard。三个文件`rustfmt --check`通过，且接口测试文件没有本地覆盖；`event_routing.rs`现有未提交差异仅属于外部binding-target工作。因此不为已消失的E0382/E0308/E0164重复制造补丁，下一次managed compile若再次报告这些位置，应先核对ticket的source HEAD/overlay而不是回退正确current source。

### 15.95 Viewport toolbar exact hit删除逐control映射与稳定key分配

15.93的第一版retained cache避免了稳定attach时的layout和hit surface rebuild，但继续复核exact-hit分支后发现它仍不是常量热路。`resolve_if_layout_matches`每次先遍历全部eligible projection controls，调用映射闭包生成`Vec<Option<String>>`，再逐项比较缓存值；`attach_viewport_toolbar_surface_frame_to_pane`还会先clone整份`SceneViewportChromeData`。四个docked pane与floating pane调用方又把已经稳定持有的`SharedString surface_key/window_id`逐次转换为新`String`。因此warm cache虽然返回同一个`Arc<UiSurfaceFrame>`，每次presentation attach仍支付`O(P)`映射、最多P个control-id字符串构造、一个临时vector，以及每pane一个surface-key分配；“有cache”并没有消除按钮交互中的这段CPU和allocator工作。

本轮把命中identity拆成几何签名与借用式hit-route key。`viewport_toolbar_hit_route_key`只包含映射函数实际读取的`mode`、`transform_space`、`projection_mode`和`view_orientation`四项`&str`；cache entry在首次构建或route变化时保存它们的owned快照。产品attach用`&pane.viewport`直接计算借用key，不再clone viewport；cache在任何projection-node遍历前比较固定四项，命中时只更新LRU generation并返回原Arc。docked/floating边界也改为借用既有surface key，只有首次插入新cache entry时BTreeMap才拥有一次key。于是稳定路径从`O(log K + P)`映射/分配收敛到`O(log K + 4)`字符串比较、一次Arc clone、零control-id重建和零surface-key分配；route变化、首次surface或跨surface shared-layout发布仍保留必要的`O(P)`小型重投影。`display_mode`、snap数值和其它不改变hit route的presentation状态不会伪失效。旧bridge API仍以无route-key模式保留逐次映射语义，只有已证明的pane产品路径调用新API，避免扩大native-host测试合同。

TDD先在pane lower test引用尚不存在的`hit_control_projection_count`，源码RED为4个调用、0个定义。实现后test-only计数只统计实际执行的projection-control映射：首次构建必须为正；相同pane/size/route、仅display/snap变化以及后续1000次稳定attach都要求计数不再增长且frame保持`Arc::ptr_eq`；改变mode必须增加映射并发布新frame，随后同route再次保持常量。另一路1000次Select/Move/Rotate/Scale风暴继续要求route id正确且layout recompute为0。source guards确认route-key early return位于`mapped_hit_control_ids`之前，三个attach调用文件不存在`pane.viewport.clone()`、surface-key/window-id `to_string()`。

source-bound profile manifest同时加入`hit_controls.rs`、`surface_frames/docked.rs`和`surface_frames/floating.rs`，当前为145项、145项唯一、0项缺失。实际安装的Pester 3.4.0完整运行`tools/tests/ui-profile-capture-output-contract.Tests.ps1`为43/43通过、0失败、process exit 0、39秒；6个Rust路径`rustfmt --edition 2021 --check --config skip_children=true`和scoped `git diff --check`通过，后者只有仓库CRLF提示。focused Rust仍未运行：当前baseline health为`degraded`，reason为`1127 unaccepted workspace change(s)`，本轮没有提交Cargo ticket。下一次baseline稳定后必须重新计算包含既有5路径加本节`hit_controls/docked/floating`的exact-8 manifest，只提交`cargo test -p zircon_editor --lib --locked viewport_toolbar` managed ticket；通过后再构建current-source Editor并运行1000次toolbar交互，要求cache miss为0、CPU/RSS/latency合同完整。未获得这些动态证据前，状态保持“稳定热路算法与静态合同完成，Rust/product validation open”。

### 15.96 Profile场景counter gate从3000行主脚本拆出

15.90已明确`tools/ui-profile-capture.ps1`超过3000行后不得继续堆叠场景门，但15.94为补齐toolbar动态证据仍把新gate放回主文件。继续在同一文件扩展SVG、resize、scroll或Runtime Diagnostics合同，会把参数解析、进程编排、artifact导出和七类独立counter判定混成一个不可审查单元。本轮按职责新增`tools/ui-profile-counter-evidence.ps1`，集中承载asset refresh、window resize、stable visual asset、viewport toolbar cache、hierarchy scroll、welcome recent scroll与Runtime Diagnostics七个gate；主脚本只保留原函数名的薄wrapper，因此现有调用方和测试按函数名定位的合同不变。新实现继续复用主authority的`Get-UiCounterTotal`、interaction scenario解析和CPU/RSS gate，没有复制timeline解析器或创建平行报告格式。

新模块由主采集脚本显式dot-source，同时进入Pester source aggregation、fixture repository与`Get-ZirconProfileCaptureToolPaths`。capture tool manifest从9项变为10项且10项唯一，新文件因此参与profile source/binary fingerprint，旧采集脚本不能配合新counter逻辑生成假GREEN。主脚本从本轮前3080余行降为2723行，新模块463行；七个wrapper与七个`Test-Zircon*CounterGate`实现一一对应，两个文件均远低于1000行模块边界。critical runtime/editor source manifest不因工具拆分改变，仍为145项、145项唯一、0项缺失。

TDD先要求主脚本加载语义模块、七个wrapper委托到对应实现、module source非空、主脚本少于3000行，并把capture tool精确计数改为10。旧代码完整Pester为42/44通过、2失败：只失败于模块尚不存在和tool manifest仍为9，其余场景合同保持GREEN。实现后第一次功能断言已44/44通过，但Pester 3.4在较长`E:\cargo-targets\...` TestDrive清理阶段触发Windows长路径错误，故没有把该轮算作GREEN；改用短且仍位于非C盘的`E:\p\m`临时根后，同一完整文件为44/44通过、0失败、process exit 0、53.2秒。四个PowerShell文件AST error为0，scoped `git diff --check`无空白错误。该切片只改善工具模块边界与证据可维护性，不代表产品UI性能变化；其价值是后续新增动态门不再继续扩大单体脚本，并保证所有新门必须进入同一source-bound工具清单。

### 15.97 Toolbar cache动态门证明exact hit没有隐藏映射

15.95的test-only映射计数能锁定下层算法，但15.94的产品gate仍只看到hit/reproject/shared/miss。旧实现即使在每个exact hit内继续遍历全部controls，只要最后增加`prelayout_surface_frame_cache_hit_count`，产品profile仍会GREEN。为消除这条证据盲区，cache在三个实际执行control-id映射的位置统一发布`hit_control_projection_batch_count`和`hit_control_projection_visit_count`；route key变化但最终hit ids未变化时额外发布`prelayout_surface_frame_cache_route_key_update_count`。exact route-key命中在记录函数之前直接返回，因此不产生batch或visit。

`Test-ZirconViewportToolbarCacheCounterGate`现在执行守恒校验：映射batch必须精确等于`reproject + shared_layout + route_key_update + resolved_hit + resolved_miss`，visit必须不少于batch；原有prelayout/resolved miss为0合同继续保留。这样合法重投影可以量化，任何exact hit上的隐藏逐control工作都会让batch多于可解释结果而失败。合成正例为8次exact hit、2次reproject、1次shared layout，对应`projection_batches=3/3`、visits=48；注入第4个隐藏batch得到`4/3`并被拒绝，把visits降到2也被拒绝。测试同时读取current Rust source与counter module，要求三条counter名在两端都存在。

TDD旧门完整运行得到43/44通过、唯一失败是隐藏第4个batch仍返回true。实现后第一次复跑只暴露Pester 3对未加括号的`Should Match [regex]::Escape(...)`解析问题；修正测试语法后，短非C盘临时根上的完整Pester 3.4.0结果为44/44通过、0失败、process exit 0、94.7秒。Rust cache `rustfmt --check`、四个PowerShell文件AST和scoped `git diff --check`均通过；source guard确认exact-hit block中不存在`record_hit_control_projection`且其return位于所有映射记录之前。该GREEN仍是合成evidence合同，不替代current-source Editor真实计数；产品验收必须同时报告hit/reproject/shared/route-key-update、projection batch/visit及CPU/RSS/latency。

本节静态收口时共享HEAD已外部推进到`f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`，baseline epoch 355仍为`degraded`，reason为`1244 unaccepted workspace change(s)`，且另有一张managed test job运行。current exact-8 toolbar manifest hash为`1d89775c9950772f488cdf7d47e5775ea0e87315580ff4e8d8bb1d0e62d70c11`，只作为未提交的静态快照；本轮未创建validation ticket，lane与baseline稳定后仍必须重新计算，不能复用该hash冒充current-source绑定。

### 15.98 稳定hover新增SurfaceFrame全产品publication硬门

继续复核15.92时先纠正两个容易把优化方向带偏的假设。第一，普通鼠标查询的`cursor_radius=0`已经直接读取单个hit-grid cell并反向遍历entry，复杂度是该cell候选数`O(M)`；只有扩展半径查询才进入候选合并、`Vec::contains`去重与排序，因此后者不是当前按钮hover卡顿的首要解释。第二，`UiSurface::hit_test_with_query`直接读取projected hit authority，不调用`surface_frame()`；所以`surface_frame()`的arranged/render/hit深复制和`ui_ecs_projection()`全树构建是“产品消费者在dirty/transient变化后请求完整frame”时的条件成本，不能错误描述为每个Runtime鼠标事件必经成本。Editor template-node frame builder、floating bridge和诊断消费者仍会请求该publication，是否在稳定hover反复触发必须用产品轨迹裁决，不能只凭源码存在推断。

本轮在真正的publication rebuild分支增加一组批量Runtime计数，且没有改变cache-hit分支。每次实际发布记录一次`publication_build_count`，并报告将被复制的arranged node、render command、hit entry、hit cell membership数量，以及本次重新生成的ECS projection node数量；hit authority只解析一次后同时用于计数和最终clone，ECS projection也只生成一次后移入frame。计数只在`profiling` feature下编译，使用一次`record_counter_batch`，不会给普通产品build增加逐字段profile锁开销。

新的`Test-ZirconSurfaceFramePublicationCounterGate`只约束精确`idle_hover` measured run。它从timeline读取`ui.surface_frame.*`全局Runtime计数，要求publication build及五类O(N)产品规模全部为0；任一非零都会使最终`RequireScenarioEvidence`合取失败。这里允许整组字段不存在并按0解释，因为“不调用`surface_frame()`”本身就是合法且更优的稳定路径；真实性由source/binary manifest保证，`frame_publication.rs`已加入critical source清单，清单现在为146项、146项唯一、0项缺失，旧Runtime二进制不能配合新门生成假GREEN。后续真实轨迹若失败，必须先按具体消费者和dirty原因定位publication调用，不能把门槛放宽，也不能在call site再造一份frame authority。

TDD先只加入gate、wrapper和manifest合同，完整Pester得到`42/45`通过、3失败，精确对应三项production尚不存在；实现后短非C盘临时根上的完整`tools/tests/ui-profile-capture-output-contract.Tests.ps1`为`45/45`通过、0失败、0跳过、process exit 0、45.94秒。Rust `frame_publication.rs`的`rustfmt --edition 2021 --check --config skip_children=true`通过，4个PowerShell文件AST error均为0，scoped `git diff --check`无空白错误，manifest实测`count=146 unique=146 missing=0`。这些是合成证据门与静态生产检查，不是Editor动态性能GREEN；当前共享baseline仍为`degraded`且两张managed Cargo正在运行，本轮没有启动raw或managed Cargo。

SVG/GPU复核也收紧了15.87的实施前置条件。当前Editor retained image painter只把逻辑frame宽高传给`RasterTargetSize::from_frame`，调用链没有携带窗口`raster_scale`，因此不能仅把`ceil()`替换成所谓physical bucket：在没有physical scale的边界上量化会把不同DPI产品错误合并。正确顺序是先把typed raster scale传到visual raster product authority，再以source/content generation、quantized physical extent、scale与variant/tint组成identity，并用像素误差合同决定bucket和hysteresis。Unreal的vector cache以`LocalSize * DrawScale`形成pixel size，Slint也把parse identity与scale相关的render/backend product分离；两者都支持这个顺序。相关Editor visual-assets与WGPU UI surface当前由UI12广域租用，本轮只形成报告和动态门，不跨写其production路径，也不回退已有SVG tree、visual raster、icon atlas和GPU local/shared cache。

最后，UI12上报的2条E0382与5条E0308/E0164在current HEAD已经由`7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`修复：pointer dispatcher借用route，navigation消费前预取target/kind，IME normalize结果直接匹配`UiInputEvent`，preedit clauses在pattern中绑定后用guard检查为空。三文件静态rustfmt与diff检查通过，接口测试文件无本地差异，`event_routing.rs`唯一未提交差异属于外部binding-target owner。本轮不为已消失错误重复编辑；若下一张managed ticket仍报告旧位置，必须核对copy HEAD/overlay与fingerprint时间，而不是修改正确current source。

### 15.99 Pointer与resize主链已转向分域状态，但动态验收仍不能省略

继续沿Editor native pointer入口核对后，current worktree已经不是“每个鼠标事件materialize整份presentation”的旧结构。`dispatch_pointer_move_body`只取得一次`HostPresentationGeneration`，其中structure、menu、page overflow、pane interaction、viewport image、hit index、theme与diagnostics均为独立`Arc`句柄；命中使用generation持有的`HostWorkbenchHitIndex`，paint则通过generation scope注入分域状态，不需要调用`HostPresentationGeneration::materialize()`。template hover setter在control identity、dispatch payload与frame均未变化时直接返回；pointer redraw在interaction快照未变化时返回idle。Workbench命中已有空间bucket与paint row index，1000次稳定命中不得重建template surface这一合同也已存在。由此，后续定位不能再把“外层presentation是一个大struct”直接等同于“每次pointer move全量clone”；必须先证明是哪一个generation、invalidation target或publication authority实际增长。

窗口resize路径也出现了同方向的外部owner候选。`update_drawer_resize_capture`先比较上一次preferred size，相同像素位置直接返回；变化时不再调用通用`mark_layout_dirty()`，而是提交`HostInvalidationMask::WINDOW_METRICS`，使`recompute_if_dirty`优先尝试`build_window_metrics_shell_snapshot()`和committed shell metrics阶段。UI Asset scoped presentation的在途改动则把floating rows从“预先clone全部窗口”改为只clone命中row，并通过`with_row_patches`复用其它row storage。这些改动都处在外部未提交路径，本轮只把它们当作current-source依赖和待验证候选，不吸收到Runtime09候选、不改写其测试，也不在managed Rust与产品profile完成前记为性能GREEN。

静态审查同时暴露一个分域快照一致性风险。`route_pointer_to_pane_with_mode`当前接收`generation.structure()`，却从structure内嵌的`presentation.pane_interaction_state.console_scroll_px`读取console scroll；而`HostContractState`已把真实interaction state拆为单独`Arc`，`replace_host_presentation`还会把structure中的旧字段重置为default。若没有其它同步合同，该route可能用陈旧scroll计算console hit。它首先是命中正确性问题，也可能因错误target引发额外leave/hover redraw；修复应由正在修改pointer generation链的owner让route显式接收同一generation的interaction snapshot，并增加非零scroll下frame/instance route等价回归，不能重新把interaction写回structure或在事件端materialize整份presentation。

Runtime Diagnostics的双阶段构建不应冒充当前按钮卡顿主因。pane conversion先构建body surface，再由surface snapshot生成debug reflector nodes，随后为变化后的nodes重建最终hit artifacts；辅助函数目前还会另建一份debug `UiSurface`。这确实存在可删除的冷路径重复工作，但只对RuntimeDiagnostics pane conversion生效，且相关文件已有其它owner在削减临时字符串分配。除非产品profile显示该pane在普通hover/resize中被错误地持续recompute，否则优先级低于全局invalidation、SurfaceFrame publication、RHI damage candidate扫描和SVG physical product identity。

下一轮动态裁决必须把上述机制分开验收。`idle_hover`的1000次移动要求structure generation、presentation rebuild、layout dirty、hit-index build、SurfaceFrame publication、full-frame redraw、SVG parse/raster/upload与GPU command回扫全部为0，同时interaction generation只在跨control边界时增长；`window_resize`的200步要求每个有效尺寸变化进入window-metrics target，full/shell-content fallback为0，重复坐标不产生transaction，且每步damage-to-submit保持有界；`viewport_toolbar_click`继续要求exact hit不执行control projection或layout。任何一项失败都应按counter对应authority修复，而不是增加第二套cache或放宽门。当前共享OS仍有一组managed `cargo build -p zircon_runtime`/`rustc`占用公共target pool，本轮继续不启动raw或managed Cargo，也不使用旧Editor binary给这些候选背书。

上述共享managed build随后自然结束。只读解析其最新fingerprint `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d\debug\.fingerprint\zircon_runtime-9c96844cadef12ec\output-lib-zircon_runtime`（2026-08-23 04:31:34）得到Rust error总数0，确认HEAD中的popup projected-hit实现与UI12先前列出的E0382/E0308/E0164旧错误没有阻断该次Runtime library build。该job属于shader06、不是Runtime09 source manifest，不能证明focused popup tests，也没有覆盖未提交的`frame_publication.rs`计数或Editor toolbar cache；因此只记为共享编译前置清零，不升级本计划的行为或性能状态。coordinator此后报告Cargo、git mutex与blocking均为0，但baseline仍为`degraded`，故本轮仍不提交可能重复触发baseline-drift的Runtime09 ticket。

### 15.100 增量布局旧几何快照改为线性伴随存储

继续核对`layout/pass/incremental.rs`后确认，传播root与visited子树已经避免无关layout island，但选中子树会先进入`BTreeSet`，再把同一K个节点的frame/clip复制进第二棵`BTreeMap`，layout结束后又对每个visited id执行一次对数查询。该结构没有提供额外identity或去重语义：roots已去掉祖先重叠，visited集合才是后续arranged/render/hit patch需要的唯一有序集合；previous geometry只在同一次调用中按节点一一消费。

实现保留visited `BTreeSet`和changed-node `BTreeSet`，只把previous geometry改为按visited迭代顺序生成的`Vec<(UiNodeId, LayoutGeometry)>`，layout后直接`into_iter`线性比较。于是快照构建和旧值查询由`O(K log K)`降为`O(K)`，并删除第二套树节点分配；root选择、measure/arrange顺序、f32 frame比较和最终changed ids完全不变。TDD先让独立Python合同因`BTreeMap`/`previous.get`存在而RED，随后Runtime源码回归与该合同转为GREEN。增量原生Flex夹具进一步锁定当前脏轮次恰好报告1次Taffy树构建、3个Taffy节点构建，`UiSurfaceRebuildDebugStats`与report保持相同数值，紧接的稳定无dirty轮次为0/0；这避免把累计layout-engine selection误当作当前轮次工作量。

`measure_node`的每容器`children.clone()`也做过替代方案审查，但没有纳入候选。`UiTreeNodes`以`BTreeMap<UiNodeId, UiTreeNode>`存储；用child index逐次短借用parent虽可消除临时向量，却会把一次`O(m)`连续复制换成m次`O(log N)`节点查找，CPU复杂度可能恶化为`O(m log N)`。正确方向是让surface generation持久化child/order派生，或让layout pass消费一次构建的连续拓扑视图；在没有该authority和动态数据前保留当前线性复制。审查期间尝试的默认slot-order快路也已撤回：它在自定义order容器上先预扫描全部children、随后又执行原排序收集，增加一轮`O(m log E)`edge lookup；更关键的是现有`UiLayoutSlotIndex`会为每条父子边预填充空索引，同cardinality替换把slot移到此前无slot的edge时，空缓存会直接返回`None`而不验证当前slots。当前`UiTree.slots`仍允许直接push、retain和整体替换，尚无可靠mutation generation；在建立mutation-owned generation前，既不能用事件时全slot扫描修补，也不能声称默认order快路正确。

观测链只增加现有authority上的字段，不引入第二套计数状态：`UiSurfaceRebuildReport`与interface debug stats新增serde-default的tree/node计数，增量rebuild取当前`layout_stats.layout_engine_report`，full compute取刚完成的`self.layout_engine_report`；Editor template projection在已有rebuild提交点把surface rebuild、visited、geometry changed和两个Taffy计数一次batch写入recorder，profiling关闭或capture未激活时不进入recorder。布局快照与观测源码进入critical source manifest，当前为174/174且无重复。独立布局合同5/5、全量性能源码合同354/354、capture-output Pester 47/47、damage evidence Pester 9/9、Rust rustfmt与scoped diff check均通过；未运行Cargo，也没有以计数存在代替产品CPU/p95。

这不是P1 auto-layout问题的终点。`UiLayoutCache`当前只有结果和text revision，没有记录desired size对应的layout input、available size/scale、visibility occupancy或child-order generation；裸用`dirty=false`跳过子树会在collapsed恢复、父约束或responsive style变化时复用错误结果。正确下一步仍是先把slot/order写入收敛到显式mutation generation，再参考Unreal `bNeedsPrepass + PrepassLayoutScaleMultiplier`与Fyrox `is_measure_valid/prev_measure`建立显式validity key，让desired/occupancy真实变化自底向上推进，最后持久化Taffy node/style identity。当前只声明线性几何快照和观测链成本下降，不声明layout visited、CPU p95或窗口缩放已经动态改善。

不启动Cargo的current-source合同补充为11/11通过：drawer resize阶段复用1项、native template hover 2项、pointer move generation/no-op 4项、template surface single-pass 2项、profile artifact gate 2项，均由`tools/tests/test_editor_*performance_contract.py`直接读取生产源码完成。另行完整执行`tools/tests/build-editor.Tests.ps1`为17/17通过、0失败、0跳过、241.82秒，证明`tools/build-editor.ps1`通过仓库managed validator发布`zircon_editor.exe`与`zircon_runtime.dll`、只允许D/E/F盘`ZirconBuilds`、失败不覆盖旧bundle并遵守root-bound staging lease。两组结果分别是源码结构合同和构建工具fixture，不是Editor产品交互profile，也没有授权绕过后续focused Rust与真实EXE压力门。

### 15.101 混合资源painter order必须以单一有序segment修复

继续纵向核对game/screen-space UI提交链，确认15.86的绘制正确性风险不是命名问题。`render.rs`按原command顺序规划时分别把solid vertices、image batches、Auto/Native/SDF text和post-text decoration写入不同数组；`render/record.rs`随后固定执行全部solid、全部image、全部text、全部post-text。于是`image -> later solid`、`text -> later image`等合法重叠都会被资源类别重新排序。Auto text还会在prepare阶段按字体资源与fallback结果转入Native或SDF，若只给solid/image增加ordered op而让文字继续整批render，仍会产生错误的半修复。未解析Image/Icon降级为实心矩形则进一步掩盖资源失败，像素可能看似“有东西”但不是请求的visual。

三份参考源码给出同一边界。Unreal `ElementBatcher.cpp:171-276`为render batch保留layer，对batch index按layer执行stable sort，同layer原始提交顺序因此仍是authority；只有layer相同且`IsBatchableWith`成立才合并。Fyrox `fyrox-impl/src/renderer/ui_renderer.rs:122`按drawing command顺序准备geometry/material，提交阶段也按同一commands序列逐项draw，不先把texture/text/solid全局分桶。Slint `partial_renderer.rs:406-555,666-680`以BackToFront访问item tree，dirty region只决定是否转发`draw_rectangle/draw_image/draw_text`，不会改变访问顺序。Zircon应采用更保守的单一`UiPaintOpSegment`流：每个segment保留command/layer/clip和source order，solid/image/text/decorations都只引用各自prepared range；record严格回放segment，batching只合并相邻且pipeline/resource/clip兼容的op。

文字prepare必须额外返回`original segment -> resolved Native/bitmap/SDF ranges`，whole-batch fallback也只能替换该segment的text op，不能把fallback文字移动到全局text尾部。正式RED至少覆盖`image -> solid`、`solid -> text -> image`、两个重叠text child、inline image与table post-text border，以及missing Image/Icon产生typed diagnostic且不生成solid placeholder。当前`text.rs`及font/prepare report存在另一owner的current-source改动，完整协议会同时修改planner、三种text backend与record；本轮没有实施只修solid/image的半方案，也没有回退这些外部改动。该P0保持design-ready，待文字owner边界可用后按lowest-layer mixed-order像素测试先行。

### 15.102 `ModelRc`顺序遍历改为有界双向patch cursor

`PersistentRowPatchMap`原先已使random `get`与单行path-copy update保持`O(log N)`，但`ModelIter`的两个overlay分支对每个base row重新调用`patches.get(row)`；因此full iter、`PartialEq`和`map_preserving_metadata`会把一次Delta优化放大成`O(N log N)`。新实现保留trie storage与版本共享语义，只新增正/反向单调cursor。cursor沿实际branch下降，frontier每个depth至多保存一个待访问兄弟；一个`usize`位图用`trailing_zeros`取得最近兄弟，row prefix由上一leaf row、高位mask和该depth恢复。frontier是`usize::BITS`个可空节点引用的栈内数组，不分配`Vec`，一个方向遍历只访问每个真实trie节点一次。`ModelIter`为`next`和`next_back`各持有一个cursor，交错消费仍由共享front/back row边界阻止重复返回。

顺序成本因此变为`O(N + P*W)`，`P`为有效patch数且`W<=usize::BITS`；在固定64位平台上对N线性，并且不牺牲random `O(log N)`或每代path-copy更新。TDD先以新静态合同得到3项预期RED，再实现游标转为3/3；随后零堆frontier门先因缺少fixed stack得到有效RED，压缩表示后合同转为4/4。最后可见性门证明`pub(crate) ModelIter`会泄漏父模块私有cursor，concrete enum改为模块私有、`iter()`只返回`impl DoubleEndedIterator + ExactSizeIterator`后合同5/5。继续审查发现`PartialEq`即使两侧共享同一storage也会完整遍历；第6项静态合同先RED，随后比较改为`row_count + shares_values_with`的O(1)短路，不同storage仍执行原深值比较。

纯标准库patch-map通过独立`rustc --test` 6/6：10k行3 patch访问节点少于128、反向顺序、cursor尺寸、replace cardinality、旧generation隔离和最高`usize`位的prefix恢复。随后使用既有受管产物中的`image`依赖只读编译完整`primitives.rs`，18/18通过，覆盖同一patch双端交汇、共享128行模型相等时比较计数为0、独立等值storage仍比较128行及现有像素共享回归。两次均使用临时`#[path] mod`包装，临时E:产物已删除；没有启动Cargo或写共享target。

这仍不是完整model generation/delta闭环。下一步必须由publication携带exact changed rows；共享storage的稳定比较已经零行访问，但不同storage的`PartialEq`仍不得被input、recompute、paint或hit热路用作代际判断。还需以patch density、trie resident nodes和仍存活old generations决定何时compaction为新的contiguous/shared-row base。需要1/100/10k/100k模型的random get、sequential visits、cursor bytes、node allocation、old-generation retention和compaction前后row identity/profile；未完成managed crate测试与产品数据前，本节只标记static candidate。`primitives.rs`与patch-map已加入critical source manifest，以防后续profile binary遗漏该顺序遍历实现；下一节继续加入Console流式paint路径后总数更新为180。

### 15.103 Console可见paint行计划改为零分配流式访问

Console current source虽已把命中限制到静态节点与可见line range，paint仍受`TemplateNodePaintTransform::row_visit_indices -> Option<Vec<usize>>`协议约束。每次paint都由`ConsoleOutputPaintMetadata::visible_node_rows`分配一个新Vec，再依次写入静态前缀、可见line节点与静态后缀；256行上限让它不会无限增长，但pointer hover、局部damage或scroll下仍产生与帧数同阶的短命分配。通用draw在得到Vec后立即单次消费，容器没有跨帧authority或复用价值。

本轮给transform增加对象安全的`stream_row_visit_indices`回调，默认返回false，所有既有Asset/Component transform继续使用原`row_visit_indices` fallback。Console单独覆盖流式入口，以`0..line_start -> visible_line_node_rows -> line_end..row_count`保持原paint order并逐row调用collector；通用draw只在transform未处理流式访问时才创建原indices计划或走all-row fallback。因此这不是全局协议切换，也不改变其它owner的可见行、damage、hover clone与command收集语义。源码TDD先因协议缺失得到1项有效RED，新增合同后Console投影组5/5、完整性能合同361/361通过；四个生产路径已加入source-bound profile manifest，总数由176增至180，manifest核验180/180且无缺失/重复，完整capture-output Pester 47/47通过。另以最小外围stub通过`#[path]`直接编译当前生产`transform.rs`、`draw.rs`与Console projector，`rustc --crate-type lib` exit 0，证明对象安全、回调闭包借用和方法签名成立；临时E: wrapper/rlib已删除。

该候选只删除paint行索引Vec，不删除command Vec，也不把最多512个severity/message retained节点变成visible slots。managed Rust尚未执行，产品门仍需证明Console稳定paint的row-plan allocation为0、visited/cloned保持`static + visible`、scroll/追加像素一致，并继续推进输出追加只重绑entered/expired slot的实例虚拟化。

### 15.104 焦点导航改为rebuild-owned最终几何索引

沿`dispatch_navigation_event`与`UiDispatchEffect::RequestNavigation`继续向下追踪，确认旧实现不是简单的候选比较慢，而是authority位置错误。每次Next/Previous/方向事件都会调用`UiTree::next_navigation_target`：先全扫tree找最高modal scope，再从所有root递归收集focus candidate，为每个candidate克隆group/modal id，按tab或paint order重新排序；方向事件最后再读取`UiTreeNode.layout_cache.frame`线性评分。单次事件为`O(N + F log F)`并产生`O(F)`短命candidate/string storage。control-anchored popup的最终位置只存在于projected hit grid，tree frame仍是placeholder，因此keyboard/gamepad与pointer还可能选择不同空间目标。

Unreal的主参考明确把这两类查询放在同一paint-time authority上。`HittestGrid.cpp:859-934`的`FHittestGrid::AddWidget`读取`GetPaintSpaceGeometry/GetRenderBoundingRect`，在paint时把最终bounding cell和layer sort写入grid；`NavigationMethod.cpp:46-105`从cell的collapsed widgets中过滤enabled/focusable/boundary；`NavigationMethodProximity.cpp:45-210`只沿grid cell查找下一focus widget，不在事件时重新排列全widget tree。Slint `internal/core/item_tree.rs:791-899`则直接沿预编译item focus chain及repeater边界前后移动，也没有为每次Tab构造全量有序Vec。Fyrox的每次Tab descendants collect+sort与Zircon旧路径相近，只适合作为低复杂度对照，不作为目标架构。

本轮新增`UiSurfaceNavigationIndex`，它由Surface rebuild持有并在projected hit authority就绪后构建。索引一次保存所有node的最终frame、focus/navigation语义、局部与全局modal scope rank；focus candidates预编译为base/modal-group/MUI-root tab lists、node position map、同scope spatial lists和manual group首项。几何先读取`UiProjectedHitTestIndex::authoritative_entry`，只有不参与pointer grid的keyboard-only node才回退到arranged frame，因此control-anchored popup不再读取tree placeholder。direct dispatcher和input effect两条产品路径统一调用`UiSurface::next_navigation_target`；`UiRuntimeTreeFocusExt`中的旧`next_navigation_target`、candidate Vec、每事件sort及tree-frame方向评分全部删除，`focus.rs`从约826行降到316行。

复杂度边界必须准确表述。相关layout/input/style/text/visible-range或popup projected geometry变化时，当前索引构建为BTreeMap插入加候选排序，约`O(N log N + F log F)`、常驻`O(N + F)`；focus/checked等纯render dirty且projected grid未变时不重建。Next/Previous通过scope list和position map查询为`O(log F_scope)`，不递归、不clone group、不分配、不排序；方向导航为保持现有“slope优先、distance次优”行为，仍线性访问预构建的当前scope candidate，即`O(F_scope)`，但删除了前置`O(N + F log F)`与全部短命候选。不能把它误报为Unreal cell级方向查询；下一阶段若真实gamepad storm仍受方向扫描限制，应在不改变slope/distance合同的前提下引入scope spatial cells或可证明等价的sector lower bound。

TDD先在现有Runtime focus test引用不存在的`navigation_index_build_generation`，静态得到test reference 2、production definition 0的有效RED。实现后用D:既有受管`zircon_runtime_interface`和`toml`产物作只读依赖，在E:临时wrapper中直接`include!`当前生产`navigation_index.rs`并执行`rustc --test`：2/2通过，分别证明发布hit geometry与tree placeholder相反时Right选择发布位置目标，以及base tab order和modal wrap保持原合同；查询前后build generation均保持不变，临时wrapper/exe随后删除。测试已下沉到`navigation_index/tests.rs`，生产模块保持864行，未继续扩大单文件职责。新增Python源码合同5/5，锁定产品入口、projected geometry、旧tree authority删除、render-only skip及生产模块规模；完整性能源码合同366/366通过。`rustfmt --check`、scoped `git diff --check`通过；critical source manifest加入`frame_hit_test.rs`、`navigation_index.rs`与收缩后的`focus.rs`，实测183/183、无缺失和重复；完整capture-output Pester为47/47、0失败、116.42秒。

本候选仍需managed Runtime focus-navigation全组与Editor product path验证。动态门至少执行1000次Next/Previous、1000次四方向gamepad storm及打开/移动control-anchored popup后的交替pointer/navigation：`ui.navigation_index.build_count`在稳定事件段必须为0，`query_count`与输入事件一一对应，input-to-damage p95不得高于1 ms，RSS不得随事件数增长，pointer与navigation最终target必须一致。相关layout/input rebuild目前仍全量构建索引；下一轮要以changed node、descendant scope inheritance和group/manual-target dependents形成patch set，并证明render-only 1000轮build为0。未取得current-source managed binary和三轮warmup/measured/quiescence数据前，本节只标记static candidate。

### 15.105 Surface frame按域共享，window/focus换代不再复制全部产品

继续沿按钮focus、window metrics与toolbar消费链检查publication，确认顶层`UiSurfaceFrame`虽已用`Arc`原子发布，但frame内部仍持有owned arranged tree、render extract、hit grid、layout report和ECS projection。任一`dirty`或仅`focus_state/window_state`变化都会重建整个frame并深克隆这些字段；toolbar bridge又比较外层frame指针，只要顶层generation变化就遍历全部hit entries、复制control id并比较control Vec。于是window-only状态可把本应常数级的快照更新放大为`O(N + C + H + E)`，focus/button路径在render patch之外额外复制layout/hit，旧frame仍被consumer持有时峰值内存也同时保留两份重域。

本轮把`UiSurfaceFrame`的arranged tree、render extract、hit grid、focus state/path、layout engine report和ECS projection硬切为serde `rc`能力支持的`Arc`字段；Arc序列化仍保持原inner value形状，新增`UiSurfaceFrameDomainGenerations`以serde-default兼容旧快照。`UiSurfaceFramePublication`持有layout/render/hit/focus/ECS脏域集合，首次/full rebuild保守替换全部域，增量rebuild从实际report与projected-hit变化精确标记，纯report变化只换顶层metadata。focus/window状态在publication时独立比较：focus只推进focus/ECS，window只推进window；layout/structure保守推进focus以覆盖reparent后的route变化，未变域通过统一`shared_domain`执行`Arc::clone`。既有clone counter现在只记录真实深复制，不再把句柄复用误计为clone。

Editor toolbar consumer同步改为保存`Weak<UiHitTestGrid>`并比较hit域allocation identity，而不是外层`UiSurfaceFrame`指针。外层frame因focus/window换代但hit域未变时直接返回，不扫描entries、不构造control字符串、不重建私有surface；origin或hit域变化仍进入原重投影和内容等价检查。下层新增三个行为回归源码，分别锁定focus-only publication复用arranged/render/hit并只替换focus/ECS、render-only rebuild只替换render/focus/ECS、window-only publication复用全部重域；toolbar回归另锁定外层generation变化但共享hit Arc时返回false。生产`UiFocusState.changes/focused_inputs`原来只有push、无任何消费/清理，现由`focus.rs`私有`UI_FOCUS_DIAGNOSTIC_HISTORY_CAPACITY=64`统一在追加前裁掉最旧项，也会收敛反序列化的超大旧历史，当前focus状态和返回事件不变。容量属于单owner诊断保留策略，没有提升成跨crate常量。

复杂度边界仍不能夸大。window-only publication的重域复制已由`O(N+C+H+F+E)`降为0，只保留顶层小状态与pipeline report组装；未rebuild的focus publication仍会全量生成ECS projection，约`O(N)`，但focus history clone已有64项硬上界；render-only rebuild虽然不再复制layout/hit，但published render Arc仍来自完整`UiRenderExtract::clone`，约`O(C + payload bytes)`，ECS也为`O(N)`。公共frame serde/Arc接口已用D:受管依赖在E:临时wrapper中直接rustc编译运行exit 0；production publication普通与profiling两个cfg也以当前源直接include编译并运行域指针断言，均exit 0，临时5文件随后删除。最终跨层静态合同5/5、完整性能源码合同371/371、critical source manifest 186/186且无缺失/重复，完整profile-output Pester 47/47通过、0失败、0跳过，耗时74.89秒。下一步先让ECS projection按changed-node、ancestor/relation dependents增量patch，再让render cache发布typed immutable segments并让frame组合segment handles；不能把本轮Arc域共享误报为command segment或ECS delta已经完成。managed Rust尚未运行，真实Editor还需以window-only、1000 click/focus和toolbar hover分别验证clone/reproject counter、CPU p95与RSS。

### 15.106 ECS projection应退出热SurfaceFrame，而不是在错误边界上建立复杂增量缓存

继续追踪15.105剩余的ECS成本后，确认`ui_ecs_projection()`每次先完整扫描render commands与hit entries构造两个`BTreeMap`，再遍历全部`UiTree.nodes`并复制node path、children、component和control id，最后又从全部projection rows聚合totals、schedule impacts与dirty-domain impacts。focus-only publication因此不是简单`O(N)`：当前实现约为`O(C log N + H log N + N log N)`并伴随字符串/Vec分配。全仓调用图同时证明这个projection没有输入、布局、渲染或调度产品消费者；唯一产品消费者是Editor Debug Reflector，其他调用均为测试或显式debug API。为它在热frame里增加persistent chunk/index会保留错误的所有权边界，并给topology、祖先disabled传播和derived rows引入第二套复杂一致性协议。

参考引擎支持把两类产品分开。Unreal `SlateInvalidationRoot.cpp`维护`FinalUpdateList`、widget proxy与cached paint/hittest authority，只处理需要更新或重绘的widget；`Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp`与`WidgetSnapshotService.cpp`则通过显式`WidgetReflector.TakeSnapshot`/snapshot service采集诊断树。Slint同样把item-local `cached_rendering_data`/dirty region用于提交热路，而不是把完整introspection payload塞进每个render frame。Zircon应保留同一代layout/render/hit/focus作为frame权威，但把retained component/path/dirty-domain的ECS DTO降为显式诊断产品。

本轮计划采用硬切而非兼容shim：从`UiSurfaceFrame`和`UiSurfaceFrameDomainGenerations`删除ECS字段，从publication删除ECS dirty、generation、全量构建与clone counter；`UiSurface::debug_snapshot*`在调用时先取得当前frame，再显式调用`ui_ecs_projection()`并传给共同的diagnostics builder。只持有`UiSurfaceFrame`的通用`debug_surface_frame*`仍能生成完全同代的arranged/render/hit/focus诊断，但其`UiSurfaceDebugSnapshot.ecs_projection`为serde-default空值，因为frame本身没有retained tree/component metadata；需要live ECS rows的调用方必须持有`UiSurface`。这不会删除ECS DTO、delta或schedule查询能力，只把构建成本从每次按钮/focus/window publication移到真正请求Widget Reflector/JSON诊断的冷路径。

TDD门先锁定源码负合同：frame/publication不得出现`ecs_projection`、`dirty_domains.ecs`、`ui_ecs_projection()`或`ui.surface_frame.ecs_projection_node_count`；surface-owned四个debug入口必须显式传projection；Runtime行为回归要证明focus变化后的surface debug snapshot仍与`ui_ecs_projection()`一致，而纯frame helper返回空projection且arranged/render/hit/focus保持有效。静态通过后再更新三份现有surface/ECS/diagnostics合同文档与critical source manifest。managed Rust、Editor EXE和真实1000次focus/profile继续等待官方validation lane；在取得`ui.surface_frame.publication_build_count`不再伴随ECS node count、CPU p95/RSS数据前只标记static candidate。

实现按该边界完成：`UiSurfaceFrame`及domain generations删除ECS字段，publication删除对应dirty/generation/build/counter；四个`UiSurface::debug_snapshot*`入口显式组合current frame与一次on-demand projection，纯frame helper保留相同geometry/render/hit/focus构建器但输出default-empty ECS。Runtime下层测试新增focus变化后的live projection断言和frame-only空projection断言；旧frame字段/代次断言与profile counter fixture同步删除。三份现有surface/ECS/diagnostics合同文档已更新，没有保留兼容字段或并行cache。

最终非Cargo证据为新边界源码合同6/6、完整性能源码合同372/372、critical source manifest 189/189且缺失0/重复0、完整profile-output Pester 47/47通过、0失败、0跳过、耗时66.95秒；相关Rust `rustfmt --check --config skip_children=true`与scoped `git diff --check`通过。按静态复杂度，普通focus/window publication删除原ECS projection的`O(C log N + H log N + N log N)`扫描与projection字符串/Vec分配，显式debug capture仍保留这项冷路径成本。managed Runtime test尚未执行，真实Editor也尚无1000 focus/window CPU p95、RSS或counter timeline，因此该项保持`static_candidate`。

### 15.107 Pipeline report成为独立共享域，focus/window换代不再重建诊断String图

15.106移除ECS后继续检查publication剩余的无条件工作，发现每次顶层frame换代仍调用`UiSurfaceRebuildReport::pipeline_report(generation)`。该函数固定构造完整stage Vec，并为Input/Focus/WidgetBehavior/Layout/PostLayout/Picking/A11y/Render/Batch等行分配dirty reasons与owned note Strings；window focus、application active或仅focus state变化都没有执行这些pipeline stage，却仍重复生成相同报告。`UiPipelineFrameReport.frame_index`因此还被顶层状态换代推进，混淆“frame状态版本”和“pipeline报告实际构建版本”。

计划把`UiSurfaceFrame.pipeline_report`硬切为serde-rc兼容`Arc<UiPipelineFrameReport>`，在`UiSurfaceFrameDomainGenerations`增加独立`pipeline`代次。full/dirty rebuild与clean-report metadata变化标记pipeline dirty；纯focus/window变化只换顶层frame/focus/window域并复用report identity。新report的`frame_index`记录它实际构建时的surface generation，之后顶层focus/window generation可继续前进而report identity/index保持不变；消费者用`domain_generations.pipeline`判断报告换代，不再把顶层generation等同于pipeline执行。

TDD先要求公共字段为Arc、publication使用`shared_domain`且不在focus/window路径重建、下层identity回归证明focus/window复用而render rebuild替换；profiling新增`ui.surface_frame.pipeline_stage_build_count`，仅在报告实际构建时记录stage数。profile-output稳定场景把该counter加入零工作门。该切片不改变pipeline stage内容或ordered contract，也不解决render command整域clone；managed Cargo与产品1000 focus/window timeline仍按共享lane授权执行。

实现后`pipeline_report`已成为Arc域并发布独立`pipeline` generation；`mark_surface_frame_rebuild_dirty`与`mark_surface_frame_metadata_dirty`标记该域，focus/window比较不会标记。下层三类frame authority测试分别断言focus-only与window-only复用report identity，render-only rebuild替换identity并推进pipeline generation；diagnostics只在显式debug snapshot冷路径克隆inner report。profile counter evidence已从退役ECS counter改为`pipeline_stage_build_count`，输出和稳定hover拒绝条件同步更新。

最终非Cargo证据：源码合同6/6、完整性能源码合同372/372、PowerShell parser error 0、critical manifest仍为189/189；完整profile-output Pester 47/47通过、0失败、0跳过、耗时52.94秒，并逐个把新counter设为1验证fail closed。production `frame_publication.rs`通过当前源direct-include wrapper的普通与`feature="profiling"`两次rustc编译/运行，focus/window复用、metadata替换与report frame index断言均通过，E:临时wrapper和两个exe随后删除。静态上focus/window换代的pipeline report工作由固定stage Vec和owned reasons/notes重建降为一次Arc clone；managed Rust和真实CPU/RSS/counter timeline仍待官方lane。

### 15.108 GPU UI planner不得为即时paint element构造缓存元数据

沿`UiRenderExtract -> ViewportRenderFrame -> ScreenSpaceUiRenderer`继续审查后确认，当前有两类成本不能混为一谈。SVG/图片资源侧已经具备`usvg::Tree`路径缓存、按尺寸/颜色缓存的`Arc<[u8]>` raster、editor icon atlas，以及WGPU按`resource_key + generation`查询resident并跳过重复像素上传的多层驻留；稳定图标不应再次读文件、parse或上传，剩余风险由现有hit/miss/upload counters在产品profile中验收。命令侧则仍在render变化时由`UiSurfaceFrame`深拷贝完整`UiRenderExtract`，renderer每次提交还遍历全部command并重建vertices/draw/text/image计划，这是后续typed immutable segment方案的真正目标。

在分段改造前先发现一个不依赖新接口的确定性浪费：`plan_screen_space_ui_batches_with_framebuffer_background`是即时GPU planner，却调用`UiRenderCommand::to_paint_elements`缓存版API。该API为paint element写入`cache_generation`与`debug_label`；generation通过`serde_json::to_writer`对整条command做稳定hash，debug label通过`format!("{:?}", kind)`分配String。renderer随后立即消费并丢弃这些paint elements，不会把其缓存元数据作为跨帧authority。实现已切到现有`to_transient_paint_elements`：所有command删除debug label分配，非文本command删除稳定JSON hash；文本路径仍在`push_text_batches`为glyph artifact/cache route identity单独调用一次`command.cache_generation()`，保持现有文本缓存失效语义，下一阶段应由render segment发布显式generation而不是继续序列化整条文本command。

TDD先扩展同一transient extraction源码合同，RED时3项中仅GPU planner用例失败，生产切换后3/3通过；Runtime下层源码断言同步要求planner只调用transient API。相关Rust rustfmt与scoped diff check通过，完整性能源码合同保持372/372；critical source manifest把production `scene_renderer/ui/render.rs`纳入后为190/190、缺失0、重复0。完整profile-output Pester 47/47通过、0失败、0跳过、耗时125.53秒，其中稳定hover逐项拒绝SVG miss、candidate rebuild、GPU image upload/allocation和SurfaceFrame重发布。managed Rust与真实Editor GPU/CPU profile尚未授权，因此本项保持`static_candidate`；产品验收应同时证明稳定SVG miss/upload为0、planner CPU下降，并继续以segment generation消除文本hash和整域command clone。

### 15.109 Versioned damage统计改为generation-owned空间候选查询

15.91确认的RHI热路在current source中仍然存在：`CompiledUiBatchPlanCache::resolve`命中相同generation/projection后，只要是局部damage就调用`draw_list.stats()`扫描全部commands；同时`command_visibility_scan_count`由accumulator固定初始化为1，不能区分访问1行或10,000行。本轮先在`zr_rhi`下层把该字段收敛为真实command predicate访问数：accumulator从0开始，`UiSurfaceDrawList::stats`及WGPU首次full/damage projection都在每次实际检查前递增。full stats快照命中仍为0并显式记录cache hit，因此公开字段现在与注释“visited command rows”一致，不再是pass布尔哨兵。

versioned `BatchDrawPlan`现持有command与compiled draw-op两份不可变`BoundsIndex`，分别保留合并前精确统计身份和合并后GPU提交身份。command索引在generation首次full projection时，从每条command经过surface与clip裁剪后的有效矩形构建；无效/完全裁空行不进入索引，但保留原command index，避免过滤后reindex错误。实现把原overlap planner的interval-tree算法抽成单一通用authority：按较稀疏轴建立center partition，两个crossing order pool各只保存每个有效row一次，常驻内存为`O(C+B)`；damage query沿interval tree剪枝、执行第二轴精确相交过滤，再按原source index排序。查询API为`query_sorted_into`，先清空并复用调用方候选存储；command scratch由compiled-plan cache持有，draw-op scratch由surface renderer持有，返回的`DrawOpSelection`只借用原op slice和index slice，不再构造`Vec<&DrawOp>`。因此generation稳定且容量预热后的局部damage候选查询不产生堆分配；scratch容量按该renderer/cache历史候选高水位保留，最坏仍受对应plan规模约束。cache-hit damage统计只对command候选执行现有`command_visible_with_damage`与`record_visible`，删除了`Some(draw_list.stats())`全量回扫。构建成本仍约为`O((C+B) log(C+B))`且首次full/damage统计仍访问C行；稳定generation的局部command/op查询分别为`O(log C + Kc_axis + Kc log Kc)`与`O(log B + Kb_axis + Kb log Kb)`，最坏大damage允许退化到全量规模。

下层Rust回归先锁定基础stats对2条命令报告2次访问，再用同一generation的10,000条互不相交横行验证：首次build同时派生full与damage统计，因此如实报告20,000次谓词访问和1条当前可见命令；改变到另一单行damage后的cache hit保持plan build 0/cache hit 1、可见命令1且只访问1条候选。第二个回归把无效首行、frame覆盖damage但clip完全移出的中间行与真实可见末行组合，证明索引使用clip后的authority并能从稀疏source index回到正确command。draw-op规模回归使用10,000个不同resource的互不相交image op，固定单行damage只返回1个op；另一个`solid -> text -> image`重叠回归证明candidate refs严格保持compiled painter order。command与draw-op各有一个10,000项复用回归：先预热单行候选，再查询另一单行并断言scratch指针和capacity均不变、候选长度仍为1。独立源码性能合同8/8及当前完整`*performance_contract.py`为380/380，critical source manifest把bounds/dependency/geometry/render-pass以及damage-clear pipeline/WGSL真实依赖纳入并达到196/196、missing 0、duplicate 0；完整profile-output Pester为47/47、0失败、0跳过、131.08秒，相关Rust均通过rustfmt解析、scoped diff/source guard。因共享Runtime/Interface current-source仍有foreign E0382/E0308/E0164阻断，本轮未运行Cargo，Rust回归与WGPU像素回归不能标记动态GREEN。

native production presentation也改为调用plan-aware record入口。无damage/full redraw继续直接回放完整slice；局部damage先用与WGPU一致的整数scissor扩张得到查询矩形，从op index取得按原op index排序的`DrawOpSelection`，再交给原recorder。recorder通过小型`DrawOpSequence`泛型同时消费完整owned op vector和借用selection，没有克隆Image key或Text command-index Vec；solid/image非文字run与text pass边界逻辑保持同一实现。只读审查指出旧逻辑在元素删除或移出damage后会保留retained cache旧像素；现已增加无vertex-buffer的全屏三角形damage-clear pipeline，在首个既有damage pass中受同一scissor约束先写opaque black，再回放候选，不增加额外render pass。空计划/空候选也会创建一次clear pass；下层像素回归先以全红cache为基线，再提交左半区空damage，要求左半区变黑且右半区保持红色。clear物理draw计入`gpu_draw_calls`，但通过内部clear计数从batch merge公式中排除，不会把技术绘制伪装成可见item或material batch。这样关闭了native damage的`O(B)`全op bounds扫描、暖态候选分配和旧残影语义，但submitted stats仍以batch union bounds/item count计数，且text prepare与image source preparation仍有各自缓存/遍历合同；相关text/image owner本轮未修改。下一切片应让text/image准备在generation cache hit时消费兼容的resource/batch候选，并增加明确op-candidate visit counter。产品验收仍要求current-source三轮warmup/measured/quiescence、10k规模、CPU/RSS/input-to-damage/damage-to-submit与SVG parse/raster/upload全链证据，不能把本节静态复杂度修复等同于按钮和窗口缩放已流畅。

### 15.110 Image/Text prepare缓存完整代次产品，外部provider保留显式探测

继续检查damage候选之后的prepare阶段，发现两个稳定代次隐藏线性项。`WgpuUiTextRenderer`虽然按`(draw-list generation, projection size)`缓存glyph batch，但命中时仍遍历全部batch统计有效renderer；现把成功构建的renderer count作为同一cache product字段发布，命中直接读取，稳定文本代次由`O(T)`降为`O(1)`。`WgpuUiImageCache`此前每次普通或damage present都遍历全部`image_upload_sources`，逐项查询resident/shared/source command；现只在无staged资源、draw-list有generation且全部source已经驻留后发布`prepared_generation + prepared_source_count`，下一次相同本地/共享资源代次在进入source loop前返回residency快照，因此稳定SVG/位图GPU准备从`O(R)`降为`O(1)`，不再重复资源解析、command探测或上传判定。

缓存资格不能只看UI generation。`WgpuUiSurfaceExternalImageProvider`目前只暴露`resolve(key, generation)`与提交后的`confirm_resident`，没有provider readiness epoch；同一UI generation内可能先返回`None`并使用CPU fallback，随后发布GPU product。若此时把本地fallback驻留误认为整代完成，会永久跳过后续provider探测。因此存在external provider时明确不复用也不发布整代prepare键，继续执行既有逐项resolve。prepare另用复用容量的source-index scratch记录本帧实际接受的external产品；render成功后只遍历这E项并`confirm_resident`，删除原先再次遍历全部R项、调用`is_resident + provider.resolve`的重复确认，提交确认由`O(R)`收敛到`O(E)`且暖态不分配。只有本地与device-scoped shared registry按不可变`(resource_key, generation)`身份进入整代快路。该边界对应Unreal Slate由resource manager持有generation-stable resource proxy、fast invalidation root复用cached element data，以及Slint以item/resource identity复用renderer cache的共同原则：缓存完整不可变生成物，不把尚无版本的异步就绪状态塞进UI代次。

TDD新增下层helper回归，锁定staged资源拒绝命中、仅完整驻留发布、external provider永不发布或复用代次键；源码性能合同另外锁定image guard位于source loop之前、text命中不扫描batch，以及external confirmation只消费prepare acceptance indices而不重复resolve。focused合同11/11、完整`*performance_contract.py` 383/383、三个Rust文件`rustfmt --check --config skip_children=true`及scoped `git diff --check`均通过。Rust用例尚未执行Cargo，真实Editor也尚无稳定SVG 1000次交互的parse/raster/upload、CPU/RSS或p95数据；本节只能标记`static_candidate`，external provider若要获得整代O(1)必须先扩展独立readiness generation合同。

### 15.111 menu/HUD剩余问题是owned extract边界，不是World全扫

重新核对open `dynamic-ui-extract-generation` handoff后，旧报告的“每次全扫World”已不符合current source。menu与HUD collector调用`dynamic_component_rows`走component sparse storage，World还发布`dynamic_component_generation(component_id)`；真正缺口是稳定capture仍重建少量目标command、文本与style String，并以owned extract跨阶段传递。

当前静态候选已完成共享所有权hard-cut。`RuntimeUiExtractCache`以`(menu generation, 两类HUD generation, viewport size)`为typed key，miss保持menu优先再HUD，hit只克隆`Arc`；无关world mutation不推进目标key，目标component变化和resize各只触发一次新publication。`current_ui_extract`、RenderFramework trait、pipelined queue、WGPU binding、`ViewportRenderFrame`和Editor submission均改为`Option<Arc<UiRenderExtract>>`，submit/build/present只借用或移动同一handle，不再以owned DTO隐藏深拷贝。project UI聚合另按viewport与各surface render generation缓存共享extract；menu input仍直接读取sparse value与算术layout，不反查presentation cache。

Editor侧也关闭了同类回退：viewport HUD以scene-mode revision、projection/display/grid和viewport size缓存唯一Arc；world-space合成以submission generation与基础Arc身份缓存，空world-space集合直接保留基础allocation，非空集合仅在任一key变化时合成一次。该结构与Unreal由invalidation root持有`FSlateCachedElementData`、Slint以`CachedRenderingData`/item generation复用renderer product的边界一致。TDD已写入stable/unrelated/target-change/resize/absent、project aggregate、Editor HUD及world-space Arc identity回归；focused source contract 7/7、相关新增模块rustfmt和scoped diff静态通过。Cargo按用户要求暂缓，1/1k/100k规模、1/60/240Hz、F2像素等价和allocation/CPU计数尚未返回，因此handoff仍保持open，不能标记完整修复。

### 15.112 布局测量改为后序计划与跨帧复用工作区

继续沿窗口resize触发的根布局检查，确认15.100删除几何快照`BTreeMap`后，measurement本身仍在每个节点克隆`children`并分配`child_desired`。Horizontal/Vertical/Block/Wrap/Grid/Masonry随后还会为slot order生成两份临时Vec；Grid与Masonry每个容器另建列宽、行高和计数Vec。因此完整根测量虽然算术复杂度为`O(N)`，堆分配次数却随内部节点数增长，宽容器还会重复申请与child count同阶的短命存储。窗口连续缩放正是这个路径的高频放大器。

本轮把测量改为单次后序计划。`measure/traversal.rs`先只读遍历目标子树，将`(node_id, collapsed)`写入紧凑post-order Vec；`collapsed`向后代传播，所以父节点Collapsed时整棵子树仍清空desired/content/virtual-window，与旧递归`collapse_node_measure`一致。随后按后序逐节点读取已经完成的child desired size并提交父结果，不再克隆children，也不为每个节点递归创建Vec。slot排序先写复用的`ordered_entries`并稳定按`(order, original index)`排序，再写复用的ordered payload；Grid/Masonry的宽高和count数组通过`clear + resize`复用历史容量。

这些buffer由Surface本来就跨帧持有的`UiLayoutSlotIndex`拥有，而不是局部函数或全局线程缓存。产品Surface在拓扑/最大分支不超过历史高水位时，measurement自身的post-order、child payload、slot sort和Grid/Masonry列数组不再发生堆增长；空间上限约为`O(N_subtree + max_degree + max_grid_extent)`。完整resize仍必须访问`O(N)`节点，本轮没有伪造subtree validity，也没有解决arrange阶段的children clone/排序；standalone `compute_layout_tree`每次新建slot index，只有单次pass内复用，不能宣称跨调用零分配。下一结构门仍是mutation-owned slot/order generation与显式layout input/available-size/scale/visibility validity key，之后才能像Fyrox `measure_valid + prev_measure`与Unreal `bNeedsPrepass + PrepassLayoutScaleMultiplier`一样安全跳过稳定子树。

TDD源码合同先得到workspace不存在、递归clone存在、容器Vec存在三类有效RED，实施后focused合同9/9、完整`*performance_contract.py` 387/387通过；六个Rust源文件通过`rustfmt --check --config skip_children=true`，scoped `git diff --check`通过。下层Rust新增Collapsed整树语义回归和10,000 sibling连续两次测量的capacity不增长回归，但因共享current-source编译仍被foreign Runtime/Interface错误阻断，本轮没有执行Cargo，不能把“用例已写入”记成动态GREEN。真实Editor验收仍需记录resize 200步的layout visited nodes、measurement workspace growth、allocator events、CPU p95和RSS；在这些数据返回前本项保持`static_candidate`。

### 15.113 布局安排复用深度缓冲并删除重复排序

15.112完成measurement后，arrangement仍在每个访问节点克隆`children`，而Block/Size/Linear/Wrap/Grid/Masonry以及Taffy入口会分别再次收集并稳定排序同一组child。宽容器因此至少生成一份与degree同阶的临时Vec；递归树还会让短命分配次数随节点数增长。Collapsed/Space/虚拟列表不可见子树则在每一层克隆children后递归清空frame与clip，深树同时承担逐层Vec和调用栈成本。这些都发生在完整resize及被扩大后的layout island上，不是命中事件本身的固定成本，但会直接放大窗体缩放卡顿。

本轮在Surface原有的持久`UiLayoutSlotIndex` workspace内增加按递归深度借用的`UiArrangeChildScratch`池。每个父节点把children写入连续的original view，并在进入容器分派前生成ordered view；两个Vec都保留历史容量，Wrap content-size可以继续读取原序，而实际排列只消费唯一ordered authority。递归child从池中借下一层scratch，返回后立即复用，因此暖态容量约为`O(sum(max_degree_per_active_depth))`，而不是按已访问节点数重新分配。排序改为`sort_unstable_by_key((order, original_index))`：显式original index保留旧稳定排序的同order语义，unstable实现本身不再需要稳定排序的额外scratch。Grid/Masonry/Taffy和各fallback helper直接消费该切片，不再各自重建顺序。

排序上提没有改变所有容器的合同。旧代码对Free、Canvas、Container、Overlay、Block、Size、Horizontal/Vertical、Wrap、Grid和Masonry使用slot order；Space和Scrollable保持tree children顺序，其中Scrollable的原始索引还是virtual-window identity。`container_uses_slot_order`因此是显式白名单，未参与旧排序的容器直接把original view复制为最终view。Wrap还有一条不同合同：排列按slot order，随后content-size重算按原始tree order分行；双视图显式保留该差异。隐藏子树另外改为单个跨帧复用的迭代DFS栈，按reverse push维持旧的前序child访问顺序；成功或`Result::Err`都归还并清空栈，删除每节点children clone且不增加每child的父节点`BTreeMap`重查。

TDD先新增两个源码合同，旧实现为9/11通过、2失败，精确命中缺少depth scratch和下层重复排序；实施后focused合同11/11、完整`*performance_contract.py` 389/389通过。下层Rust另写入10,000 sibling连续两次arrange capacity不增长、同order稳定性、Free遵循slot order、Scrollable保持tree order、异构Wrap排列后仍按原序重算content-size、隐藏链复用单栈及错误返回归还scratch回归；当前共享编译链按用户要求继续冻结，所以这些Rust用例尚未执行，不能记成动态GREEN。scoped Rust均通过`rustfmt --edition 2021 --config skip_children=true`解析，`git diff --check`无空白错误。

这不是完整layout validity或零分配声明。`resolve_linear_child_main_extents`、Scrollable positions、Wrap row/payload、Taffy layout inputs/outcomes以及Masonry column arrays仍有独立临时Vec；完整arrange仍访问`O(N_subtree)`节点，standalone调用每次新建slot index时也不能跨调用保留高水位。下一非重叠切片应把这些容器特有buffer并入同一workspace并分别锁定可见性/fallback语义；跳过稳定子树仍必须等待mutation-owned child/slot order generation与available-size/scale/visibility validity key，不能用当前dirty位猜测缓存有效。

### 15.114 Linear求解与容器专用arrange缓冲进入同一高水位

15.113列出的容器临时存储继续沿实际调用链拆解。Linear旧路径先分配`Vec<AxisConstraint>`，公共`solve_axis_constraints`再分配resolved Vec；growth/shrink会各收集priority Vec，并在每个priority的饱和迭代里反复收集active index Vec；最后layout pass又把resolved映射成main-extents Vec。Scrollable则先完整生成positions Vec再完整消费，两个循环访问同一children；Wrap fallback每次创建row-items Vec，content-size重算另收集child desired Vec；Masonry每次创建column heights/counts两条Vec。它们的算术仍为线性或受priority迭代约束，但堆行为会在连续resize中逐轮重复。

约束求解核心现在提供crate内`solve_axis_constraints_into`，调用方传入resolved、priorities与active-indices缓冲；公开`solve_axis_constraints -> Vec<ResolvedAxisConstraint>`继续是原合同，只用局部buffer调用同一核心，Editor与外部调用方无需迁移。Runtime layout的`UiLinearArrangeScratch`同时持有constraints与三条solver buffer，arrange直接读取`ResolvedAxisConstraint.resolved`，不再生成main-extents Vec。priority排序、去重、growth从高到低、shrink从低到高、权重与max/min饱和循环及最后兜底shrink顺序均未改变；四条复用Vec每轮先clear，exact-fit轮次也不会留下之前的priority/index长度。

Scrollable删除positions Vec但保留旧的两阶段错误合同：父scroll state提交后先按tree order无分配验证全部direct children，任何缺失都在修改child subtree前返回；验证通过后才执行流式position/arrange。消费循环对每个child仍先按`occupies_layout`决定前置gap，保存当前position，再执行visible-window arrange或hide，最后只对占布局节点推进desired extent和placed count，因此Hidden、Collapsed与offscreen的索引/间距语义和旧两遍算法一致。Wrap row-items与按原tree order重算content-size的desired payload进入父层depth scratch；Masonry column height/count改为`clear + resize`。父层scratch在递归child期间不在pool中，child借用下一深度scratch，所以这些可变Vec不会被递归覆盖；所有成功与错误返回最终都由统一recycle清长度并保留容量。

TDD分两步执行：Linear源码门从11/12通过、1失败转为12/12；容器门再从12/13通过、1失败转为13/13。当前完整`*performance_contract.py`为391/391，scoped `rustfmt --check --config skip_children=true`与`git diff --check`通过，旧`let indices: Vec`、axis constraints/main-extents collect、Scrollable positions helper、Wrap局部row Vec与Masonry `vec!`源码扫描均为空。下层Rust新增固定oracle的growth priority/max saturation、zero-weight、ascending shrink、exact-fit与minimum-floor结果，加上owned/retained capacity稳定和256-child Linear四缓冲复用；Scrollable同时覆盖Collapsed间隔流式位置，以及“前child含缺失后代、后direct child缺失”时必须先返回后direct id且前subtree frame不变。Wrap row/desired与Masonry列数组均覆盖二次容量稳定。共享Cargo继续按用户要求冻结，故只记为已写入、未执行。

剩余主要临时分配已收缩到Taffy桥接的`layout_children`、`hidden_children`、`child_inputs`和backend outcome，以及其它不在本切片的公开owned solver调用。完整layout仍为`O(N_subtree)`，current pass也尚无allocator/CPU/RSS动态样本；这项不能表述为“resize零分配”。下一步应先判断Taffy backend是否允许 caller-owned output/scratch，再设计单一桥接workspace，不能为了复用而复制一份Taffy布局树或改变fallback诊断。

### 15.115 增量布局以依赖路径和精确几何输入复用稳定子树

继续审查15.112-15.114后确认，更大的剩余问题不是单个Taffy Vec，而是`compute_incremental_layout_tree_with_text_measure_cache`在选出incremental root后仍先递归收集整个子树、为全部节点建立几何快照、完整measurement，再完整递归arrangement。窗口尺寸变化会把tree root加入layout roots，所以这些前置步骤仍为`O(N)`；`apply_mui_responsive_layout`还会先扫描全部节点发现responsive候选，即使绝大多数节点没有responsive metadata。Unreal Slate的`SWidget::SlatePrepass`只在fast update路径上处理`bNeedsPrepass`，`FWidgetProxy::ProcessLayoutInvalidation`重算当前widget desired size后仅在结果变化时继续失效父级，`FSlateInvalidationRoot::ProcessPrepassUpdate`按失效堆处理并跳过已经包含的widget range。Fyrox `UserInterface::measure_node`以`measure_valid + prev_measure`命中精确available size，`arrange_node`以`arrange_valid + prev_arrange`命中精确final rect。这些证据共同要求缓存完整输入身份和局部失效路径，不能只依据一个surface dirty布尔值跳过布局。

本轮建立单一增量布局访问账本。dirty layout/style/text/visible-range节点沿parent链上推到已经选定的layout root，形成`layout_dependency_paths`；incremental measurement的post-order planner只进入这组路径，未失效child直接读取retained desired size。Collapsed节点仍强制清理整个子树；从Collapsed恢复时，零frame的required container强制重测后代，避免复用此前被清零的desired size。arrangement只有在“节点不属于required路径，且输入frame与最终clip_frame逐值等于缓存”时才能返回；required ancestor即使几何相同也必须继续下降。隐藏/离屏入口使用同一判定，已经为零几何的干净子树不再重复DFS。每次实际写frame/clip前直接把node id加入visited并比较old/new记录geometry delta，删除了layout前的`collect_subtree_nodes`与`snapshot_geometry`全扫描。诊断统计使用measurement与arrangement访问集合的并集；layout engine route替换只消费实际进入arrangement写入/隐藏路径的`layout_engine_route_node_ids`，因此被零frame恢复逻辑测量、但几何精确复用的container不会丢失旧selection。同时保留独立的removed-node账本：若删除节点原来拥有route，快速补丁回退到现有完整合并并剔除失效selection，而不把删除项伪计为实际layout访问。

responsive候选索引同时进入Surface持久的`UiLayoutSlotIndexState`。首次full layout或节点/slot数量变化时构建，之后只对pending mutation/dirty node id执行BTreeSet membership patch；root resize直接遍历已索引的media-query、visibility、container和implicit-grid候选，不再先扫描全部`UiTree.nodes`。responsive值在取得`node_mut`前先以不可变读取比较，未变化候选不会进入mutation ledger。新增责任从接近1000行的`responsive_mui.rs`提取到`responsive_mui/candidates.rs`，保留responsive orchestration与candidate ownership的模块边界。

复杂度必须按实际容器输入表述。全子树前置收集和几何快照由`O(N)`降为0；依赖路径构建约为`O(D*H*log A)`，D为本轮layout dirty节点、H为到layout root高度、A为required路径节点数。measurement访问A个节点，但每个受影响container仍需读取其direct child desired，因此成本是`O(A + sum(degree(affected containers)))`。arrangement访问required路径以及frame/clip实际变化的后代；父容器仍必须计算可能依赖父frame的direct child输入，所以最坏的根resize仍允许退化为`O(N)`，不会伪报常数复杂度。responsive候选发现从每次`O(N)`变为mutation patch `O(D log R)`；无implicit Grid时slot阶段常数返回，有候选时当前仍为`O(R + S)`扫描slot表，后续只有取得slot mutation generation后才能安全维护parent-to-slot index。candidate/required/visited/delta BTreeSet常驻或单轮空间分别受R或A约束。Taffy direct-child bridge临时Vec和backend tree build仍是下一独立热点。

TDD先用新源码门确认production缺少dependency/exact-geometry authority并得到有效RED；实现后focused性能合同14/14、完整`*performance_contract.py` 392/392，相关Rust均通过`rustfmt --check --config skip_children=true`，scoped `git diff --check`无空白错误。下层回归已写入root resize只访问root且不触发深层Text measure、Collapsed恢复重测、same-cardinality responsive metadata patch、未变化responsive不生成mutation，以及已归零隐藏子树不重复访问；本轮仍按共享阻断约束未运行Cargo，所以这些Rust用例只能记为已写入。真实Editor还必须以200步resize和10,000节点记录layout visited/geometry changed、responsive candidate visits、text measure、allocator events、CPU p95与RSS；在这些current-source动态数据返回前本项保持`static_candidate`。

### 15.116 Taffy bridge复用按递归深度持有的backend tree与输出缓冲

15.114留下的Taffy临时所有权逐项确认后，旧路径每次进入一个eligible container都会新建`layout_children`、`hidden_children`、`child_inputs`、Taffy node-id和child-frame五条Vec，并重新构造包含多组SlotMap的`TaffyTree`。这些对象的元素数都与direct child数K同阶；即使连续resize保持同一拓扑，allocator仍会为每个访问到的Taffy container重复增长和释放。Taffy 0.10.1自身提供`TaffyTree::clear()`，其合同是删除节点但保留tree对象；因此可以复用backend arena，而不需要缓存可能过期的layout result。

本轮给Surface持有的layout workspace增加`UiTaffyArrangeScratch`深度池。scratch持有visible/hidden child id、Zircon/Taffy backend node id、输出frame以及唯一`TaffyTree<()>`；首次创建时关闭rounding，之后每轮用`clear()`清节点与长度并保留历史容量。所有child先完整完成Unsupported visibility、axis priority、invalid value、slot canvas/padding/alignment验证，保持旧fallback优先级；第二遍才把短生命周期`UiTreeNode`/`UiSlot`引用逐项转换成Style并立即move进本轮backend leaf，不把引用或完整Style留在跨帧高水位Vec。Grid placement在同一push中更新轻量columns/rows计数，parent track维度仍由同一ordered child序列推导。父scratch在递归child期间不归还，嵌套Taffy container从池中取得下一份；成功、fallback或`UiTreeError`返回后统一清长度并归还，所以没有`RefCell`重入，也不会在错误路径丢失高水位缓冲，tree-build/compute fallback统计仍按实际已建leaf数发布。

复杂度不能写成布局缓存。每次被访问的Taffy container仍创建K个backend leaf、一个parent并执行Taffy compute，CPU保持`O(K + backend solve)`，layout-engine tree-build/node-count诊断也继续报告本轮真实构建。变化的是堆所有权：拓扑/最大分支不超过历史高水位后，上述五条Vec和backend tree arena不再按container pass重新申请；持久空间上界约为`O(sum(max direct children at simultaneously active Taffy depths))`。Grid parent的template row/column Vec仍随track数创建，standalone layout每次新建slot index也不能跨调用复用；这两点不计为已关闭。

TDD新增源码门先精确命中缺少bridge scratch、局部Vec和局部`TaffyTree::new`，实现后focused Taffy合同1/1、完整`*performance_contract.py` 393/393通过，相关Rust均通过`rustfmt --edition 2021 --check --config skip_children=true`，scoped `git diff --check`无空白错误。下层Rust回归已写入256-child连续两次Taffy arrangement，要求frame逐值相同、深度池只有一个scratch且五组容量不增长；共享current-source编译仍受foreign blocker约束，本轮没有运行Cargo，因此该回归只能记为已写入。真实产品仍需在200步resize中采集Taffy tree-build count/node count、allocator events、layout CPU p95与RSS；本节保持`static_candidate`。

### 15.117 向量raster尺寸身份必须在pixel/GPU cache之前收敛

继续沿SVG tree、visual pixel、Chrome command stream、icon atlas与WGPU image cache纵向核对后，确认当前源码并不是“没有GPU图像缓存”。SVG解析树已经由canonical path、文件stamp与source fingerprint持有；visual pixel cache以logical source、target size与tint为key；非atlas像素使用最终`(width,height,RGBA)`内容寻址；presenter-local cache和device-shared registry都以`(resource_key,generation)`复用GPU texture。生产RenderFramework/queue链也只是移动owned `UiRenderExtract`，没有逐层深克隆。剩余可复现的identity漏洞位于这些缓存之前：`raster_size_from_frame`对每个resize物理像素尺寸执行`ceil`，该精确target直接进入pixel key。连续改变一个可伸缩SVG的frame时，tree可命中，但每个新像素尺寸仍会重新raster、产生新content key并上传新GPU texture，直到64 MiB/entry上限开始淘汰。

Unreal的对照边界位于`SlateRHIResourceManager.h:260-276`与`SlateRHIResourceManager.cpp:299-301,365-370`：brush/vector resource由resource manager返回稳定proxy，vector graphics cache与atlas page是renderer持有的资源产品，而不是每次paint重新加载文件。Slint `internal/core/graphics/image/cache.rs:40-87`同样以`ImageCacheKey`持有受内存权重约束的image product，`graphics/image/svg.rs`把解析后的SVG与cache key绑定。Zircon不应复制两者的具体容器，但必须在raster前建立稳定、有限的尺寸identity，并让后续CPU/GPU层继续消费同一个key。

本轮给`RasterTargetSize`增加向上取整的自适应向量桶：最大边不超过32/64/256 px时分别使用2/4/8 px步长，更大目标使用16 px步长，最终仍受4096 px上限约束。current workbench/template在Host painter之前已经把presentation scale乘入node frame；Runtime render-resource路径不再从逻辑frame重算，而是消费brush已有的`resource_state.pixel_size = render_bounds * dpi_scale`，仅在旧状态没有合法pixel size时退回frame。因此两条入口都在bucket前获得物理像素target，不会跨DPI错误共享或在高DPI生成低分辨率Vector。语义Icon、显式`SvgIcon`角色、带`.svg`后缀（含query/fragment）的主Image以及显式`UiRenderResourceKind::Vector`应用该桶；显式Vector和SvgIcon即使URI没有扩展名也不会退回精确尺寸。普通Image及带PNG/JPEG/WebP等明确位图扩展名的Icon不量化，避免把位图放大后再缩小。draw command继续保留原frame和clip，量化只改变采样纹理的像素分辨率；向上取整保证纹理不会低于显示target。

template主`media_source`与icon fallback不再合并为一组、预先猜一次类型。现在主media使用自己的candidate group、target和cache key；只有typed loader返回`Missing`才查询icon组，`Deferred`直接返回而不会并行制造fallback I/O。icon组再按实际语义/扩展名选择向量桶或精确位图target，并持有独立cache identity。由此missing PNG primary + SVG fallback会使用向量桶，missing SVG primary + PNG fallback会保持精确尺寸；missing source的fingerprint仍由既有cache invalidation跟踪，文件随后出现会使主组missing entry失效并恢复主源优先级。missing-icon placeholder使用icon组同一target/key，不创建第三种identity。

对1..512 px方形连续resize，精确identity为512个，自适应桶为64个，减少448个（87.5%）并形成8倍复用；这不是对任意宽高组合宣称固定64个，也不改变visual cache的4096 entry/64 MiB硬上限。单个桶首次仍需要一次异步文件选择、SVG raster与GPU upload；之后同桶paint在候选路径构造前命中pixel cache，RGBA content key、icon atlas slot和device texture保持稳定。SVG tree parse原本已按source缓存，所以本项的主要收益是减少raster、RGBA驻留和upload churn，不能把它写成“所有parse降为0”。

TDD先新增4项源码合同并得到4项有效RED；首轮实现后的独立review发现主图片/fallback声明优先级和54 px旧精确尺寸断言两项Important，补充静态合同后得到2项有效RED。跟进review又发现“实际fallback仍沿用缺失主源target”与Runtime render-resource丢失typed physical pixel size两项Important；两项都先扩展同一focused合同得到有效RED，再按上述single authority修复。修复后focused合同5/5、完整`*performance_contract.py` 398/398，相关Rust文件`rustfmt --check`与scoped `git diff --check`通过；最终限定复审确认Critical 0/Important 0。Rust单元新增普通bitmap target逐值不变、SVG suffix/query识别、语义Icon与显式位图Icon分类、Missing/Deferred区分、missing位图主源到SVG fallback实际加载、分组cache identity、Runtime physical pixel size优先、`SvgIcon`绘制frame与bucket纹理解耦、小尺寸bucket以及512步identity计数；共享Cargo仍按用户要求不启动，故只能记为已写入。动态验收必须在current-source Editor中记录连续resize的`visual_assets_render_svg_raster`、visual cache hit/miss、GPU image upload writes/bytes、resident bytes、CPU p95与RSS；在这些数据返回前本节保持`static_candidate`。

### 15.118 稳定鼠标移动只返回generation内借用命中视图

继续沿workbench命中索引到hover transaction复审后，当前主要成本已不在空间索引查询，而在命中结果物化。`template_node_pointer_hit_with_frame`即使只处理稳定hover，也会复制control/action/binding/dispatch/component/value/edit/commit及table identity等字符串；popup另外重建dispatch/action/value，pane路径再复制pane id，按钮激活还会复制完整hit。窗口层虽然已经在写interaction前比较旧状态，但这些复制发生在比较之前，所以同一目标上的高频pointer move仍会产生堆流量。

Unreal `FHittestGrid`以`FWidgetIndex(grid, widget index)`和`TWeakPtr<SWidget>`恢复widget/bubble path，事件阶段不复制widget语义；Slint `ItemRc`同样以`ItemTree`所有权加`u32 index`定位item，并通过`borrow()`读取当前数据。Zircon对应边界应由`HostPresentationGeneration`持有`ModelRc<TemplatePaneNodeData>`和命中索引，pointer move只返回生命周期受该generation约束的`TemplateNodePointerMoveHit<'generation>`。该视图仅携带借用的control/action/value、枚举化popup kind以及栈上`FrameRect`；普通node不复制按钮binding/component/table字段，popup直接借用投影期已经规范化的menu action或option id。需要跨generation保存和执行callback的press/release路径暂时继续物化现有`TemplateNodePointerHit`，两条合同不得以额外clone互相转换。

空间查询复杂度仍为`O(C)`，C是命中cell候选数；从命中row取得语义引用为`O(1)`，move hit构造的堆分配、字符串clone count与clone bytes必须为0。稳定目标上的hover transaction在首次状态写入后，后续比较必须保持interaction mutation、damage和present为0。RED源码合同先锁定move route不能调用owned hit函数，Rust下层回归再用指针身份证明control/action/value来自generation-owned row，并覆盖普通node、menu popup和option popup；按钮回归必须证明owned payload语义不变。动态验收使用同一current-source Editor执行1000次同目标move和1/1k/100k节点规模，记录命中候选访问、owned hit materialization、allocator events/bytes、pointer CPU p50/p95/p99、damage/present和进程RSS。Rust与真实profile未完成前，本节只允许标记为`static_candidate`。

实施后，workbench候选选择由唯一`WorkbenchTemplateHitTarget` authority完成；move route投影为借用hit，button route仍投影为owned hit。普通move不再读取或复制binding/component/table语义，menu/option直接借用generation-owned row；primary press还先计算damage再把owned hit move进callback，删除了第二次完整hit clone。源码合同先以borrowed类型、move route和下层回归缺失得到3项有效RED，再以primary press二次clone得到1项有效RED；修复后focused 5/5、完整`test_*performance_contract.py` 403/403、scoped rustfmt与diff check通过。下层Rust回归已写入node/menu/option字符串指针身份及borrowed/owned frame和语义等价，但按用户约束未启动Cargo；1000次真实pointer storm的allocator、CPU、damage/present和RSS证据仍未取得，因此状态保持`static_candidate`。

### 15.119 Pane pointer route必须借用同一presentation generation

15.118只关闭了workbench窗口的owned hit物化，docked/floating pane仍通过`PanePointerRoute`在每次move/click/scroll中拥有目标。`AssetTree`、`AssetContent`、`AssetReference`、`Viewport`与`ViewportToolbar`把静态模式、surface key和control id复制成`String`；template pane先物化包含十余个字符串的`TemplateNodePointerHit`，move实际只读取control id与frame。asset回调随后又对route中的mode/list kind执行一次clone，因此稳定move会先为route分配，再为callback ABI分配。`FrameRect::clone()`只是四个`f32`的栈复制，不是该热点的堆根因；不能用删clone语法冒充性能修复。

参考实现支持把route身份绑定到当前generation，而不是增加第二份字符串缓存。Unreal `FHittestGrid`以stable sparse-array index和weak widget恢复`FWidgetData`，`FWidgetPath`只持有widget/geometry path；Slint的`ItemRc`由`ItemTree`所有权加`u32 index`定位item，`MouseInputState`保存`ItemWeak` path并在分发时借用item；Fyrox消息只把代际`Handle<UiNode>`作为destination。Zircon当前pane route从一次同步的`HostPresentationGeneration`开始并在事件函数返回前完成，不需要跨帧保存route，因此最小且足够深的落点不是新建pane-id resolver，而是`PanePointerRoute<'generation>`/`PanePointerTarget<'generation>`借用`generation.structure()`中的pane、surface和template row。若未来需要跨generation capture，再单独引入带generation校验的稳定handle；本轮不为不存在的长期存储增加索引和解析失败面。

实现必须保留唯一几何/排序authority。floating occlusion、dock顺序、viewport toolbar、asset panel、popup和template node仍由同一`route_pointer_to_pane_with_mode`调用树裁决，move/click/scroll只改变允许的语义投影，不复制geometry算法。asset surface/list使用小型typed enum，只有进入现有`SharedString` callback ABI时才转换一次；viewport surface key和toolbar control id直接借用presentation；template target返回覆盖完整激活语义的borrowed hit，普通node/table row/popup都借用generation-owned数据。menu action规范化及需要跨调用保存的command/callback payload只在activation boundary物化一次owned `TemplateNodePointerHit`，move/scroll不得调用owned builder。所有route引用必须在持有generation的同步dispatch栈内结束，编译期生命周期即stale-generation保护，不把borrowed target写入interaction state。

复杂度保持与现有命中结构一致：floating窗口最坏`O(F)`、dock选择`O(1)`、asset metadata lookup按现有缓存、template hit为`O(C)` cell候选或可见console候选。变化的是每事件所有权：route构造的mode/surface/control/template字符串clone与堆分配从`O(S)`降为0，S为命中payload字符串总字节；asset callback仍允许一次ABI要求的`O(mode + list)`物化，button activation仍允许一次完整owned payload，稳定template move/scroll必须为0。持久内存不新增cache、index或generation pin；栈上route只携带借用引用、typed tag、坐标和`FrameRect`。

实施顺序先写focused source RED，要求pane route/target带generation lifetime、target不含`SharedString`或owned `TemplateNodePointerHit`、move/click/scroll继续共用同一route authority、owned template payload只在activation boundary出现。下层Rust回归覆盖docked与floating route、occlusion、asset tree/content/reference、viewport/toolbar、普通template node、table row、menu/option popup和非零console scroll；借用字段用pointer identity证明来自generation，owned activation与旧语义逐字段等价，旧generation释放后route无法逸出由类型系统保证。静态GREEN要求focused合同、完整performance-contract、scoped rustfmt/diff及no-owned-route source guard通过。动态验收必须使用current-source Editor做1000次同目标move、1000次scroll和代表性click，报告route owned materialization、allocator events/bytes、pointer CPU p50/p95/p99、damage/present、interaction generation与RSS；在managed Rust和产品profile返回前只能标记`static_candidate`。

本轮实现已把`PanePointerRoute`/`PanePointerTarget`改为generation-borrowed类型；asset surface与reference list改为Copy枚举，viewport/toolbar直接借用surface/control，docked/floating template route使用完整的`TemplateNodePointerRouteHit`。popup与普通节点共享同一target/geometry选择，旧的popup owned全扫描入口删除；move只读取borrowed control/frame，scroll不物化template payload，primary press通过唯一一次`to_owned_hit()`进入原callback协议。asset与viewport现有`SharedString` callback ABI未扩散进route，字符串只在实际调用该ABI的最后一层生成。

TDD证据为focused pane-route合同先得到6项预期失败，实施后扩展为8/8通过；下层回归以非零18 px interaction scroll命中generation-owned `line-2`，对pane/control/action/binding/dispatch/value六个字段执行`std::ptr::eq`，另覆盖docked viewport surface key、floating window id、table row identity及menu/option popup借用身份，并验证menu规范化和全部activation owned payload语义等价。完整`test_*performance_contract.py`为416/416，相关Rust通过`rustfmt --edition 2021 --check --config skip_children=true`，scoped `git diff --check`无空白错误，no-owned-route/source guard无命中。当前没有启动Cargo，也没有current-source EXE的1000次move/scroll profile；因此只确认热路径所有权由每事件`O(S)` clone收敛为route构造0堆分配，产品CPU、allocator、damage/present和RSS仍待managed Rust及真实Editor动态验收，状态保持`static_candidate`。

### 15.120 滚动后的arranged/hit派生几何必须按精确clip子树修补

继续核对Hierarchy、generic virtual list、arranged tree、hit grid与render cache后，需要先纠正“列表已经虚拟化”的范围。当前Editor Hierarchy不再为全部逻辑行创建模板control：template reconciliation限制为10个authored controls，native hierarchy renderer只绘制可见range，pointer route通过行高直接进行`O(1)`索引计算。该产品路径已经有窗口化边界。Runtime通用`UiVirtualListConfig`则仍把全部logical children保留在tree中，measure/arrange仍遍历所有child，只把窗口外节点发布为零frame；因此它是几何窗口化，不是真正的实例虚拟化。行为测试已hard cut为`retained_virtual_list_only_arranges_visible_window`，只声明arranged geometry结果，不再以“materializes”冒充实例数证据。

本轮关闭的是其后的确定性放大。旧`set_scroll_offset`除layout/hit/render/visible-range外还设置input和legacy state dirty，使局部layout分支失去资格；arranged geometry patch遇到任何clip ancestor就拒绝；hit build丢弃零面积entry，导致行进入/离开viewport时entry拓扑变化；rebuild又用原layout dirty集合而不是实际clip影响集合同步各派生authority。一个scroll offset变化因此会在layout之后再次全量构建arranged tree和hit grid。

实现后的唯一几何集合由`patch_arranged_tree_geometry`返回：changed clipping node展开其descendants，逐项验证非几何字段不变后才替换frame、clip和slot；任何缺失或语义漂移仍fail closed到全量authority。`UiHitTestIndex`为所有符合pointer语义的节点保留稳定entry，零面积entry只拥有空cell membership；geometry patch可以在现有grid范围内激活、移动或停用cell，不改变entry index与route identity。`rebuild_dirty`把arranged返回的精确集合传给base/projected hit、navigation frame patch及report accounting；scroll offset不再伪造input dirtiness。下层回归覆盖clip parent变化时返回完整受影响子树、稳定entry从不可见到可见再退出时cell membership变化，以及surface滚动后frame/instance hit authority同时命中新可见行且arranged/hit没有全索引fallback。

复杂度边界必须保持保守。设最终clip影响子树为G，局部arranged/hit派生更新由全表`O(N + H)`收敛为`O(G + affected cells)`；但上游通用virtual-list layout仍遍历全部retained children，不能写成滚动总成本`O(visible rows)`。稳定零面积entry以持久内存换取拓扑稳定：16,384节点、12,288 pointer entries的压力模型保留12,224个inactive entries，按声明的64 bytes/entry只估算payload为782,336 bytes；这不是Rust `size_of`、allocator footprint或RSS。更重要的是render extract的flat command topology仍在行进入/离开零frame时可能改变command count，当前`UiSurfaceRenderCache::patch_nodes`对此fail closed到full extract；该问题必须作为后续独立render-topology slice处理，不能把本轮arranged/hit GREEN扩张成“滚动不再全量重建”。

TDD源码合同先锁定dirty domain、精确affected-set返回、稳定zero-area hit entry、rebuild集合传递和压力模型，当前为5/5；相邻scroll/hit/navigation/projected-order focused组为29/29。完整Runtime performance-contract discovery为422/423，唯一失败是独立的旧`runtime_session_linked_module_registration_m0`源码断言仍要求退役的`linked_extensions.registry.modules()`循环；architecture mirror仍只有既有9项surface inventory漂移，未因本轮hit geometry子模块增加。Rust lower regressions已经写入，scoped Rustfmt、Python compilation、压力产物重生成与targeted diff check均通过。共享Cargo和current-source Editor profile仍未获本轮授权，因此状态保持`static_candidate`。`E:\zircon-profiles\runtime-ui-scroll-geometry-patch-pressure-20260827.json`（SHA-256 `6c0a42e0478b6815b61e0aac379a26e44e630478add8fae99c5ac12604a87801`）模型中，4,096次更新、16,384节点、64节点affected subtree把post-layout arranged/hit work从117,440,512降至524,288，比例224.0x；模型明确排除layout child iteration、variable render-command patch、true instance virtualization、CPU、RSS、allocator与cache效果。

### 15.121 通用列表必须把逻辑数据与有界row实例分离

M2.46之后重新纵向审查`UiVirtualListConfig`、measure/arrange、Table/TreeView window metadata、Editor template virtual rows与Asset Browser projection，确认当前Runtime没有通用repeater/materializer authority。`UiVirtualListConfig`只有`item_extent`和`overscan`，无法表达logical count、stable item key、row prototype或rebind；measurement仍接收全部retained child的desired size，`arrange_scrollable_children`仍先验证全部child再逐项arrange/hide。Table/TreeView能发布`viewport_start/count`与`requested_start/count`，Editor也有独立的row clone/prune和materialized budget，但这些上层实现没有共同拥有slot生命周期、逻辑身份与派生索引同步。因此现状的内存为`O(L*T)`，scroll arrangement至少为`O(L)`；L是logical item count，T是一个row template的retained node数。M2.46只把后续arranged/hit修补收敛到精确clip子树，不能改变这个上游复杂度。

Unreal `SListView.h:982-1080,1524-1700`提供主参考：`FWidgetGenerator`只维护当前生成item到row widget的映射，每个generation pass把未再次看见的row reset/release；`ReGenerateItems`从scroll offset开始，仅生成到viewport填满并在末尾必要时backfill。Slint `internal/core/model/repeater.rs:118-151,176-359,646-675`提供Rust侧独立佐证：持久state只保存model offset、平均item height、anchor和前一viewport；`update_visible_instances`按viewport splice实例、确保对应model row已更新并清理窗口外实例，scrollbar extent由cached height乘logical row count得到。两者共同要求逻辑model与UI实例分离，而不是为全部数据先建立widget/tree node再把窗口外geometry设零。

Runtime目标设计分四个authority。第一，serialized `UiVirtualListConfig`继续只描述fixed-extent几何和overscan，不把model或callback塞入可序列化layout类型。第二，`UiSurface`持有non-serialized materialization state，以owner node为key记录materialization generation、logical count、requested window、稳定slot node ID、slot当前logical index与stable logical key；外部model仍是数据唯一authority。第三，reconcile入口接收当前model/window/prototype并只对进入窗口的slot执行rebind；窗口重叠slot保持node/render/hit identity，离开slot在确认没有capture/IME/edit transaction后循环复用，否则该次回收fail closed。第四，所有row事件、selection、focus与accessibility item identity通过`logical key + materialization generation`解析，不能把可回收slot node ID持久化成数据身份。

普通fixed-extent scroll的目标复杂度为：window计算`O(1)`，slot映射与最坏大跳rebind `O(V)`，小步滚动rebind `O(delta)`或在首版保守为`O(V)`，row subtree常驻内存`O(V*T)`，layout/render/hit更新`O(changed slots*T)`；外部model保持其自身`O(L)`数据内存。V为`ceil(viewport/item_extent)+2*overscan`并受logical count clamp。viewport resize只允许把pool调整到新的V，model shrink必须清理越界逻辑映射。普通scroll必须保持row subtree和每slot command topology稳定，使render cache只更新重绑slot payload/geometry；flat command splice仅服务真正的prototype/structure变化，不能作为虚拟滚动主路径。variable-height列表以后使用Slint式cached height或prefix extent index单独设计，不能让fixed-extent首版偷偷扫描全部L项。

实施顺序保持bottom-up。M2.47a先新增纯window/slot planner及lower RED，覆盖logical counts 1/100/10,000/100,000在相同viewport下得到相同最大slot数、一步scroll只重绑边界slot、大跳不超过V次rebind、model shrink、stable key与generation。M2.47b再建立surface-owned materialization state和prototype subtree slot pool，覆盖capture/focus/IME回收保护、node pool容量、layout frame、projected hit、navigation、accessibility和render command-count稳定。M2.47c迁移一个真实Editor列表并删除其私有row clone/prune机制，随后迁移Table/TreeView window metadata到同一authority。旧retained-child geometry window合同必须改名，迁移完成后才考虑hard cut。静态合同、压力模型和Rust lower tests不能替代managed Cargo与current-source产品数据；动态验收要求在1/1k/100k logical rows上分别采集live node/slot count、rebind count、layout/render/hit访问数、full extract fallback、allocator events/bytes、CPU p50/p95/p99、RSS与GPU upload，且这些量除外部model内存外必须对L近似不敏感。

M2.47a已经建立纯planner但没有提前伪造surface集成。`UiVirtualListSlotMap`以`logical index mod slot capacity`保持物理slot identity，实际实现使用逆映射从slot直接求logical index，因此每次reconcile只遍历`max(previous V,next V)`，不遍历logical range；调用方提供并复用change Vec。相同状态不推进generation；稳定capacity下一行滚动只重绑一个边界slot；大跳最多V项；model shrink先报告被删除slot再截断。首轮source RED因production文件不存在得到3项预期错误，基础GREEN为3/3；加入压力模型RED后因工具模块不存在得到预期import error。复审随后发现整数viewport在fractional scroll offset下会同时出现两个partial boundary rows，旧capacity少1；扩展RED准确命中Rust helper与压力模型后，capacity统一增加一个boundary slot。最终focused合同4/4，Runtime UI性能合同116/116；完整Runtime性能合同238/239，唯一失败是无关的旧`runtime_session_linked_module_registration_m0`字符串断言。六个Rust lower cases已经写入但未运行；新Rust模块rustfmt、Python compilation及targeted diff check通过。共享`layout/mod.rs`存在本轮之前的外部re-export排序漂移，因此没有为格式门机械重排该文件。此前委托的`event_routing`与window/IME共7条编译错误形态在current source中的定向守卫计数为0，相关外部修改未被本轮重复覆盖；这里不据此声称Cargo通过。

`E:\zircon-profiles\runtime-ui-virtual-list-slot-materialization-pressure-20260827.json`（SHA-256 `65e7dd25fb9e1d85ca642fc38043a9594f0b3d20fdaffd5e4694aa6adc2279c1`）建模100,000 logical items、每row六个retained nodes、800 px viewport、24 px fixed extent、overscan 3、4,096次scroll（含64次large seek）。旧retained-child形态为600,000 row nodes和409,600,000 direct-child visits；bounded target为41 slots/246 row nodes（含一个fractional boundary slot）和167,936 planner visits，两项声明计数均为2,439.02x差异，rebind上界6,656。该JSON明确写入`surface_materializer_wired=false`、`layout_render_hit_reduction_measured=false`、`cpu_or_rss_measured=false`；当前完成的是正确且有界的slot算法，不是UI tree、render cache或Editor产品性能修复。下一步必须先补slot absolute-content position与stable logical key，再进入M2.47b，不能直接把外部未跟踪的Asset Browser slot实现覆盖掉。

M2.47b已经把planner接入`UiSurface`，但仍按验收边界称为static candidate。non-serialized materialization index以owner为key保存logical count、stable item key、generation、物理slot root及完整subtree反查；每次回收先检查focus、capture、pressed、pointer capture/lock/high precision、IME、drag与drop owner，命中受保护节点时整次reconcile回滚。layout slot index独立发布logical count与物理root到logical index的映射，fixed-extent arrange直接以logical index计算绝对内容位置，只访问V个物理root。prototype pool首次捕获一棵完整authored subtree blueprint，按V复制完整node/slot拓扑；容量缩小时整棵detach到有界`UiSurfaceNodePool`，再次增长优先复用，稳定容量不创建节点且不改变slot root identity。lower Rust源码回归覆盖100与100,000 logical rows拥有相同物理tree尺寸、完整subtree/slot拓扑、稳定root、六节点缩放复用、stable/key-only generation、protected rollback、descendant lookup以及layout visits对L不敏感；共享Cargo冻结期间这些测试尚未执行。

M2.47c首个Editor产品迁移选择Inspector component properties。模板把`WorkbenchInspectorMeshProperties`定义为fixed-extent虚拟scroll owner，只保留一个authored row；Editor删除旧retained tree inventory、max node-id扫描、按L clone/prune和静态control枚举，转为Runtime stable-key reconcile及prototype pool。bridge缓存当前不可变property snapshot及预计算stable-key数组；snapshot更新仍检查V个slot，滚动只读取缓存key，并从`UiVirtualListMaterializationChange`解析Delta个变化slot、只重写这些row metadata。此前componentized pointer route会把完整`UiPointerEvent`降级成kind/point而丢失`scroll_delta`，导致嵌套Runtime scroll owner无法移动；现在`route_pointer_input_event`保留button/delta，`apply_default_pointer_scroll`与dispatcher fallback共享唯一nested-scroll实现，Editor把scroll layout与Delta rebind合并进现有一次pointer refresh。真实Windows pane callback也已经从旧`ScrollSurfaceHostState -> no-op setter`路径切换：它把inspector-local坐标还原到mounted workbench surface，路由同一个Runtime scroll authority，提交pending host-projection patch，仅在retained surface没有滚动时才使用旧状态fallback。下层Rust测试验证完整event delta及统一scroll authority，产品Rust测试以64 properties滚动真实owner并要求slot 0 logical binding/value同时切换；测试已写入但未获Cargo执行授权。

focused源码合同当前20/20，scoped Rustfmt与diff guard通过。`E:\zircon-profiles\editor-ui-virtual-row-prototype-slot-pool-pressure-20260827.json`（SHA-256 `20a733a01947b4af6196aeea018ac99ae02b9b5c4ac4f9682438514fae868fdf`）对4,096次reconcile、16,384 surface nodes、V=41、Delta=1建模：退役的两次全tree inventory为134,217,728 node visits；候选为167,936 bounded slot checks及4,096 metadata rebinds，tree-visit/slot-visit比799.22x，并避免163,840次未变化slot metadata写入。该结果只证明复杂度和声明操作数，不是CPU、allocator、RSS、GPU upload或交互延迟实测。当前还不能宣称M2.47完成：必须等待managed Runtime/Editor回归、current-source EXE profile、render command topology/full-extract fallback与accessibility identity证据，并继续迁移Table/TreeView/Asset Browser。

### 15.122 可回收row的逻辑身份必须拥有逐槽代次

M2.47b复审发现已有`UiVirtualListNodeBinding`虽然同时携带owner、logical index和128位stable item key，但只暴露owner级materialization generation。若辅助技术、延迟提交或其它跨publication消费者把owner generation作为token，一步滚动只重绑一个边界槽也会让其余V-1个未变化row全部失效；若完全不带generation，又会让旧物理node id在slot回收后误指向新逻辑项。两种做法都不满足“稳定逻辑身份 + stale action拒绝”的合同。

当前实现把identity DTO拆到`virtual_list_materialization/identity.rs`，区分稳定的`UiVirtualListItemIdentity(owner_id, logical_index, item_key)`与当前物理`UiVirtualListNodeBinding`。owner继续维护单调generation，但只把当次generation写入`UiVirtualListMaterializationChange`实际覆盖的active slot；窗口重叠且未重绑的slot逐字段保持原binding。`virtual_list_binding_is_current(node_id, binding)`通过现有`node -> slot`反查做有界当前性校验，不分配、不扫描logical model；protected rebind失败时candidate generation同slot/key映射一起回滚。lower Rust源码回归覆盖变化槽拒绝旧identity、未变化槽保留assignment generation，key-only rebind同样推进目标槽代次。

这一步没有伪装成AccessKit产品闭环。当前neutral accessibility snapshot与AccessKit adapter仍以物理`UiNodeId`为主键，OS adapter、不可变a11y frame generation和旧publication action路由尚未接入；新token只是为这些消费者提供下层正确性原语。Inspector编辑回调当前同步读取当代metadata，focused/IME/capture节点又由materializer禁止回收，因此没有为了表面“使用新字段”复制第二份Editor状态。Table/TreeView复核也表明它们目前只发布`viewport_start/count`与`requested_start/count`并操作已有row节点，没有prototype/model-to-slot实例owner，不能在state reducer层声明已迁移。Asset Browser已有独立的bounded slot和算术pointer authority，paint仅访问materialized window，但其slot binding仍缺共享stable key/assignment generation；后续迁移应替换该身份层，而不是退回全量节点。

focused source-contract先以identity模块、逐槽generation和stale guard缺失得到预期RED，实施后虚拟列表组合为21/21，新增identity组4/4；新Rust文件与materializer的scoped rustfmt、Python compile和targeted diff check通过。Cargo仍按共享lane约束未启动，因此Rust lower cases未执行。`E:\zircon-profiles\runtime-ui-virtual-list-logical-identity-pressure-20260827.json`（SHA-256 `21aae3f49c5a56cb6af28233392443256fbc0b6e759d5727399ad909d17bb72c`）对V=41、4,096次scroll和64次large seek建模：owner级全量token刷新为167,936，逐变化槽刷新上界6,656，保留161,280个未变化token，声明刷新次数比例25.23x。JSON明确标记`runtime_cpu_measured=false`、`allocator_or_rss_measured=false`、`accesskit_adapter_wired=false`；M2.47仍需managed Rust、current-source产品profile、a11y publication接线和Table/TreeView/Asset Browser真实迁移后才能完成。

### 15.123 Asset Browser有界槽池还必须保持相交窗口的物理身份

继续检查Asset Browser的真实paint/projector路径后，确认它虽然已把逻辑资产与materialized card/row数量分离，但旧binding仍以`start_row * columns + slot_index`解释每个物理槽。窗口只向下移动一行时，V个slot的logical item、selection、text、preview identity全部同时变化；有界实例数因此没有转化为有界的增量变化。这个问题不会增加UiTree节点数，却会让下游相等性检查、render command/resource identity和未来逐槽generation看见V次rebind，是“节点已虚拟化但滚动仍抖动全部槽”的结构性残留。

当前候选把逻辑行稳定映射到`logical_row mod materialized_row_count`的物理行，列仍由`slot_index mod columns`保持；`y_offset`改为逻辑行与物理行的相对差，因此物理节点不用重排也能落到最终内容坐标。窗口尾部按logical row count回填完整materialized row pool，避免深滚动到末尾只剩最后一行；若外部构造传入非整行且仍可滚动的槽池，则显式回退到连续映射，优先保证不漏逻辑项。实际Asset Browser先按保守上限创建slot，再在最终grid width确定后以真实列数裁剪，正常大数据产品路径仍进入整行取模快路。

TDD源码合同先在缺少row-modulo helper、相对slot geometry和bottom backfill时得到预期RED，实施后4/4通过；Rust lower cases覆盖一步滚动只替换进入物理行、列不漂移、bottom backfill和非整行正确性回退，两个既存projector产品用例改为验证新逻辑项复用离开窗口的首个物理行并保持item-specific child geometry。相关Rust路径的scoped rustfmt与targeted diff check通过；Cargo仍未获授权，因此这些Rust用例尚未执行。`E:\zircon-profiles\editor-asset-browser-slot-reuse-pressure-20260827.json`（SHA-256 `cbc7eb76a07d876ae5e75154d708afbb905b7cde81372964dc28bc6802a75f3f`）对54槽、6列、4,096次滚动（含64次large seek）建模：旧窗口相对binding产生221,184次slot rebind，行取模为27,648次，保留193,536次不变binding，声明rebind次数比例8.0x。该JSON明确排除item projection clone、equality check、allocator、CPU、RSS、render command patch、texture upload和GPU工作；本节关闭的是Asset Browser assignment churn，不等于共享Runtime stable key/逐槽generation已迁移，也不等于产品交互延迟已实测。

### 15.124 Accessibility全量快照内部也不能重复遍历同一树结构

继续检查辅助技术路径后发现，即使当前还没有把virtual-list逐槽identity接入AccessKit，单次neutral snapshot本身也存在确定性超线性工作。旧extract在relation-target discovery和node build两遍循环中分别沿每个node的parent链计算effective hidden，深树为`O(N*h)`；`filter_children`又对全部tree nodes逐个递归其excluded descendants，只有递归结束后才判断该owner是否实际发布到a11y map，深链最坏为`O(N^2)`；hidden relation target在`nodes.into_values()`后再对Vec逐项`find`，为`O(K*A)`。这些都不是增量publication缺失本身，而是一次全量publication内部的重复结构访问。

当前候选新增`EffectiveHiddenIndex`：从roots迭代传播ancestor hidden状态，合法tree的每条parent/child edge只访问一次；orphan/cycle防御路径使用memoized parent resolution，不会改变正常tree语义。构建接收budget deadline callback并在每个结构节点推进时检查，保留`max_processing_time_micros`的中断合同；bounded dynamic API在进入extract前已经验证`source_nodes <= max_items`，因此O(N)辅助map有现成输入上界。child flatten只从实际发布的a11y node出发，遇到下一个included node即停止递归；hidden relation target则在BTreeMap阶段以key定位并清空children/actions，再转换为snapshot Vec。secure text input的value/name/action过滤等当前共享修复保持不变。

源码TDD在visibility模块、included-node traversal和map-stage hidden-target清理不存在时得到3项预期失败，实施及deadline复审后4/4通过；新增Rust lower case覆盖hidden ancestor向后代传播，相关Rust的scoped rustfmt与targeted diff check通过，Cargo仍未运行。`E:\zircon-profiles\runtime-ui-accessibility-extraction-pressure-20260827.json`（SHA-256 `e8e1a365c03ebaf5c091501bb848b25fd2216b6fba77511c816eaa89d7663d02`）以16,384节点、仅root发布的合法深链建模：两遍ancestor walk加全owner child flatten的结构edge visits从402,628,608降为32,766，声明比例12,288x；另记录128个hidden target在8,192 a11y nodes上的旧Vec probe上界1,048,576与新128次map lookup，但不把BTree内部比较数混为同一单位。模型明确排除relation/name resolution、serialization、budget accounting、allocation、CPU、RSS、AccessKit、OS流量和增量publication；当前只关闭full snapshot内部的超线性结构遍历，逐dirty-domain a11y publication及recycled-row logical identity仍是后续验收项。

### 15.125 稳定tooltip hover不得先构造owned候选和祖先路由

针对“按钮仍然跟手性很差”继续复核真实pointer-move主链后，先排除了一个容易误判的方向。`HostMenuPointerBridge`当前只在顶层菜单切换、submenu path变化或popup开关等拓扑变化时调用`rebuild_surface()`；普通稳定move不重建，大popup scroll还通过算术row projection保持固定surface authority，已有10,000项回归锁定scroll前后authority generation不变。因此菜单surface拓扑重建仍需后续局部patch，但它不是每个鼠标像素都发生的全量工作。

真正位于每条非touch move上的额外路径是workbench tooltip观察。旧`icon_tooltip_target_at_point`先调用`bubble_route()`构造祖先`Vec`，命中metadata后又把`label`和`control_id`各复制成owned `String`，最后才与上一候选比较。即使指针稳定停留在同一图标且最终`changed=false`，这些分配和route元素写入仍然重复。当前实现改为`IconTooltipTargetRef`借用metadata字符串，先逐字段与stored owned candidate比较，只有候选确实变化才`into_owned`；祖先查找直接沿`UiTreeNode.parent`从top hit向root遍历，不再物化bubble route。候选变化路径也先借用candidate更新input manager，最后把owned candidate直接移入state，不再为state额外clone两条String。tooltip内容、owner、delay及由control id派生的tooltip id语义不变。

intro timer原来也会在每次tick克隆owned candidate的两条String并重新格式化tooltip id；现在tick临时取得candidate所有权，以borrowed target执行visible/show判定，在传播任何show/transition错误前恢复candidate，并通过固定前缀`strip_prefix`比较id。只有真正首次显示时才为surface metadata复制label/control id。源码TDD锁定borrowed candidate、ownership顺序、metadata零复制、无bubble-route Vec、tick无candidate clone/format五组不变量，RED准确命中后与相邻pointer feedback batch合同合计9/9通过；Rust文件通过scoped rustfmt，相关路径通过diff check。`E:\zircon-profiles\editor-workbench-icon-tooltip-candidate-pressure-20260827.json`（SHA-256 `1f4d38298b2b32eb45757938baf76f0c86a8d93ae655ecfbcca0535b4a69da23`）对65,536次tooltip-bearing move、64次候选切换、平均祖先深度6和8次intro tick建模：旧路径的String/format/Vec分配点执行为196,824，新路径为192，避免196,632次，声明比例1,025.12x；同时消除393,216次bubble-route元素写入。该模型明确不测allocator实现、hit-grid查询、parent读取、CPU、input latency、RSS、paint或GPU。

事件泵仍有一个需要独立设计的未闭环点：每个原生`PointerMoved`目前都同步执行tooltip surface hit、native workbench hit和结果damage合并，没有“每次present前只消费最新可合并位置”的latest-value mailbox。高采样率鼠标可能因此形成输入队列，即使单事件已经有界。不能简单丢弃move，因为capture、range drag、tab drag、viewport高精度输入、输入sequence/outcome诊断和button/scroll顺序都要求不同策略。下一步应先将move分类为可合并hover与不可合并capture/drag/high-precision两类；前者在event batch边界保留最新physical position并记录被合并sequence区间，后者保持逐事件或按累计delta处理。该修改必须由当前event-loop owner完成lower input-order回归，并用真实1000 Hz/8000 Hz输入的queue depth、event-to-damage p95/p99和damage-to-submit验证，不能用本节allocation模型代替。

### 15.126 Module Plugins业务projection缓存不能代替host节点generation缓存

历史产品profile中`convert_pane_module_plugins`两次样本的p95约为2,251.14 ms；该目录早于current source，只能用来定位结构热点，不能作为本轮性能结论。current source已经有app侧`ModulePluginPaneProjectionCache`，它以不可变`EditorPluginStatusReport Arc` identity避免重复执行status report到`ModulePluginsPaneViewData`的业务投影；但每次host pane转换仍先对全部插件生成row/title/meta/features/diagnostics及最多六个button节点，再用`map_model_rc`遍历全部插件并复制二十余个`SharedString`字段。稳定业务generation因此仍是`O(P * node/action fanout)`，app cache并没有覆盖这层派生。

本轮先hard cut删除没有任何生产消费者的host `ModulePluginStatusData`及`ModulePluginsPaneData.plugins`镜像；renderer、hit test、geometry profile与shell patch的唯一消费authority都是`module_plugins.nodes`，不再为无用副本执行第二遍插件扫描和二十余字段字符串复制。随后新增独立的`ModulePluginsPaneProjectionCache`，由`RetainedEditorHost`持有并沿full scene与shell-content patch的同一conversion调用链传递。有效性同时要求pane id、模板retained document identity、是否使用模板、content width/height bits、源plugins `ModelRc`存储identity和diagnostics相同；命中在任何template projection或row generation之前返回最终`ModulePluginsPaneData`的共享node模型clone和一份diagnostics字符串。新的插件`ModelRc`、模板reload generation、尺寸或diagnostics变化都会miss并重建，内容相同但新模型存储也不会被错误当成同一publication。cache最多驻留8个pane entry并以使用序列淘汰；稳定命中没有插件行扫描，只有有界map lookup、identity/string比较、`ModelRc` clone和diagnostics clone。

这项修复把稳定重复apply的host转换从每次`O(P * fanout)`收敛为`O(log 8 + diagnostics bytes)`，真正的source/template generation变化仍正确支付一次`O(P * fanout)`。profile stream新增`host_projection_cache_hit_count`、`host_projection_cache_miss_count`和`host_projection_source_row_count`，后续产品采集必须同时证明稳定交互hit增长、miss不增长且source rows为0，不能只比较总span。它没有把插件列表实例虚拟化：首次打开、1,000插件更新或尺寸bucket变化仍会建立全部行节点，常驻host模型仍为`O(P * fanout)`；后续必须把该面板迁移到15.121的Runtime有界slot materializer，并让滚动只重绑进入窗口的物理行。不能以本节cache命中替代首帧、滚动或内存验收。

TDD source合同先因cache文件、cache入口和pane接线缺失得到2项error与1项failure；dead mirror hard-cut合同随后准确命中旧type/field/mapping。实现后与相邻pane projection合同合计10/10、tooltip/pointer相邻组40/40通过。两条待managed执行的Rust lower回归分别要求同一source generation复用nodes模型存储、内容相同但新plugin模型存储使nodes失效；相关Rust路径通过scoped rustfmt与targeted diff check。全量Editor performance-contract discovery执行471项，其中464项通过，7项外部漂移失败分别来自已删除的global interaction函数名、3个缺失material test fixture、scene paint调用形态、UiAsset text ownership旧断言和viewport snap旧返回形态；这些路径不在本候选中，本轮没有越权修改。`E:\zircon-profiles\editor-module-plugins-host-projection-cache-pressure-20260827.json`（SHA-256 `a72a365e28922be9d895c91c9907a935151057a98b5c26a08dbf05c158af5380`）对1,000插件、4,096次presentation apply和64次projection generation变化建模：旧路径plugin row与status mapping合计8,192,000次source row reads，候选为65,000，避免8,127,000次，声明比例126.03x；稳定cache lookup的source row reads为0。JSON明确排除真实template projection、host node count、allocator、CPU、RSS和产品延迟；Cargo与current-source Editor profile仍冻结，因此状态保持`static_candidate`。

### 15.127 鼠标移动当前绕过batch并同步支付两套命中查询

沿真实Winit产品入口复核后，15.125里把latest-value mailbox描述为“event batch边界”还不够精确。`UiWindowInputPumpBatch::push_coalesced`当前只去重相邻redraw request；Editor的每个`WindowEvent::PointerMoved`会立即translate、分配sequence并同步进入`handle_pointer_moved`，并未先进入该batch。handler随后先调用`invoke_workbench_pointer_input`，在Runtime template surface的published hit grid上查tooltip top hit并沿parent链查metadata；接着调用`dispatch_native_pointer_move`，capture不存在时又在`HostPresentationGeneration.workbench_hit_index`上查询一次published workbench hit，更新native hover并合并damage。因此普通idle hover每个原生move确定支付两次空间查询及两套状态比较，现有batch redraw去重对这段CPU和输入队列没有作用。

两套查询目前不能直接视为重复结果。tooltip authority需要`UiNodeId`、parent链、tooltip label/control id/delay等Runtime metadata；native move authority返回借用presentation generation的control/action/value/frame语义视图，没有Runtime node id或tooltip metadata。capture入口还会在查询workbench index前优先处理native resize和tab drag，而viewport高精度输入、range drag、pointer sequence/outcome诊断及button/scroll顺序分别有不同合同。把所有move简单替换为最后一个绝对坐标会丢失累计delta、拖拽中间态或每sequence outcome，不能作为修复。

实现顺序应拆为两个可独立验收的层。第一层在publication时把可复用的tooltip semantic target纳入workbench hit product，或发布从presentation hit identity到Runtime node/tooltip metadata的稳定有界索引；单次idle hit同时驱动native hover与tooltip候选，禁止事件热路径再回查另一棵树。该product必须随template/hit generation一起失效，popup stack、clip、scale和control ancestor tooltip继承必须与Runtime hit authority等价；capture提前返回时不得伪造hover target。第二层才在Winit event-loop owner增加latest idle-hover mailbox：仅合并连续、同window/device/pointer、mouse source、无capture/resize/tab drag/range/high-precision状态的Move；任何button、scroll、enter/leave、keyboard、IME、resize、scale或不可合并move都是顺序屏障，并在屏障前及`about_to_wait`刷新最新位置。被覆盖sequence必须发布明确的coalesced outcome/range，而不是静默消失；capture与相对运动若要压缩，只能累计delta并逐合同另行实现。

第一层current-source候选已经按该边界实现。surface host projection现在发布`Option<UiNodeId>`与由共享`workbench_icon_tooltip_text`规则计算的tooltip eligibility；legacy/non-surface projection显式发布`None/false`，不会伪造Runtime identity。`HostWorkbenchHitIndex`的bucket membership扩展为`dispatchable || (surface identity && tooltip eligible)`，但press query仍把`is_dispatchable`作为独立predicate，move hit另携带`dispatchable`位；因此disabled显式tooltip能复用同一次几何查询，却不会变成native hover或click target。native move返回同一次borrowed hit的`surface_node_id`，capture/resize/tab drag及非workbench路径返回`None`；Winit handler先完成native query再把该标量交给tooltip callback。tooltip bridge只沿给定Runtime node的parent链读取metadata，不再调用`surface.hit_test`。componentized direct pointer路径也先生成唯一`UiPointerRoute`，再复用`route.target`，capture时显式拒绝tooltip target。

源码TDD先得到publication、hover/press predicate与single-hit接线三类预期RED，实施后新增合同4/4、相邻tooltip/native pointer合同合计38/38通过。lower Rust另写入tooltip-only disabled节点move命中返回原始`UiNodeId`而press命中为None、eligibility变化使索引membership失效的回归；相关Rust除一个未由本轮改动的既存import顺序漂移文件外均通过scoped rustfmt解析，targeted diff check无空白错误。全量Editor performance-contract discovery为475项，其中468项通过，7项外部漂移仍是缺失global interaction旧函数名、3个material fixture、scene paint调用形态、UiAsset text ownership旧断言和viewport snap旧返回形态；本候选没有新增失败。`E:\zircon-profiles\editor-workbench-pointer-single-hit-pressure-20260827.json`（SHA-256 `4d41d33a2628f544b1cd84fd5e729379ac3a80277fca3a0ebe1913d5b40b43e8`）对65,536次idle move建模：旧tooltip surface与workbench index合计131,072次空间查询，候选为65,536次，避免65,536次，声明比例2.0x。模型明确标记CPU、allocator/RSS、input latency与GPU均未测；该artifact生成时第二层尚未实现。后续静态类型复审还发现回调调用已经扩为`(UiPointerInputEvent, Option<UiNodeId>)`，但`UiHostCallbacks.workbench_pointer_input`仍存储单参数`Callback1`；新增合同先RED，存储改为对应`Callback2`后恢复GREEN，避免Cargo恢复时形成确定性arity编译错误。

第二层current-source候选现已实现为Winit event-loop自有的固定单槽`UiIdlePointerMoveMailbox`，不建立无界sequence或event容器。只有primary `PointerSource::Mouse`、没有已按下鼠标按钮、没有已武装tab drag且没有native resize capture时才把move放入mailbox；mailbox保留`device_id`，设备切换先flush旧设备位置，绝不跨设备覆盖。新move以`Option::replace`覆盖同设备旧move并为旧sequence发布独立`Coalesced`终态。未知、touch和tablet来源以及任何按键/capture态move继续逐事件即时处理。所有非可合并window event先flush再translate当前事件，因此`Move A, Move B, Button`严格按`B -> Button`进入产品handler；scroll、enter/leave、keyboard、IME、resize、scale、focus和redraw同样是屏障。`about_to_wait`与proxy wake也会flush尾部move，保证damage先于present/redraw调度可见。profiling新增`ui.idle_hover.received_move_count`、`coalesced_move_count`、`dispatched_move_count`和有界`ui.input.outcome.coalesced_first_sequence/last_sequence/count`，被覆盖输入不再从诊断账本静默消失。

TDD先让single-slot、屏障顺序、wait/proxy flush、capture bypass和coalesced outcome五组合同全部RED；继续收敛观测与metadata分配后，mailbox focused源码合同为9/9，mailbox replacement和coalesced range另有待Cargo执行的Rust lower case。完整Editor性能源码合同最近一次执行484项，477项通过，仍只有既存7项外部漂移：一个旧global interaction函数名、三个缺失material fixture、scene paint调用旧形态、UiAsset text ownership旧断言和viewport snap旧返回形态，本候选没有新增失败。相关Rust通过scoped rustfmt parser，targeted diff check无空白错误；D盘受管产物池未在限定时间内找到可复用`winit` rlib，故没有把依赖源码签名核对冒充Rust编译。`E:\zircon-profiles\editor-idle-pointer-move-mailbox-pressure-20260827.json`（v3，SHA-256 `32af5e7bc0b1569ec9fa740488161dde9f73ad921d7533a6d255fb723353a305`）以65,536次idle move、256个event batch和4,096次capture move建模：capture move仍全部4,096次分发，idle分发为256、合并65,280次，总空间查询由single-hit路径69,632降为4,352，声明模型比例16.0x；相对最初double-hit路径139,264为32.0x。`idle_event_batch_count`是显式模型输入，JSON明确标记CPU、allocator/RSS、input latency与GPU均未测。

动态验收仍必须使用current-source Editor EXE记录1,000 Hz与8,000 Hz的received/coalesced/dispatched move、最大queue depth、event-to-hit、event-to-damage、damage-to-submit p50/p95/p99、CPU与RSS，并单独验证resize/tab drag、窗口失焦及viewport轨迹。最坏不可合并输入仍为`O(E * query)`，单槽只把连续idle hover收敛到`O(B * query)`，其中B为event-loop flush batch数；它不会降低一次实际dispatch内部的布局、hit、paint或GPU成本。Cargo与产品运行继续冻结，所以两层都只记为`static_candidate`，不能标记完整交互性能GREEN。

### 15.128 并行审查收口与current-source阻断复核

三项独立只读审查均已对current source完成归并。分段UI提交链已经在Runtime的`PublicRuntimeFrame`、动态session、RenderFramework、WGPU和Editor viewport边界统一为共享`Arc<UiRenderSubmission>`；扁平producer只在明确边界用`UiRenderSubmission::single`包装，稳定project aggregate按surface render generation复用已发布submission。focused source contract 7/7通过，因此不再修改这些共享owner，也不把测试曾经指向旧`Arc<UiRenderExtract>`形态的历史结果写成当前缺陷。render cache审查确认现有`Weak<PreparedScreenSpaceUi>`保留策略没有新增正确性或稳定代次缓存问题，本轮不更换所有权模型。

导航局部几何审查曾指出一个真实风险：仅按节点自身navigation语义判断patch authority时，被其它候选引用的modal root可能在frame改变后漏更新。current source已经显式维护`referenced_modal_root_node_ids`，rebuild时把group root登记为几何authority，geometry patch同时接受本地navigation authority与外部引用modal root；对应lower regression `externally_referenced_modal_root_patches_frames_but_rebuilds_for_order_changes`也已存在。focused性能合同5/5通过，证明非候选跳过、候选frame patch、projected geometry过滤及外部modal root回归均被源码合同覆盖。该修复属于共享工作树现有变更，本线程不重复编辑或声明所有权。

UI12转交的7条current-source编译诊断也已逐项静态复核，而不是按旧行号重复修补。`event_routing.rs`的pointer dispatch现借用route，navigation dispatch在消费前预取`route_target`与`route_kind`，不再发生两条E0382；IME clause测试直接匹配已经normalize后的`UiInputEvent`，adapter合同在pattern中借用`preedit_clauses`并在guard调用`is_empty()`，不再把`Vec::new()`写进pattern，对应E0308与四条E0164均已从源码结构消除。由于Cargo lane仍冻结，这里只能报告current-source source assertion与rustfmt/parser结果，不能宣称共享27-error编译指纹已经动态清零。

mailbox再审确认其诊断终态不会进入`UiInputOutcomeTracker.active`或`pending_present`，因此覆盖旧move不会中断正在处理的输入或伪造damage batch；mouse button计数在button handler完成后更新，后续move进入不可合并路径，focus loss重置计数；不同device先flush旧位置，所有非合并WindowEvent、proxy wake与`about_to_wait`都是顺序屏障。仍未覆盖的产品风险是窗口系统是否在极端capture/focus组合下遗漏release事件，以及8 kHz输入下真实winit queue深度；这些只能由授权后的Windows产品测试裁决，不再用额外静态模型外推。

### 15.129 mailbox观测不能在8 kHz capture中制造逐事件全局锁

对第二层候选继续审查profiling实现后发现，最初版本虽然把hit与dispatch合并到flush边界，却在每个deferred move上立即记录`received_move_count`，每次覆盖又分别记录`coalesced_move_count`和`coalesced_sequence`。Runtime recorder的单counter入口会获取全局recorder锁，并为静态stream/name各构造owned `String`；因此capture开启时，一个包含M个move的mailbox epoch仍产生`M + 2*(M-1) + 1 = 3M-1`次锁与counter allocation。该成本只污染测量构建，但恰好会放大本来要测的1/8 kHz输入，是不能忽略的observer effect；非profiling构建因cfg裁剪没有这组recorder写入。

当前实现把同一epoch的观测也变成固定大小product。mailbox在最后一个move的无分配metadata/device/position快照之外，只维护`received_count`和连续`UiCoalescedInputRange { first_sequence, last_sequence, count }`；每次覆盖只更新标量，不写recorder。flush使用一次`record_counter_batch`同时发布received/coalesced/dispatched三项累计值；若存在覆盖，再在同一锁内发布coalesced first/last/count。sequence与timestamp仍在原生事件到达时分配；window metrics和modifier只在屏障前flush时读取，任何能改变它们的非move事件都会先flush，因此不会跨状态边界取错上下文。range要求连续且count等于`last-first+1`，没有把被合并输入静默删除，也没有建立随事件数增长的Vec。

采集端`Get-ZirconTypedInputOutcomeEvidence`现原生消费该range，不把它展开成逐sequence对象。parser要求三项counter相邻且同timestamp，验证范围单调、计数精确，并把range与damaged/quiet/rejected单项统一排序，任何range-range或range-single重叠都会使`typed_input_outcome_complete=false`；`input_outcome_count`包含range内真实输入数，并新增`coalesced_input_outcome_count`。合法range与错误count/重叠两条Pester回归连同既有延迟证据为9/9，mailbox源码合同升级为7/7；相关Rust rustfmt、PowerShell AST与targeted diff均通过。

v3压力artifact在65,536 idle move、256 flush的模型下，将旧逐事件recorder lock从196,352降为256，结构比例767.0x；counter样本从196,352降为1,536，比例127.83x。该模型假设256个batch均至少发生一次覆盖，并明确不测真实锁等待、String allocator、CPU或输入延迟；它只能证明capture的写入次数边界从`O(E)`收敛为`O(B)`。source-bound critical manifest补入mailbox、input metadata reservation、window metadata owner及`UiWindowId(String)`类型合同，并删除一个既存重复的workbench pointer路径，直接审计为243条、243 unique、0 missing，四个本层关键owner各1次。Cargo与Editor产品profile仍未授权，因此本节仍是`static_candidate`，动态验收必须确认counter ring不再因8 kHz move本身overwrite，并同时报告recorder lock wait和真实CPU。

### 15.130 mailbox不能在覆盖前先构造owned窗口标识

继续沿defer热路径检查allocation后发现，原实现虽然只dispatch最后一个move，却在每个原始move上先调用`translate_platform_input_event`。该调用通过`native_input_metadata`执行`UiWindowId::new("editor.main")`；`UiWindowId`当前由`String`承载，因此被下一次move立即覆盖的`PlatformInputTranslation`仍会先完成一次堆分配和完整Winit规范化。空间查询已从`O(E)`降为`O(B)`，但窗口标识分配仍是`O(E)`，会在8 kHz idle hover上制造与最终dispatch数无关的allocator压力。

current-source候选把该边界改为两阶段metadata。事件到达时`reserve_input_metadata`只生成`UiInputTimestamp`、`UiInputSequence`及默认标量字段，`window_id`保持`None`；单槽保存最后一个metadata、`device_id`和physical position，覆盖只读取被替换metadata的sequence形成有界range。flush才由`translate_reserved_pointer_move_event`附加一次稳定window id、重建同语义的primary mouse `PointerMoved`，并进入共享Winit translator。arrival timestamp与sequence不变；window metrics和modifiers不会跨越状态变化，因为所有其它window event、proxy wake和`about_to_wait`都是先flush的屏障。即时capture、drag和resize路径仍使用原逐事件translator，不改变精度或顺序。

新增源码合同先以缺少reservation、defer仍调用translator及压力模型无allocation字段得到3个RED，实施后mailbox focused合同9/9通过；metadata lower test固定“reserve时window id为空，attach后timestamp/sequence不变”，待Cargo授权执行。v3 artifact对65,536次idle move、256个flush建模：deferred window-id分配与完整Winit平台规范化均由65,536降为256，各避免65,280次，结构比例均为256.0x；这些数字是源码结构模型，不是allocator、CPU或RSS实测。四个本层Rust文件通过rustfmt check，Python脚本通过`py_compile`，targeted diff check无空白错误；完整profile输出合同48/48通过，Editor性能源码合同484项中477项通过且仍仅有7项既存外部漂移。动态验收仍需用current-source EXE确认allocator调用、CPU、RSS与event-to-hit p95/p99，当前只记为`static_candidate`。

### 15.131 Hierarchy full reflow不得同时维护有序身份索引和重复展示字符串

对Scene Hierarchy重新沿source generation、retained bridge、native model、paint和pointer路径逐层核对后，早期“每个逻辑行仍物化一个UiTreeNode”的结论已经过期：`scene_tree_rows.rs`只保留10个authored control；native hierarchy paint只访问visible range，为`O(V)`；pointer按row stride直接求logical index，为`O(1)`。完整逻辑集合由`SceneEntries`中的共享`Arc<[WorldInspectionHierarchyRow]>`承载，full reflow另外发布一份跨所有Hierarchy pane共享的`ModelRc<SceneNodeData>`。这部分N行展示模型是当前产品合同所需的一次线性publication，不能通过继续加cache假装消失。

剩余放大器位于`SceneHierarchyProjectionState`：旧实现为了按entity处理稀疏消息，又把全部行收集到`BTreeMap<EntityId, SceneHierarchyRowState>`，并在每个row state内复制`display_name: String`。因此一次必要的`O(N)`reflow被额外放大为有序索引的`O(N log N)`模型工作与N个重复字符串；selection-only patch还会从projection再次克隆name并重建id/depth。Unreal `SListView.h:982-1173`用`TMap<Item, Widget>`维护身份到当前实例的直接查找，`:1524-1688`从scroll offset开始只生成到viewport填满；Slint `repeater.rs:118-330`把model row offset、可见instance集合和`ensure_updated(row)`分开。两者共同说明，身份查找、逻辑源和可见/展示实例应分层，identity index不应再次拥有完整展示payload。

current candidate把`rows_by_entity`改为`HashMap<EntityId, SceneHierarchyRowState>`，row state只保存`row_index/parent/depth`，不再拥有名称。该map从不参与迭代顺序，只用于entity membership、topology一致性和row index查找，因此随机迭代序不会改变可见顺序；presentation顺序仍由authoritative row slice和`ModelRc`控制。逻辑delta进一步拆成两类：真实rename/content row携带一个`SceneHierarchyLogicalRowContent` replacement；纯selection delta只携带row index与最终selected。`patch_presented_hierarchy_rows`在selection-only路径从同一shared generation取现有`SceneNodeData`，保留id/name/depth，只更新selected；content与selection同实体时，按row index合并后的content replacement携带最终selection，不丢rename。稀疏patch仍为`O(Delta log Delta)`的有序publication map，完整identity index期望构建为`O(N)`，没有在事件/paint热路径增加全量扫描。

压力模型`E:\zircon-profiles\editor-scene-hierarchy-generation-index-pressure-20260827.json`（schema 1，SHA-256 `62A4B9181F7D579482B0856299D08AE7086A8D9D6F145CEFE8076654B5FEBD50`）以100,000行、平均24 UTF-8 bytes名称和65,536次selection-only patch计算：有序索引的`ceil(log2(N+1))`工作单元模型为1,700,000，Hash identity模型为100,000，比例17.0x；projection避免100,000个名称和2,400,000 payload bytes，selection-only避免65,536次名称复制和1,572,864 payload bytes。该模型没有测BTree真实比较次数、Hash碰撞、String header/capacity、allocator、CPU、RSS、输入延迟或GPU，不能当作产品性能实测。

TDD先得到3项源码合同RED和1项模型GREEN，实施后focused合同5/5、既有Hierarchy authority合同8/8、profile-output Pester 48/48通过；critical source manifest扩展为248/248 unique、missing 0，PowerShell AST、Python `py_compile`、Rust rustfmt与scoped diff check均通过。完整Editor性能合同执行489项、482项通过；余下4 errors/3 failures仍是同组外部漂移：全局interaction旧函数、3个Material fixture缺失、Scene paint旧调用断言、UI Asset text ownership旧断言和viewport snap旧断言。本层Rust新增selection-only保留id/name/depth回归，但未获Cargo授权执行；真实验收必须用managed current-source Editor分别测1/10k/100k full reflow及连续Delta=1 selection/rename，记录identity-index build、presentation row build、名称/SharedString分配、CPU p50/p95/p99、峰值RSS和旧generation滞留。当前状态仍为`static_candidate`，不宣称按钮或窗口缩放整体达标。

### 15.132 可见行绘制不得先克隆完整ModelRc行

继续从Hierarchy展示模型沿原生paint向下复核时，确认另一个确定的稳态放大器：`draw_hierarchy_rows`虽然已经用clip与scroll算术把遍历限制在V个可见行，却通过`ModelRc::row_data(index)`按值取得`SceneNodeData`。该类型包含`id/name`两个owned String，因此每次稳定paint都会先复制两段文本，再以引用交给row painter；菜单栏、popup、Welcome recent projects、asset tree frame、debug overlay、page overflow和template popup也存在相同模式。这里的问题不是全量N遍历，而是有界V读取仍产生与paint次数成正比的row/String clone；对频繁hover、damage repaint和resize freeze snapshot复用都会形成不必要的allocator噪声。

`ModelRc`本身已经提供`get`和`iter`借用API，因此本轮没有新增缓存或第二套生命周期。15个paint owner改为直接借用稳定row；Hierarchy inline rename与row draw继续消费同一引用，diagnostics与viewport直接迭代model。template menu/option popup仍需把最终label/shortcut复制进owned `HostPaintCommand`，但不再先克隆整行再复制一次文本。submenu只在进入下一层时克隆其有界children model handle，保持跨循环层级的所有权边界。Unreal `SListView.h:982-1173`把generated widget和visible item实例保存在generator中并按当前实例读取；Slint `repeater.rs:118-330`只更新可见instance并复用未变化实例。本轮借用读取与这两者的共同原则一致：稳定实例读取不产生新的展示对象。

TDD源码合同先在Hierarchy、native pane、menu/Welcome和template popup四组得到预期4项RED；生产修改后focused合同5/5通过。压力artifact `E:\zircon-profiles\editor-borrowed-model-paint-pressure-20260827.json`（schema 1，SHA-256 `F684821D93FAFC6EE284A4FEB82287FE2FF19B1164F9C913F3A90085AD7AC139`）以10,000次稳定paint、64个可见row、每行2个owned文本字段、平均24 UTF-8 bytes建模：退役路径为640,000次row clone、1,280,000次文本字段clone和30,720,000 payload bytes，借用读取模型为0。该数字不包含String header/capacity、allocator metadata、command-owned文本、CPU、RSS、输入延迟或GPU，不能当作真实收益。

Inspector P1结论也随current source复核修正：其主产品路径已由Runtime fixed-extent virtual owner、一个authored prototype和Delta slot rebind掌握scroll/instance authority；旧`set_inspector_scroll_px`空setter只存在于Runtime owner未处理事件时的fallback兼容链，不应再据此断言主路径按全部P项常驻。后续应删除或显式隔离该fallback，并用managed产品测试验证大量properties、active edit pin和pixels/hit/field-id一致性。critical source manifest本轮加入15个borrowed-paint owner，由248扩为263，实际审计263/263 unique、missing 0；focused与相邻组合Python合同21/21、profile-output Pester 48/48、15个Rust owner的`skip_children` rustfmt、Python compile与targeted diff check均通过。完整Editor性能源码合同494项中487项通过，仍只有既存7项外部漂移；Cargo按共享授权冻结，故Rust编译与current-source Editor allocator/CPU/RSS/p95尚未执行，本节保持`static_candidate`。

### 15.133 Segmented options paint不得物化owned中间集合

借用paint审查继续进入segmented control后，发现它没有`row_data`单点问题，而是更完整的中间集合物化：`segmented_options`每次paint遍历`node.options`，逐项克隆`String`并收集`Vec<SharedString>`；`push_segmented_control`随后只读该Vec计算均分geometry、selection和label，label进入`HostPaintCommand`时又生成最终owned String。这里K通常很小，不是N级全量重建，但在hover/damage/resize重复paint中仍会制造每控件每帧一次Vec allocation和K次无必要String clone。

current candidate把options投影改为`impl Iterator<Item=&str>`，直接借用`ModelRc::iter()`并过滤空项。commands先通过同一iterator取得非空`option_count`，body再遍历一次借用项进行geometry与paint；最终display label仍由command拥有，生命周期边界没有外泄。保留空项过滤意味着由一遍owned materialization变为两遍borrowed traversal，这是明确的时间/分配权衡。Unreal `SUniformGridPanel.cpp:48-72`的arrange直接按`Children[ChildIndex]`读取slot，不先复制child集合；Slint `item_rendering.rs:234,285`通过borrowed item-tree visitor遍历实例。本轮采用同一原则，但没有假设“两遍一定更快”，动态门仍要比较实际K和allocator成本。

源码合同先得到iterator、command dispatch和segment body三项预期RED，实施后4/4通过；与相邻borrowed-paint合同组合为12/12。压力artifact `E:\zircon-profiles\editor-segmented-options-paint-pressure-20260827.json`（schema 1，SHA-256 `6D6A17CC549536458F4FB9329E88537B7B725440BF9A4C44D8335AB9C2F1C7E8`）以10,000次paint、3个option、平均8 UTF-8 bytes建模：避免10,000次Vec allocation、30,000次String clone和240,000 payload bytes，同时增加30,000次borrowed option visit。模型排除最终command label、iterator overhead、String header/capacity、allocator metadata、CPU、RSS、输入延迟和GPU；它只说明复杂度/操作数交换。critical manifest再加入options/commands/body三个owner，实际审计266/266 unique、missing 0；profile-output Pester 48/48、四个相关Rust文件的`skip_children` rustfmt、Python compile与targeted diff check通过。完整Editor性能源码合同498项中491项通过，仍仅有既存7项外部漂移。Cargo冻结期间没有Rust编译或产品profile，真实收益必须由managed current-source产品allocator/CPU/RSS/p95裁决。

### 15.134 Render增量patch仍在frame publication退化为全命令复制

2026-08-27重新沿`UiSurface` rebuild到`UiSurfaceFrame`的current-source所有权检查，确认15.65/15.92留下的render publication问题仍然存在，但边界已经比旧报告更窄。跨域共享已正确完成：`UiSurfaceFrame.render_extract`是`Arc<UiRenderExtract>`，focus/window-only publication会复用该Arc。真正未闭环的是render域本身发生变化时，`UiSurfaceFramePublication::surface_frame`仍执行`self.render_extract.clone()`；profiling也把`next_frame.render_extract.list.commands.len()`完整记为`ui.surface_frame.render_command_clone_count`。Surface内的per-node range patch即使只改一个control，发布边界仍复制完整`UiRenderList.commands`。每条`UiRenderCommand`又拥有resolved style、可选resolved text layout、`String`和image引用，因此该clone不是只复制固定宽度标量。

直接把`UiSurface::render_extract`改成`Arc<UiRenderExtract>`并在patch处使用`Arc::make_mut`不是修复。上一个`UiSurfaceFrame`仍被consumer持有时strong count大于1，下一代第一次局部写会在mutation边界复制同一完整command Vec；复杂度和payload bytes与当前publication clone相同，只改变计时位置。把segments放进普通`Vec<Arc<Segment>>`后每代复制整张segment handle表，也会留下`O(R/S)`提交成本。正确目标必须同时解决payload和目录两层：

1. rebuild-owned render cache把稳定painter order切成固定上限segment，初始候选为64 commands；segment持有稳定identity/generation、bounds/clip/layer/order摘要，以及分离的geometry metadata和共享typed payload/resource handle。content变化只替换受影响segment和payload，geometry-only变化复用text/image payload identity。
2. `UiRenderFrame`持有持久有序目录根而不是flat command Vec或每代重建的flat Arc表；初始fanout为32，局部替换只path-copy受影响leaf到root。插入、删除、跨segment reorder或segment密度失衡必须有typed fallback/compaction reason，不能静默全量flatten。
3. `UiSurfaceFrame`、Runtime/Editor frame、RenderFramework和one-slot queue只传`Arc<UiRenderFrame>`；publication只增加handle引用。`UiRenderExtract`保留为authoring/serde/debug冷路径DTO，不能继续作为热提交ABI。renderer按segment generation缓存compiled elements/batches，stable present可遍历cached batches，但不得重新hash、clone或compile未变command payload。
4. 下层合同先验证stable、focus/window-only和单command content/geometry变化的frame/segment/payload `Arc::ptr_eq`矩阵，再验证painter order、clip、text/image、multi-surface namespace与full-reference pixels。计数至少包含full extract clone、segment rebuild、persistent-directory node clone、payload clone bytes、command flatten visits、compiled batch rebuild和typed fallback reason。

现有`ui_surface_frame_domain_sharing_pressure.py`已扩展为三路算法模型，并以测试明确拒绝Arc/COW伪修复。artifact `E:\zircon-profiles\runtime-ui-render-publication-segmentation-pressure-20260827.json`（SHA-256 `71E6A3548DD18AA1B1C322579C59873C89FA5DD3162D9867CEA45714602D67C6`）使用10,000 commands、10,000次单command render update、64 commands/segment、32路持久目录和每command 24 owned payload bytes的保守输入：current publication与direct Arc/COW都复制100,000,000 command entries、模型payload 2,400,000,000 bytes；segment目标复制640,000 entries、15,360,000 payload bytes和20,000个目录路径节点，command clone work下降156.25倍。24 bytes只代表显式模型输入，不含String header/capacity、resolved style/text layout、allocator metadata、CPU、RSS、延迟或GPU；artifact不是动态收益证据。

Unreal本地源码再次确认责任边界：`SlateInvalidationRoot.cpp:356-372`把root-owned `CachedElementData`压入window element list；`PaintFastPath:723-833`只消费`FinalUpdateList`和reindex heap；只有`PaintFastPath`转slow或`ClearAllFastPathData`时才在`:839-842`、`:1407-1447`清空cached element authority。Slint `partial_renderer.rs:42-74,174-209,378-525`则让item内cache index/generation指向backend cache，并按dirty region/property tracker更新。两者都支持“稳定generation共享、changed owner局部替换、显式slow fallback”，不支持每个changed frame复制完整draw DTO图。

本切片只修改可执行模型、合同和设计记录，没有编辑当前存在大范围外部改动的Surface/render生产路径，也没有运行Cargo。focused模型合同2/2通过；生产切换保持`design_ready_e3`，必须在owner窗口稳定后按`interface handle -> Surface immutable segment authority -> shared queue ABI -> renderer changed-segment cache`依赖顺序实施，不能从中间call site加parallel cache。

### 15.135 Measurement validity不能由父节点向全部后代放大

继续复核Runtime layout current source后，15.115与canonical计划中“用零frame冒充未测量”的结论已经过期。`UiLayoutCache`现有serde-default的`measure_valid`，成功measurement调用`complete_measure()`，布局输入、结构、responsive、pool复用与Collapsed清理调用`invalidate_measure()`；零尺寸frame保持独立有效。已有回归覆盖零frame父节点只进入required branch、Collapsed恢复重建失效后代，以及child insertion使parent measurement失效。这一层已经建立正确的显式状态，不能再按旧blocker重复实现。

但复核planner发现第二层放大：`plan_measurement_post_order`原先只要required parent的`dirty.input`或`measure_valid=false`，就把`force_children=true`传给全部children。向一个已经完成measurement、包含深层clean sibling subtree的auto-layout parent插入一个新child时，parent和new child确实需要重测，clean sibling只需提供缓存desired size；旧算法却把parent invalid扩大为整个clean subtree的post-order work。父节点随后聚合直接children仍需`O(K)`，但不应进入每个有效child的后代。这里必须区分“当前节点缓存失效”和“整个后代缓存失效”：前者是node-local validity，后者目前只有Collapsed清理这一显式原因。

本轮planner改为三个规则：full measurement仍遍历全部；incremental节点若不在required set且自身`measure_valid=true`则立即截断；自身invalid即使不在dependency path也会被测量，但不再强制其有效后代。`force_subtree`只由既有force或Collapsed继续传播。这样Collapsed pass对每个descendant执行`invalidate_measure()`后，恢复时会沿各节点invalidity自然进入整棵失效子树；普通parent invalid或input dirty不会再越过有效sibling root。新增Rust lower regression先warm一条256层clean branch，再向root插入一个fixed child，要求measured set精确为root + new child、probe count为root与两个direct children共3，clean branch validity保持true；既有Collapsed恢复和zero-frame回归继续作为正确性门。

Unreal本地参考支持这个责任边界。`SWidget.cpp:674-684`在global fast update路径只对`bNeedsPrepass` widget进入prepass；`WidgetProxy.cpp:122-175`重算当前widget desired size，只有结果改变或visibility post-update才向parent传播；`SlateInvalidationRoot.cpp:1160-1190`按prepass heap处理，并跳过已由前一个有效widget range覆盖的项。Fyrox `fyrox-ui/src/lib.rs:1830-1843`以node-local `measure_valid`与精确`prev_measure`决定cache hit。Zircon没有复制其类型，但采用相同不变量：失效必须属于具体node/input，父失效不是后代全部失效的替代信号。

压力artifact `E:\zircon-profiles\runtime-ui-measure-validity-locality-pressure-20260827.json`（SHA-256 `FE04FED7F0023E7B1408AE06D6B3F7CE1067BAC5721016A7936674185DC7640E`）使用10,000-node clean subtree、10,000次parent + new-child update、每次2个required measured nodes和2个root direct children建模：退役父级force路径写入100,020,000个post-order measured entries；node-local validity路径写入20,000个并执行30,000次顶层probe，消除100,000,000 measured-node work，比例5,001倍。该数字不代表真实CPU、栈深、BTreeSet、child desired聚合、allocator、RSS、延迟或GPU；宽parent仍需读取K个direct child desired，深度过大仍需单独验证迭代栈安全。

源码合同经历预期RED后转绿，validity focused源码/模型合同2/2，完整incremental-layout与模型Python合同20/20；两个Rust文件已通过Rust 2021 `rustfmt --check --config skip_children=true`。source-bound profile manifest新增measurement traversal、layout cache和tree mutation authority，由266增至269，PowerShell 7 profile-output Pester为48/48。Cargo仍未运行，因此新增Rust regression、Collapsed/zero-frame相邻用例和真实10,000-node产品测量计数尚无动态结果，本节保持`static_candidate`而不是性能GREEN。

### 15.136 Child order必须由拓扑代次发布，不能在每次layout pass重新排序

继续沿宽parent的剩余`O(K)`工作拆解后，确认其中混合了两类性质完全不同的成本。父容器为计算desired size或连续child frame读取K个直接child，在一般Linear/Grid/Wrap语义下仍是必要工作；但child order是拓扑/slot输入，不应在每次measurement和arrangement中重新推导。current source的`UiLayoutSlotIndexState`只缓存edge lookup、parent关系、root resize依赖与responsive候选，`ensure_initialized`只比较node/slot数量。measurement的`prepare_ordered_child_desired`每次重建`ordered_entries`并`sort_unstable_by_key`，arrangement的`order_children_for_container`又独立执行同一组slot lookup和sort。因此一个slot order完全稳定、仅一个child desired持续变化的K-child parent，每次pass仍支付两套`O(K log K)`顺序推导；复用Vec容量只删除allocator，不改变该复杂度。

不能直接在`UiLayoutSlotIndex`塞入一个按parent缓存的Vec。`UiTree.slots`仍是公开可变序列，现有index依靠每次lookup验证edge来容忍同数量slot替换；`slot.order`原地变化不会改变`slots.len()`。无mutation generation的缓存会把当前性能问题改成陈旧顺序正确性问题。正确authority必须在Runtime/Interface边界一次收口：序列化slot仍是数据合同，但生产slot插入、删除、替换和order变更必须经过typed `UiTree` mutation入口，记录受影响parent并推进runtime-only topology/slot generation；deserialization和完整tree build发布初代generation。`UiLayoutSlotIndex`在rebuild边界按受影响parent生成一个稳定、精确相等的ordered-child product，至少绑定container kind、tree child identity/order和相关slot值；measurement与arrangement共享同一个generation handle。相同generation只借用/克隆handle，不做slot lookup、hash或sort；缺generation、反序列化、同数量替换、container/slot kind变化和node-pool detach都必须显式重建或typed full fallback，不能在input event期lazy扫描。

参考实现支持这一所有权边界。Unreal `Children.h:460-512`让`TPanelChildren`直接持有有序slot数组并由`AddSlot`修改，`:696-707`在scoped slot提交时显式Add/Insert；`SWidget.cpp:739-757`把父子关系变化标成`ChildOrder` invalidation，而prepass只消费已发布children，不在每次prepass重排。Slint `item_tree.rs:1132-1236`把`children_index/children_count`存入item tree，`:1375-1386`按连续索引和遍历方向直接访问。Fyrox `fyrox-ui/src/widget.rs:1074-1097`在add/remove child时失效layout，后续measure遍历稳定children slice。Zircon不需要复制其容器类型，但必须采用同一不变量：顺序在结构变异时确定，layout pass只消费。

压力artifact `E:\zircon-profiles\runtime-ui-layout-order-authority-pressure-20260827.json`（SHA-256 `979E61BB2B6E5CFEAE5AD457121A158F65E9A5A0A0C3F949E1019B3B417B7088`）以10,000个direct children、10,000次单child layout update和1次拓扑发布建模。当前pass-local排序为10,000次、按`K * ceil(log2 K)`计1,400,000,000个比较工作单元；generation-owned目标排序1次、140,000个工作单元，消除1,399,860,000，结构比例10,000倍。模型同时保留100,000,000次required child aggregation，明确不把父容器本身的语义工作虚报为零；比较单元也不是Rust sort真实比较次数、CPU、allocator、RSS、cache miss、延迟或GPU数据。

本切片现已进入`static_candidate`。`UiTree`发布runtime-only `layout_order_generation`和受影响parent集合，生产slot插入/删除/order或kind变化以及node-pool detach统一经过typed mutation入口；canvas placement和z-order等非顺序变化不会误推进代次。`UiLayoutSlotIndex`按parent持有`Arc<[UiNodeId]>`有序产品，同代查询复用相同Arc identity，空叶节点复用索引级空Arc；增量layout在measurement前只同步dirty parent，完整layout在既有rebuild边界生成全部产品。measurement traversal按该句柄直接收集desired size，arrangement复制同一顺序到depth scratch，旧`prepare_ordered_child_desired`、`order_children_for_container`以及两套order scratch已hard cut；Wrap arrangement与content-size也不再消费不同顺序。下层源码回归覆盖同order稳定性、同代`Arc::ptr_eq`、同数量order原地变化、公开child同数量重排的防御性patch、非顺序slot变化不推进generation、Free/Scrollable顺序和10,000 sibling容量；focused算法/源码合同23/23、Python compile、scoped rustfmt与diff check通过。critical-source manifest保持272项，压力artifact与10,000倍结构比例不变；Cargo、真实allocator、CPU p50/p95/p99、RSS及产品resize仍未运行，因此不宣称宽parent动态达标。

### 15.137 Renderer必须继续消费publication的不可变command leaf身份

15.134对current source的结论已经部分过期：`UiRenderFrameExtract`现已用64-command不可变leaf、32路持久目录和局部range patch发布render generation，`UiRenderSubmission`也按surface保留共享frame segment；changed surface不再在publication边界复制整份flat command DTO。复核后剩余放大器已经下移到renderer：`ScreenSpaceUiPlanCache`原先每个submission surface只保存一个plan entry，并以整份`UiRenderFrameExtract Arc`作为identity。一个surface内只替换一个leaf时，frame根Arc必然变化，renderer仍会重新访问、投影并hash该surface的全部commands，publication的leaf sharing没有穿透最后一个consumer。

current candidate没有建立第二套分段缓存。`UiRenderFrameCommands`新增固定深度栈的borrowed exact-size leaf iterator，直接暴露既有`Arc<[UiRenderCommand]>` identity且不flatten目录；renderer仍按`UiRenderSubmissionSegment`取得route tree、node projection和normalized raster scale，但把每个surface segment进一步展开为command-leaf plan entry。命中键要求leaf `Arc::ptr_eq`、route tree、projection、DPI和传入background generation同时一致。planner跨leaf共享background tracker；重建leaf产生的background effects若与上一代相同则复用outgoing generation并保留suffix，否则分配新generation使后续leaf显式失效。这样不会把背景依赖错误地当作独立叶片，也不会在输入或surface rebuild路径增加扫描。最终`PreparedScreenSpaceUi`继续保留各leaf的共享plan，composition不复制vertices/draw/text payload。

lower Rust regression `screen_space_ui_plan_cache_reuses_unchanged_command_segments_within_surface`构造130条commands并只patch第65条，要求3个published leaves中首尾plan保持`Arc::ptr_eq`、中间plan重建且publication只clone一个leaf；它已写入但尚未获Cargo执行。源码/模型和相邻segmented submission、surface-frame sharing、render evidence、GPU image/dependency cache合同合计91/91通过，四个相关Rust文件通过Rust 2021 scoped rustfmt。新增`ui.screen_space_ui_plan.command_leaf_cache_hit_count`、`command_leaf_count`与`command_leaf_rebuild_count`用于后续产品证据；exact frame hit、全leaf复用和局部重建三条分支现在发布同一组leaf count/hit/rebuild/visit指标。stable evidence v2要求leaf hit等于leaf count且rebuild为0；delta evidence v2把image/text surface-segment守恒与planner command-leaf守恒分开。兼容`segment_cache_hit_count`当前只与新leaf hit对拍，不能再解释为surface命中。

source-bound artifact `E:\zircon-profiles\runtime-ui-command-leaf-plan-cache-pressure-20260831-r1.json`（schema v6，SHA-256 `30BDA2A9F4478C00088A42848195D5238E0F31B40A0AC2CC7C876EC10D65864A`）绑定5个implementation owner、Unreal `SlateInvalidationRoot.cpp`和Slint `partial_renderer.rs`。默认64 surfaces、每surface 512 commands、64 generations、每代一个非background leaf变化的模型中，surface级缓存访问65,024条commands，leaf级访问36,800条并命中32,193个leaf，避免28,224次command visit和4,064,256 modeled vertex-hash input bytes；composition payload clone为0。该模型不测Arc原子成本、hash实现、allocator、CPU/GPU、RSS、帧延迟或真实背景覆盖。Cargo与current-source Editor profile仍冻结，本节状态为`static_candidate`；动态验收必须分别覆盖普通image/text变化、background变化导致suffix失效、DPI/route/projection变化、resize连续输入和最终pixels一致性。

### 15.138 典型浅层派发不应为visited集合支付逐事件堆分配

事件路由从owned deep copy切换为借用slice后，pointer与navigation dispatcher仍各自在每次 routed event 开始时构造一个`HashSet<UiNodeId>`。它用于跨主route、stacked pointer candidate route和注入target去重，语义上不能删除；但典型按钮、菜单和窗口chrome route深度远小于通用HashSet的必要规模，因此稳定hover/click/navigation仍会保留“一次事件至少一次visited堆分配”的固定税。这个成本与前一轮消除的route节点复制是两条独立分配链，不能用route sharing的旧结论覆盖。

current candidate新增单一底层`UiDispatchVisitedNodeSet`：前16个唯一node id写入固定栈内数组并以线性比较去重，第17个唯一节点才一次性提升到按expected route容量建立的`HashSet`，同时把前16个节点种入fallback。pointer与navigation dispatcher共享同一实现；pointer的容量提示包含target path、stacked candidate slice和可选注入target，navigation包含target path。浅层路径不分配，深路径或候选route祖先分叉仍保留hash复杂度与原有去重语义。这里没有声称所有深度10 pointer事件都零分配：共享祖先fixture的14个唯一node保持栈内，而4条完全分叉的保守上界为40个唯一node，必须进入fallback。

lower regression覆盖16以内稳定inline与重复节点拒绝、17节点只提升一次且既有membership不丢；pointer/navigation源码合同同时禁止dispatcher直接恢复`HashSet::with_capacity`。source-bound artifact `E:\zircon-profiles\runtime-ui-dispatch-route-sharing-pressure-20260831-r4.json`（schema v3，SHA-256 `B97ACA0D0A7F5AE561718E051955560990CAA628065D85DEC0B32CE655F5FBAA`，source manifest SHA-256 `A2F50F98FC386D245ECFC4FBF2E0C1AB2570B6224233E6771109A383427A2C8C`）绑定current HEAD与5个dispatcher implementation owner。1,000,000事件的结构模型中，depth 1与depth 10共享祖先场景把pointer/navigation visited heap allocation都从1,000,000降为0；depth 100与depth 10四条分叉候选上界仍各保留1,000,000次fallback allocation。历史owned-route模型的pointer 616,000,000 node-id copy、4,928,000,000 minimum payload bytes和11,000,000 Vec allocation，以及navigation 300,000,000 node-id copy、2,400,000,000 bytes和3,000,000 Vec allocation保持为0。所有数字均是源码结构模型，不是allocator、CPU、RSS或输入延迟实测。

TDD与相邻route-sharing合同合计14/14通过，Python compile、四个Rust owner的Rust 2021 `rustfmt --check --config skip_children=true`及scoped diff check通过；新增Rust lower tests尚未获Cargo执行。产品profiling preflight `E:\zircon-profiles\ui-profile-preflight-20260831-r17.json`（SHA-256 `ABE052D947C9C598CC1A74ECDE4784B57DC8F847145A0B7F34C70155A5DF1C5A`）已绑定current HEAD与E盘managed release pool，但当前唯一blocker仍是该pool缺少`zircon_editor.exe`和`zircon_runtime.dll`；WPR/xperf/wpaexporter存在。故本节保持`static_candidate`，后续必须用current-source产品二进制实测按钮、菜单、连续resize下的visited fallback次数、allocator、CPU、event-to-hit p95/p99和像素正确性，不能由模型推导“交互已经流畅”。

### 15.139 Geometry-only命名不能替代semantic pane共享所有权

WindowMetrics链路的current-source复核修正了一个容易误判的状态：Editor现已新增`HostWindowGeometryPresentationData`、`to_host_contract_host_scene_geometry_with_retained_panes`和native floating geometry入口，职责命名上开始区分geometry与semantic pane；但底层dock/floating字段仍直接拥有`PaneData`。scene conversion为了保留四个dock semantic payload执行4次`.pane.clone()`，geometry `apply_to`再从current presentation复制同四个payload；两个floating conversion分支还各有一个`active_pane.clone()`站点。current source没有`Arc<PaneData>` carrier，因此这不是共享产品已完成，而是所有权拆分仍停在接口命名层。

正确修复不能继续把完整presentation或现有owned pane包进geometry DTO。semantic generation必须发布不可变共享pane identity，geometry generation只发布frame/clip/visibility/z/hit等派生数据；一次WindowMetrics transaction通过同一generation manifest组合两者，semantic identity保持不变且clone/rebuild计数为0。paint和input消费已发布的借用组合视图，不能在事件或绘制时再物化monolithic presentation。Unreal invalidation root保留widget/cached-element authority、由dirty update list驱动局部几何和paint更新的边界仍是本项主参考；Zircon现有Runtime frame-owned hit authority也要求Editor不要在resize时把语义与几何重新粘回一个owned payload。

current-source artifact `E:\zircon-profiles\ui-window-metrics-pane-clone-pressure-20260831-r3\ui-window-metrics-pane-clone-pressure.json`（SHA-256 `F0F089FF7F84ABF290DED99088C3AAF2EA807BD72E46A0A6B4FCC32ABBCE1017`，source manifest SHA-256 `0173E3C7637EA68A36716D60279FFE89AA21FB4EFE48163AA2343F4658386133`）绑定HEAD `050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f`。600个零浮窗resize frame、显式1 MiB pane payload输入下，现状固定为4,800次semantic clone和5,033,164,800 modeled bytes，目标为0；该byte值不是`PaneData`实测大小，也不包含allocator、CPU、RSS或延迟。focused fail-closed合同3/3通过并确认4 + 4 + 2站点未漂移。相关production owner全部已有外部修改，本轮只更新证据与设计报告，不吸收其半成品；managed产品二进制缺失时仍不能宣称resize动态达标。

### 15.140 SVG/GPU缓存存在，但无版本external provider仍关闭稳定代次快路

重新把15.110的正确性保护与15.139之后的current-source产品接线合并审查后，需要修正“稳定SVG/位图GPU准备已经普遍从`O(R)`降为`O(1)`”这一过宽结论。`WgpuUiImageCache::prepare`确实在无staged resource、无external provider且完整驻留时复用`prepared_generation`，并在进入`image_upload_sources`循环前返回；但主`WgpuRenderFramework::create_ui_surface_presenter`为每个产品UI Surface无条件构造`WgpuViewportProductProvider`并传入`Some(provider)`。`reusable_image_prepare_generation`与提交helper又都明确要求`!external_images_present`，所以实际产品Surface永远不能进入该整代快路。每个稳定source仍调用provider `resolve`，而`ViewportProductRegistry::resolve`每次都取得`products`互斥锁后查询resource key/generation。

这不是“SVG被反复加载”的证据。canonical source、parsed tree、raster product、immutable atlas page、device-shared texture allocation、Surface bind group和in-flight pin均已有缓存；稳定场景的文件读取、tree parse、raster和GPU upload仍为0。残余是缓存命中前的依赖查询税。新source-bound artifact `E:\zircon-profiles\editor-svg-gpu-residency-pressure-20260831-r16.json`（schema v5，SHA-256 `DE6CDF732A3F9BDBD52644188C3447A0AB95B199203F38B0386828424A5EE276`，source-set SHA-256 `8B1DCC8F5264CF21735AEBE92AD953A3E647C53BB5634E3BFED81D2D49E8926B`）绑定current HEAD与20个owner。默认10,000次present、16个稳定图像source时，current contract执行160,000次provider resolve和160,000次registry lock acquisition；这些是确定性源码结构计数，不是CPU时间、锁竞争、allocator、RSS、GPU时间或输入延迟实测。模型中20,480,000个image-command visits只保留为退役逐command重建baseline的概念上界，不再描述当前WGPU source loop。

正确修复不能简单忽略provider，也不能把provider readiness错误并入UI draw-list generation。同一UI generation内，viewport GPU product可能从`None`变为ready；旧保护因此必须保留。缺失的authority是provider自身的单调product revision或等价不可变snapshot identity。目标prepare key应至少包含`draw-list generation + provider revision + device generation`并持有完整prepared allocation/bind set；三者未变时每个present只做一次`O(1)`revision check并在source loop前返回，provider revision变化才重新解析external dependencies。若后续journal能精确列出changed resource identity，可在revision miss后只patch受影响source；没有精确journal时允许typed全external-source fallback，但不能在普通稳定帧逐项锁registry。默认模型目标保留10,000次revision check、把provider resolve从160,000降为0，稳定复杂度从`O(P * R)`收敛为`O(P)`。

这个边界与Unreal `SlateRHIResourceManager`为static/dynamic/vector资源返回稳定resource proxy、由resource manager显式更新/释放的做法一致：paint/draw热路消费已经发布的resource identity，而不是每帧重新探测所有资源。当前四个production owner `image_cache.rs`、`ui_surface.rs`、`wgpu_framework.rs`和`viewport_product_registry.rs`都有外部worktree修改，本轮未编辑或吸收；UI12在`surface/rebuild.rs`的device-pixel raster-scale合同与`metrics_dirty.rs`回归也完整保留。focused Python合同已覆盖provider安装、fast-path禁用、逐source registry锁和目标revision模型；Cargo与current-source产品二进制仍不可用，因此本节保持`design_ready/static_candidate`，不能宣称按钮、缩放或SVG动态性能已达标。

### 15.141 pointer/focus/navigation/raw已消费发布态，缩放错序不再退回全Surface扫描

再次核对`RuntimeUiSurfaceSet`后，generic `dispatch_input`先拒绝typed raw `MouseMotion`并记录`ui.surface_set.input.unrouted_reject_count`，Keyboard/Text/IME按保留的active focus owner直达单一Surface，Navigation/Analog按published navigation owner直达焦点Surface或topmost eligible root；dynamic session仍先把原始delta送入gameplay input buffer。navigation eligibility直接读取rebuild-owned `UiSurfaceNavigationIndex.spatial_all`的O(1)空/非空状态，不在事件时扫描树。uncaptured pointer先查询`RuntimeUiInputPublication`的global bounded cell directory，只对真实候选Surface做topmost fallthrough；hit generation不变的Surface在publication patch中不重扫entry，captured pointer也直达owner。上述pointer/focus/navigation路径都以`rebuild_before_dispatch=false`进入`UiInputManager`，事件调用链不再探测`rebuild_dirty`，text owner只为真实目标同步一次。query admission现已把`Published`、`Unpublished`和带原因的`Rejected`分开：仅冷启动`Unpublished`保留reverse fanout与event-time rebuild；non-finite pointer、degenerate viewport或affine overflow在任何Surface dispatch前O(1)拒绝并分别计数。

resize事件可能先于新frame publication到达。当前directory query保留已发布viewport，把新viewport物理坐标按x/y分别映射到旧publication坐标；`UiHitTestQuery.point`保持原始physical point，`virtual_pointer.current/previous`携带命中坐标，经`UiInputManager -> surface input dispatch -> pointer dispatch -> UiSurface::dispatch_pointer_event_with_query_and_modifiers`贯通。这样正常非退化resize不因viewport mismatch返回directory miss，也不扫描arranged nodes/render commands。下层Rust回归覆盖640x360发布后在1280x720查询同一逻辑位置、physical/virtual坐标合同，以及非候选dirty Surface不被事件重建；这些回归仍未获Cargo执行。

current-source artifact `E:\zircon-profiles\runtime-ui-surface-input-publication-20260901-r13.json`（schema v11，SHA-256 `0E09FD6F22F06833B2FBB7080E85C392F4AC42C592617EEDDA9DFBF1AE7264FD`，source-set SHA-256 `18FC9D1B746A54679AEB325B875C32E301E356DF294791FF426FA6937A090B13`）绑定HEAD `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`及10个dirty critical sources。它会在pointer publication落到fallback之后、invalid rejection落到fallback之后、physical/virtual mapping消失、query-aware manager/pointer/Surface链路被绕开、navigation eligibility退化为树扫描、Navigation/Analog owner落到fanout之后、window pump参数错位、retained cell-stamp/footprint复用消失、per-entry cell-vector allocation或consumer-independent footprint sort恢复、compact u32 membership漂移、raw/focus路径漂移或同步owner变化时fail closed，并拒绝向C盘写profile产物。

默认64 Surface、pointer/focused/navigation/unrouted各100,000事件的legacy基线为25,600,000次Surface dispatch、25,200,000次event clone和25,600,000次event-path rebuild probe。当前模型中的四类已收敛为400,000次真实Surface dispatch、100,000次必要pointer fallthrough clone、零event-path rebuild probe与400,000次text-owner sync，分别避免25,200,000、25,100,000、25,600,000和50,800,000次对应调用；current-to-target为1.0，baseline-to-target为64.0。invalid admission另为零Surface dispatch、零event-path rebuild probe。publication patch也不再为每个dirty Surface分配510-cell布尔占用表和替换footprint：默认64 Surface warm patch移除64 + 64次分配与32,640 transient bool bytes，常驻stamp scratch为2,040 bytes；同一stamp去重还移除了64次无消费者footprint排序和32,640个排序输入项；无分配cell visitor再移除640,000次per-entry临时cell-vector分配。它们仍只是确定性调用/分配次数，不是CPU、allocator latency、RSS或输入延迟。focused pressure/design合同21/21通过；当前Surface vector构造后不可变，因此index已是现有生命周期内稳定identity，动态add/remove/reorder只能在先引入独立Surface ID后开放。managed Rust验证与产品1/4/16/64 Surface p95仍open，不能宣称鼠标或缩放已动态达标。

### 15.142 Projected hit grid 已复用共享容量门禁，超宽 entry 改为自适应 coarsening

current-source复审发现base `UiHitTestIndex`已有每轴128、总计16,384 cell、单entry 4,096 cell与finite geometry保护，但popup `UiProjectedHitTestIndex`在publication rebuild时仍自行执行`columns * rows`并维护第二套cell映射。这样control-anchored popup投影后的巨大或非finite frame仍能绕开base保护，且两套规则后续必然漂移。lower RED先要求frame/instance authority已有的popup grid同时满足finite拒绝、bounded backing和共享helper；production随后把dimension/cell-span/finite helper提升为tree内`pub(crate)`权威，base build、base geometry patch、projected full build与projected incremental patch全部复用它。projected backing allocation改为checked multiplication，重复`projected_cells_for_frame`已删除。UI12在`surface/rebuild.rs`的device-pixel scale与`metrics_dirty.rs`回归没有被编辑。

本地Unreal源码确认`FHittestGrid::SetHittestArea`按窗口hit-test area确定`NumCells`，`AddWidget`在paint时读取`GetPaintSpaceGeometry/GetRenderBoundingRect`并把cell坐标夹在窗口网格；Fyrox虽递归pick全树，也先用screen-size clip拒绝不可见subtree。Zircon下一步不应继续让authored union bounds拥有索引容量，而应把viewport/surface budget变成authority。第一版安全fallback曾把超宽entry压到1×1，虽然不会OOM却会让所有query扫描全surface entry；本轮lower RED因此进一步要求保留空间分区。当前共享算法在任一entry超过4,096 membership时把cell size加倍：由于初始网格每轴已封顶128，coarsened网格最多64×64，所有entry自然不超过4,096 membership。1,000,000×1,000,000单entry压力模型从旧64像素规则的244,140,625个cell降为4,096，结构比例59,604.64倍且`global_grid_collapsed=false`；10,000节点paint-order cursor模型同时把旧累计49,995,000次max-order visit降为0。这些都是算法基数，不是CPU、allocator、RSS或输入延迟。

source-bound artifact为`E:\zircon-profiles\runtime-ui-tree-hit-grid-admission-pressure-20260831-r3.json`，SHA-256 `77F70F264B2182CAA448F7F8FF8F37546C584E0281C72EACFE3768A32EB42F58`，source-set SHA-256 `2A343F27FC3F3F79BED2569DA32439D5CF96C1571437BD6A6EA91C94F4D5E014`，绑定HEAD `17b92691e1d67c3df09376a2ca599bf2e07a061d`与6个dirty critical sources。两个focused Python suite合计11/11、pycompile、scoped rustfmt和diff check通过。本轮无Cargo授权，因此Rust regression与产品验证仍open；必须继续补surface级node/entry/membership/bytes typed budget、adaptive-coarsening/query-candidate counters，以及resize/popup场景的CPU、RSS与input p95，不能把确定性基数改善误报成“已经流畅”。

### 15.143 Inspector 全量投影仍是双阶段 O(P)，但普通 resize 已不属于该基线

复核Editor Inspector从authoring snapshot到pane publication的完整链路后，确认普通presentation/value refresh仍有两个连续的全量阶段。`EditorState::snapshot_with_inspector_customizations`枚举选中实体的dynamic components，执行reflection/JSON property projection、field-editor resolution和每组件属性排序，构造拥有全部`String/Vec`的`InspectorSnapshot`；`pane_payload_builders/inspector.rs`随后再次遍历全部components/properties，把字段、值、editor kind和asset markers复制进`InspectorPanePayload`。下游converter又会建立Surface/layout/host model。Runtime virtual slots只能约束最终物理slot数，不能消除这两个上游`O(P)`物化阶段。

本轮同时纠正了旧模型把所有resize都算入Inspector全量投影的错误。current `WindowMetrics` transaction先从`committed_shell_state`取得chrome/model/pane payload和retained shell presentation；`apply_window_metrics_geometry_presentation`成功后直接重新提交这些identity并返回，不调用`build_chrome`或Inspector pane builder。只有committed-stage miss或geometry publication失败才进入全量fallback。因此复杂度应写成`O((1 + R + D + F) * P)`，其中`R`是普通presentation recompute，`D`是当前尚未增量化的value update，`F <= Z`是明确计数的metrics fallback；正常resize `Z - F`对Inspector snapshot/pane物化为0。后续产品证据必须分别报告metrics fast-path与fallback，不能继续用“200 resize = 200 full Inspector rebuild”推导热点。

本地参考源码支持单一retained model而非leaf cache：Unreal `SDetailsView`以`RootTreeNodes`作为稳定tree source，`ShouldSetNewObjects`拒绝无变化对象集重置，并独立刷新node visibility；Fyrox `InspectorContext`保留property editor widget handles，`sync`只生成属性消息；Slint `ModelTracker/ModelNotify`提供row changed/added/removed通知。Zircon对应目标仍是source owner发布selection/schema/value generations、共享logical property source和exact changed field IDs，由`RetainedEditorHost`持有唯一Surface/frame/model publication。仅把DTO字段替换成`Arc<String>`不能移除双遍历，也会留下错误authority。

current-source artifact为`E:\zircon-profiles\editor-inspector-projection-pressure-20260901-r2.json`，SHA-256 `479E7F2B10CBB5488081583CBCA563074906F0DA156F8017F91FA8551552952C`，source-set SHA-256 `E1D516EE51F954AA65717180FD794FEAB311B779E61B36DB7830B0F6F4A608E7`，绑定HEAD `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`、7个Editor owner和4个reference source。默认10,000 properties、1,000 stable presentation recomputes、200 metrics resize、0 fallback、1,000 single-field updates下，current模型为20,010,000次snapshot property materialization与20,010,000次pane property copy，合计40,020,000 property-record work；target为11,000次record update，对应1,819.09x snapshot ratio与3,638.18x双阶段ratio。current Surface build为2,001，拒绝的all-resize-fallback基线另加200。focused模型/合同9/9通过；这些仍是确定性操作次数，不是CPU、allocator、RSS或input-to-present timing。生产owner均有外部改动，本轮不建立第二authority，也不声称按钮、缩放或Inspector交互已经动态达标。
