---
handoff_kind: failure
status: open
created_at: 2026-07-26
summary_slug: native-plugin-load-report-private-fields
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/tests.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib native_plugin_load_report --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader::discover --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked --jobs 1 -- --ignored --exact export_runtime_multilingual_text_product_framebuffer_png --test-threads=1
---

# Runtime06: NativePluginLoadReport private fields block Text01 framebuffer proof

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 MVP 产品 WGPU framebuffer 截图门。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 交接原因：编译错误位于 Runtime06 所有的 native plugin discovery/report 边界；Text01 不应破坏 `NativePluginLoadReport` 的封装或为截图测试建立特例。

## 失败现象与复现证据

Text01 受控 GPU run `987eee5889cf42ccb6b1c88734d5bd38` 于 2026-07-26 启动了原始产品证据命令，但在执行测试前编译失败。stderr 的唯一阻断错误为：

```text
error: cannot construct `NativePluginLoadReport` with struct literal syntax due to private fields
  --> zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs:431:26
  ...and other private field `projection` that was not provided
```

复现命令：

```powershell
cargo +1.94.1 test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked --jobs 1 -- --ignored --exact export_runtime_multilingual_text_product_framebuffer_png --test-threads=1
```

测试没有开始，`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260722.png` 未生成；不得以纯文本或旧图替代该 WGPU framebuffer 证据。

## 最低共享层根因

`discover/authority.rs` 仍直接构造 `NativePluginLoadReport`，而 report 类型新增/保持私有 `projection` 字段。Runtime06 的 report construction API 与 discovery owner 没有同步，导致任何依赖 `zircon_runtime` 的测试目标在 Text01 前失败。

## 架构修复验收

- Runtime06 使用 report owner 提供的构造/默认路径，或在同一 owner 内恢复完整受控初始化；不得暴露字段只为调用方绕过封装。
- Runtime06 focused native-plugin discovery/report 测试与受管 `zircon_runtime` 编译通过。
- 重新执行本 artifact 中的原始 Text01 WGPU framebuffer 命令；它必须实际运行、导出 `docs/tests/runtime/text` 下的产品 PNG，并通过像素/布局断言。

## 禁止临时方案

- 不要将 `projection` 改为 public、添加兼容别名、静默默认 fallback、测试专用初始化或调用点例外。
- 不要跳过 native plugin loader 编译、弱化 Text01 的 framebuffer/像素断言，或用纯文本图替代产品截图。

## 当前前向修复状态

- `NativePluginLoadReport` owner 已提供 crate-scoped `diagnostic_only` 受控构造路径；它只接收诊断、其余字段走 owner 的 `Default` 初始化，因此不会要求 discovery 调用方了解私有 `projection` 缓存。
- raw vectors 已收窄到 native loader 内部；公开消费者通过不可变 `discovered()`、`loaded()` 与 `diagnostics()` 切片读取，native loader integration contract 已迁移到访问器。
- owner regression 覆盖空 discovered/loaded 集合、诊断保留和 `projection()` 的单次初始化身份。
- `discover/authority.rs` 由另一有效租约持有。覆盖该文件和 report owner-private fields 的原子 delayed patch 已通过 `git apply --check`；它将调用侧迁移到 `diagnostic_only` 与 `from_discovery`，最终集成与受管编译回执仍须由持有者协调，不在本会话越权修改。
- report owner 现已为 discovery/loading/live-host 调用侧提供受控构造、消费与变异 API；`take_*`、`push_*` 与 `restore_discovered` 均会清空冻结投影。回归覆盖先冻结再追加诊断的刷新结果，已租约的 `load_discovered` 和 live-host 调用侧不再直接变异原始集合。
- 二次独立审查发现的 entry 保留 P1 已前向修复：runtime/editor entry 的任一加载错误现在只记录诊断，另一 entry 的成功结果仍会与稳定库句柄写入同一 `LoadedNativePlugin`。回归覆盖 sibling 成功结果、错误诊断和最终 `push_loaded` 顺序，避免随后加载 editor entry 时丢弃已成功的 runtime entry。
- 随后独立审查发现的 P2 已前向修复：`try_into_discovered` 以 `Result<_, Self>` 拒绝包含 loaded plugins 的混合 report，hot-update 调用点将此转换为明确失败；回归确认发现/loaded/diagnostics 均随错误 report 保留，不会在 release 下静默释放动态库句柄。
- P2 修复后的第三次独立复核为 0 P1 / 0 P2：成功消费仅移动 discovery candidates，混合 report 返回完整 `Err(self)`；唯一 hot-update 调用先复制只读 diagnostics，再显式传播 discovery-only 不变量违反。
- 二次独立审查发现仍有一个 P1：缓存投影冻结后，native loader 内部的 `discovered`、`loaded` 与 `diagnostics` 仍保留 module-scoped 直接写入路径，特别是 `discover/authority.rs`。完整前向修复必须由 authority owner 应用已验证的 `diagnostic_only` delayed patch，并将 report fields 设为 owner-private；在该有效外部租约完成前，本 failure 保持开放，不能以文档承诺替代实现。
- authority 外的 live-host 诊断路径及 registration/behavior/bridge 测试夹具已全部迁移到受控 accessors 或 `push_loaded`；私有化前置的 scoped format、diff 与 delayed-patch apply 检查均为绿色，但这不是 Cargo 或 Text01 framebuffer 的终态证据。
- 同次审查将一项只校验输出和计数器的测试由“线性”性能断言更名为正确性/确定性断言，避免它对复杂度给出未证明的结论。
- 静态格式与 scoped diff 检查已完成；受管 focused native report 以及原始 Text01 WGPU framebuffer gate 尚未产生终态证据，failure 保持 `open`。
- 2026-08-24 current-source owner hard cut 已完成：`NativePluginLoadReport` 的
  `discovered`、`loaded`、`diagnostics` 与 `projection` 全部收回 report module 私有；
  discovery authority 只通过 `from_discovery` 与 `push_diagnostic` 构造/变异 report，
  discovery regression 只通过 `discovered()` 读取。新增 owner-boundary source contract
  先对四个 visibility、authority literal 与 direct diagnostic mutation 共 6 项 RED，修复后
  6 项清零；exact-four `rustfmt --check` 与 `git diff --check` 通过。受管 Rust 与 Text01
  framebuffer 仍待 terminal evidence，因此本记录继续保持 `open`。

## 修复结果与回传

Open state: `source_repair_confirmed / managed_native_report_and_Text01_framebuffer_validation_pending`。上方旧租约和 delayed patch 描述仅是当时的待办；2026-08-24 owner hard cut 已落入现行源码，不再等待同一补丁集成。受管编译、focused native-plugin 测试和原始 Text01 产品 framebuffer 均无匹配现行源码的完整通过记录；no dynamic pass is claimed.

### 2026-09-24 当前源码与原票据核对

- `discover/authority.rs` 的快照投影调用 `NativePluginLoadReport::from_discovery`，I/O lane 与失败报告走 `diagnostic_only`，追加诊断走 `push_diagnostic`；现行 authority 不再直接构造 report 字面量或写入私有字段。四个字段保留在 report owner 私有作用域，回归源码校验字段封装、受控构造与投影失效。
- 两份生产 owner 的 SHA-256 分别为 `d89ceb7e81bc42b34e31284530fa70ba839e8200888295f7806d93c856d88d67`（authority）与 `0c698a7bebb88303674605b48d46ce40b2a647f460b6a7626c4867b546569198`（report）；`discover/tests.rs` 为 `0bfbc7d0334dfad32fdc5220b351733ad1a43b70cdc8131fc75b546c1388a502`。最初 snapshot 3742 的 `native_plugin_load_report/tests.rs` 为 `1c552d2d351ae35e145a11e0dc3f51c692b1a611d5d708c4e41c0f8e42e38a4c`；复审后此文件的精确修复另记于下。生产源码与 discovery 测试均未编辑。
- 原受管 focused 票据 `7f8e1c2c33244d9f862abe0f784bb29a` 于 2026-08-24 在 validation copy `closure_planning` 阶段因 `validation_copy_compile_time_resource_missing` 失败，缺少 `core/framework/state/machine.rs` 对 `tests/state/hook_index.rs` 的 compile-time resource；它没有跑到 Rust 编译或目标测试，且旧票据 authority 哈希与现行源码不同，不能复用为通过证据。
- 现行 Text01 framebuffer 测试路径属于上行 owner，工作区有其他会话修改；不吸收入 Runtime06 scope。需待可用受管验证通道按上述 focused → 原始 GPU 命令重跑，保留测试实际执行/PNG 像素验收凭据，并完成独立 C/I/M 复审后才可回传/closeout。

### 2026-09-24 独立审查 Moderate 修补

- Snapshot `3742` 的初次只读独立审查为 `Critical=0 / Important=0 / Moderate=1`：report 测试的字段私有性守卫只拦 `pub(...)`，把 `discovered` 改为裸 `pub discovered:` 仍误判通过。该 finding 已按现行测试 owner 的精确范围修补，未更改生产 report 字段或 Text01 上行文件。
- 修订后的测试限定 `NativePluginLoadReport` 声明，检查四个字段各有唯一且无可见性前缀的声明；对每个字段在内存里分别注入裸 `pub`、`pub(crate)` 和受限 `pub(in ...)`，断言 12 种公开变体均不能通过守卫。原守卫的裸 `pub` 假绿已在编辑前复现；Windows `rustfmt +1.94.1 --check --edition 2021`、scoped `git diff --check` 和同语义 Python 静态变体检查均通过。修订后 test owner SHA-256 为 `1439e85cdb816f85810eb988ac9fffbb0258f1533f2780a89bc0c7b847ee6384`。
- Snapshot `3743` 修补后独立只读复审为 `Critical=0 / Important=0 / Moderate=0`；复审再次验证四字段 12/12 种公开变体均被拒绝，源码/文档哈希一致且四份源码格式检查通过。静态守卫、格式和审查通过不等于 Rust 测试运行；受管 Cargo 与 Text01 GPU PNG/像素断言仍待完成，failure 继续 `open`。

Fixed state must replace the open text with:

- 根因：<final root cause>
- 架构修复：<owners and invariants changed>
- 验证：<commands and exact results>
- 回传：Text01 WGPU framebuffer gate can resume.
