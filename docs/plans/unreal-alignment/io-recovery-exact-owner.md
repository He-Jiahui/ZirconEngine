# Windows 配置恢复可能误用另一目标的暂存文件或备份

**状态：独立 Draft 候选，仅源码取证。** ZE 引用固定在 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，与本轮审阅的相关规范树代码等价；UE 参考为本机 [Build.version:2–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>) 声明的 6.0.0、BranchName `UE5`、CL 0，源码提交未知。UE 链接是本机取证锚点，非公开可读源码。下文的两个场景是源码推导，不是已运行的测试或产品复现。本问题只涉及现有短文件名的旧版恢复候选判定，**与 #159 的长文件名暂存命名边界独立**；不需先合入或修改 #159。未发布、未执行 Cargo/Git/远端操作、未修改引擎或测试。

## 1. 两边的实际入口

**ZE：Foundation 先恢复文件，再解析 JSON。** 正常 [Foundation 工厂:25–46](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/module.rs#L25-L46>) 创建 `DefaultConfigManager`；显式绝对路径同样可经 [new_with_file_path:82–113](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager.rs#L82-L113>) 进入。生产 [new_with_options:131–156](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager.rs#L131-L156>) 在路径 gate 内捕获实际 I/O 路径，调用 [recover_and_read_from_disk:224–236](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager.rs#L224-L236>)。后者通过 [Core Resource I/O 转发:5–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/resource/io/mod.rs#L5-L11>) 先调用 `recover_missing_target_from_backup`，再打开目标、解析 JSON；构造成功后才合并读到的值。这里不是测试专用恢复入口。Windows 正常配置路径也可使用短叶名 [config.json:14–25](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_path.rs#L14-L25>)。

**UE：比较的是如何选择已知文件，不是相同的自动恢复流程。** [LoadConfigFileWrapper:567–601](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L567-L601>) 未被加载委托处理时，文件读取分支使用原 `IniFile`。实际配置备份比较从 [ConfigContext:769–781](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigContext.cpp#L769-L781>) 进入 `FConfigFile::ProcessSourceAndCheckAgainstBackup`；写端 [SaveSourceToBackupFile:3432–3459](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L3432-L3459>) 和读端 [ProcessSourceAndCheckAgainstBackup:3463–3501](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L3463-L3501>) 均用专用 `Config/CoalescedSourceConfigs` 目录与配置 `Name` 组合出同一明确 `.ini` 路径。它用于比较默认配置层变化并传播差异，不是在缺失目标时枚举临时兄弟文件并恢复，不等于物理文件 ID 或跨所有配置路径的身份模型。

## 2. 有界影响与两个独立见证

共同前置：Windows；一个真实、可写、独立的父目录；目标 `config.json` 缺失；每个场景在**不同目录**各只有一个能被旧判定接受的普通文件，其他 I/O 正常。`42-1` 表示合法的 PID/序号样例，仅用于说明文件名形状，不要求模拟真实进程 ID。

| 场景 | 唯一候选（属于另一个短目标 `config.json.zr-backup-other`） | 真实生产来源（源码推导） |
|---|---|---|
| A：角色错误 | `.config.json.zr-backup-other.zr-staging-42-1` | [单文件暂存:16–40](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L16-L40>) 先[创建 staging:119–127](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L119-L127>)，写好后交给待提交对象；对象仍持有或异常退出时，可以出现未发布 staging。 |
| B：owner 错误 | `.config.json.zr-backup-other.zr-backup-42-1` | 对另一已有目标替换，先生成 [backup:76–81](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L76-L81>)；[Windows 替换:256–311](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L256-L311>) 在发布后同步失败时可保留它。 |

若重命名及同步成功，旧 [单候选恢复分支:76–94](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L76-L94>) 会把任一候选移到 `config.json`，消耗原候选路径。若移动的是另一源的合法 JSON 对象，Foundation 可将其值当成当前配置；若内容不能解析，构造会报错，但移动早于解析。不是任意同目录文件都会被移动：[已有正常目标或非文件目标:15–29](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L15-L29>) 不走该分支；多个被采纳的候选报歧义而不是任选一个；[非 Windows 缺目标恢复:96–99](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L96-L99>) 不做该目录扫描。未测实际触发频率。

## 3. 根因与 UE 做法的可用部分

ZE 的[配置恢复合同](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_runtime/foundation/config-manager.md#L39-L44>) 明确要求唯一的同属主备份；下面的前缀家族并非有意定义的恢复所有权。

ZE [旧生成规则:20–44](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/pathing.rs#L20-L44>) 将完整可表示的用户叶名、角色、PID、序号串接。Windows [恢复过滤:50–74](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L50-L74>) 先查前缀 `.config.json.zr-backup-`，再独立调用 [形状识别器:47–68](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/pathing.rs#L47-L68>)，后者接受 staging 或 backup 的合法数字尾部。两个判定分别为真，不代表**属于所请求 leaf 的完整 backup frame**：A 用另一个角色通过末尾检查，B 用较长的另一个 leaf 沾上了前缀。普通文件检查无法补足此身份判断。

借鉴 UE 的只是**以已知目录和精确名称选择文件**：[SaveConfigFileWrapper:619–635](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L619-L635>) 也显式持有本次临时路径与原目标，未由前缀重新猜测目标。这里不移植 UE 的 INI 合并、保存位置或移动语义，也不把 UE 的确定路径读写当作一种已存在的等价崩溃恢复保证。

**UE 本机取证指纹（SHA-256）：**

| 参考文件 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [ConfigCacheIni.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp>) | `4960815ec3d489ebfdd74643c6c24b0e76b41bc6264a3499db8fddc7c2105ae0` |
| [ConfigContext.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigContext.cpp>) | `74cfc53410cd527313c7c996d43695b31a47f705d8ea886c8a6b6e377c0c1ee6` |
| [FileTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/FileTest.cpp>) | `32423bef2cee04e38b032938490190bf3597ce0b281b0216e60850a1a62a418e` |

## 4. Rust 最小改法（未实施）

只改 `zr_resource` 的 Windows 旧名 backup 归属判定。对现有精确 `.<leaf>.zr-backup-` 前缀，检查去掉它之后的**整个**余串必须恰为原数字 `PID-sequence` 形状：非空 ASCII 十进制 PID，恰一个分隔符，非空且可解析为 `NonZeroU64` 的十进制序号，不允许额外 marker、字符或后缀。首选复用现有尾部校验，不另行收紧旧 PID 的数值范围/规范化，也不要求 PID 是当前进程。这样既不接受 A 的 staging，也不接受 B 的较长 leaf。通用形状识别仍可供资产过滤，不把它当作 Windows 恢复归属凭证。

保留原 target `Path`、同目录扫描、候选的普通文件检查、零候选为 false、唯一才移动、多个或非普通候选拒绝，以及 [现有 Windows 发布与文件同步:39–75](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/platform.rs#L39-L75>)。不更改命名预算、平台原语、数据格式、配置 gate、大小写别名或历史非 Unicode fallback，不声称文件名能认证有意伪造的同形文件。若 #159 的新格式先落地，本问题仍只改旧名分支，不捆绑新格式工作。

## 5. ABI / ECS 差异与修改所有权

UE 是 `FConfigContext`/`FConfigFile` 的 C++ 配置层比较；ZE 是 Foundation 调用 `zr_resource` 的 Rust 路径与 `io::Result` 恢复。根因在 I/O owner 解析候选，不在实体、组件、注册顺序或配置序列化。`zr_resource` 为同仓库 [workspace path 包:61–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.toml#L61-L65>)；仅修内部判定，无须改变 ECS、C ABI、对外 API、JSON/WAL 线格式、依赖或 Foundation 的生产接线。

## 6. 已有测试、后续 RED 和验收范围

- **ZE 既有控制：** [Windows recovery lookalike / 非普通文件:103–135](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs#L103-L135>) 覆盖非数字后缀和目录项，但不覆盖“前缀吻合且整个候选恰好是另一源合法产物”。[Foundation 唯一与歧义启动恢复:862–920](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs#L862-L920>) 覆盖真备份的正、负对照，不替代两条错源回归。
- **UE 邻近测试，不是同例原测：** 本地 Core/Tests、Core/Private 的 `*Test*.cpp` 与 EditorConfig/Private/Tests 检索 wrapper、config backup/recovery、`SaveSourceToBackupFile`/`ProcessSourceAndCheckAgainstBackup`，未定位等价的错源自动恢复用例。[FileTest — FFileCompetingCaseDirectoriesTest:323–349](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/FileTest.cpp#L323-L349>) 以不同内容区分实际读到的文件，适合作为精确字节断言的参考；它测试大小写路径解析，不是备份扫描。[FFileAppendTest:95–175](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/FileTest.cpp#L95-L175>) 验证普通写入与覆盖，也不是该恢复原测。
- **先独立复核、去重、发布 Draft，再取得 RED。** 在现有 atomic-file lib harness 新增仅 Windows 运行的恢复用例，利用其 [test_root:333–347](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs#L333-L347>) 下分别布置 §2 A/B，先确认目标真实缺失且候选可读、字节准确。对真实恢复入口分别要求 `Ok(false)`、原目标仍缺失、错源文件精确路径与字节不变。两个错例必须分开，否则原代码会先报多候选歧义，掩盖单候选错恢复。正对照用一份精确合法备份，验证恢复到原名并只消耗该文件；保留多真候选歧义、非普通候选及已有目标的不变控制。
- **验证门：** 授权后先运行原生产实现上的 `zr_resource` lib 定向 Windows 测试（默认 features 禁用、无额外 features、合规盘根 `cargo-targets`，仅相应测试过滤），确认两条失败来自错误移动／归属，不是 fixture 环境或编译。之后才窄修并用同一回归及既有控制验证；可另选 Foundation startup 作产品接线证据，但不是首个 RED 的前置。不引入新 harness，不把未运行的源码推导宣称为实际损坏或修复通过。
