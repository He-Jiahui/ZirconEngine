---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: runtime-module-structure-cfg-fence
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_runtime/frameworks/03
related_code:
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
tests:
  - zircon_runtime-d204f127672c3c4e.exe builtin::runtime_modules::tests::registration::structure::runtime_module_assembly_keeps_specialized_flows_in_child_owners --exact --nocapture --test-threads=1
resolved_at: 2026-07-11
---


# Frameworks 03：Runtime module 结构守卫未识别复合 cfg test fence

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Plan10 M2.1 Runtime 全包测试门 exit 1 归属诊断
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 交接原因：最低失败位于 Frameworks03 所有的 `builtin/runtime_modules` 可选域 cfg 化及 Runtime 全测试门；Plan10 的 AssetRef/.zmeta 聚焦测试已全绿。

## 失败现象与复现证据

使用已由受管 Runtime 全包门生成的 `D:\targets\zircon-engine\lanes\test-861c17af83854cc78f2453f8c825dee2\debug\deps\zircon_runtime-d204f127672c3c4e.exe` 做只读分组诊断：

- `--list --format terse` 共 7553 项测试。
- `animation::` 24/24 通过。
- `builtin::` 14/15，通过之外的唯一失败为 `builtin::runtime_modules::tests::registration::structure::runtime_module_assembly_keeps_specialized_flows_in_child_owners`。
- 精确复跑稳定 0/1、exit 101；断言位于 `zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs:205`，要求 production source 不包含 `RuntimeExtensionRegistry`。

## 最低共享层根因

结构守卫以 `.split("#[cfg(test)]")` 截取生产源码；`assembly/registration_inputs.rs:195` 已硬切为 `#[cfg(all(test, feature = "graphics"))]`，所以 fence 不再匹配，后续测试专用 `RuntimeExtensionRegistry` import 被误判为生产依赖。生产 assembly 本身没有违反该结构合同，失败属于 cfg 声明与结构守卫共同事实漂移。

## 架构修复验收

- 让测试专用 owner 的 cfg fence 与结构守卫使用同一可扩展事实，不再依赖单个字符串拼写。
- 精确测试 `runtime_module_assembly_keeps_specialized_flows_in_child_owners` 通过。
- 复跑 `builtin::runtime_modules::tests::registration::`，再复跑 Frameworks03 Runtime/App 全测试门。
- 修复后回传 Plan10 M2.1，使其 Runtime 全包 exit 1 归属可关闭并继续 M2 统一测试。

## 禁止临时方案

- 禁止删除或放宽 `RuntimeExtensionRegistry` production 边界断言来隐藏误判。
- 禁止把测试 import 搬入 production、添加兼容 re-export，或为一个文件写调用点特判。
- 禁止把 Plan10 聚焦 9/9 通过冒充 Runtime 全包通过。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Frameworks 03 M1 | runtime module cfg fence 与结构守卫同源 | `未通过-全包唯一首因已定位` | 2026-07-11 | 7553 项现有 binary 分组诊断：`animation::` 24/24、`builtin::` 14/15；唯一失败精确复跑 0/1，复合 `cfg(all(test, feature = "graphics"))` 未被只识别 `cfg(test)` 的 production-source guard 截断。 |

## 修复结果与回传

- 根因：Inline test ownership forced a brittle cfg(test) string split; compound feature cfg exposed test imports as production text.
- 架构修复：Moved tests to a child owner and made the structure guard inspect the complete production file without cfg spelling compatibility.
- 验证：Current default Runtime lib-test compiled; exact structure 1/1 and registration group 7/7 passed.
- 回传：Plan10 M2 Runtime full gate can resume; no compatibility path was added.
