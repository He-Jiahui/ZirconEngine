# 同次 Cook→Pack：显式清单既是 Cook 输入，又被当成 Pack 成品

基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本提交只调查一个已交付的命令行交接，不改引擎、不改 Pack 格式。两个独立阶段各有明确合同；矛盾出现在**一次命令确实先执行 CookAssets、再执行 Pack**。这里不把旧项目文本扫描、实际运行时挂载、桌面向导或“没有实现 UE 的完整烘焙系统”列为本题。

## 1. 普通用户入口与对照

在支持 `library_embed` 的桌面工程中，执行 `python -B -m tools.export --profile windows-release --project <project>/zircon-project.toml --out <out> --resume-from cook_assets --asset-manifest <project>/source_assets.json`。使用工程里确实存在的默认场景、模型和材质，以及一份 Cook 明确支持的 JSON、普通 Windows 路径分隔符。不要伪造哈希、报告或制造不同版本的工具。[CLI 参数帮助](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/cli_arguments.py#L143-L158)说明这是同一选项的两种含义。[恢复时的执行顺序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/pipeline_stages.py#L22-L65)包含 CookAssets、Pack，再是后续交付阶段。

对照只改**从哪里指定同一份文件**：在工程清单使用 `asset_manifest = "source_assets.json"`（命令行不再传此选项），或依次执行 `--stage cook_assets --asset-manifest ...` 与 `--stage pack`。使用新的 out，核对两种 Cook 输入的内容、生成清单、过滤标签和包内路径。另需证明直接 `--stage pack`、从 Pack 开始 `--resume-from pack` 时，显式已规范的 Pack 输入仍然有效。

## 2. 精确的生产者与消费者

[CookAssets 读源再写归一化产物](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/cook_assets.py#L122-L164)；[报告分开记源和产物](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/cook_assets.py#L171-L199)。[每个阶段重新复制原参数](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/cli.py#L98-L110)导致 Cook 用过的显式源仍在后一个阶段的参数里；[Pack 的参数默认值](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/cli.py#L138-L156)和 [真正的 Pack stage](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/pack_stage.py#L60-L90)都只在其为空时才取 Cook report。[实际标签为 --manifest 的 packer 实参](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/pack_stage.py#L273-L297)因此选的是**输入来源**，不是本次 Cook 产物。Rust [writer 按这个参数打开清单](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_export_pack/run.rs#L46-L80)并记录 `asset_manifest`。这与两份文件是否同名、内容是否恰好一样是两回事。

## 3. 具体影响与限制

源清单可以包含 Windows 分隔符，由 Cook 负责归一化；见[原 ZE 规格测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/test_cook_assets_manifest_schema_gate.py#L346-L392)。若 Pack 收到未归一的源，严格 writer 可拒绝；即便已是 `/` 且刚好能写出包，Cook 注入的 profile 过滤可能不在源清单中，[最终源/产物同一性校验](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/pipeline_report_cook_assets_pack_handoff.py#L13-L43)仍应拒绝两条不同的来源路径。这里是**源码推导**，真实 Pack 阶段和容器内容尚待实测；不声称已经发布了坏包或任何终端用户已丢失文件。

## 4. UE 采取的办法、现有原测与不可比之处

`BuildCookRun.DoBuildCookRun`（`Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs:251–267`）依序 Cook、Stage 和 Package。`从平台 Cook 目录准备 StageFiles`（`Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs:2156–2204`）按已选平台输出归集；`ApplyStagingManifest`（`Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs:6528–6562`）将同一 staging context 交给选定的 pak 路径。ZE 可沿用的是**让后续阶段消费前一阶段实际产物**，无需照搬 Unreal 的生产类型或 IoStore。

现存 Gauntlet `UE.CookByTheBookEditor`（`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.CookByTheBookEditor.cs:70–99`）及其 `IsCookedContentPlacedCorrectly`（`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.CookByTheBookEditor.cs:171–207`）核对烘焙后内容目录，`ContentFolderEqualityComparer`（`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/CookUtils/ContentFolderEqualityComparer.cs:9–35`）检查预期项存在，并非逐字节整树相等。没有把它当成 ZE 这组双义 CLI 参数的同形自动测试。参考镜像为 6.0.0 / UE5 / CL 0；未执行 UE、不包含 UE 源码，也没有可靠上游 Git revision。

## 5. 容器和历史合同的边界

[文档规定显式 Pack 输入优先](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/tooling/zircon-export-tool.md#L2655-L2666)与[最终报告规定同一次发布必须来自 Cook 产物](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/tooling/zircon-export-tool.md#L2685-L2705)同时存在。正确方案应明确作用域，而不是放松路径规范或修改日志让两者看上去相等。建议将同一次已成功 Cook 的输入归 Cook，随后 Pack 读取这次的报告；直接 Pack 的显式覆盖、空值拒绝及没有执行 Cook 的 dry-run 应保持可区分。若需兼容完整主链中专门的独立 Pack override，应另明确命名/早期诊断，不能把它伪装成同源交付。并不预先断定新开关或人为改变发布判据是唯一修法。

[已有 Pack/恢复/空值测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/test_pipeline_resume_flow.py#L382-L492)从 Pack 开始；[已有 Stage Report 来源测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/test_pipeline_report_stage_metadata_assets_pack_schema.py#L220-L252)约束的是报告。Tooling03 的正则扫描、Runtime207 的 typed dependency 图以及 [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) 的构建目标身份是不同的问题。已比对全部状态的 PR 标题、正文、相关文件及当前开放路径，没有发现相同入口与交接冲突的 Draft。

## 6. 等价回归与最小修复门

先在本 PR 增 tests-only：普通 RenderableEmpty 默认场景与真实材质/模型，明确 Cook 源，在 CLI `main` 中分别输入项目字段与命令行参数；记录两次真实 Cook 生成的字节、所有路径、过滤以及 Pack 实参。若测试在外部 packer 入口用非零哨兵，不据此声称写包成功。另用与原提交绑定的 `zircon_export_pack --features native-host` 复核真包：baseline 与修后均实查所读清单、Pack exit、真实 trim/写入记录及包内 manifest 和 source bytes；可在后续 Bundle 前明确停止，不填造 Bundle 或完整 Report 的成功。保留 writer 的严格规则、最终路径拒绝与独立 Pack override。没有到实际业务断言前不修改生产。

UE 对照文件 SHA-256：

- `Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/CookUtils/ContentFolderEqualityComparer.cs`：`e37ec247824fcba98cab014bbb33357e7b1e4e0593a6f8912d1f6b71acc84461`
- `Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.CookByTheBookEditor.cs`：`aa9f4de9d25077bf5e2b23ad6ab927012cbf2c040448e30de71d38aac747fda8`
- `Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs`：`4b937241960bb31887399ba1821d2d43cdd3a354f2d548b42d0133554742c582`
- `Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs`：`afe9c16de253e2b6a5f25d4a7db4520ab9d6c74b4db75d266c00348be1293245`

## 7. 真实写包，而不止是 Python 参数

真实 Windows Pack 复现：命令行显式指定与项目字段相同的源清单，两次 Cook 写出相同规范化字节，但 Pack 用了原输入。反斜杠写法让 writer 返回 2、无包；用 / 时写出了第四个 debug-only 资产，随后同源检查正确拒绝。

配置：真实 RenderableEmpty 工程模板中的场景、材质、OBJ；相同工程的两份正常清单数据（Windows 和 `/` 的路径写法分别为两组）。在 `/` 组有一个可达的独立调试资产，带 `debug` 标签；profile 的实际 Cook 默认为 `shipping`。每一组中项目声明 / 显式命令是同一份**源文件**、同一工程、独立新 out，Cook 的输出清单字节各自相同。

| 正常输入写法 | 项目字段指定 | 同一文件从命令行指定 |
|---|---|---|
| Windows 路径 | Pack 成功，3 个实际资产；来源门通过 | Pack 2，无包 |
| 正斜杠、含调试 root | Pack 成功，3 个实际资产；来源门通过 | Pack 成功，4 个实际资产；来源门拒绝 |

成功的包不只据 exit/report 判断：读取真实 ZRPK v1 header、嵌入的 chunk/asset manifest，再按 offset/size 与工程源文件逐字节对照；`--determinism-check` 由真 writer 运行。使用既有匹配 profile 的 Validate 夹具而非执行完整 Validate；仅在实际 Pack 成功后在 PlatformBundle 边界非零停止，没有假造发布成功，也未运行宿主。最后的同源检查以实际生成的两份 stage report 输入原 `cook_assets_pack_manifest_handoff_diagnostics`，不是全量 Report。

[受测提交](https://github.com/He-Jiahui/ZirconEngine/commit/3d03471f416ee7956f27f726098bdf8a46c406a9)；Python 新增用例 0 过 / 1 败，停在最终实参断言；已有恢复、显式路径和报告门 39 过 / 0 败。Windows x64/MSVC 1.94.1，单包 `zircon_runtime`，`--no-default-features --features native-host --bin zircon_export_pack --profile dev`。

| 实物 | SHA-256 |
|---|---|
| 构建回执 / 真 writer | `a757e13ece17cf1f552efad6dd6248f6eac399f599ab0cf3491e191ccca10307` / `5fd1027b509bcbb5e620276be37e18424c77a9f41b41151e993954e832368ed1` |
| Python 回归 / 邻近用例回执 | `2f8fdfdfc9ac20b747e2a4225d536a88222673482584b18fa889a7f12464b232` / `26d5d9113d5044727c5f7c3a7f1ea0b7ba33c3a2cb000d9a14b8582d1b4ac8aa` |
| 真实 Cook/Pack 运行 / 四条结果与内容校验记录 | `479e9eaa4907108d816caa095d38e02cce28c425a58f51423ff940adea983370` / `15f28906be65c2a5e77b0f23cf549da264b4a57373276b2b5db15d45ea7e7c48` |
| 3 个不写包的预演 + 显式空 Cook 源拒绝 | `8a9b43ca4956ac18f82bc8e27fc9a2d9ba1c3e9799c999ab84af925061dc6787` |
