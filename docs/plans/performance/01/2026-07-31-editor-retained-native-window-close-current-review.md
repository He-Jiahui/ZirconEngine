---
related_code:
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close
  - zircon_editor/src/ui/retained_host/app/close_prompt/model.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
tests:
  - inline source guard: 1
  - external close-prompt behavior tests inspected: 4
  - rustfmt check: blocked by pre-existing import-order drift in 5 externally modified files
  - current-source managed Windows Cargo pending
  - close/save scale counters and WPR/Tracy product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained native-window-close当前源码复核（2026-07-31）

## 范围

`zircon_editor/src/ui/retained_host/app/native_window_close.rs`与`native_window_close/**`当前源 **7/7** 个Rust文件、**262** 行、**1** 条内联`#[test]`已逐文件阅读；path+raw-content SHA-256为`7e9b2a94469f5f060c14b52485b3491f4c8bc0e4ab4166a7ac8ce62435d8a49d`。5个叶文件只有外部未提交import顺序差异，本轮按current source只读审查，未修改Rust。另完整回查`close_prompt/model.rs`、事件分发、layout apply/session metadata、scene-inspection publication与4条外部close-prompt行为测试。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| close request root | 1/1 | 63 | 1 | main/floating dirty preflight与prompt入口 |
| floating close | 1/1 | 65 | 0 | workspace实例收集、逐实例关闭与最终window检查 |
| action routing | 2/2 | 39 | 0 | action解析与pending prompt分派 |
| completion/presentation | 2/2 | 65 | 0 | exit/window close完成与prompt window选择 |
| saving | 1/1 | 30 | 0 | UI asset/animation同步保存 |

## 发现

- **正向边界**：main-window close已复用一次`current_view_instances()`结果；内联source guard锁定该边界。无dirty时不构造prompt；未知action在读取pending prompt前早退；`UiHostWindow::clone_strong()`是窗口生命周期handle clone，不误报为presentation深复制。
- floating close预检先调用`recompute_if_dirty()`，随后`current_layout()`深clone完整layout以定位window并递归收集`k`个instance id，再`current_view_instances()`深clone全部`V`个实例。`dirty_close_views`还clone candidate Vec并对每个实例线性`any`候选，复杂度为`O(V*k)`；dirty row再复制instance/descriptor/title。无dirty时关闭完成后又clone完整layout确认window是否存在。
- valid prompt action先`clone()`完整`PendingClosePrompt`，复制target、全部close ids及dirty row strings。save中途失败时原pending prompt保留，重试会从旧列表重新保存先前已成功的文档；这同时放大I/O并缺少“全部支持后再开始”的事务预检。
- **PERF-MVP-602 / 逐标签完整事务放大**：floating window含`k`个tab时，`close_floating_window_without_prompt`逐个构造`CloseView`事件。每个事件都进入dispatcher、取得shell锁、修改layout并调用`recompute_session_metadata`；该函数每次重新收集全layout placements、clone全部剩余open instances、重建window registry、retain editor sessions并同步window-host manager。整体至少为`O(k*(L+V))`，多tab关闭趋近二次工作。
- 每个changed layout event随后还调用`publish_scene_inspection_publication()`，重新取得scene-inspection、shell和authoring-world锁并观察scene，尽管纯layout close不改变scene；再执行workbench invalidation/reflection、trace和journal record。host逐event应用effects时还无条件调用`sync_pending_play_decisions()`。循环末尾才做一次完整retained recompute，因此“只重算一次UI”没有消除上游`k`次元数据、world observation、事件和副作用成本。
- **主线程同步保存**：UI asset保存直接序列化、`fs::write`、复制disk baseline、asset import、workspace refresh、hydrate与instance sync；animation保存也在session save后同步import与sync。多个dirty views在native prompt button回调中串行执行，没有job admission、bytes/time预算、取消、进度或generation commit gate，慢磁盘/导入会直接阻塞窗口事件循环。
- `finish_prompted_close`先清pending状态，再忽略floating close返回值；异步化前必须定义`Prompting -> Saving -> Committing/Failed -> Closing`唯一状态机和generation校验，不能只把现有循环包进任意worker。

## 参考与目标

- Godot `dev/godot/editor/editor_node.cpp:2655-2681,3461-3484,4258-4273,6826-6865`先建立`tabs_to_close`队列，逐项处理需要用户确认的场景，并用`save_editor_layout_delayed()`合并布局持久化。它仍有串行交互成本，但证明close queue与coalesced layout commit应由统一owner持有，而不是每tab触发一次完整host刷新。
- Unreal `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Subsystems/AssetEditorSubsystem.cpp:459-477`先复制稳定open-editor key集合，完成全部close协商后只广播一次`CloseAllAssetEditors`；`MainFrameActions.cpp:348-359`先收集dirty packages，再进入一次统一`PromptForCheckoutAndSave`。Zircon应采用typed batch transaction和统一save request，而不是复制其具体UI。

EditorUI08新增原子`CloseViews`/`CloseFloatingWindow`事件合同：一次generation-checked preflight取得目标实例与dirty摘要，一次shell mutation关闭全部可关闭实例，一次session metadata/window registry重建，一条event record和一次dirty-domain发布；纯layout close不观察authoring world。Editor09发布统一typed save-all request/result与dirty-generation commit，先验证全部view可保存再提交。Editor14为保存/导入链提供明确的有界interactive lane、per-resource mutex group、取消/进度和completion budget；不得落到当前默认无限`Misc`批量提升，也不得让worker持UI/window对象。

## 动态验收

按window tabs `1/8/128/1K`、open views `1/1K/100K`、dirty `0/1/all`、target `main/floating`、action `cancel/discard/save`、save failure `first/middle/last/retry`记录layout/view snapshot builds与clone bytes、candidate probes、layout events/records、shell/world lock acquire/wait/hold、placement visits、metadata/window-registry rebuild、scene-inspection observes、host invalidations/recomputes/native hide、save jobs/fs writes/imports/retries、UI blocked p50/p95与RSS。

验收要求：floating preflight不clone完整layout/view集合，candidate工作近`O(k)`；关闭`k`个tabs的layout transaction、metadata/window-registry rebuild、event/journal、host invalidation/recompute与native hide各不超过1，纯layout close的scene-inspection/world lock为0；stable/重复close为no-op；保存UI线程文件I/O与import为0，每dirty view每generation最多保存一次，retry只处理未成功项，全部成功且generation仍匹配后才关闭。现有4条prompt行为、main exit、cancel、unsupported save、partial failure、focus/order/session cleanup、Cargo、F4 native-window像素与WPR/Tracy通过前保留在`pending.md`，不进入`review.md`。该路径不提交GPU工作，RenderDoc不是本模块主验收工具。
