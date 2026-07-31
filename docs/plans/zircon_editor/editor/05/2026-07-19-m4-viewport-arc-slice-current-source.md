Plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
Milestone: M4
Status: in_progress
Files: ["zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs", "docs/plans/zircon_editor/editor/05/failure-2026-07-19-viewport-shared-extract-arc-slice-iteration-compile-regression.md"]

# Editor05 M4 Viewport Arc Slice 当前源记录

## Scope Delivered

- `precision_candidates_from_layout` 对 handle、scene gizmo、renderable 三个共享 `Arc<[T]>` 容器统一使用 slice `.iter()`，关闭 `&Arc<[T]>` 的 E0277 consumer 漂移。
- 未 clone `Arc` 内容、未回退 `Vec<T>` 双轨、未改变 handle -> gizmo -> renderable 候选追加顺序。
- 编译修复只属于 Editor05 pointer candidate consumer；同文件既有 projection/capacity 改造保持其原 Editor05 owner，不在本记录中重复声明。

## Fresh Testing Evidence

- 当前源码 SHA-256 `c7594435c3496c5e5b4bcb044b69ae5eabd4a23552cfef8fccf2805c61eae10b`，与 snapshot `559` 完全一致；snapshot `577` 再次固定同一源码与更新后的 failure 记录。
- `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs`：通过。
- `git diff --check -- zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs docs/plans/zircon_editor/editor/05/failure-2026-07-19-viewport-shared-extract-arc-slice-iteration-compile-regression.md`：通过。
- viewport focused / `zircon_editor --lib` Cargo：未执行；原 Editor15 job 为 source-raced diagnostic，禁止计入本切片验收。

## Review

- 当前源静态自审确认三个 Arc slice consumer 迁移完整，候选过滤与 push 顺序不变，也没有引入内容复制。
- 独立复审尚未执行；本记录在 current-source Cargo 与 independent Critical/Important 清零前保持 `Status: in_progress`。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 未完成项 |
| --- | --- | --- | --- | --- |
| 2026-07-19 05:31 +08:00 | M4 pointer candidate Arc slice consumer repair | `源码完成 / 静态门通过 / 受管验收待办` | successor Session `editor05-viewport-arc-slice-compile-repair-r2-20260719` 声明 exact3；源码从 snapshot `559` 到 `577` 零漂移，failure 子计划已同步真实完成状态。 | 等待 Coordinator01 immutable validation-copy fixed return；随后运行 viewport focused 与 fresh `zircon_editor --lib`、独立复审、`Status: accepted`、failure return 与受管提交。 |
