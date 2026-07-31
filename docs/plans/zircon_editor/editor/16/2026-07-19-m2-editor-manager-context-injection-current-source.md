Plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
Milestone: M2
Status: in_progress
Files: ["zircon_app/src/entry/entry_runner/editor.rs", "zircon_app/src/entry/entry_runner/editor/tests/cli_operation.rs", "zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs", "zircon_app/src/entry/entry_runner/editor/tests/host_config.rs", "zircon_app/src/entry/entry_runner/editor/tests/mod.rs", "docs/plans/zircon_editor/editor/16/failure-2026-07-18-editor-state-context-constructor-hardcut.md"]

# Editor16 M2 EditorManager Context 注入当前源记录

## Scope Delivered

- `run_editor_operation` 在构造 `EditorState` 前解析唯一 `EditorManager`，并把 `manager.context().clone()` 注入 `EditorState::with_default_selection_with_context`。
- 原 `editor.rs` 内联测试机械拆到 `editor/tests/{cli_operation,gui_startup,host_config}.rs`，模块清单已纳入同一原子 manifest，未改变 CLI 断言语义。
- 未恢复生产可见的无 Context 构造器、兼容 facade、第二套 transaction/jobs/command service owner 或全局静态 Context。
- 对应 failure 已规范化为 `open`，保持 Cargo、独立复审、fixed return 与受管提交未完成的真实状态。

## Fresh Testing Evidence

- `python -m unittest tools.tests.test_editor03_scene_transaction_hardcut_contract -v`：12 passed / 0 failed；覆盖 CLI manager Context 注入、Context transaction engine、旧 scene/gizmo owner 物理删除、唯一 fallible owner、multi-selection、play/project 生命周期与测试结构合同。
- `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check`：`editor.rs` 与 4 个拆分测试模块全部通过。
- `git diff --check -- zircon_app/src/entry/entry_runner/editor.rs docs/plans/zircon_editor/editor/16/failure-2026-07-18-editor-state-context-constructor-hardcut.md`：通过。
- `cargo test -p zircon_app --features target-editor-host --locked`：未执行；Coordinator01 immutable validation-copy 终态证据 failure 仍为 open，禁止用共享工作树裸 Cargo 替代。
- 联合 `zircon_editor --lib` immutable gate 已启动但 inner exit 101 且 stdout/stderr 为空，不能作为 Editor16 编译结论；对应 Coordinator01 failure 为 `validation-copy-nonzero-cargo-output-missing`。

## Review

- 当前源静态自审确认 manager/context owner 单一，CLI operation 参数、operation routing、runtime gateway 与退出码路径未因本修复改变。
- 独立 exact46 终审 C/I/M=0/3/1；三条 Important 与文档 Minor 已由 r7 完成源码整改，增量 source 复审为 C/I/M=0/0/0。本记录因 Cargo 与 failure return 未完成继续保持 `Status: in_progress`，不得用于 milestone commit。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 未完成项 |
| --- | --- | --- | --- | --- |
| 2026-07-19 05:23 +08:00 | M2 CLI operation Context 单源硬切 | `源码完成 / 静态门 5/5 GREEN / 受管验收待办` | exact3 successor Session `editor16-editor-state-context-constructor-hardcut-r2-20260719`；生产源与 canonical failure 的前序 snapshot `578` 无漂移；本记录声明的业务 manifest 仅含两条业务路径，协调器自动把本记录作为第三条 manifest 路径。 | 等待 Coordinator01 validation-copy fixed return；随后执行 target-editor-host Cargo、独立复审、把 `Status` 提升为 `accepted`、failure return 与 milestone commit。 |
| 2026-07-19 10:33 +08:00 | M2 Editor03/Editor16 原子 current-source closeout | `exact31 frozen / Coordinator01 evidence RED` | successor `editor03-editor16-context-hardcut-atomic-closeout-r2-20260719`；snapshot `608`，M2 manifest `4d56cc596a1545c8ade20e56775683c7`；补齐 4 个 CLI 测试模块后 31/31 overlay 零差异，静态合同 6/6、rustfmt/diff guard GREEN。 | inner run `82329a4e961e4ce3ad3894768f9be29c` exit 101 且无 stdout/stderr，副本已删除；等待 Coordinator01 fixed return 后重建副本，再执行 `target-editor-host` Cargo、独立复审与受管提交。 |
| 2026-07-19 11:09 +08:00 | M2 原子 review 整改 | `exact33 source / static 9/9 GREEN` | successor r3 扩展两个 viewport 生产入口；Editor03 选择保持、play 共享门禁、gizmo 失败 rollback 与 host/binding 错误传播完成，CLI Context 注入路径未改变。 | 重新生成 current-source snapshot/manifest 后执行独立复审；`target-editor-host` Cargo 仍等待 Coordinator01 evidence failure 返回。 |
| 2026-07-19 11:57 +08:00 | M2 原子生命周期整改 | `exact46 source / static 12/12 GREEN` | successor r6 物理删除旧 gizmo lifecycle owner、补 project/play/error 路由并拆分 oversized state tests；CLI manager Context 注入路径与 4 个拆分 CLI 测试模块未改变。 | 重新生成 exact46 snapshot/manifest 后执行独立终审；`target-editor-host` Cargo 仍等待 Coordinator01 evidence failure 返回。 |
| 2026-07-19 12:35 +08:00 | M2 排他切换整改 | `exact47 source / static 13/13 GREEN` | exact46 终审 0/3/1 后，successor r7 增加 transaction engine exclusive transition、普通 action pre-capture cancel 与 finally cleanup；新增 locking regression，CLI manager Context 注入路径未改变。 | 重新生成 exact47 snapshot/manifest 后执行独立复审；`target-editor-host` Cargo 仍等待 Coordinator01 evidence failure 返回。 |
| 2026-07-19 12:49 +08:00 | M2 source 终审 | `independent C/I/M=0/0/0 / Cargo pending` | r7 增量复审确认上轮 0/3/1 全部关闭，且 CLI manager Context 注入路径没有类型、隐私、借用或 move 风险。 | snapshot `623` / exact47 manifest `8bb61cd3...`；`target-editor-host` Cargo 仍等待 Coordinator01 evidence failure 返回，不提交。 |
