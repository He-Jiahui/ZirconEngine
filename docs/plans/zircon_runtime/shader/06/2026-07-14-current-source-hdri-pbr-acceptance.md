# Shader06 Current-Source HDRI PBR Acceptance

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md

Milestone: M1

Status: completed

Files: ["docs/plans/zircon_runtime/shader/06/2026-07-14-current-source-hdri-pbr-acceptance.md", "docs/plans/zircon_runtime/shader/06/fixed-2026-07-15-deferred-lighting-nested-include-resolution.md", "docs/plans/zircon_runtime/shader/06/fixed-2026-07-15-runtime-operation-phase-terminal-matcher.md", "docs/tests/runtime/shader/runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260715.png", "docs/zircon_runtime/tests/runtime_shader_pbr_hdri_export.md", "zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs", "zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs", "zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs", "zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative/math.rs"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | EC-M1 当前源码方向、PBR 数学与真实 HDRI 产品复核 | `completed` | 2026-07-15 | 独立审查后收紧为非 ignored readback 门禁、真实 Lakes PMREM-reference 64-cell 门禁、逐半径 10% 上限和不可变 20260715 证据文件。最终 current-source managed job `77c7b62ea56343339237e62348fc1abc` 完整重编译并 1/1 GREEN；dated export 1/1，overwrite guard 1/1 并拒绝已有证据。Render18 与 Editor03 两个跨计划失败均已回传 fixed。 |
| M1 | EC-M1-T testing 当前源码 HDRI PBR 产品门禁 | 通过 | 2026-07-15 | managed product gate `77c7b62ea56343339237e62348fc1abc` 1/1；post-review atomic evidence tests `a8c1d39f641840e293a6a65fa6238ff7` 2/2；coordinator action validation 24/24；independent review Critical 0 / Important 0。 |

## Scope Delivered

- 8x8 metallic 0 -> 1 / smoothness 0 -> 1：最终 `runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260715.png` 为 1600x1200、1,381,872 bytes、SHA256 `0EEBCDAD9071B999585F94ADBB9F31103D5585F014A837D15C54597237246527`；报告 1,382 bytes、SHA256 `0A4B778824F2890FFF7C46B0DB3E965DACA7E6EB816C66DAA509646D88377473`。
- Lakes PMREM512 精确五视角：`runtime_shader_pbr_real_hdri_lakes_pmrem512_angular0003_exact_multiview_contact_sheet_20260713.png`，1920x1016，2,092,876 bytes，SHA256 `969718512A5B126EAB24F569C097F76D97715428906CCDD43869AC4FCE1286FA`。原图复核确认 front、yaw +/-120、pitch +/-120 的天空、湖岸、道路和球面反射方向一致，镜面球不是白球且无旧 16x8 块状伪影。
- Realtime procedural 五视角：`runtime_shader_pbr_procedural_realtime_ibl_mirror_cardinal_120deg_contact_sheet_20260714.png`，4000x600，663,618 bytes，SHA256 `B41F470CA6119405AAFB8B5441C0276258F6680353381BFB4230C5FB67BCE9FF`。
- DX12 capture：`zircon_shader_pbr_viewer_pmrem512_angular0003_dx12_renderdoc_20260714_capture.rdc`，48,412,516 bytes，SHA256 `438AF4D4D796AE982E7128843E6741266A19CC3A2686FE61B4FFC01BE6825B6B`；`D:\Tools\renderdoc\renderdoccmd.exe replay ... --loops 1` fresh exit 0。
- 当前交付查看器：`E:\ZirconBuilds\shader-pbr-viewer-pmrem-artifact-layout-20260714\zircon_shader_pbr_viewer.exe` 的 `--help` fresh exit 0；以 Lakes 2K、source64、PMREM64 启动，全程 `Responding=True`，53.81s 完成 scene prepare，窗口进入 `Ready - yaw 0 pitch 0`，stdout 报告 source/PMREM 均为独立 64x64、7 mip staged artifact。
- 结构预算：`mipmap.rs` 372 行、`pmrem.rs` 367 行、source cubemap tests 762 行；当前 PBR product root/matrix/quantitative/quantitative-math/sphere owners 为 628/491/660/154/572 行，viewer `scene.rs` 136 行，均低于适用的 800 行预算。
- 左右掠射回归：8x8 产品测试新增常量对称 cubemap 第三组 GPU 帧；对最光滑电介质球的五组左右掠射半径取对称 3x3 patch 并扣除 diffuse-only 基线，要求镜面响应可见、左右聚合能量相对差 `<= 0.05`，且每个半径相对差 `<= 0.10`。聚合门禁覆盖总体偏置，逐半径门禁阻止相反方向误差互相抵消。
- 当前源码首轮 RED：managed job `459d93506df64b9f99953de4e13a8c35` 在测试体前发现资产管理器接口硬切后，integration fixture 仍把 `Arc<ProjectAssetManager>` 传给生产 `SceneRenderer::new(ProjectAssetManagerAccess)`。夹具已改用专门的 `SceneRenderer::new_for_test(...)`，后续 GREEN 复跑不得省略。
- 第二轮 RED：managed job `c60744f9a7c1438dbc92898428fce66c` 证明 `new_for_test` 只在库单元测试编译中可见，integration product test 不应扩大测试专用公开 API。夹具现与 viewer 一致，注册并激活真实 foundation/tasks/asset 模块，从 `project_asset_manager_handle(...)` 构建 `ProjectAssetManagerAccess`，再走生产 `SceneRenderer::new(...)`；下一轮必须从该结构化生产合同取得 GREEN。
- 第三轮 RED：当前源码已编译产品 test binary，精确执行 `pbr_matrix::render_product_environment_pbr_matrix_quantitative` 实际运行 1 项，但在创建 deferred lighting pipeline 时失败；最终 WGSL 仍含 `#include <zr_irradiance_volume.wgsl>`，Naga 拒绝解析，结果 `0 passed; 1 failed`。最低原因归 Render18 AF-M2 的 deferred module dependency consumer，已登记 [deferred-lighting-nested-include-resolution](../../render/18/failure-2026-07-15-deferred-lighting-nested-include-resolution.md)，本记录不提前宣称产品门禁通过。
- 第四轮 RED：Render18 current source 消除 nested include 后，managed job `e3c19cb1117c40d9ae6fc0184a18de19` 已通过完整编译并进入真实 GPU 产品测试，但在 12 分钟后以 Windows stack overflow 退出，未生成 panic 文本或改写截图。量化测试现复用同文件 HDRI 导出既有的 128 MiB named-thread 模式，避免三套 1600x1200 HDR readback/量化链耗尽默认测试线程栈。
- 第五轮 RED：managed job `5b43633018d64ab7bcb49dd9c0513eb2` 实际执行 1 项并稳定进入新增掠射门禁，单像素/逐点口径测得 `max_relative_delta=0.058334`；改为五半径对称 3x3 patch 后，job `7138fe470e31468bba813dcce213a7e9` 测得左右聚合均值 `0.078899/0.075075`，聚合相对差约 `0.0497`，而逐半径峰值 `0.0850`。这证明 BRDF 总体左右能量符合 5% 门槛，逐点峰值主要由小球光栅覆盖离散造成；断言现锁定聚合能量并继续报告逐半径峰值。
- 收紧后最终 GREEN：managed non-ignored job `77c7b62ea56343339237e62348fc1abc` 结果 `1 passed; 0 failed; 20 filtered out`，测试体用时 217.00s。常量环境左右掠射聚合/逐半径最大相对差 `0.049672/0.085009`；镜面 SSIM `0.998674`、Lakes 64-cell PMREM-reference 最小 SSIM `0.981621`、controlled-HDR 最小相邻粗糙度差 `0.00000165`、电介质 delta E `0.806798`、中心 F0 `0.041188`、Lakes 掠射响应 `0.266949`、粗金属亮度 `0.493901` 均通过。独立 dated export 1/1；已有日期再次 export 明确失败，证明不会覆盖 canonical evidence。

## Fresh Testing Evidence

- Passed before review tightening: managed Windows exact 8x8 product test `44dad6ba11c04a6ea61c91054037a4fa`; direct current-binary 8x8 grid/endpoints contract 1/1; legacy-path static scan; UE PDF/source-LOD static contract scan; visual hash/dimension audit; original-resolution visual inspection; viewer startup; RenderDoc replay; scoped `git diff --check`; file-budget scan; and handoff validator.
- Passed after review tightening: final managed non-ignored real-Lakes PMREM-reference + controlled-HDR roughness run `77c7b62ea56343339237e62348fc1abc`; ignored dated evidence export 1/1; overwrite unit guard 1/1; duplicate dated export rejected; fresh hashes and original-resolution visual inspection; independent re-review findings resolved.
- Final post-review compile/test guard: managed Windows job `a8c1d39f641840e293a6a65fa6238ff7` rebuilt the current integration test after the atomic evidence reservation and quantitative-math module split; both exclusive-claim and partial-pair rollback tests passed (`2 passed; 0 failed; 20 filtered out`).
- Cross-plan current-source compile failures were fixed and returned as [deferred nested include](fixed-2026-07-15-deferred-lighting-nested-include-resolution.md) and [runtime operation matcher](fixed-2026-07-15-runtime-operation-phase-terminal-matcher.md). Orphaned/RED attempts remain diagnostic history and are not acceptance evidence.
- Render18 lower-layer WGPU reproduction also passes 1/1 on current source; managed job `4108772f5a3b4f0784cfab0925a914fd` exits 0. The canonical nested-include handoff has been returned as [fixed](fixed-2026-07-15-deferred-lighting-nested-include-resolution.md), so no Shader06 test or dependency gate remains red.

## Review

- Product evidence was inspected at original resolution: the Lakes skybox is continuous rather than block-pixelated, smooth metals carry recognizable road/tree/lake reflections, rough rows broaden reflection energy, and the five front/yaw +/-120/pitch +/-120 views preserve environment orientation.
- The 3x3-patch aggregate gate intentionally measures BRDF symmetry under a constant cubemap. It keeps per-radius maximum delta in diagnostics while avoiding a one-pixel raster-coverage threshold that would not represent the analytic left/right BRDF contract.
- Final independent review found no Critical issue. Its two Important findings were closed before acceptance: dated PNG/report reservation now uses exclusive `create_new` claims with partial-pair rollback and failure cleanup, while the module document now distinguishes the Lakes 64-cell selected-PMREM SSIM gate from the controlled-HDR 56-transition monotonicity gate and correctly names the default product gate as non-ignored.
- No Render18 or Plugins12 source is included in this Shader06 milestone manifest; those changes retain their owning sessions and commits.
