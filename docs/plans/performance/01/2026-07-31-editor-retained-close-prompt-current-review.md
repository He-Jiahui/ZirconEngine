---
related_code:
  - zircon_editor/src/ui/retained_host/app/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
tests:
  - inline tests: 0
  - external close-prompt behavior tests inspected: 4
  - rustfmt check: blocked by pre-existing import-order drift in 2 externally modified files
  - current-source managed Windows Cargo pending
  - dirty/preflight/prompt allocation counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained close-prompt当前源码复核（2026-07-31）

## 范围

`zircon_editor/src/ui/retained_host/app/close_prompt.rs`与`close_prompt/**`当前源 **7/7** 个Rust文件、**220** 行、**0** 条内联`#[test]`已逐文件阅读；path+raw-content SHA-256为`875e6485a638198c7ffe4149b8c2855a44911a8b74baee92ea88636493666a2a`。root与`presentation/data.rs`只有外部未提交import顺序差异，本轮按current source只读审查，未修改Rust。另核对4条外部prompt行为测试和`native_window_close/**`完整consumer。

| 模块 | 文件 | 行 | 当前边界 |
|---|---:|---:|---|
| root/action | 2/2 | 17 | 固定三action解析与模块导出 |
| model | 1/1 | 75 | dirty筛选、pending target/instances/rows ownership |
| presentation root/data | 2/2 | 46 | prompt DTO、save capability与窗口提交 |
| layout/text | 2/2 | 82 | 窗口尺寸布局与prompt strings/details |

## 发现

- **正向边界**：action parser是固定三分支且无分配；prompt layout是常数工作；details最多读取3个title后join，更多项只追加省略号；save capability为一次dirty-row线性扫描。prompt仅在close请求或save error时生成，不是idle/per-frame路径，因此这些小分配不单列热点。
- `DirtyCloseView`只保留save/prompt需要的instance、descriptor和title，不复制完整`ViewInstance`；main close的`all_dirty_close_views`单次线性扫描合理。`UiHostWindow`由consumer strong-handle选择，不存在本模块内的完整presentation clone。
- **PERF-MVP-602复证**：floating `dirty_close_views`先把candidate ids再collect成Vec，然后对全部`V`个view逐个线性查找`k`个candidate，达到`O(V*k)`；caller此前已经clone完整view集合与candidate ids。应由session/layout owner在同一generation下按目标window直接投影dirty摘要，不能仅把Vec换成临时HashSet后保留full snapshot。
- `PendingClosePrompt`与三层row均derive `Clone`并完全owned。valid action consumer当前clone整份prompt，所以`k`个close ids与`d`个dirty title/id被复制；这是consumer状态转移问题，不应在model里叠加第二Arc/cache。异步save落地后由唯一prompt state按move/take转移所有权。
- `host_prompt_data`每次show/retry重新读取native size，构造target/message/details Strings并提交完整close-prompt DTO；这是低频有界工作。优化优先级低于避免partial retry重复save与逐tab完整事务；验收只要求同一Saving generation不重复show/build，而不是缓存跨尺寸/跨target的陈旧布局。
- 当前model只以descriptor字符串判断save capability。Editor09 canonical save preflight必须成为唯一typed authority并返回逐view结果；prompt可以消费其summary，不能复制一套domain-specific save dispatch表。

## 参考与目标

Godot `dev/godot/editor/editor_node.cpp:3461-3484,6826-6865`让统一close queue持有待处理场景，prompt只推进队列；Unreal `AssetEditorSubsystem.cpp:459-477`对稳定open-editor集合完成close协商后统一广播。Zircon保留单prompt交互，但由EditorUI08持有唯一generation-checked close state，Editor09提供typed dirty/save summary，Editor14执行有界save job；model不持第二layout、dirty或job事实源。

## 动态验收

复用PERF-MVP-602矩阵，额外记录views/candidates/dirty `1/8/128/1K/100K`下full-view clones、candidate clones/probes、dirty-row/title bytes、pending prompt clones、prompt DTO/String builds、native size reads与show/set次数。要求floating preflight candidate工作近`O(k)`且full-view clone=0；valid action pending deep clone=0；同一Saving generation的重复prompt build/show=0；target/size/dirty generation改变时各build一次；现有cancel/discard/save/unsupported/error与窗口布局语义等价。managed Cargo、规模counter、WPR/Tracy/F4与independent review完成前保留在`pending.md`，不进入`review.md`。
