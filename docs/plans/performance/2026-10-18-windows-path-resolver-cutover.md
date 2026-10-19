# S69：Windows 路径解析器导入迁移

必需导入仍指向已不存在的 `tools/common`、`tools` 根目录和 `tools/analysis/common`，使验证入口在读取产品证据前停止。本次已将 11 个实际消费者接到现有 [maintenance 解析器](../../../tools/maintenance/WindowsPathResolver.psm1)，保留导入顺序、参数和函数实现。

`Invoke-MvpAcceptance.ps1:38` 先导入 BuildSummary，后者第 10 行已引用缺失路径；因此入口第 55 行与传递依赖需一起迁移。验收入口及其测试最后一次强制导入继续保留。解析器的 `OperationalPath` 用于物理路径和 I/O，`DisplayPath` 用于显示及 System.Drawing；现有契约保持一致。

| 消费者 | 修改行 |
|---|---:|
| [tools/mvp/Invoke-MvpAcceptance.ps1](../../../tools/mvp/Invoke-MvpAcceptance.ps1#L55) | 55 |
| [tools/mvp/MvpBuildSummaryEvidence.psm1](../../../tools/mvp/MvpBuildSummaryEvidence.psm1#L10) | 10 |
| [tools/mvp/MvpTestFixturePaths.psm1](../../../tools/mvp/MvpTestFixturePaths.psm1#L3) | 3 |
| [tools/mvp/RenderExtractBaselineEvidence.psm1](../../../tools/mvp/RenderExtractBaselineEvidence.psm1#L4) | 4 |
| [tools/mvp/RenderExtractMachineEvidence.psm1](../../../tools/mvp/RenderExtractMachineEvidence.psm1#L4) | 4 |
| [tools/mvp/ResourceManagementJsonEvidence.psm1](../../../tools/mvp/ResourceManagementJsonEvidence.psm1#L4) | 4 |
| [tools/tests/mvp-acceptance.Tests.ps1](../../../tools/tests/mvp-acceptance.Tests.ps1#L340) | 340 |
| [tools/mvp/RenderExtractSourceIdentity.psm1](../../../tools/mvp/RenderExtractSourceIdentity.psm1#L6) | 6 |
| [tools/tests/test_managed_cargo_storage_windows_powershell_contract.py](../../../tools/tests/test_managed_cargo_storage_windows_powershell_contract.py#L8) | 8 |
| [tools/analysis/validation/check-runtime-domain-features.ps1](../../../tools/analysis/validation/check-runtime-domain-features.ps1#L13) | 13 |
| [tools/analysis/validation/check-runtime-profile-features.ps1](../../../tools/analysis/validation/check-runtime-profile-features.ps1#L13) | 13 |

3 个 Luna/max 并行审查了解析器、消费者和引用范围，Sol/max 独立裁决了这 11 项修复。实施后的独立复核验证了 11/11 逆向 SHA、其余字节、BOM 和换行保持一致，所有新目标均解析到同一现存文件。`zircon-change-guard` 对实际补丁返回 pass，errors/warnings/suppressed 均为 0；静态 diff 检查通过。

实际基线为 `main a80b4c1e23d7d4910782112c2cdfc8025851262b`。本次范围差异为 11 行替换、净增 68 B；[完整指纹与审查报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s69-windows-path-resolver-cutover.json) 记录前后版本。guard 检查的 [精确补丁](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s69-resolver-import-cutover.patch) SHA-256 为 `4bf0ed879c26372209e527a206f9b863fc4008dd9e6c46d845297a6f9786108a`，对应 [实际 guard 回执](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s69-resolver-import-cutover.guard.json)。历史报告继续绑定其当时源码。

既有问题 `S56-MVP-FIXTURE-RETIRED-DEPENDENCY` 继续开放：本次完成解析器位置子问题；夹具第 7/16 行仍引用退役的 `zircon-session.ps1`，其 Jenkins 归属、物理命名空间和清理生命周期仍待迁移。统一问题队列仍为 27 项。

Jenkins 后续先核对补丁、语法和目标路径，再用新 PowerShell 进程执行现有解析器契约、模块导入和 Windows PowerShell 存储契约；MVP 及 render/resource evidence 回归需有实际受管夹具 runner。两项 feature checker 的完整 Cargo 矩阵不属于本次导入修复所需验证。

本轮完成源码修复与独立静态复核。模块导入、行为测试、当前源 MVP00、产品和性能验收均待执行，遵循 [当前 MVP 门槛](../../testing-and-validation/mvp-acceptance-index.md#current-execution)。[Jenkins 交接消息](../../../.jenkins/records/engine-audit-20261009-01a12097-s69-resolver-import-cutover-handoff.md) 已保留，读取与执行回执仍待返回；原 S50 身份保持不变。

保留本轮源码、补丁和必要证据；实际释放 0 B。其他会话及未知归属产物继续保留，清理状态未关闭。
