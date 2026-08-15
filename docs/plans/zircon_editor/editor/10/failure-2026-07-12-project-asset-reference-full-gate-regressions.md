---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: project-asset-reference-full-gate-regressions
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/10
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/workbench/project
tests:
  - cargo test -p zircon_editor --lib --locked project -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked asset_references -- --test-threads=1
---

# Editor 10：当前全量门 ProjectAuthority / AssetRef 回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor03 / Editor08 M1 统一行为门
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：工程与资产引用业务不属于命令注册或事务内核；必须由 Editor10 在唯一 ProjectAuthority / AssetRef 权威上精确归因。

## 失败现象与复现证据

Windows 受管 job `520d85713df249afae31661a7697ad07` 使用 `cargo test -p zircon_editor --lib --locked --jobs 1` 完成编译并进入测试后，至少 10 个 ProjectAuthority / AssetRef 归属用例失败：工程边界守卫、场景/动画/物理引用追踪、创建与打开、损坏 workspace 回退、preset 文件往返、welcome 回退，以及工程文档 roundtrip/renderable-template scaffold。原始失败名保存在 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`。

测试 harness 随后在全包资源停滞中被终止，未生成逐项 panic summary；因此本记录不臆测产品实现与旧断言谁错误。命令 registry、when/palette 与事务引擎专属用例在同一 binary 中已通过，不能把本组失败归到 Editor03/08。

## 最低共享层根因

当前可证实的最低归属是 Editor10 的 `ProjectAuthority`、工程文档/模板和 AssetRef/reference tracking 合同。精确根因须由功能 owner 以 fully-qualified exact 单线程复现取得 assertion/typed error 后再细分；不得在上层命令或宿主测试中吞错。

### 2026-07-12 22:44 精确复现补充

Editor 15 将当前 test binary 按顶层 owner 拆分后，`core::` 65 tests 得到 64 passed / 1 failed。
唯一失败为
`core::project::tests::boundary::project_authority_core_has_no_ui_dependency_or_retired_template_generator`：
`zircon_editor/src/core/project/tests/boundary.rs` 递归扫描 `core/project/**/*.rs` 时没有排除自身，导致测试源码
中声明 forbidden needles 的字符串字面量被当成 6 个生产违规：`crate::ui`、`super::ui`、
`DEFAULT_PBR_WGSL`、`DEFAULT_CUBE_OBJ`、`library_root(`、`runtime_cache_root(`。

这提供了此前缺失的 exact panic summary。Editor10 owner 应让结构守卫只检查生产源码，或以不自命中的
结构化方式定义规则；不得删除真实禁用词检查，也不得恢复退休路径。

## 架构修复验收

- 分别运行 project boundary、bootstrap/startup、workbench project roundtrip 与 asset references exact 组，记录每项真实 assertion/typed source。
- 保持 `ProjectAuthority`、manifest v2、`.zircon/` 与 GUID-first `AssetRef` 单一权威；如断言仍要求退役路径/DTO，应硬切测试而不是恢复兼容面。
- 聚焦组全绿后重新运行 `cargo test -p zircon_editor --lib --locked --jobs 1`，并确认 Editor03/08 gate 不再被本组阻断。

## 禁止临时方案

- 禁止恢复旧 UI startup project DTO、`library/`、`.zircon-cache`、路径 ID fallback 或第二份 asset reference registry。
- 禁止批量忽略失败、把 typed error 改成空成功，或在 Editor08 command handler 中复制工程业务逻辑。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor10 M1/M2 / Editor03+08 M1 | ProjectAuthority / AssetRef 当前全量门回归 | `open-待功能owner精确复现` | 2026-07-12 | job `520d85713df249afae31661a7697ad07` 完成编译并复现至少 10 个 project/reference 失败；原始日志 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`，全包因线程/资源停滞未形成 panic summary。 |
| Editor10 M1/M2 / Editor15 M1 | ProjectAuthority core boundary exact failure | `open-已取得精确根因` | 2026-07-12 | 当前 Editor test binary 的 `core::` 分片 64/65；唯一失败为 boundary guard 扫描自身并命中 6 个 forbidden needle 字面量。 |
| Editor10 M1/M2 / Editor09 M1 | 当前源码完整门停滞前复现 | `open-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 再次记录 ProjectAuthority boundary、asset references 与两个 workbench project roundtrip 失败；asset-reference exact 为 `left=[]`、`right=6 GUIDs`。日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`；不在 Editor09 复制 ProjectAuthority/reference registry。 |
| Editor10 M1/M2 | 当前源码功能修复 | `open-实现与静态门完成-独立复审/受管行为门待执行` | 2026-07-17 | boundary guard 已排除自身测试目录且保留全部生产禁用词；ProjectAuthority generation、document roundtrip 与 AssetRef consumer 当前源码已收束，详见 `2026-07-17-project-authority-reference-regression-closeout.md`。未取得 current-source exact Cargo 输出前不改 fixed。 |
| Editor10 M1/M2 | clean-HEAD 不可变完整门 | `open-目标测试未执行-上层编译阻断` | 2026-08-05 | 协调器 validation-copy job `ac4153eb48e74c5bb47194928a5772f8`、run `459c4c003f4140e5babeb8c04a8d3b8f` 固定 HEAD `94cf43ec3` 与输入哈希 `e19f08dcdad2a54765b4795551cc5998a23ba1b8c9d1559dcc3a0a50c5824346`，显式包含 17 个 renderable-empty 模板和 pinned `zr_vm` 外部源；Runtime 成功编译后，Editor test binary 因 686 个无关编译错误退出 `101`，`0 tests`。首个 `project` 聚焦组未执行，后续 `asset_references` 与全量门被 `&&` 正确阻止；因此不执行 `failure return`。 |

## 修复结果与回传

- 状态：`open / 待修复`；Editor10 完成 exact 归因和修复后在本文件回填验证，并向来源 Editor08 回传。
- 2026-08-05 的不可变副本已排除模板遗漏、外部本地依赖和共享工作树脏改动三类环境干扰；
  剩余阻塞是 clean HEAD 的 Editor test binary 编译失败。声明的 project/reference 行为门仍未开始，
  本 handoff 保持 `open`。
