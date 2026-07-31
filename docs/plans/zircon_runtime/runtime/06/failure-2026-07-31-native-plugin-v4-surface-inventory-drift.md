---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: native-plugin-v4-surface-inventory-drift
origin_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
fixing_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
origin_child_dir: docs/plans/zircon_runtime/runtime/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/mirror_docs.rs
  - zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - tools/tests/test_runtime06_native_plugin_surface_inventory.py
tests:
  - python -B -m unittest tools.tests.test_runtime06_native_plugin_surface_inventory -v
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary.RuntimeTechStackBoundaryTests.test_runtime_06_current_backend_command_and_folder_guard_owners_are_clean -v
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary -v
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --exact --nocapture --test-threads=1
---

# Runtime06: V4 native plugin public-surface inventory drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 来源执行切片：Runtime01 Kira/Sound owner inventory 的完整 `runtime_tech_stack_boundary` 上行门。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 交接原因：失败全部落在 Runtime06 的 native plugin public-surface 分类、App 调用面清单和 Rust mirror；Runtime01 不应通过放宽 tech-stack 门掩盖 Runtime06 漂移。

## 失败现象与复现证据

已集成当前源的 Runtime01 上行门最初为 7/8，唯一失败是 `plugin_surface_lifecycle_boundary_audit`：

```text
native_namespace_reexport_count: actual 74, expected 68
native_namespace_symbol_group_count: actual 6, expected 5
unclassified_native_namespace_symbols:
  NativeHostApiV4RegistrationPolicy
  NativeHostApiV4RegistrationScope
app_native_plugin_file_count: actual 8, expected 7
risks: 4
```

新增 App 调用点是 `zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs`。新增 V4 policy/scope 承载完整 runtime-interface host API capability/resource authority，不是 bridge-method 专属合同；它们需要稳定的 host-API adapter owner，也不能通过扩大 `plugin` 根公共面解决。

## 最低共享层根因

Runtime06 的审计清单仍冻结在 native namespace 68 个导出和 7 个 App 调用文件；父计划还把四个不同边界压成“单一 V3/behavior-host V4”二分。当前事实是 descriptor/entry 与 plugin-to-host callback table 为 V3，behavior callback table 与 runtime-interface host API 当前面为 V4；`NativeHostApiV3RegistrationScope` 是独立旧 adapter debt。V4 policy/scope 未进入真实 host-API adapter 分类，导致合法公共面被错误标成 unclassified。与此同时 Rust mirror 仍钉住旧计数，三个 V2 byte DTO 仍作为物理类型存在、V3 名称仍是 alias。

## 已完成的前向修复

- public-surface classifier 已建立 `native-host-api-adapter-public-debt` 稳定 owner，并把 V3/V4 registration scope/policy 从 bridge-method 组迁入；没有恢复 root re-export。
- lifecycle audit 当前期望值更新为 source files 17、native namespace 74、symbol groups 6、App call-site files 8；风险聚合现在比较实际 source count，消除 17/expected14 false-green。
- root hard-cut scanner 已从单一 `native_plugin_loader::{...}` 文本形态扩展到完整 `pub use` statement，负例覆盖 `native::{...}`、`self::native::*` 与 crate-qualified re-export。
- Runtime06 父计划已改为 V3 descriptor/entry + host-function-table、V4 behavior + runtime-interface host API 的精确矩阵；M2 登记为源码硬切完成但 managed 验收待关闭。
- M3 已重新打开：V2 byte DTO 物理类型与 V3 alias 必须删除，不得以 compatibility、冻结或长期别名名义保留。
- 第二审查修复后的当前源证据：focused inventory 3/3、Runtime06 standalone boundary 1/1、完整 runtime tech-stack boundary 8/8，Python py_compile 通过；这些仍是静态证据，不替代 Rust/Cargo hard-cut 验收。

## 尚未完成与后续动作

- `zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/mirror_docs.rs` 仍钉住 source 14、namespace 68、groups 5、App 7 与旧日期；必须在 Runtime source quiet window 内原子更新为 2026-07-31 与 `17/74/6/8`，并通过 managed Rust 1.94.1 focused mirror gate。
- M3 必须把 V2 byte-slice/buffer/callback-status 实体迁成唯一当前 V3 物理类型，删除 V2 public names 与 V3 aliases，并删除旧 `NativeHostApiV3RegistrationScope`；当前 `NativePluginHostFunctionTableV3` 仍是 entry ABI 的合法 callback table，不得误删。
- M2/M3 新增的 world-runtime-extension 并发门和 native callback panic/global-hook 门仍需 managed current-source 证据。
- 在上述 Runtime Rust 切片、二次审查和 managed gate 完成前，本 failure 保持 `open`，Runtime06 保持 `in_progress`。

## 禁止临时方案

- 不得把预期值退回 68/7、忽略新增 App 测试文件、放宽 `risks` 或允许 unclassified symbol。
- 不得把 V4 policy/scope 暴露回 `zircon_runtime::plugin` 根、误归 bridge-method，或用一次性 catch-all 分类掩盖 host-API adapter owner。
- 不得把 V2 物理类型/V3 alias 宣称为长期兼容策略；禁止恢复 V1/V2 entry、descriptor、loader fallback、alias 或 shim。
- 不得用 Python focused 3/3 或 tech-stack 8/8 代替 Rust mirror、native/plugin workspace 和 Runtime06 全量验收。
