---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: dynamic-scene-format-version-root-export-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_editor/editor/11
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document/schema.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --no-default-features --features target-client --locked --offline --jobs 1 --no-run
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-14
---


# Editor11：DynamicScene version hard-cut root export drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 FR-M2 post-review current-source product framebuffer 与 Editor02 upward regression
- 修复责任计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 交接原因：Editor11 正在持有 DynamicScene version shell hard-cut 文件租约；Text01 的两个当前源编译均在进入文本断言前被同一 owner 边界错误中断。

## 失败现象与复现证据

受管 Windows GPU job `ca3e2c917bc849c1b35d4d6447f8205c` 与 lib-test job `4d34f26189144bf4afd056ee2dd10bfd` 分别执行产品 exporter 和 `scene::` upward gate。二者均在 `zircon_runtime/src/scene/mod.rs:69` 报 `E0432`：

```text
unresolved import dynamic_scene::DYNAMIC_SCENE_FORMAT_VERSION
no DYNAMIC_SCENE_FORMAT_VERSION in scene::dynamic_scene
```

GPU job 在 17m43s 后以 1 error / 85 warnings、exit 101 结束；lib-test job 以同一唯一错误 / 359 warnings、exit 101 结束。Text01 的 post-review compile job `d4fd827abfc3450090d20275d91b57ee` 在 Editor11 本轮改动前已退出 0，因此失败不属于字体或文本模块。

## 最低共享层根因

Editor11 已在 `dynamic_scene/document/schema.rs` 把格式版本收口为 `VersionedSchema::VERSION` 与 typed `PayloadHeader`，并从 `dynamic_scene/mod.rs` 删除旧 `DYNAMIC_SCENE_FORMAT_VERSION` re-export；但上层 `scene/mod.rs` 仍尝试公开重导出该已删除符号。hard-cut 的新 owner 已建立，root public surface 没有同步收束。

## 架构修复验收

- `scene/mod.rs` 与全部消费者迁移到 Editor11 当前 typed schema/header owner，不恢复已删除的 public format-version 常量。
- 不新增 compatibility re-export、alias 常量、双版本真相或 Text01 条件编译绕过。
- 先运行 Editor11 的 DynamicScene version/serialization focused tests，再确认上述两条 Text01 current-source Cargo 命令不再出现该 E0432。

## 禁止临时方案

- 禁止在 `dynamic_scene/mod.rs` 或 `scene/mod.rs` 重新声明 `DYNAMIC_SCENE_FORMAT_VERSION` 兼容常量。
- 禁止修改 Text01 测试、feature gate 或产品 exporter 来避开 scene root 编译。
- 禁止让 runtime root 直接读取私有 schema module，必须由 Editor11 设计的 typed public contract 承接真实消费者。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor11 M2.2 | DynamicScene version shell hard cut | `未通过-待-editor11-owner-修复` | 2026-07-14 | 两条不同 feature surface 的受管 Cargo 编译均只有同一 `scene/mod.rs:69` E0432；Text01 路径无诊断。 |

## 修复结果与回传

- 根因：Editor11 removed the obsolete DynamicScene format-version export from the typed schema owner, but scene/mod.rs still re-exported the deleted DYNAMIC_SCENE_FORMAT_VERSION symbol.
- 架构修复：Removed the stale scene-root re-export and kept VersionedSchema/PayloadHeader as the only typed version contract; no compatibility constant, alias, or Text01 bypass was restored.
- 验证：Managed default-feature zircon_runtime lib-test compilation completed and launched without E0432; managed target-client product job d80d6dabac754907b50aa3ae2c1c1056 then exited 0 with 1 passed/0 failed, proving the former scene-root compile drift is absent on both feature surfaces.
- 回传：Editor11 hard-cut root export drift is fixed and validated upward; Text01 product execution is unblocked.
