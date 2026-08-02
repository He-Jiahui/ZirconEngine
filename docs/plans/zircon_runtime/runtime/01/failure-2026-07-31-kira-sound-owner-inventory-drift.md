---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: kira-sound-owner-inventory-drift
origin_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - docs/engine-architecture/runtime-tech-stack.md
  - zircon_plugins/sound/runtime/Cargo.toml
tests:
  - python -B -m unittest tools.tests.test_runtime01_kira_sound_owner_boundary -v
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary.RuntimeTechStackBoundaryTests.test_current_optional_text_and_backend_feature_declarations_are_clean tools.tests.test_runtime_tech_stack_boundary.RuntimeTechStackBoundaryTests.test_jolt_backend_is_feature_gated_and_plugin_owned -v
  - python -B -m unittest tools.tests.test_runtime_tech_stack_boundary -v
---

# Runtime01：Kira Sound owner inventory drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 来源执行切片：Frameworks03 canonical ZrVM executable-command hard cut 的完整 tech-stack upward gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：失败来自 Runtime01 自有依赖清单和技术栈权威仍把 `kira` 视作全仓禁用依赖；Frameworks03 不拥有 Sound backend 选型或跨 manifest 依赖治理。

## 失败现象与复现证据

2026-07-31 完整 `tools.tests.test_runtime_tech_stack_boundary` 运行 8 项，仅 5 项通过。三项失败中，两项 Runtime01 测试都因为报告包含 `Removed or editor-only dependencies entered Cargo manifests.`：

- `tech_stack_source_inventory.py` 的 `NON_DEPENDENCIES` 仍包含 `kira`；
- 当前受跟踪 `zircon_plugins/sound/runtime/Cargo.toml` 已合法固定 `kira = "0.12.2"`；
- Sound runtime 的生产 `kira_bridge`、播放、设备、mixer graph 与 lifecycle owners 已实际消费 Kira，Plugins02 记录也明确这是从旧 CPAL/软件 mixer 到 Kira 0.12.2 的 hard cut；
- `docs/engine-architecture/runtime-tech-stack.md` 仍声称 Kira 未引入且 Sound 使用旧 CPAL/custom mixer，形成第二份陈旧真相。

第三项完整套件失败来自 Runtime06 plugin public-surface 审计，独立归 Runtime06，不属于本节点。

## 最低共享层根因

Runtime01 用无 owner 的全仓 `NON_DEPENDENCIES` 元组表达“当前未采用”决策。Plugins02 完成 Sound backend hard cut 后，清单、扫描边界和权威文档没有一起切换为“插件唯一 owner + 禁止向 core/editor 泄漏”的可执行合同，因而把正确的新架构误判为非法依赖。

## 架构修复验收

- 从全仓禁用清单移除 `kira`，但新增唯一 owner 合同：全仓产品 manifests 中只允许 `zircon_plugins/sound/runtime/Cargo.toml` 声明 Kira，且精确固定 `0.12.2`。
- manifest 扫描只覆盖根 manifest 与当前顶层 `zircon_*` 产品树；不得把 `.codex/state` validation copy、`target` 或参考源码当成 current source，也不得因此漏掉嵌套 `zircon_plugins` workspace。任一产品子树或 manifest 读取失败必须进入 `manifest_scan_errors` 并使审计 fail closed。
- 技术栈权威同步为 Kira Sound owner 当前事实，并由审计锚常驻守卫；不得保留“未引入 Kira / 旧自研 mixer”双重真相。
- 新 focused lower-layer 回归 4/4、原两项 Runtime01 reproduction 2/2 通过；完整 tech-stack upward gate 中 Runtime01 风险归零。若只剩已单独归属的 Runtime06 public-surface 漂移，应记录为外部失败而不冒充全套 GREEN。

## 禁止临时方案

- 不得删除 Sound 的 Kira hard cut、恢复旧 CPAL/软件 mixer、给 Frameworks03 测试加跳过或放宽 `risks == []`。
- 不得把 Kira 改成任意 manifest 均可使用，也不得靠忽略整个 `zircon_plugins` 树消除错误。
- 不得扫描 `.codex/state` 或旧 validation copy 并把它们当 current dependency owners。

## 修复结果与回传

- Python inventory now removes `kira` from the four global non-dependencies, scans only the root manifest and current top-level `zircon_*` product trees, parses TOML package identity so dependency aliases cannot escape the owner rule, and requires exactly one Sound runtime declaration pinned to 0.12.2. Focused negative cases reject metadata decoys, metadata-only text, a second alias, and dev/build/workspace-only declarations while accepting an exact target-specific runtime declaration; an injected unreadable product subtree produces `manifest_scan_errors` and a fail-closed risk.
- `runtime-tech-stack.md`, its anchor inventory, Markdown renderer, M0 review, interface-convergence mirror, and Runtime01 child output record now describe the Sound-owned Kira hard cut instead of the retired CPAL/custom-mixer claim.
- Runtime Rust guards now remove Kira from the global non-dependency list, parse Cargo TOML package identity across root/workspace/dev/build/target tables, require one Sound runtime owner and one production `0.12.2` declaration, accept canonical/alias/target production forms, reject metadata and non-runtime decoys, and fail closed on product-tree enumeration, entry, file-type, manifest-read, and TOML errors. The mirror guard now pins count 4 plus owner, declaration-count, version, pin, violation, and manifest-scan evidence.
- The first independent post-implementation review returned `Critical 0 / Important 4 / Minor 1`; all five findings were forward-fixed without compatibility: Python/Rust pin semantics were aligned, directory traversal became fail closed, `.zircon` cache exclusion became effective, missing mirror evidence was added, and this handoff state was refreshed. Current static evidence is focused Kira owner/pin/scan 5/5 GREEN, original Runtime01 reproduction 2/2 GREEN, complete tech-stack 8/8 GREEN, Python py_compile, Rust 1.94.1 rustfmt, and scoped diff-check GREEN.

Open state: `implementation_complete_second_review_rerun_and_managed_rust_validation_pending`; no fixed return or accepted closeout is claimed.
