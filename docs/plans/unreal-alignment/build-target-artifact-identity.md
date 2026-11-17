# BUILD-01：显式 target 的产物查找丢失身份，可能复制旧 host／另一 target 的同名文件

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

## 1. 差异：两边路径、符号与调用链

**结论：一条确定的独立实现缺陷；不是要求 ZE 复制 UE 架构。** 已完成目录盘点后才评估本问题，目录数量不作为行为审计证据。本报告不含修复、不执行 Cargo/UBT/引擎测试、不修改引擎或 Git，不声称发生过实际发布事故。

ZE 原任务指定只读 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；lead 要求正式固定 blob 引用采用 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。**本问题两边同时存在**：`find_artifact` 的 AST 完全相同。本次使用绑定的 Python **3.12.14**，按 `ast.dump(fn, include_attributes=False)` 的 UTF-8 字节计算，SHA256 为 `69809f2338b721684a19a7ebcbcef0e440cf5edf9ee29b98ed706e78b3fee78b`；该 AST 序列化散列不保证跨 Python 版本相同，应优先用末尾固定 blob 原字节 SHA256 核对。两次函数级复现分别从两个 commit 的 Git blob 在内存提取；不是拿本地工作区代替固定版本。e893 的历史 `main` 仍含 Jenkins 转发入口，本报告没有执行该历史 CLI，也不以函数复现声称它已实际完成 staging；本问题与入口退役差异独立。

| ZE 固定证据 | 符号与已读事实 |
|---|---|
| [zircon_build_config.py:79–82](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_config.py#L79-L82>) | `BuildConfig.effective_target_triple` 选择 product profile 的 target，或调用者显式 target。 |
| [zircon_build.py:859–890](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L859-L890>) | `run_cargo` 将这个 target 传给 Cargo；在本机 cb 对应 [849–880](<E:/Git/ZirconEngine/tools/build/zircon_build.py#L849-L880>)。 |
| [zircon_build.py:903–943](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L903-L943>) | `copy_artifact` 调用 `find_artifact(target_dir, profile_dir, artifact_name)`，未传 target；后者优先 `<target_dir>/<profile>`，随后按排序扫描所有 `*/<profile>` 并取首个同名普通文件。在 cb 为 [893–933](<E:/Git/ZirconEngine/tools/build/zircon_build.py#L893-L933>)。 |
| [zircon_build.py:637–724](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L637-L724>)、[752–778](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L752-L778>)、[811–827](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L811-L827>) | runtime DLL／preview、editor host、native plugin 都通过上述查找落到最终复制路径。 |

ZE 调用链：`build → build_runtime / build_editor → run_cargo(--target=T) → copy_artifact → find_artifact(没有T) → copy_file`；native plugin 的链在 `build_native_dynamic_plugin` 直接进入同一个 `find_artifact`。

UE 对应链（版本 6.0.0、BranchName UE5，仅作短说明，不转载正文）：[UEBuildTarget.cs:5646–5668](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildTarget.cs#L5646-L5668>) 的 `MakeBinaryPaths` 由 target/platform/configuration/architectures 计算确定输出；[UEBuildBinary.cs:578–588](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildBinary.cs#L578-L588>) 将确切路径登记为 `BuildProduct`；[UEBuildTarget.cs:2495–2536](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildTarget.cs#L2495-L2536>) 的 `PrepareReceipt` 保留 target 身份及产物路径；[CompileTask.cs:158–199](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CompileTask.cs#L158-L199>) 经 `UnrealBuild.Build`，从 `targetToManifest[target]` 获取输出。它不是在共享输出树按 basename 猜产物。

## 2. 影响：已复现什么，未验证什么

当相同 target-dir/profile 被复用，且存在优先命中的旧 flat 产物，或排序优先于请求 triple 的其他 target 同名产物时即可触发，不要求编译失败；并非任意同名旧文件都必然触发。两份固定 blob 原函数均在纯内存虚拟路径上获得下列结果；虚拟文件不是本机真实产物，也没有写任何 DLL。

| 虚拟输入 | 正确选择 | 实际选择（e893 与 cb 一致） |
|---|---|---|
| flat `shipping/zircon_runtime.dll` 旧文件，与 `x86_64-pc-windows-msvc/shipping/zircon_runtime.dll` 新文件并存 | 明确请求的 x86_64 新文件 | flat 旧文件 |
| 没有 flat，但 `aarch64-pc-windows-msvc/shipping/zircon_runtime.dll` 与 x86_64 新文件并存 | x86_64 新文件 | 排序靠前的 aarch64 文件 |
| 无显式 target，只有 flat 文件（兼容性对照） | flat 文件 | flat 文件，正确 |

又以 e893 原 `copy_artifact`、`find_artifact`、staging 的 `_source_candidates` / `_find_source` 联合做纯内存验证：配置明确持有 x86_64 target，新旧文件字节不同，**实际 copy 来源和 sidecar 来源仍为 flat 旧文件**；模拟暂存后的 SHA256 provenance 查找接受这个 flat 旧来源。相关源码为 [zircon_build_staging_manifest.py:142–180](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_staging_manifest.py#L142-L180>)：显式 target 下候选依然包含 legacy profile root。

后果是“编译了请求目标，却暂存另一个构建的文件”，可能造成旧功能闭包、DLL/host 不配套、错误架构或旧符号。Shipping 的 PE/ABI 校验可阻止部分错误架构，但**不能仅凭架构/导出形状保证是本次新构建**；同架构旧库不会因此必然被拒。这里验证的是函数选择与哈希来源接受，不是用真实 PE 跑通全部发布门。

## 3. 根因与 UE 的解决方式

根因是 target 身份只到达“生产命令”，没有到达“选择产物”的接口；为兼容旧布局保留的优先级因此覆盖了显式 target。不是 Cargo 返回值问题，也不是运行时 ECS 问题。

“先清理 product output 就不会触发”不能反证本问题：[clean_product_output:34–43](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_product_output.py#L34-L43>) 清理 engine/symbols/reports/archive 等 generation 根，**不清空 `targets_root` 的 Cargo 缓存树**。复用缓存本身合理，不能因此丢失明确 target。

UE 的具体做法是保持确定产物路径及本次 invocation 的 manifest：[UnrealBuild.cs:145–208](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/UnrealBuild.cs#L145-L208>) 在 UBT 调用前删除本次指定的旧 manifest，运行后读取指定 manifest 并按 BuildTarget 绑定；[UEBuildTarget.cs:2255–2299](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildTarget.cs#L2255-L2299>) 写入确切 BuildProducts；[CompileTask.cs:193–199](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CompileTask.cs#L193-L199>) 找不到对应 target 的 manifest 就报错。`TargetReceipt` 记录 platform/configuration/architectures；其文件名是否带 architecture 后缀由平台策略决定，不能概括成所有平台都强制同一种目录命名。

## 4. Rust 借鉴：借身份传递，不移植 UBT

ZE 已有 Cargo、`BuildConfig.effective_target_triple` 和 product build-set 身份，不需要新建 Rust 构建器或引入 UE 的 CSharp rules 系统。可借鉴的最小原则是：**确定的构建选择必须贯穿命令、产物路径选择及 provenance**，不能事后仅用文件名或 mtime 猜身份。

若 Rust 侧消费现有构建结果，也应从现有配置/回执传入目标、profile、逻辑产物及来源路径，按同一身份核对；不把 Rust ABI 可加载当作产物来源证明。本问题当前 owner 是既有 Python 构建工具，修复应先留在这个 owner，不把无关引擎模块重构夹进来。

## 5. 接口与 ECS 差异：明确不改哪些边界

UE 的 `BuildProduct` / `BuildManifest` / `TargetReceipt` 是构建与打包数据，不是 Actor/组件存储。ZE 的 Cargo package、动态 runtime DLL 和 neutral runtime-interface 也不等同 UE 的 Build.cs 模块身份。

因此修复不涉及 ZE ECS entity/component、scene world、runtime lifecycle、UI ABI 或插件加载 ABI；也不需要把 UE 的 UObject/Actor 反射模型搬过来。只需在现有构建函数接口保留调用者已拥有的 target 身份。缺陷范围限于本链条的目标选择及来源约束；本报告不另立架构、性能或安全问题。

## 6. 具体 ZE 改法草案、测试迁移表及证据身份

**草案，尚未实施：**

1. 给原 `find_artifact` 增加显式 target 参数，由 `copy_artifact` 和 native plugin 调用点传入既有 `effective_target_triple`。有 target 时只在对应 `<target_dir>/<T>/<profile>` 的允许位置查找；不得先选 flat，也不得扫描其他 triple 来兜底。找不到请求目标则报错。
2. 无显式 target 时保留既有 host flat 布局的正常用法；与修复无关的符号、发布命名和缓存复用不改。不要用“每次删光共享 target”掩盖选择错误。
3. staging 的 `profile_candidates` 在显式 target 下也限定同一 target，防止同一身份丢失链条被 provenance 的 legacy fallback 重新接受。无需新增全局产物架构。
4. 回归放入现有 Python 工具测试 owner。下面名字带“拟新增”的都不是既有 UE/ZE 测试名。本次禁止写引擎/测试文件，因此只交草案和内存复现，不交修复或伪造 GREEN。

**原测试搜索范围：**已 glob/grep [UnrealBuildTool.Tests](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool.Tests>)、[EpicGames.Build.Tests](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.Build.Tests>)、[EpicGames.BuildGraph.Tests](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.BuildGraph.Tests>)、[AutomationTool.Tests](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool.Tests>) 中的 CSharp 文件，关键词为 `MakeBinaryPaths / BuildProducts / TargetReceipt / CompileTask / artifact / receipt / manifest / output`。没有找到直接覆盖“共享 Cargo 输出树中旧 host 与显式 triple 同名文件竞争”的 UE 原测试；这不是对 UE 全仓测试的不存在证明。

| UE 原测试名／路径行或推导来源 | 输入 | UE 原期望／本问题建议期望 | ZE 现有落点与迁移状态 |
|---|---|---|---|
| `TestTargetRequiresUniqueEnvironmentFirstOrder`，[TargetRulesTests.cs:14–87](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool.Tests/TargetRulesTests.cs#L14-L87>) | 未改规则、修改需独立环境的 warning、显式 override | 原测试分别检查不需要／需要／override 后不需要独立环境 | 对照 [ProductBuildProfileTests:37–61](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_product_profiles.py#L37-L61>) 的 target/identity 现有测试；**仅相关原测试阅读，不作为本缺陷直接迁移**。 |
| `NodeTest`，[GraphTests.cs:27–52](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.BuildGraph.Tests/GraphTests.cs#L27-L52>) | UpdateVersion → Compile → Cook，显式 Requires 与 DefaultOutput | Cook 的传递输入依赖为 2，Compile 的为 1，按 producing node 关联输出 | 对照 [RuntimeBuildOrchestrationTests:23–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_runtime_orchestration.py#L23-L65>)；该 ZE 现有测试 mock 了 copy 层，**不是本问题原回归迁移**。 |
| **源码推导，非迁移；无直接 UE 原测试**。[CompileTask.cs:193–199](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CompileTask.cs#L193-L199>) 的 target→manifest 约束 | 旧 flat DLL 与新请求 triple DLL 并存 | 应取请求 triple；旧 host 不得覆盖。现函数实测错误 | 现有 [test_zircon_build_runtime_orchestration.py](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_runtime_orchestration.py>)，拟新增 `test_explicit_target_artifact_ignores_flat_host_output`。 |
| **源码推导，非迁移；无直接 UE 原测试**。[MakeBinaryPaths:5646–5668](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildTarget.cs#L5646-L5668>) | 无 flat；另一 triple 名排序靠前；请求 triple 存在／不存在 | 存在则只取请求 triple；不存在则报错，不能取 sibling | 同一现有 orchestration 测试 owner，拟新增 `test_artifact_lookup_is_scoped_to_requested_target`。 |
| **源码推导，非迁移；无直接 UE 原测试** | 无显式 target，仅 flat host | 应保持旧 host 布局；内存对照已正确 | 同一现有 orchestration 测试 owner，拟新增 targetless 兼容性用例。 |
| **源码推导，非迁移；无直接 UE 原测试**。[staging source candidates:153–180](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_staging_manifest.py#L153-L180>) | 显式 T、新 T 文件与 staged 旧 host 字节不符，但旧 host 字节匹配 | 不允许以 legacy hash match 证明请求 T 的来源；当前联合内存验证接受了旧来源 | 现有 [test_manifest_binds_staged_files_to_sources_and_hashes:11–89](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_staging_manifest.py#L11-L89>) 仅覆盖 flat；拟补 target 约束用例。 |

现有测试只读审查，没有运行这些测试套件：所读部分工具测试会创建真实临时目录，超出本次仅允许 research 报告输出的范围。内存复现只执行从固定 blob 选取的原函数 AST，路径/内容/复制操作全是虚拟对象；没有导入构建工具、调用 Cargo 或接触发布目录。独立只读复核再次确认两基线的选择、copy/sidecar 和 provenance 结果一致；未发现清理或 ABI 层能推翻本问题的上游不变量，仍保留“不等于真实 PE／完整产品发布验收”的限制。

**UE 源文件 SHA256（仅元数据，供本机复核）：**

| 源文件 | SHA256 |
|---|---|
| [UEBuildTarget.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildTarget.cs>) | `f868dfab180c93b8ebbebc16d9f3406f7666e26ff337e786df9106cfe824bb06` |
| [UEBuildBinary.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Build/UEBuildBinary.cs>) | `4c0fca77b09afdb34c0d0a7bc7a12038bedf0d2a65d853126006418ef9192297` |
| [TargetReceipt.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs>) | `eb0ae686d5c99b4d54c96dee099bf22b42306177c28fa4429fc536fa1140cbd7` |
| [CompileTask.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CompileTask.cs>) | `e00cdb87b1f52816db78b7fdbbfd0019dd0d57fafb053ec5d9f860c6b89f9669` |
| [UnrealBuild.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/UnrealBuild.cs>) | `7d49c97ac8b9dad22138a4dc44d16ab731eaffb7463b14c793a52efca4cea0ac` |
| [TargetRulesTests.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool.Tests/TargetRulesTests.cs>) | `89a438e2e92e494c5e870290b36ca19493a127102d4e6a027fdc73f7ea50c345` |
| [GraphTests.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.BuildGraph.Tests/GraphTests.cs>) | `245bf798fd5a6a3e89b0575f1b2b4b15172d57af3b77d69c50708d5631960ace` |
| [DotnetProjectInfoTests.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.Build.Tests/DotnetProjectInfoTests.cs>) | `8dcc133ace2eff5eaada0c4cc95a3508277890d2134e626c898299ee386dc813` |
| [PackageGraphTests.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Programs/AutomationTool.Tests/Oplog/PackageGraphTests.cs>) | `40a93124cfcadd290c88f46bb27bc7fb0bc3d7f845aa5391b6e84fe3e58aa6fc` |

**ZE 固定 blob SHA256：**主构建文件 e893 为 `38c70befc02ec23b18e461345a22208cbf17cf6ab52efba3747abaf9de8aef35`，cb 为 `b81044950db5e3d0ff850ee2ff3aaf3ff8da048d900e9fb9061894593a210857`。整体文件差异来自入口退役与输出路径处理等本地提交，但本问题函数 AST 相同；并非将 cb 已修复问题重提。e893/cb 的 staging owner 相同，SHA256 `93ec10729671f5e6588b6d37e54ffb2ee0e42fea479d6feeef289b5389f64d1d`；BuildConfig 相同，SHA256 `692c73bd2fc1b1743a5dba3976dc5b25fc6f03eef2249328ef3173e5418ed06a`。
