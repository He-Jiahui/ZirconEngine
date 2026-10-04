---
related_code:
  - zircon_runtime/src/ui
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_editor/src/ui
  - zircon_editor/src/ui/workbench
  - zircon_editor/src/ui/retained_host
  - zircon_runtime_interface/src/ui
  - tools/analysis/profiling/ui/ui-profile-capture.ps1
  - tools/analysis/profiling/ui/ui-profile-scenarios.ps1
  - tools/analysis/profiling/ui/ui-profile-surface-pipeline-metrics.ps1
  - tools/analysis/profiling/ui/ui-profile-latency-evidence.ps1
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor
  - dev/bevy/crates/bevy_ui
  - dev/bevy/crates/bevy_ui_render
  - dev/Fyrox/fyrox-ui
  - dev/godot/scene/gui
status: assessment_complete_plan_pending_implementation
---

# UI 整体架构与性能重审报告（2026-09-02）

## 1. 结论摘要

本轮只完成 current-source 代码/计划审查，没有修改生产实现，也没有把静态测试数量或历史 profile artifact 当作动态性能结论。现阶段最主要的问题不是某个 `Vec` 或单个 layout 函数，而是 UI 的 authority、generation 和 presentation 边界仍未统一：

1. Runtime UI 的 retained tree、layout、hit-test、navigation、render extract 和 Editor retained host 已有增量基础，但部分路径仍以受影响 subtree/full scan 为工作单位；Taffy bridge 逐 container 创建临时树，无法提供跨帧 graph/cache。
2. Editor 与 game UI 仍有两套终端 renderer。game renderer 按 primitive 类型全局分桶，存在破坏 painter order 的结构性正确性风险；Editor renderer 有 generation/cache/damage 基础，但普通 live frame 仍可能没有 producer generation，damage path 还必须验证 alpha/删除等价性。
3. Editor 的 workspace/layout、UI asset authoring 与 runtime UI 的数据同步存在多处“先修改 live state、后验证/写盘/import”的事务风险；性能优化前若不先固定 revision/generation/transaction，任何 profile 结果都可能测到不一致状态。
4. 文本、字体和图标已经有大量真实实现，但 cache、font collection、atlas、Editor/game renderer 和 device lifetime 尚未统一。文本/字体静态修正不能等同于实际 WGPU、内存、功耗或多语言产品资格。
5. 已有 profile 工具能够产生结构化的 timeline、stage duration、dirty/visited/reused/rebuilt、damage、GPU timing、input latency、RSS/进程证据和 scale fixture；但当前仓库证据仍缺少受管执行结果，不能宣称达到 Unreal 或其他引擎合理值。

因此，首个实现方向不能是“继续局部微优化”，而应先建立唯一的 `UiRuntimeService`/surface lifecycle、generation-qualified publication、统一 ordered presentation artifact 和可回滚的 Editor transaction；之后再对布局、虚拟化、文本和 GPU batch 做数据驱动优化。

## 2. 当前证据边界

| 领域 | 当前证据 | 可得结论 | 未证明内容 |
|---|---|---|---|
| Runtime UI tree/layout/input/a11y | 11A E2-E3 静态纵向审查 | 发现 owner、数据流和结构性缺口 | 真实输入到 present、OS a11y、CPU/RSS/power |
| Runtime text/font/IME | 11B E2-E3 静态纵向审查 | 发现字体 artifact、shaping outcome、长文本、编辑与 IME 边界 | 多语言 WGPU、真实 IME、百万行、性能 |
| UI GPU/atlas/submit | 11C E2-E3 静态纵向审查 | 发现 painter order、icon、damage、两套 renderer 与 cache 边界 | 像素结果、GPU 时间、带宽、VRAM、device loss |
| Editor layout/authoring | Editor13/23 E3 静态审查 | 发现 restore/save/authoring transaction 和重复 authority 风险 | 崩溃恢复、多实例、真实窗口/DPI、规模延迟 |
| Profile tooling | tools 脚本和测试静态审查 | 已有 E4 采集路径设计与 E-drive 输出约束 | 本轮没有新的受管 capture artifact |

现有工作区包含大量未提交变更；实施前必须重新计算相关 source fingerprint，并将未跟踪/在途文件纳入审查。隔离 worktree 在 Windows 长文件名上无法创建，因此本报告没有将任何隔离副本的结果混入主工作区。

## 3. 结构性瓶颈排序

### P0：正确性与 authority 先于性能

- **唯一 runtime UI owner**：`UiModule` 当前 driver 与 dynamic session 私有 surface 生命周期未完全收敛。必须由 `UiRuntimeService` 统一 world/window/target、clock、input queue、host request、publication 和 teardown。
- **统一 generation/transaction**：Tree mutation、Editor workspace restore、UI asset save、binding/model update、async compile/import 和 preview 结果都必须带 asset/surface/document/generation，并采用 validate-then-commit。旧结果只能成为 stale/pending，不能覆盖新状态。
- **统一 ordered presentation**：game renderer 不能继续把已排序 command 拆成 solid/image/text 全局数组；应先保持 painter token 顺序，再在证明不改变重叠关系时合批或安全重排。
- **damage 等价性**：Editor retained texture 必须明确 clear/backdrop/replace 语义，full redraw 与 damage redraw 逐像素一致后才可继续优化 partial present。

### P1：算法和数据结构

- **Persistent layout graph**：surface generation 持有 persistent Taffy/等价布局图；当前逐 container 临时树无法有效复用 measure/layout cache。
- **Changed frontier 而非 subtree 全收集**：局部 rebuild 仍可能遍历完整受影响 subtree，并在 arranged/hit/render/navigation 多个 artifact 上重复工作；需要统一 changed-set/frontier 和精确 reused/changed/allocated 统计。
- **真实 virtualization**：当前 virtual list 主要是全量 retained child + 可见性裁剪，不是 data-source virtualization；100k/1M item 必须只物化 visible + overscan，并提供 variable extent/anchor correction/focus/a11y logical model。
- **Persistent navigation/hit index**：导航按键和 popup/focus 路径不能每次全树收集排序；hit index 需要 surface/viewport-owned budget、候选数和退化策略。
- **增量文本模型**：编辑器仍有完整正文 clone/projection 路径；需要 paragraph/chunk storage、grapheme/line index、局部 shape/layout invalidation 和 bounded history。

### P2：可测量的热路径

- 每帧 owned vector、JSON generation hash、CPU quad/tessellation、全 vertex hash、字符串/TOML解析、text pass split、image draw/bind 切换。
- global/process font database clone、thread-local font system rebuild、atlas relocation/eviction、4K retained texture full copy。
- Editor projection、hierarchy/palette/inspector/diagnostics 全量 projection 与未虚拟化列表。

## 4. Unreal 参考应吸收的机制

本地 Unreal 源码审查应以机制而不是类名复制为准：

- `SlateInvalidationRoot`、widget list/heap/index 表明 invalidation 应由明确 root、reason、stable index 和 changed worklist 驱动，而不是任意树遍历。
- `FSlateApplication` 表明 window、tick、focus、popup/modal、input 和 accessibility 需要一个产品级 application owner。
- `ElementBatcher`/`SlateRenderBatch` 表明 batch key 必须包含 resource/material/clip/effect 等语义，并保持 layer/painter order；只合并相邻或经 overlap 证明安全的元素。
- `FHittestGrid` 表明 hit-test backing grid 的容量由窗口/target domain 约束，不能由异常 authored geometry 无界决定。
- UMG Editor 的 factory/compiler/designer/hierarchy/palette 表明编辑器必须围绕真实 asset、preview instance、transaction、drag/drop、诊断和生成结果闭环，而不是固定展示数据。

不应照搬 Unreal 的 singleton、历史配置格式或兼容性债务。Zircon 应保留自身 typed DTO、content-addressed artifact 和 Rust ownership，但必须达到相同的生命周期、失败、顺序和验证纪律。

## 5. E-drive 性能基线计划

所有输出根目录固定为 `E:\zircon-profiles`，Cargo target 使用 `E:\zircon-build\targets\...` 或 `D:\cargo-targets\...`；禁止 C 盘产物。每个 artifact 必须记录 source fingerprint、git revision、feature set、binary hash、OS/backend/DPI/font/locale/profile、generation 和 workload hash。

### 5.1 基线矩阵

| 轴 | 最小场景 |
|---|---|
| UI tree | 1 / 1k / 10k / 100k nodes；深链、宽树、popup/backdrop、NaN/Inf/超大 frame rejection |
| list/editor | 100k / 1M logical items；fixed/variable extent、scroll、jump、selection、near-caret edit |
| text | 1 / 100 / 1k / 10k graphemes；Latin/CJK/Arabic/Indic/emoji、LTR/RTL、rich、VerticalRl、wrapped/editable |
| renderer | solid/image/Glyphon/bitmap/SDF/icon/decoration 重叠矩阵；1080p/1440p/4K；1/4 window；DPI 1.0/2.0 |
| interaction | click/hover/drag/wheel/resize/filter；1k、100k、1M event pressure；input-to-damage/submit/present |
| lifecycle | cold/warm、font/theme/locale hot reload、surface lost/device reset、multi-window、slow subscriber |
| persistence | restore 10k tabs/100 windows/100 placeholders；Nth write/import failure、crash/restart、双实例 conflict |

### 5.2 必采集指标

- CPU：`ui.surface_rebuild.*` stage p50/p95/p99/max、layout visited/measure/arrange probes、Taffy build nodes、hit/navigation candidates、projection/rebuild counts。
- Rendering：ordered op count、batch/layer/dependency、draw/pass、pipeline switches、geometry rebuilt/reused、upload/hash bytes、atlas occupancy/relocation/eviction、damage area、retained copy bytes、GPU timestamp。
- Memory：allocation count/bytes、peak RSS、CPU/GPU resident bytes、cache hit/miss/eviction、subscriber queue depth/bytes/age。
- Latency：`input_to_damage_us`、`damage_to_submit_us`、`input_to_present_us` 的 31-sample cold/warm p50/p95/p99；记录 dropped/coalesced/rejected outcome。
- Correctness：pixel hash/diff、geometry diff、ordered painter token、source/runtime/font/device generation、full-vs-damage equivalence。

现有阈值（如 input-to-damage 1000 us、damage-to-submit 8000 us、input-to-present 9000 us）只能作为仓库当前 gate 的起点，不代表 Unreal 基线或普适行业标准。必须先采集同一硬件、同一内容和同一 backend 的自身 baseline，再决定是否收紧阈值。

## 6. 实施顺序（评估后才开始）

### M0：冻结证据与失败不变量

重取 fingerprint，确认工作区重叠；建立 ordered painter、damage、tree cycle/depth/budget、generation stale、restore/save no-op 失败测试；先取得 clean managed Cargo/profile 资格。

### M1：唯一 UI runtime service

把 surface registry、clock/tick、window/lifecycle、input/result、host bridge、teardown 归入 `UiRuntimeService`；dynamic session、Editor/Play 和测试适配同一 owner。

### M2：generation-qualified data publication

统一 `UiSurfaceHandle`/`UiNodeHandle`/document revision；Tree validate-then-commit；async compile/import/preview/binding 结果按 generation 提交；删除 bool-only/字符串反射成功旁路。

### M3：canonical ordered presentation

建立 backend-neutral `UiPaintScene`/`UiGpuPresentation`；game 与 Editor producer 适配同一 ordered draw-op、clip/resource/color/device contract；先修 painter/icon/damage 三个 P0，再测像素。

### M4：persistent layout 与真实 virtualization

将布局图和 measure cache 变成 surface-owned persistent artifact；以 changed frontier 驱动 layout/arrange/hit/navigation/render；引入 data-source virtualization、pool budget、variable extent 和 anchor correction。

### M5：文本/资源统一 owner

让 cooked font/glyph/icon/image artifact、device generation、atlas page/slot/lease/residency 进入同一 resource service；Editor 不再创建独立字体/图标真值。

### M6：Editor transaction 与同步

workspace/UI asset save、restore、undo/redo、cross-asset operation 统一 staging、CAS/revision、atomic durability、rollback、crash recovery 和 conflict receipt。

### M7：动态资格与对比

运行上述矩阵，补真实 WGPU/窗口/IME/DPI/device loss/clean package；在相同内容和画质下才与 Unreal Slate 做 CPU/GPU/带宽/VRAM/p95 对照。没有对照数据，不得宣称“达到 Unreal”。

## 7. 当前决策

- **暂不开始局部优化实现**：先完成 M0 的 current-source fingerprint、失败不变量和动态 profile 资格。
- **不以 `rebuild_dirty` 当前的局部 patch 统计宣布增量布局完成**：其报告目前仍可能把 patch 记成 rebuilt，且内部仍有 subtree/full-scan 成本。
- **不以已有静态 profile artifact 宣布性能达标**：它们能证明计数模型或 source-bound 下界，不能替代 managed CPU/GPU/RSS/power。
- **不再新增第二套 UI/text/font/icon/navigation authority**：后续实现必须接入现有 owner 或明确硬切 owner。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| UI-REVIEW-R0 | UI 布局、渲染、Editor、文本/字体、同步与 profile tooling 只读重审 | completed_review_only | 2026-09-02 | 6份优化计划、Runtime/Editor关键源码、Unreal/Bevy/Fyrox/Godot参考边界、E-drive profile脚本已交叉核对；未运行Cargo或新建产品profile |
| UI-REVIEW-M0 | fingerprint、失败不变量、受管动态基线 | pending | - | 必须先取得 current-source 与 managed artifact |
