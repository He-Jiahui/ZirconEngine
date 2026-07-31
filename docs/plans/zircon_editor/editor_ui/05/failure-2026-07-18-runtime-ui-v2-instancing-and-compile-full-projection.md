---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-v2-instancing-and-compile-full-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/surface_builder.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - dev/slint/internal/interpreter/dynamic_item_tree.rs
tests:
  - prototype validation once-per-generation counter
  - overlapping component root linear compile test
  - preview stable-generation zero-artifact-rebuild test
---

# Runtime UI v2实例化与编译重复全图投影

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：v2 component instancer/compiler/surface builder审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 联动责任：EditorUI04消费compiled style artifact；EditorUI02消费typed layout/slot contract。
- 交接原因：prototype store、compiled artifact及preview/hot-reload ownership由EditorUI05统一拥有。

## 失败现象与复现证据

PERF-MVP-273/274与306/312：每component instance重复validate prototype、线性扫imports、clone definition/stack/patch；当前`template/asset` prototype frame还为每child复制node、mounts、token/param maps与slot fill。compiler对root和每个component root分别DFS overlapping nodes，随后arena/component graph/surface/runtime tree多层复制metadata、path与TOML payload。

## 最低共享层根因

prototype graph没有generation级validated/indexed representation，compiled output也没有canonical owner与root/component handle/range，导致每阶段靠owned BTreeMap/String/TOML projection传递。

## 架构修复验收

- 每prototype generation验证一次并建立qualified component index；instance resolve近O(1)，task只携带Arc context+handle/cursor。
- canonical compiled arena一次O(N+E)遍历建立root/component/source/control/slot索引，重叠roots不重复DFS。
- surface/preview持Arc artifact与必要runtime delta，稳定generation artifact rebuild=0，同payload authoritative owner=1。
- 1/100/10k nodes/instances、1/100 components记录graph visits、import probes、clone bytes、peak RSS与compile/preview p95。

## 禁止临时方案

- 不得仅把BTreeMap换HashMap而保留每instance validation和每root全图投影。
- 不得让preview、runtime surface和persistent writer各持一份独立compiled graph。

## 修复结果与回传

Open state: `等待EditorUI05回传validated prototype DAG、canonical compiled arena和preview规模证据`。
