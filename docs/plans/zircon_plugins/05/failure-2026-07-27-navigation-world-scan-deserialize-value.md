---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: navigation-world-scan-deserialize-value
plan_link_mode: child_record_only
origin_plan: docs/plans/mvp/04-f3-persistence.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/mvp/04
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - zircon_runtime/src/navigation/runtime/world_scan.rs
tests:
  - "pwsh -NoProfile -File .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter post_persist_save_sync_failure_is_diagnostic_only"
---

# Navigation 05: world scan Deserialize Value 编译失败

## 来源执行者

- 来源计划：`docs/plans/mvp/04-f3-persistence.md`
- 来源执行切片：F3 managed `zircon_editor` focused validation
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：`world_scan.rs` 由 Navigation 05 的 typed projection owner 持有；Editor 仅经 runtime 依赖链暴露该基础层错误。

## 失败现象与复现证据

2026-07-27 的 Windows coordinator managed run 在编译 `zircon_runtime` 时失败：

```text
error[E0599]: the method `as_ref` exists for reference `&serde_json::Value`,
but its trait bounds were not satisfied
  --> zircon_runtime/src/navigation/runtime/world_scan.rs:230:67
```

同一错误也出现于 obstacle descriptor 的第 245 行。复现命令见 frontmatter；它在运行 F3 测试之前的 package build 阶段失败。

## 最低共享层根因

`World::dynamic_component_rows` 返回的 row value 已经是 `serde_json::Value`。`NavMeshAgentDescriptor::deserialize(value.as_ref())` 与 `NavMeshObstacleDescriptor::deserialize(value.as_ref())` 错把 `serde_json::Value` 当作实现 `AsRef` 的包装类型，因此无法通过 Rust 类型检查。这里需要保持 projection 读取单一 dynamic-component snapshot，而不是让 Editor 或上层测试引入临时反序列化分支。

## 架构修复验收

- Navigation owner 以直接的 `&value`（或等价 `&serde_json::Value`）完成两处 descriptor 反序列化，保持 current-generation projection 的单次读取语义。
- 运行 Navigation projection focused tests，覆盖 agent 与 obstacle 行。
- 重跑本记录的 managed Editor reproduction，并向上恢复 F3 document/save focused validation。

## 禁止临时方案

- 不在 Editor、F3 测试或调用者增加兼容 alias、fallback、test-only bypass 或第二次 component scan。
- 不删除或弱化 typed projection 测试来掩盖该编译错误。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
