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
- M3 已重新打开并先落 guard：V1/V2 descriptor、entry、behavior、host function table、byte DTO、function pointer、fixture feature 与 V3-to-V2 alias 全部纳入零容忍扫描；V2 物理类型与 V3 alias 必须删除，不得以 compatibility、冻结或长期别名名义保留。
- 第二审查修复后的当前源证据：focused inventory 3/3、Runtime06 standalone boundary 1/1、完整 runtime tech-stack boundary 8/8，Python py_compile 通过；这些仍是静态证据，不替代 Rust/Cargo hard-cut 验收。

### 2026-08-03 guard-first RED 基线

- `native_plugin_public_surface.py` 不再把 `NativeHostApiV3RegistrationScope` 归入当前 host-API adapter 公共面；该符号在生产源码删除前会被现有 unclassified-symbol 守卫阻断。
- `plugin_surface_lifecycle_boundary.py` 新增独立零容忍报告：`native_v3_alias_files`、`retired_host_api_adapter_files`、`v2_fixture_feature_files`，并把 `zircon_plugins` 扫描扩展到 Rust 与 Cargo TOML。
- focused 回归 `test_v2_descriptor_entry_dtos_aliases_and_fixture_feature_are_hard_cut` 已按 TDD 取得预期 RED。初始扫描计数为 loader 9、plugins 7、V3 alias 2、retired adapter 4、fixture feature 2；二次审查把 Runtime 扫描扩到完整 `zircon_runtime/src/plugin` owner 后，准确计数更新为 Runtime plugin 10、plugins 7、V3 alias 2、retired adapter 4、fixture feature 2，其中新增的第 10 个路径是公开 facade owner `plugin/native.rs`。
- 三个已修改 Python 文件均通过 `ast.parse`，scoped `git diff --check` 通过（仅既有 CRLF 提示）。Windows ACL 阻止 `py_compile` 写入 `__pycache__`，因此本轮不把 py_compile 计入证据，也未运行 Cargo。
- 当前 Runtime06 会话仅持有审计脚本、focused Python test、本 failure 与 output record 的写入范围；生产 Rust/plugin fixture 硬切范围尚未生效。上述 RED 是生产迁移的可执行基线，不是回滚或放宽守卫的理由。
- audited ownership-transfer preview request `354d81c6b91749e683c1c9c91c001bc7` / fingerprint `f36eb5dddc61b1b36d9ca51ef65c2619d00d033cdbc27e6c917936de56bca464` 在 baseline epoch 245 原子审计 22 个生产/测试路径：10 个 eligible，12 个仍由 executable Session 持有。Frameworks04 持有 editor/gltf fixture 2 路径；Frameworks02 持有 native dynamic fixture、plugin SDK native owner/tests 与 Runtime host callbacks 共 5 路径；Plugins01 持有 behavior calls、host API adapter/tests 与 live-host tests 共 5 路径。
- 因 descriptor/entry probe、behavior callbacks、DTO/function pointers、host callbacks 与 fixtures 必须在同一硬切内闭合，禁止只转移并修改 10 个 eligible 路径。preview 仅作为 coordinator 后续 owner-aware transfer/wakeup 证据，未 apply、未抢占任何 executable owner，也未轮询其状态。

### 2026-08-03 guard 二次审查与前向修复

- 首轮独立二次审查为 Critical 0 / Important 2 / Minor 1：root `pub use` scanner 可被私有 alias 与 token 间注释绕过，V1/V2/V3-alias scanner 遗漏 `plugin/native.rs`，且 unrelated `crate::other::{native::...}` 会被误报。
- 第一次复核为 C0/I1/M0，发现只追踪 native alias 不能解析 `crate::plugin as p -> p::native as n`；第二次复核为 C0/I1/M0，继续发现 grouped `crate::plugin::{self as p}` 叶子未归一。两项均以精确 mutation 先 RED，再由通用 use-binding path 展开与 grouped-self prefix 归一前向修复。
- 第三次复核为 C0/I2/M1，发现文件级 alias 字典混入嵌套 module/function scope，且普通/raw string 正文可制造 `pub use` 假阳性或用 `/*` 吞掉后续真实导出。新增作用域漏报/误报、字符串假导出和 raw-string 后续真实导出四个 mutation 后，scanner 改为保留 offset/换行的 Rust comment/string/char/raw-string 非代码遮罩，并仅收集根 delimiter depth 的 `use`。
- 第四次复核为 C0/I2/M1，发现 regex-first statement 匹配可从 macro token tree 内的无分号伪 `use` 吞到后续真实 root re-export，`extern crate self as runtime_alias` 未进入 alias graph，且 `^\s*` 把诊断位置提前到注释/空行。三项均先新增精确 RED mutation，再把 root import 收集改为非代码遮罩后的 delimiter-stack scanner；scanner 只在根深度识别 `pub` visibility / `use` / `extern crate`，并以真实 token offset 生成位置。
- 第五次复核为 C0/I2/M0，发现任意根级 `extern` 都会被消费到下一分号，导致合法 `extern "C" fn` 函数体的局部 alias 污染根图；ASCII-only tokenizer 同时漏掉 raw identifier 并把 Unicode alias 后的 `native` 错切为直接根路径。正反 extern-scope、raw identifier 与 Unicode alias mutations 全部先 RED；当前 scanner 只在确认 `extern crate` 后消费语句，并以 Rust raw/Unicode identifier-aware tokenizer 展开 use tree。
- 第六次复核为 C0/I2/M0，发现 Rust 对 Unicode identifier 做 NFC 判等，而 scanner 保留原始码点，且本地 `macro_rules!` 可生成根导出或生成 alias 后供显式根导出消费。tokenizer 现对每个 raw/Unicode identifier 做 NFC；plugin facade 根级宏调用则设为独立零容忍风险并把 M4 gate 置为 `root-macro-invocation-present`。该约束拒绝所有不可静态审计的宏生成根面，不实现不完整的 macro_rules 展开器。
- 第七次复核为 C0/I1/M1：raw identifier tokenizer 会把 `r#macro_rules` 归一成 `macro_rules`，导致合法同名 raw 宏调用被错误当成关键字宏定义忽略；原有宏回归也只断言计数非零，且缺少 definition-only 与嵌套调用负例。新增 raw-name 精确计数、未调用根定义、函数内调用和嵌套 module 定义/调用回归先取得预期 RED；scanner 现只排除没有 `r#` 前缀的 `macro_rules!` 定义，`r#macro_rules!()` 仍被根级零容忍门捕获。
- 第八次复核为 C0/I2/M1：互斥 `#[cfg]` 下的同名 alias 会被单值图按源码顺序覆盖，`NativePluginDescriptorFnV1/V2` 与 `NativePluginEntryFnV1/V2` 未进入 V1/V2 库存，且生产 intended-RED 的首个断言会遮蔽后续库存分支。两种 alias 顺序和仅含 descriptor/entry V2 function-pointer 的隔离 mutation 先取得精确 RED；alias 图现保留每个 binding 的所有候选并以逐分支 cycle guard 解析，任一候选到达 native owner 即命中。function-pointer 四个名称已补齐；生产 RED 改为一次比较完整库存映射，并为 plugin TOML、retired adapter、fixture feature 增加独立 subtest mutation。
- lifecycle scanner 现覆盖完整 `zircon_runtime/src/plugin` owner；native owner 判断基于展开后的完整路径，不再搜索任意嵌套 `{ native }` 子串。alias/comment/grouped-self/cfg-candidate/scope/literal/macro/extern-crate/raw-identifier/Unicode/NFC controls、unrelated nested-native negative control、macro definition/nested-call negatives、descriptor/entry function-pointer、isolated inventory branches 与 public-owner V2/V3-alias mutation 共 14 个 control test 全绿。完整 inventory 为 17 tests：14 passed / 3 expected production-migration failures；三个 Python 文件继续通过 `ast.parse`，scoped `git diff --check` 通过（仅 CRLF 提示），未运行 Cargo。第八轮独立审查的 C0/I2/M1 已以 mutation-first 前向修复；修复后最终 exact5 复核为 C0/I0/M0，guard-first 静态切片 Source Ready。
- `native_plugin_public_surface.py` 当前为 964 行，尚未越过约 1000 行门，但新增 lexer/root scanner 已触发 modularization warning。当前 coordinator write scope 是不可变 exact5，未包含可创建的新 helper path，因此本切片不得越权扩文件。最小后继边界已锁定为把 `_mask_rust_non_code`、Rust identifier/use-tree/root-import/root-macro scanner 一组整体提取到语义化 `rust_root_imports.py`；在 coordinator 赋予该路径前不继续向本文件堆叠解析职责。

## 架构修复验收

- `zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/mirror_docs.rs` 仍钉住 source 14、namespace 68、groups 5、App 7 与旧日期；必须在 Runtime source quiet window 内原子更新为 2026-07-31 与 `17/74/6/8`，并通过 managed Rust 1.94.1 focused mirror gate。
- M3 必须把 V2 byte-slice/buffer/callback-status 实体迁成唯一当前 V3 物理类型，删除 V2 public names 与 V3 aliases，并删除旧 `NativeHostApiV3RegistrationScope`；当前 `NativePluginHostFunctionTableV3` 仍是 entry ABI 的合法 callback table，不得误删。
- M2/M3 新增的 world-runtime-extension 并发门和 native callback panic/global-hook 门仍需 managed current-source 证据。
- 在上述 Runtime Rust 切片、二次审查和 managed gate 完成前，本 failure 保持 `open`，Runtime06 保持 `in_progress`。

## 修复结果与回传

Open state: `hard_cut_guard_red_production_scope_pending`; no fixed return or Runtime06 completion is claimed.

## 禁止临时方案

- 不得把预期值退回 68/7、忽略新增 App 测试文件、放宽 `risks` 或允许 unclassified symbol。
- 不得把 V4 policy/scope 暴露回 `zircon_runtime::plugin` 根、误归 bridge-method，或用一次性 catch-all 分类掩盖 host-API adapter owner。
- 不得把 V2 物理类型/V3 alias 宣称为长期兼容策略；禁止恢复 V1/V2 entry、descriptor、loader fallback、alias 或 shim。
- 不得用 Python focused 3/3 或 tech-stack 8/8 代替 Rust mirror、native/plugin workspace 和 Runtime06 全量验收。
