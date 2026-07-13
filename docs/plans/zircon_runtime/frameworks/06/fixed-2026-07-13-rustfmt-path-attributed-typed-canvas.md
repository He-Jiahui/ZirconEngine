---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: rustfmt-path-attributed-typed-canvas
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/ui/retained_host/ui.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/typed_canvas.rs
tests:
  - cargo fmt -p zircon_editor -- --check
  - cargo fmt --all --check
resolved_at: 2026-07-13
---


# Editor Layout 15：path-attributed projection 的 typed-canvas rustfmt 解析失败

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：Frameworks 06 M1 Windows testing stage / G3 fmt gate
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：最低共享原因位于 Layout 15 正在实现的 typed Preview Timeline / Workbench projection 模块挂载；Frameworks 06 只拥有 fmt 守卫与 CI 接线，不拥有 Editor retained projection 的模块树。

## 失败现象与复现证据

2026-07-13 Windows 当前工作树执行：

```text
cargo fmt -p zircon_editor -- --check
```

稳定 exit 1：

```text
Error writing files: failed to resolve mod `typed_canvas`:
zircon_editor/src/ui/retained_host/ui/typed_canvas.rs does not exist
```

同一问题也令 `pwsh -NoProfile -File tools/check-conventions.ps1 -Only fmt -Json` 返回非零。单独对 `workbench_window_projection.rs` 执行 `rustfmt --edition 2021 --check` 为 exit 0，说明格式内容本身不是根因；失败只在 Cargo 从 `ui.rs` 的 path-attributed module tree 解析时出现。

## 最低共享层根因

`ui.rs` 用 `#[path = "ui/workbench_window_projection.rs"] mod workbench_window_projection;` 挂载文件，而该文件又用普通 `mod typed_canvas;` 声明 child。当前真实 child 位于：

```text
ui/workbench_window_projection/typed_canvas.rs
```

Cargo rustfmt 从 path-attributed parent 解析 nested module 时却寻找：

```text
ui/typed_canvas.rs
```

因此 Rust 编译图可以继续推进，但全库 fmt 守卫无法解析模块树。这是模块挂载路径单源问题，不是格式规则误报。

## 架构修复验收

- 保持 `typed_canvas.rs` 为 `workbench_window_projection/` 下唯一 child owner，并让 rustc 与 cargo fmt 通过同一个显式模块挂载解析它。
- `cargo fmt -p zircon_editor -- --check` exit 0，且不再寻找 `ui/typed_canvas.rs`。
- `cargo fmt --all --check` 不再报告该 Editor 模块解析失败；若仍有其他 owner 的格式差异，应按实际 owner 单独报告。
- 运行 Layout 15 typed canvas / Preview Timeline focused tests，确认投影行为没有因模块挂载修复改变。

## 禁止临时方案

- 禁止复制或移动实现到 `ui/typed_canvas.rs`，禁止保留两个 typed-canvas owner。
- 禁止增加 re-export、compat module、空占位文件、cfg 绕过或跳过 Editor 的 fmt。
- 禁止弱化 Frameworks 06 的 `cargo fmt --all --check` 或在 CI 使用 `continue-on-error` / `|| true`。

## 修复结果与回传

- 根因：path-attributed parent 中的普通 nested mod 让 cargo fmt 从 ui 根层解析 typed_canvas，和真实 directory-backed child owner 不一致
- 架构修复：在 workbench_window_projection.rs 显式挂载 workbench_window_projection/typed_canvas.rs，保持唯一 child owner，不增加 shim、复制文件或兼容模块
- 验证：cargo fmt -p zircon_editor -- --check exit 0；受管 zircon_editor --lib --no-run job 3c153af7879543cfb4f1d8b2cf6529ef exit 0；typed Preview Timeline 1/1、timeline strip 3/3
- 回传：Editor Layout15 模块挂载已硬切到唯一显式 child 路径，fmt 解析与 rustc 编译图重新一致，交接回迁 Frameworks06 fixed 归档
