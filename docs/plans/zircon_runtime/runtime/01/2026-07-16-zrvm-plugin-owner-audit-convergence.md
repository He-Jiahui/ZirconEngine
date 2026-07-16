---
record_kind: milestone_output
status: accepted
completed_at: 2026-07-16
milestone: M1
plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
  - tools/tests/test_runtime_tech_stack_boundary.py
tests:
  - python -m unittest tools.tests.test_runtime_tech_stack_boundary -v
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
---

# Runtime01 M1：ZrVM plugin owner 审计收敛

## 结果

- 状态：`runtime_01_zrvm_plugin_owner_audit_current_source_passed`
- `zircon_plugins/zr_vm_language/runtime/Cargo.toml` 是唯一 concrete ZrVM backend manifest owner。
- `zircon_runtime/Cargo.toml` 不允许 ZrVM 语义 feature、binding dependency、Cargo renamed binding package 或兼容转发 owner。
- 插件可以声明一个或多个 `zr_vm_rust_binding*` 依赖，但每项必须保持 optional、位于规范化后的外部 `zr_vm` owner 根内，并由 `backend-zr-vm` feature 显式 gate。

## 验证

- TDD RED：新增 current-owner regression 首次以缺少 `zr_vm_plugin_manifest_present` 报告字段失败。
- Python suite：7/7 passed。
- Review 后定向 negative-path regression：rename / traversal / alias / Runtime reintroduction 3/3 passed。
- `py_compile` 与 path-scoped `git diff --check` passed。
- full Runtime structure audit 的 Runtime01 分区：`dependency_boundary_violations=[]`、`risks=[]`。
- 独立最终 review：critical 0、important 0。

## 范围说明

- 未修改并行中的 `zircon_plugins/zr_vm_language/runtime/Cargo.toml`。
- 核心 product manifest count 保持 5；插件 manifest 单独作为 backend owner 审计。
- 全量审计仍报告 Runtime02/10/14 的并行 owner 漂移，本里程碑未吸收这些 foreign scope。
