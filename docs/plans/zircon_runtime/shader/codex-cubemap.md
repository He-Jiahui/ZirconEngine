# 真实 HDRI Cubemap IBL 与 PBR 反射修复设计

## Summary
- 将 2026-07-04 的 `polyhaven_lakes_1k.hdr` 截图标记为“临时 16x8 采样验证”，不能作为 cubemap/PMREM 完成项。
- 按虚幻的 Reflection Environment 思路实现真正的 HDR cubemap、specular mip 预滤波、SH diffuse irradiance，并用 `texture_cube + textureSampleLevel` 驱动天空盒和 PBR 反射。
- `dev/cmft` / `dev/cmftStudio` 用作 CPU 参考与边缘修复、solid-angle 权重验证来源；运行时视觉标准以 Unreal GGX PMREM 为准。

## Key Changes
- 文档状态：实现时更新 `docs/plans/zircon_runtime/shader/index.md`、Plan 11、Plan 13，明确旧截图被拒绝的原因，并按每个实现切片追加状态记录；所有验证图放到 `docs/tests/runtime/shader`，不放进 `target`。
- Cubemap 资产：新增一等公民 cubemap 描述，包含 HDR 线性数据、face size、mip count、source hash、6 face 顺序；不要把 cube 伪装成普通 2D 材质贴图。
- GPU 上传：为 cubemap 增加独立上传/视图路径，创建 `TextureViewDimension::Cube`，采样器启用线性与 mip 过滤；避免继续增大当前大型资源文件，按结构规范拆到子 owner。
- 环境资源：新增 `GpuEnvironmentMap`，包含 source cubemap mip0、prefiltered specular cubemap mip chain、SH9 diffuse、average brightness、bake key。
- Shader：cubemap 路径停止使用 16x8 scene uniform 表；天空盒采样 source cubemap mip0，PBR specular 按 roughness 映射 mip，diffuse 使用 SH9。
- 旧 `SampledEquirectangularEnvironment` 仅保留为 debug/临时 fallback，不再用于 HDRI cubemap 验证截图。

## Prefilter Algorithm
- 输入流程：HDR equirectangular 以线性 HDR 读入，双线性投影成 cubemap base face；默认 Poly Haven 1K 生成 512 face，测试可降到 256 face。
- 普通 mip：先生成 source cubemap mip chain，参考 Unreal 的 cubemap downsample，至少使用中心加邻域 tap 的模糊下采样，不能 nearest。
- Specular PMREM：每个 face/mip 执行 GGX importance sampling；验证导出使用 1024 samples，运行时可预留质量档。
- Roughness/mip 关系采用 Unreal 常量：`ROUGHEST_MIP = 1`，`ROUGHNESS_MIP_SCALE = 1.2`，并实现正反 roundtrip 测试。
- 高粗糙度使用 cosine hemisphere，低粗糙度 `< 0.01` 直接取 mip0；sample mip 用 solid-angle 公式选择，保持 HDR 线性直到最终渲染 tonemap。
- cmft 参考路径实现 solid-angle weighted CPU oracle，覆盖 `EdgeFixup::Warp`、face seam 与 mip 能量趋势，用于测试而不是强制逐像素一致。

## Test Plan
- 单元测试：equirect → cubemap face 顺序、cube view descriptor、mip/layer 上传偏移、roughness ↔ mip roundtrip、SH9 constant-color、PMREM blur 单调性。
- Shader 检查：cubemap HDRI 路径必须包含 `texture_cube` / `textureSampleLevel`，并确认不再走 16x8 采样表。
- 产品截图：重新导出 8x8 PBR 球阵，列为金属度 0→1，行为 smoothness 0→1，使用真实 HDRI cubemap 天空盒和环境反射。
- 验证图：保存为 `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_YYYYMMDD.png`，可额外输出 mip 诊断图到同目录。
- 质量验收：天空盒无 16x8 马赛克；高 smoothness 金属球能看到清晰环境；低 smoothness 反射随 mip 逐级模糊；高频能量随 roughness 单调下降。
- 路径验收：扫描确认新验证图没有写入 `target`。
- 最终验证：运行 `zircon_runtime` 相关 render/texture/environment 测试；共享接口改动完成后跑仓库现有验证矩阵的 runtime 范围。

## Implementation Milestones
1. 纠正计划状态与验收标准：记录旧截图问题，锁定 cubemap/PMREM/SH 为未完成项。
2. 完成 Cubemap asset 与 GPU cube upload 基础，先解决 Plan 13 的 cubemap/mip 支撑。
3. 接入 environment bind group、cubemap skybox 与 PBR `texture_cube` 采样。
4. 实现 Unreal-style GGX PMREM 与 SH diffuse，并用 cmft-style CPU 参考验证。
5. 重新导出真实 HDRI PBR 8x8 截图，补齐文档状态、测试记录和结构规范检查。

## Assumptions
- 采用完整引擎方案，不做“提高 16x8 表分辨率”的临时视觉修补。
- Unreal 的 roughness-to-mip 与 GGX PMREM 是运行时标准；cmft/cmftStudio 是参考实现与测试 oracle。
- HDR 数据在预滤波阶段始终保持线性 HDR，不提前 tone-map。
- 当前 workspace 按主线直接实现，不额外创建分支或 worktree。
