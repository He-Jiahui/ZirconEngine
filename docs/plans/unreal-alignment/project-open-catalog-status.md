# 项目打开诊断：没有已发布的 Editor 目录代时，不应标记 `completed`

阶段：独立问题定义，只有文档。ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。这是**完成状态的误报**，不是对“为什么目录没有发布”的归因；纠正标签也不会让一条资产出现。以下分类由该基线生产源码和已有验收规则确定，新增 Rust 回归尚未执行。

## 1. 具体差异

[打开工程的生产路径](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/project_access.rs#L27-L74) 完成 Runtime 激活、调用 Editor 的刷新、加载默认文档，随后从 **Editor** 快照取 `catalog_revision`、`publish_epoch`、行数，连同 **Runtime** 的就绪统计写入 `editor_project_open`。日志里前两个数虽叫 `project_generation` 和 `project_generation_publish_epoch`，并非 Runtime 输入序号。[格式化器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/project_access.rs#L389-L431) 只用 Runtime 统计和设置来源选择 `completed/degraded`，把 Editor 代身份当作纯展示字段。

可精确判定的一组输入：4 项资产、4 项 Ready、0 项失败，设置 `persisted-v1`，但 Editor `(revision, epoch, rows)=(0,0,0)`。生产表达式仍给 `result=completed`。[目录默认代与首次递增](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/generation.rs#L72-L91) 明确区分尚未发布的代与已经发布、恰好没有行的代；不能以行数代替代身份。

## 2. 影响与不受影响的边界

- [创建阶段的存活探针](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProcessLivenessProbe.psm1#L30-L37) 将 `editor_project_open result=completed` 当作**进度标记**。它可能先报告打开阶段已到达，却没有与此匹配的目录代。不是说最终验收误通过。
- [正式校验先看 result](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProjectOpenEvidence.psm1#L153-L165)，并且[另查代身份与模板资产](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProjectOpenEvidence.psm1#L227-L255)：任一个身份为 0 即拒绝；至少 4 个目录项只属于 RenderableEmpty 产品样例，**不是**所有项目的下限。阶段脚本在[读取本次诊断文件后解析记录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/Stage-MvpProducts.ps1#L1751-L1789)，取的是[这段文本最后一条打开标记](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProjectOpenEvidence.psm1#L29-L54)。稍后刷新目录不会改写一条已发的日志。
- [#157 的原生现场](https://github.com/He-Jiahui/ZirconEngine/blob/7657c511d41cf261f4c78f5cd226bdd373a9090b/docs/plans/unreal-alignment/project-scene-suffix-case.md#L96-L111) 说明初始 0/0 快照确实可被已注册服务返回；它**没有捕获该条日志内容**。本问题不把上述 4/4/0 例子假称为那次实测，不声称窗口、目录最终一致性、磁盘持久性或 F1/F5 在本轮已验证。

## 3. 为什么会误报；UE 在哪里区分“仍在加载”

根因限日志 producer：它已经拿到两个身份，却未把它们纳入完成条件。`refresh_from_runtime_project` 的 `Ok(())` 也不是首代一定已发布的凭证；例如[过期候选允许无副作用退出](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/project_sync/sync_from_project.rs#L311-L343)。不必先解释某次为何过期，才能判断一条给定输入的日志是否误报。

本机 Unreal 参考树 `6.0.0 / UE5 / CL 0`，上游提交未知。`Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp:4691–4717` 的 `IsLoadingAssets`、`IsGathering` 显式区分初始搜索完成与追加搜索；`Engine/Source/Editor/ContentBrowser/Private/CollectionAssetRegistryBridge.cpp:142–156` 在加载中订阅 `OnFilesLoaded`，已经就绪则直接运行完成处理。借用的是**用就绪证据解释完成状态**，并非 UE 实现了同一日志协议或同一种 Editor 目录代。两文件 SHA-256 分别为 `071f916324ff4e714329416559b3dce37be493b9df22a3dcb98988c58a4ca9fc`、`52ae8765567cc2820fab888282ad038d3d042106597cbaec327406ae72c05e9c`。

## 4. 适合 Rust 的做法

在 `project_opened_diagnostic` 生成 `result` 时，同时要求已有 Runtime/设置条件成立、Editor 两个身份均非零；否则沿用原有 `degraded`。保留实际数值和字段名，保留打开函数的 `Result`、日志严重度和用户操作语义。不等待、不重试、不主动填行。不要把“目录项必须 ≥4”塞进通用分类器。

## 5. 接口、ABI、ECS 和所有者

这是 Editor 私有诊断格式化行为，不是 ABI、插件协议、ECS 或运行时序号分配变更。公开日志的 key 及现有 result 枚举不变。[启动状态文案](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/startup/create_or_open.rs#L35-L65) 另有共享 Runtime/settings 谓词且手中没有该快照；不把这个局部观察传遍 UI，也不混改 watcher、Runtime fence、资源导入、场景后缀、正式校验脚本。

## 6. 等价回归与修复门槛

- **真实 UE 原测参照（源码已核，未执行）：** `Engine/Plugins/Tests/CQTest/Source/CQTestTests/Private/Helpers/CQTestAssetHelperTests.cpp:27–41,132–143,168–179` 的 `FindAssets_WithFilter_ReturnsAssetArray` 和 `FindAssetPackageByName_WrongPackagePath_ReturnsNullOpt` 在实际 Editor AssetTools 创建并通知 registry 后，检验正确包查询恰得一项有效对象、错误包无结果；测试辅助 `Engine/Source/Developer/CQTest/Private/Helpers/CQTestAssetHelper.cpp:13–31,96–101` 在加载中可同步搜索。两文件 SHA-256 `764cf587bfb5462d9c416f69b2a0739b0fd26e97d25a3eb0b2200f65bf0df567` 和 `a02260f755b389fc11b81085b57476e43b9bd0feaf74ee90109e08cdb17ae0e3`。这些不是 UE 对 ZE 诊断字符串的断言；没有看到可以逐字迁移的该类 UE 测试。
- **ZE 新增，非 UE 原测：** 在[已有 production formatter 单测组](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/tests/project_access.rs#L13-L130) 固定参数，只改变代身份或行数。验证 0/0 及任一身份为 0 均退化；两者非零且 0 行仍可完成。保留原有成功、导入失败、未全 Ready、设置 fallback、Windows 路径转义。校验完整结果 token 与原始数字，不能依靠名字或字符串源扫描。
- **同 PR 顺序：** 先提交测试及原生产实际 RED；独立核实施改不影响共享谓词与现有正式拒绝条件，再只改 formatter，重跑同一组冻结用例。若需证明具体项目在 staged Editor 的真实日志或消费者效果，另用标准创建/重开产品门；此局部修正不能让缺行的目录回归变绿，也不能代替首帧及收尾证据。

## 7. Rust 生产者：原实现实际输出（2026-11-18）

[纯测试提交 `1fb3a09f8380ba81a89cb199fdeef739843e64fe`](https://github.com/He-Jiahui/ZirconEngine/blob/1fb3a09f8380ba81a89cb199fdeef739843e64fe/zircon_editor/src/ui/host/tests/project_access.rs#L132-L191) 直接调用原 Rust `project_opened_diagnostic`，不调用 PowerShell，不读模拟结果。Windows x64/MSVC，Rust 1.94.1；`test -p zircon_editor --no-default-features --lib ::project_open_diagnostic_ --locked --offline --jobs 1 -- --test-threads 1`。原有 5 案加 4 个新案，共 **6 过、3 败、0 忽略、10,535 过滤；exit 101**。

| Runtime 资产 / Ready / 失败 | Editor revision / epoch / 行 | 原生成器实际 `result` | 期望 |
|---|---|---|---|
| 4 / 4 / 0 | 0 / 0 / 0 | `completed` | `degraded` |
| 4 / 4 / 0 | 0 / 9 / 4 | `completed` | `degraded` |
| 4 / 4 / 0 | 3 / 0 / 4 | `completed` | `degraded` |
| 0 / 0 / 0 | 3 / 9 / 0 | `completed` | `completed` |

三份失败的输出向量各有 13 个 token，**唯一差异是 `result`**；真实代数值、行数、Runtime 计数、设置来源和编码后的路径等均一致。两个单零是防御性输入，不称它们是正常发布现场。旧的成功、Windows 路径、失败导入、未全 Ready、设置 fallback 均通过。[#202](https://github.com/He-Jiahui/ZirconEngine/pull/202) 的现场原本已 `degraded`，不用于填补这三个 RED；8 组 PowerShell 消费端对照也不是本次结果。

回执 `54607124af706f11b99520e0f3d709c1a4503aec33e0a8507f7d074dc11ee101`、stdout `89b86c49233b05fa23f7f0b02cf4e99e55d9e356a511bc18e5f9e96c39941a6f`（5,005 字节）、stderr `bf3c8e440202d523f3a2b6d9abb4ba3f960f6007d1bcf9cf290a4591496b5959`（992,262 字节）。测试文件 SHA-256 `07ec0ed5b8ffd255fadf330b949945b6fa5f3efac6e9299cfc1d6bc0f09795cf`，18 项选定输入复核，非整包/产品验收。此阶段只加测试和证据，未改生产。
