---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - command palette theme/metrics/search/row/native/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - open/query burst and 1/100/10000 catalog/visible-row/clone/theme/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template command palette逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_command_palette*`共 **39/39** 个Rust文件、**1,280** 行已逐文件阅读，并回查已在editor-core批次验收的`commands/registry.rs`与`app/command_palette_actions.rs`候选生成入口。覆盖identity/open gate、panel/search/empty、row state/surface/indicator/label/detail、theme/metrics/geometry及native/pixel tests。Current-source baseline全批次另有2个无关source-guard漂移，open/query/规模trace尚未完成，因此仍留在`pending.md`。

## P0：catalog多份O(n)表示与paint晚裁剪

打开palette时，registry先收集完整`Vec<EditorCommandPaletteEntry>`；open state再把全部entries转换为一份`UiValue` commands，并克隆全部id构建另一份`filtered_commands`，同一catalog至少存在多份O(n) owned表示。Rust当前只看到query作为已投影字段，未发现可复用的typed search index/generation。

Paint对全部structured options先执行owned `row_data`，再由surface/indicator/label/detail leaf分别判断clip。每个可见row约5–6次metrics投影并复制label/description；panel/search也分散读取metrics/palette，search text每paint复制。长候选集即使只显示少量结果也按完整row_count clone/iterate。

PERF-MVP-211要求Editor08发布immutable command catalog generation与typed search/index result，只共享entry handles和bounded visible/top-K ids，不重复materialize完整catalog。EditorUI08按fixed row metrics在`row_data`前计算visible+overscan，并为changed row编译`CommandPaletteRowPaintSpec`；panel/search共享一次frame theme/metrics。不得用painter-local catalog cache替代Editor08 owner。

## 动态验收

在1/100/10,000 commands、open/close 100次、query 1,000 keystrokes、0/1/20/1000 results和scroll top/middle/end记录catalog builds/owned bytes、when eval、search visited/comparisons、row_data clone、visible/offscreen visits、theme/metrics/text/build/commands与input p95。Stable catalog open不深clone完整entries；query工作由匹配结果而非重复catalog materialization决定并有top-K/visible预算；paint visited=visible+overscan且offscreen clone=0。保持enabled/when、selection/focus/commit、search/empty、row detail、clip/order和pixels parity。
