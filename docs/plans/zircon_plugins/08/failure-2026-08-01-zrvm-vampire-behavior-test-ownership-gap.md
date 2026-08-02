---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: zrvm-vampire-behavior-test-ownership-gap
origin_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_runtime/runtime/10
fixing_child_dir: docs/plans/zircon_plugins/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_zr_vm_language_runtime -Features backend-zr-vm -LibTests -TestFilter vampire
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -NoDefaultFeatures -LibTests -TestFilter dynamic_api::session::tests
---

# Plugins08：Vampire real-VM 行为测试 owner 缺口

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 来源执行切片：2026-08-01 ignored tests、源码门禁与 dead-code 静态清理审阅
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：Runtime10 的旧动态 session 测试声明真实 ZrVM 覆盖已迁至 Plugins08，但插件 owner 没有等价 Vampire 行为测试；必须先补齐真实 owner，才能删除旧 ignored tests 与 support code。

## 失败现象与复现证据

`frame_diagnostics.rs`、`vampire_gameplay.rs`、`vampire_hud.rs` 与 `vampire_menu.rs` 合计保留 10 个 `#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]` 测试。它们覆盖 W 输入、自动攻击、敌人行为树、击杀循环、world HUD、长期 tick、capture、开始菜单、game-over retry 与 runtime diagnostics。当前 `zircon_plugins/zr_vm_language` 全树没有引用 `examples/vampire`、`gameplay.key_pressed`、`gameplay.menu_state` 或对应 `vampire_project_session_*` 行为测试。

这些 Runtime 测试仍通过 `mod.rs` 进入每次 lib-test 编译。四个 test 文件连同 508 行的 `vampire_runtime_support.rs` 共 1,263 行；support 中大多数 Vampire fixture helper 只服务 ignored tests，但 `diagnostic_current`、`diagnostic_series`、`small_headless_frame_request` 仍被 `frame_diagnostics.rs` 的非 ignored headless diagnostics 消费。因此 support 不能整文件先删：必须先把这三个通用 helper 移到小型 current owner。其余现状既没有可执行回归价值，又不能安全删除，因为声明中的替代覆盖并不存在，Runtime15 当前结构守卫还把 support owner 当作必须存在的命名锚。

## 最低共享层根因

测试 owner 的迁移只改了 `#[ignore]` 原因，没有完成“新 owner 先建立等价行为验收、旧 owner 后删除、结构计划同步”的原子 hard cutover。Runtime10 因此继续承担不会运行的 real-VM fixture 编译成本，Plugins08 的 M4 real backend 状态又缺少 Vampire 产品行为证据，Runtime15 则固化了本应退休的中间 owner。

## 架构修复验收

- Plugins08 在 `zr_vm_language` real backend 下建立 generation-owned Vampire fixture，覆盖 gameplay 输入/攻击/AI、HUD/menu、diagnostics 与至少一条真实 capture/product path；不得只复制源码字符串断言。
- 新测试在 managed Windows `backend-zr-vm` lane 有精确 executed count 与 GREEN，默认无 real backend 的插件矩阵继续可编译。
- 等价覆盖 accepted 后，Runtime10 删除 10 个 ignored tests；保留的 WASD source-policy test 移到不依赖 VM fixture 的小 owner；先迁出三个通用 diagnostics helper，再在 support 无消费者时删除 `vampire_runtime_support.rs`。
- Runtime15 同步移除“support 文件必须存在”的命名/状态锚，改为禁止旧 owner 回流；Runtime10/Plugins08 计划各保留 fixed return。

## 禁止临时方案

- 禁止把 ignored tests 改名、继续永久 ignore、只删除 `#[ignore]` 后依赖外部未配置 VM，或用 source-string marker 冒充真实 VM 行为。
- 禁止在 Runtime 与 Plugins08 同时维护两套 Vampire fixture、兼容 re-export 或测试专用 fake backend。
- 禁止先删除旧规格再承诺未来补测；hard cutover 顺序必须是新 owner GREEN、结构锚迁移、旧 owner 删除。

## 修复结果与回传

Open state：已完成静态 inventory、consumer scan 与 owner 核对；未删除任何测试或 support code。Plugins08 当前无 active primary Session，本记录保留到 real-backend 等价覆盖、Runtime10/Runtime15 清理和 current-source managed 验收完成。
