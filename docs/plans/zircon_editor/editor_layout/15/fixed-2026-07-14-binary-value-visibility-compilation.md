---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: binary-value-visibility-compilation
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_editor/editor/11
related_code:
  - zircon_runtime_interface/src/serialization/binary/mod.rs
  - zircon_runtime_interface/src/serialization/binary/value/mod.rs
  - zircon_runtime_interface/src/serialization/binary/value/binary_value.rs
  - zircon_runtime_interface/src/serialization/binary/value/error.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace/preview_viewport.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput
  - focused Blend Space Preview contract and ignored screenshot capture after Editor11 repair
resolved_at: 2026-07-14
---


# Editor11 M3.1：BinaryValue 可见性编译阻断

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：S15.4/S15.5 Blend Space Preview toolbar 原子组件密度与当前源码截图门禁。
- 修复责任计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 修复责任范围：Editor11 M3.1 Binary 版本壳。
- 交接原因：布局切片必须先通过当前源码的共享 Runtime Interface 编译面，不能借用旧 editor 测试二进制、移除 Binary 接线或把错误转成 editor 私有兼容层。

## 失败现象与复现证据

2026-07-14 在 Windows coordinator 受管 target 上执行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput
```

命令已经越过先前的 Runtime Text 构造器诊断，但在编译 `zircon_runtime_interface` 时失败，因此未执行本切片 focused contract，也没有刷新任何截图。

`serialization/binary/mod.rs` 尝试向 `serialization` 父域重导出 `BinaryValue`，`value/mod.rs` 又重导出 `BinaryValue`/`BinaryValueError`，但叶子声明仍为 `pub(super)`。编译器因此报告 11 个 `E0364`、`E0365` 与 `E0603`，涉及 `binary/mod.rs`、`encode.rs`、`envelope.rs` 和 `decode.rs`：父 owner 的 re-export 范围超过叶子声明的可见域。

## 最低共享层根因

这是 Editor11 M3.1 将二进制 wire 拆为 `binary/{encode,decode,envelope,value,wire}` 后的 owner-tree 可见性收束不完整。`BinaryValue` 与 `BinaryValueError` 是 Binary owner 的内部协作类型；其叶子和 owner 根没有以同一受限域声明，导致 `zircon_runtime_interface::serialization` 无法通过新 root 消费，早于 Runtime、Editor retained host 和布局截图执行。

## 架构修复验收

- 只将 `BinaryValue` 与 `BinaryValueError` 提升到 Binary owner 及其父 `serialization` 实际需要的最窄可见域，并使各层 re-export 与该域一致。
- 保持 `serialization` 的公开 API 中性；不得把二进制内部值类型提升为 `pub`/`pub(crate)`，不得添加旧 `UnsupportedFormat(Binary)` 分支、兼容 reader 或 Editor facade。
- 运行 Editor11 M3.1 的受管 `zircon_runtime_interface` 验证，确认上述 `E0364`/`E0365`/`E0603` 不再出现；随后回传来源 Layout 15，重跑受管 editor 包验证、focused Preview 合同和三档 ignored capture。

## 禁止临时方案

- 不得在 Layout 15 跳过当前源码 Cargo、复用 7 月 13 日测试二进制或把旧 PNG 作为本次证据。
- 不得删除 Binary M3.1 接线、放宽成 crate-wide/public 可见性，或在 editor 添加 JSON-only 条件旁路。
- 不得向任何 `target` 目录写截图；恢复后验证图只能写入 `docs/tests/editor`。

## 修复结果与回传

- 根因：BinaryValue and BinaryValueError leaf declarations used pub(super) while their value owner re-exported them to crate::serialization, exceeding the leaf visibility domain
- 架构修复：Raised both leaf declarations and value-owner re-exports only to pub(in crate::serialization); kept binary-root test helpers behind cfg(test) and did not expose either type publicly or crate-wide
- 验证：Windows managed interface build job 45fb57fca3a54bc89206a871df0c3862 exited 0; serialization focused contracts passed 32/32 in the fixing Session; scoped rustfmt and git diff checks passed; handoff validator reported 115 artifacts and 0 errors
- 回传：Layout15 may resume current-source editor validation and focused Preview capture because the Runtime Interface binary owner now compiles with its narrow visibility contract
