---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: material-abi-layout-expectation-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/shader/04
related_code:
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-14
---


# Shader04：material ABI 测试仍断言旧布局

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 声明的默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md`
- 交接原因：当前 canonical material ABI 已扩展，但 Shader04 自有 render-product 测试仍断言旧 payload 长度与旧 binding11 拒绝语义；Editor02 不拥有 material layout 或 renderer readiness diagnostics。
- 受管 Windows 日志：`E:\ZirconBuilds\editor02-m1-runtime-scene-default-after-text01-fix-20260714.log`。

## 失败现象与复现证据

- 原计划命令成功编译并运行 1702 个匹配测试，结果 `1690 passed / 6 failed / 6 ignored`；其中两项稳定归属 Shader04：
  - `render_product_material_properties_prepare_uniform_payload`：生产结果为 `Some(192)`，测试仍断言 `Some(144)`；
  - `render_product_streamer_reports_shader_material_layout_abi_diagnostics`：测试仍要求 `pipeline_layout.group2.binding11` 产生“仅支持 group 2 bindings”的旧诊断，但当前 material ABI 已接受该布局，因此诊断不存在。
- 两个测试文件自 2026-07-04 后未变化；失败不是 Editor02 world-sync、generation 或 inspection 代码产生。

## 最低共享层根因

Shader04 已扩展 canonical material uniform/layout ABI，但 render-product 回归期望仍固定在扩展前的 144-byte payload 与 binding11 非法语义，测试契约没有随唯一布局 owner 同步升级。

## 架构修复验收

- 由 Shader04 根据当前 canonical material layout owner 派生 payload 长度和合法 binding 范围，更新两项测试及必要的结构守卫；不得在测试中再复制容易漂移的裸布局常量。
- 明确验证新增 ABI 字段/绑定确实进入 layout hash、uniform encoding 与 readiness diagnostics，而不是仅放宽断言。
- fresh 重跑 `cargo test -p zircon_runtime --lib scene:: --locked`，两项测试通过且无新增 Shader04 失败。

## 禁止临时方案

- 禁止把生产 payload 截断回 144 bytes。
- 禁止恢复 binding11 的旧拒绝分支、兼容 alias 或仅为旧测试伪造诊断。
- 禁止在 Editor02 路径屏蔽、过滤或跳过 material 测试。

## 修复结果与回传

- 根因：Shader04 advanced PBR extended the canonical material uniform and group2 ABI, while two render-product tests still duplicated the pre-extension 144-byte size and binding11 rejection expectation.
- 架构修复：The uniform product test now reads GPU_MATERIAL_UNIFORM_MIN_SIZE from the canonical GPU owner; the readiness test no longer invents an illegal binding11 and instead proves the canonical validator requires semantic clearcoat-normal texture and sampler resources.
- 验证：Managed Windows job 173a1b539a4b46c3943b571d8b501414 generated fresh lib-test binary SHA256 45DF8F0B7D6C143B68C8A1EA8DF5707E54AFE770E058C8FD33F309EBF1906A2D. Both target tests passed 1/1. Segmented scene coverage accounted for all 1706 matching tests: 1698 passed, 6 ignored, and 2 failed only in foreign Plugins08 dynamic-reflection ownership; no Shader04 failure. The one-process scene run reached 418 passed and 0 failed before a Windows access violation, so broad scene green is not claimed. rustfmt check and scoped diff check passed.
- 回传：Editor02 material ABI expectation drift is cleared. Its aggregate scene gate may resume, but it must still rerun after the two Plugins08 dynamic-reflection failures are resolved; this return does not claim the aggregate gate green.
