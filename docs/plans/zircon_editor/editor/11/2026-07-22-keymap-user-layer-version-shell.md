---
owner_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
milestone: M2
slice: keymap-user-layer-version-shell
status: source_complete_static_green_validation_pending
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/persistence.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
  - tests/fixtures/serialization/editor-keymap-user-layer/v0/keymap-user-layer.json
tests:
  - tools/tests/test_editor11_keymap_version_shell_contract.py
  - zircon_editor/src/core/commands/keymap/tests.rs
---

# Keymap User Layer Version Shell

Plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
Milestone: M2
Status: source_complete_static_green_validation_pending
Files: ["docs/zircon_editor/core/keymap.md", "tests/fixtures/serialization/editor-keymap-user-layer/v0/keymap-user-layer.json", "tools/tests/test_editor11_keymap_version_shell_contract.py", "zircon_editor/src/core/commands/keymap.rs", "zircon_editor/src/core/commands/keymap/persistence.rs", "zircon_editor/src/core/commands/keymap/tests.rs"]

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-22 10:31 +08:00 | `source_complete_static_green_validation_pending` | `EditorKeymap` 已增加共享版本壳用户层。内建 TOML 继续作为唯一默认 owner；typed user delta 独立保留重绑、新增与 `null` 解绑，effective bindings 只是排序后的派生投影，不再从 effective/base 反推并丢失暂时缺席插件命令的 tombstone。内建与用户 command id 均经唯一 `EditorOperationPath` 词法 owner 校验。schema 固定为 `zircon.editor.keymap-user-layer` v1，payload 无第二版本字段；真实仓库 v0 fixture 经显式迁移链读取，future version fail closed。应用前验证全部 id/chord，再一次性构造新 keymap；`LoadError`、`WriteError` 与带 `ErrorKind` 的 I/O source 可供 Editor17 分类。 | TDD RED：生产 persistence owner 与记录不存在时，Python 合同 1/4 通过、3/4 失败。首轮 GREEN 为静态 4/4；独立复审 C/I/M=`0/4/0`，指出缺席命令 tombstone、operation path、typed errors 与真实 fixture 四项缺口。r1 在快照前取消，r2 以含 fixture 的 immutable exact-seven scope 接管；新增 10 个 Rust 回归覆盖原六项及 tombstone 跨默认回归、非法 id 原子拒绝、NotFound/写失败、损坏 payload 分类。整改后 Python 静态合同 5/5、exact-three rustfmt check 与 exact-seven diff check 通过；终审 C/I/M=`0/0/0`。managed Cargo 仍待追加，当前不声明 Rust 编译通过。Editor08 的同 chord/when 域冲突诊断与 Editor17 settings 路径/变更事件仍开放，父 M2.1 不勾选。 |

2026-07-22性能补充：不改版本壳/用户delta语义，仅让`chord_for_command`利用effective bindings按command id排序做binary search；key alias normalization与Display删除临时lowercase/parts分配。输入方向的chord→command仍线性扫，归Editor08/PERF-MVP-074 generation index；受管Cargo仍pending，本记录状态不提升。
