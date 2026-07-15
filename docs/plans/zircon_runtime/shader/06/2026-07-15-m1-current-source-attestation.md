# Shader06 M1 Current-Source Attestation

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_runtime/shader/06/2026-07-15-m1-current-source-attestation.md"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | 当前源码真实 Lakes HDRI 8x8 PBR 产品重新证明 | `completed` | 2026-07-15 | Windows managed job `136865cf76784cc4abdc7a07c8ba3d64` 在当前共享源码上精确执行非 ignored 产品门禁，结果 1/1 GREEN；协调器记录 exit 0、released 且无存活进程。不可变 20260715 PNG/TXT 哈希、原尺寸目检与 target 零副本扫描均重新核验。 |
| M1 | 历史清单完整性合规处理 | `completed` | 2026-07-15 | 旧 M1 commit `c228d91c9ff7b2a167237570513c9257e05bee66` 的 recorded manifest hash 无法由 commit tree 重建，因此不导入、不覆盖、不伪造。本记录以新的一文件 manifest 提供当前源码证明。 |

## Scope Delivered

- 仅重新证明当前源码的 8x8 metallic 0 到 1 / smoothness 0 到 1、真实 Lakes HDRI、PMREM 模糊反射、左右掠射对称与电介质/金属端点产品合同。
- 不修改 M1 生产实现、产品测试、PNG/TXT 证据或旧提交；本 manifest 只包含本证明文件。
- M2-M5 继续使用各自独立的历史导入或 current-source attestation，不把本记录扩大为后续里程碑完成声明。

## Fresh Testing Evidence

- Managed job `136865cf76784cc4abdc7a07c8ba3d64`, target `F:\cargo-targets\zircon-engine\pool\832e9caf94cdbc5bb2fbeb3ffd49f9e6d203390ac62d5f892b18bcb9fe6d3c30`, exit 0, status `released`, live process count 0。
- `WGPU_BACKEND=dx12 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export pbr_matrix::render_product_environment_pbr_matrix_quantitative --locked --target-dir <managed-pool> -- --exact --nocapture --test-threads=1`: 1 passed, 0 failed, 0 ignored, 21 filtered out；测试体用时 187.11s。
- 左右掠射 3x3 patch 聚合相对差 `0.049672 <= 0.05`，逐半径最大相对差 `0.085009 <= 0.10`；mirror SSIM `0.998674`，Lakes 64-cell PMREM-reference 最低 SSIM `0.981621`。
- 报告继续通过 controlled-HDR 最小相邻粗糙度差 `0.00000165`、电介质 delta E `0.806798`、中心 F0 响应 `0.041188`、Lakes 掠射响应 `0.266949`、粗金属亮度 `0.493901`（允许范围 `[0.211641, 1.688753]`）。
- PNG `docs/tests/runtime/shader/runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260715.png`: 1600x1200, 1,381,872 bytes, SHA256 `0EEBCDAD9071B999585F94ADBB9F31103D5585F014A837D15C54597237246527`。
- TXT `docs/tests/runtime/shader/runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260715.txt`: 1,382 bytes, SHA256 `0A4B778824F2890FFF7C46B0DB3E965DACA7E6EB816C66DAA509646D88377473`。
- 原尺寸目检确认 Lakes 天空、湖岸和道路连续，8x8 球阵列完整，粗糙行逐步展宽环境响应，光滑金属球可辨识环境镜面内容；D/E/F 三个 Cargo target 根对该 dated PNG/TXT 的精确扫描为 0 个副本。
- 早先 job `b807a44c2e384030bd01f2bafd693473` 在外层监督超时后成为 orphaned，`exit_code=null` 且未绑定 milestone validation；它明确不作为本记录验收证据。

## Review

独立审查只读核对了单文件 manifest、fresh managed job 终态与原始日志、不可变 PNG/TXT 哈希和尺寸、原尺寸视觉检查、D/E/F target 零副本扫描，以及历史 M1 不可导入说明，结论为 Critical `0`、Important `0`、Minor `0`。审查确认本证明足以进入 coordinator M1 review gate；协调器仍须记录该 review 并通过 milestone validation 后才可提交。
