---
related_code:
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_sources:
  - dev/godot/editor/debugger/editor_debugger_server.h
tests:
  - 1191-id unique registry/index parity RED then GREEN
  - existing preview-action route sample suite and Windows focused Cargo pending
  - 1k known/unknown action lookup profile pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Workbench Preview Actions 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

根文件 `workbench_preview_actions.rs` 与子目录 `workbench_preview_actions/extensions.rs` 共 **2** 个 Rust 文件，已逐文件阅读 **2/2**。常量清单共 **1,191** 个唯一 action id；extension 文件仅声明静态切片。动态 Cargo/action storm 未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- 旧 `is_workbench_preview_action()` 对 root 303 项和 extension 888 项依次 slice `contains`；unknown 或尾部 action 每次最多比较 1,191 个字符串。该函数位于 editor event、common callback 与 componentized Workbench 三条 dispatch 路径。
- 已用 `LazyLock<HashSet<&'static str>>` 从现有两个静态切片一次建索引；稳态 lookup 不分配、均摊 O(1)，清单仍是唯一可读真源。现有 uniqueness test 扩展为索引长度 parity，源码 RED→GREEN。
- Godot 的 editor debugger protocol registry同样用 `HashMap<StringName, handler>` 表达稳定名称→callback 查找；Zircon 使用静态 `&str` 避免 intern/owner 扩张。

## 待验收

运行 preview-action registry 与三条 dispatch focused tests；对首项、尾项和 unknown 各 1k 次记录 lookup p95、比较/分配计数，并确认 Lazy index 只建一次。通过前不进入 `review.md`。
