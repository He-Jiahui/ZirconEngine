# 真实 HDRI Cubemap IBL 与 PBR 反射修复设计

## Summary
- 2026-07-04 的 `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 是旧 16x8 sampled-equirect 路径的失败证据:天空盒和反射都呈马赛克,不能再作为 cubemap/PMREM 通过项。
- 真实 HDRI 路径必须分成两张 cube:skybox 采样 source cubemap mip0;standard PBR specular 采样单独的 GGX PMREM cube mip chain。
- 运行时视觉标准以 Unreal Reflection Environment 的 roughness/mip 对数映射、GGX filtered importance sampling、split-sum BRDF LUT 和 SH9 为准;`dev/cmft` / `dev/cmftStudio` 作为投影、solid-angle、edge/seam、final mip、IEM/PMREM 消费分工参考。

## Key Changes
- 文档状态：实现时更新 `docs/plans/zircon_runtime/shader/index.md`、Plan 11、Plan 13，明确旧截图被拒绝的原因，并按每个实现切片追加状态记录；所有验证图放到 `docs/tests/runtime/shader`，不放进 `target`。
- Cubemap 资产：新增一等公民 cubemap 描述，包含 HDR 线性数据、face size、mip count、source hash、6 face 顺序；不要把 cube 伪装成普通 2D 材质贴图。
- GPU 上传：为 cubemap 增加独立上传/视图路径，创建 `TextureViewDimension::Cube`，采样器启用线性与 mip 过滤；避免继续增大当前大型资源文件，按结构规范拆到子 owner。
- 环境资源：`GpuEnvironmentMap` 至少包含 source cubemap mip chain、prefiltered specular PMREM mip chain、SH9 diffuse、BRDF LUT、average brightness、bake key;IEM 是质量档可选项。
- Shader：cubemap 路径停止使用 16x8 scene uniform 表；天空盒采样 source cubemap mip0，PBR specular 按 Unreal roughness/mip 映射采样 PMREM，diffuse 默认使用 SH9，可选 IEM。
- 旧 `SampledEquirectangularEnvironment` 已退出 runtime 验收路径;任何 16x8 sampled-equirect 截图只可作为失败回归样本。

## Prefilter Algorithm
- 输入流程：HDR equirectangular 以线性 HDR 读入，双线性投影成 cubemap base face；面尺寸按 cmft/计划 06 规则 `clamp(next_pow2((H + 1) / 2), 64, 1024)`，所以 Poly Haven 1K(1024x512)生成 256 face，2K(2048x1024)生成 512 face。
- 普通 mip：先生成 source cubemap mip chain，跨 face 的下采样 tap 必须重投影到相邻面；source mip chain 可供 skybox 缩小显示和 FIS 选源 mip，不得直接当作粗糙反射 PMREM。
- Specular PMREM：每个 face/mip 执行 UE 风格 GGX filtered importance sampling；默认 CPU bridge 使用 low/mid/high 32/64/128 样本，GPU/offline reference 可保留 1024 样本质量档用于对拍。
- Roughness/mip 关系采用 Unreal 常量：`ROUGHEST_MIP = 1`，`ROUGHNESS_MIP_SCALE = 1.2`，并实现正反 roundtrip 测试。
- 低粗糙度 `< 0.01` 直接取 source mip0；常规 GGX 样本用 `PDF = D_GGX * 0.25` 和 solid-angle 公式选择 source mip；高粗糙度饱和尾部可从上一层 PMREM 继续降采样，最终 1x1 mip 六面平均。
- cmft 参考路径提供 face order、lat-long 公式、exact solid angle、edge/seam 邻面访问、final mip averaging 与 irradiance/IEM 参考；cmft 的 Blinn/Phong power 滤波和线性 LOD 只作对照,不作为 Zircon PBR 运行时标准。

## 2026-07-06 Reference Design Lock
- **拒收旧图**:`runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 使用 16x8 scene uniform sample table,不是 cube texture,也没有 PMREM;其马赛克效果是旧路径缺陷,不是可调参数问题。
- **天空盒**:必须显示 source cubemap mip0。PMREM 是反射用辐射贴图,不能拿来当背景天空,否则低 face size 或粗 mip 会直接污染第一屏。
- **反射**:材质 roughness 只映射到 specular PMREM;source mip chain 仅是 FIS 的输入 pyramid。低 roughness 金属应能看清 HDRI 高频结构,高 roughness 反射应随 mip 逐级模糊且高频能量下降。
- **diffuse**:默认 SH9,可选 32^3 IEM。cmftStudio 的 `u_texPmrem` / `u_texIem` 消费分工可作为 UI/调试参考,但 Zircon 的 specular 必须走 UE GGX + BRDF LUT。
- **后续代码切片**:不要继续膨胀 `source_cubemap.rs`;新增 IEM、GPU/offline bake、derived cache 或更严截图指标时拆分到独立 owner,同步模块文档和状态表。

## Test Plan
- 单元测试：equirect → cubemap face 顺序、cube view descriptor、mip/layer 上传偏移、roughness ↔ mip roundtrip、SH9 constant-color、PMREM blur 单调性。
- Shader 检查：cubemap HDRI 路径必须包含 `texture_cube` / `textureSampleLevel`，并确认不再走 16x8 采样表。
- 产品截图：重新导出 8x8 PBR 球阵，列为金属度 0→1，行为 smoothness 0→1，使用真实 HDRI cubemap 天空盒和环境反射。
- 验证图：保存为 `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_YYYYMMDD.png`，可额外输出 mip 诊断图到同目录；2026-07-06 的 `runtime_shader_pbr_real_hdri_lakes_1k_pmrem_mip_diagnostic_20260706.png` 是 source mip 与 GGX PMREM mip 的 raw 数据检查图，低 mip 被放大显示时天然更粗，不等同于运行时天空盒或 trilinear PMREM 采样效果。
- 质量验收：天空盒无 16x8 马赛克；高 smoothness 金属球能看到清晰环境；低 smoothness 反射随 mip 逐级模糊；高频能量随 roughness 单调下降。
- 路径验收：扫描确认新验证图没有写入 `target`。
- 最终验证：运行 `zircon_runtime` 相关 render/texture/environment 测试；共享接口改动完成后跑仓库现有验证矩阵的 runtime 范围。旧 2026-07-04 图必须保留为 rejected baseline,不得作为通过图复用。

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
