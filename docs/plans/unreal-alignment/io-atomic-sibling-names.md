# 文件 I/O：原子发布的临时文件名使合法的长目标名无法写入

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

**初始调查状态（创建 Draft 时）：仅源码审查；后续回归状态见文末。** ZE 引用基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，只读本地 HEAD `cb495bb1c952a24c86f4513b8099bc4942c3a87d` 的相关产品路径无差异。UE 为 [本机 Build.version:2–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>) 声明的 6.0.0、BranchName `UE5`、CL 0；源码提交未知。**未运行 UE/ZE 测试或产品，未调用 Cargo、未改引擎/索引/远端。**

## 1. 与 UE 不一致（两边可达路径）。

ZE 的 `native-host` [Foundation:5–16](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/mod.rs#L5-L16>) 用 [模块工厂 `module_descriptor`:25–46](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/module.rs#L25-L46>) 建默认 `DefaultConfigManager`；默认路径可由 [`ZIRCON_CONFIG_PATH`:14–20](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_path.rs#L14-L20>) 指定，公开的 [`new_with_file_path`:82–113](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager.rs#L82-L113>) 也接收绝对文件路径并安装真实 writer。
 [`set_value` / `flush`:201–217](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager.rs#L201-L217>) → [`run_worker`:346–378](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager/worker.rs#L346-L378>) 序列化 JSON → [`AtomicConfigFileWriter::write`:44–48](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager/writer.rs#L44-L48>) → [Core I/O 转发:5–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/resource/io/mod.rs#L5-L11>) → [`stage_atomic_write_with_fault`:16–40 / `create_staging_file`:119–127](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L16-L40>) → [`unique_sibling_path_with_sequence`:20–44](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/pathing.rs#L20-L44>)。
后者把整个 UTF-8 leaf 放进 `.<leaf>.zr-staging-<pid>-<序号>`。
UE 的 `Core` 路径是 [`FConfigCacheIni::Flush`:4815–4847](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L4815-L4847>) → [`SaveBranch`:1494–1521](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L1494-L1521>) → [`SaveConfigFileWrapper`:607–640](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L607-L640>)；需临时写入时只取目标 base 的前 32 个字符作为前缀，由 [`FPaths::CreateTempFilename`:1704–1713](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/Paths.cpp#L1704-L1713>) 加 GUID/扩展名，写入后将原目标路径单独传给 `IFileManager::Move`。
## 2. 有界实际影响。

在目标所在卷接受 255 个 ASCII 字符的单一文件名分量、且目录可访问的前提下，用 `"a" × 250 + ".json"` 作为配置文件名：旧配置可作为普通文件读取，但任何需要持久化的新值都无法完成同目录暂存；新文件不能创建，已有文件不会进入替换。 [`ConfigPersistenceWorker::flush`:206–247](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/config_manager/worker.rs#L206-L247>) 等到该代失败后返回持久化错误（或先达到调用者超时），不等于内存写入失败，也不代表原数据损坏。只读主机证据：本机 D:、E: 均报告 NTFS；在 D: 同一已有目录用 `\\?\` 绝对路径、仅调用 `[IO.File]::GetAttributes`，255 ASCII 单元返回 `0x80070002`（未找到文件），256 及按 ZE 公式拼接的 275 单元返回 `0x8007007B`（名称不合法）。这是卷/路径解析证据，不是引擎写入复现；未创建文件。较短的常见 `config.json` 不触发，其他卷、UNC 及真实产品端到端未验证。
## 3. 根因与 UE 方法。

ZE [`path_entry`:35–46](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/mod.rs#L35-L46>) 先探测派生名，随后 [`create_staging_file`:119–127](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L119-L127>) 用 `create_new` 打开；无论失败落在探测还是打开，均到不了写入。相同拼名也用于[替换前的 backup:76–81](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L76-L81>)；仅缩短 staging 不足以恢复覆盖写。这里不是 Windows 全路径 `MAX_PATH` 问题，后续 [`windows_api_path`:122–172](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/platform.rs#L122-L172>) 不能提高磁盘单分量上限。UE [`WriteTempFileThenMove`:2812–2820](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp#L2812-L2820>) 仅在 desktop editor 的非 game、非 unattended 情形启用，否则直接写目标；临时名在 `ProjectSavedDir`，不保证与目标同目录。UE [`FFileManagerGeneric::Move`:299–323](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/FileManagerGeneric.cpp#L299-L323>) 可先删除已有目标；Windows 物理后端 [`FNormalizedFilename`:1135–1158](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformFile.cpp#L1135-L1158>)、[`MoveFile`:1216–1219](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformFile.cpp#L1216-L1219>) 规范化最终目标后调用 `MoveFileW`。这里仅比对**临时名长度与原目标解耦**，不推断 UE 各分支都原子或具有 ZE 的故障恢复语义。
## 4. Rust 沿用 UE 思路，不换架构。

只借鉴“临时名有独立、有界预算”。保留原始目标路径、同目录暂存、排他创建、Windows `ReplaceFileW` / Unix 原有提交、sync 屏障、提交 fence 和既有 recover 缺失目标行为；不用 UE 的先删后移，不改 JSON 存储结构，不加配置服务或 I/O 框架。
## 5. ABI / ECS 差异与修改所有权。

UE 是 `FConfigCacheIni`/`IFileManager` 原生 C++；ZE 是 `Path`/`io::Result` 和 Rust worker/fence，错误沿已存在路径传播。无 C ABI、ECS、数据格式或依赖版本更改需求。`zr_resource` 是 ZE 自有同仓库 [workspace path 包 `Cargo.toml`:2–9、61–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.toml#L61-L65>)，[`Cargo.lock`:10291–10302](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L10291-L10302>) 没有 `source` 或 Git revision；基线 `git ls-tree` 的 `pathing.rs` 是 `100644 blob bbeeeab183ca9fba8a98572ef54a8ef9885eaa93`，不是子模块或外部仓库。后续可在同一独立 ZE PR 做内部窄修。
## 6. ZE 改法草案（未实现）。

在 `zr_resource` 的 `io/atomic_file` owner 给 staging/backup 有界 ASCII 名前缀；backup 需要保持与目标关联，长名不要只取共同前缀，可用稳定、统一编码的 leaf 摘要配合既有 role/PID/序号（按本机 Windows 文件名比较语义区分同目录目标，避免有损字符串），同时更新 Windows [`recover_missing_target_from_backup_platform`:32–94](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L32-L94>) 及保留识别约束。确保两个同前缀的目标不会串恢复，兼容现存旧版有界名的唯一备份；不从单纯文件名匹配推断可删除别人的文件。顺序：独立问题 Draft → 在受控真实卷用普通 I/O 对照先证实 255 目标名可创建，加入同路径 `atomic_write` 新建/替换、字节与残留、Windows backup 恢复以及注入替换失败时旧内容不变的回归 → 再修并复跑。可增加实际 `DefaultConfigManager::new_with_file_path` 的 `set_value` + `flush` 接线证据；不得用截断用户目标名、放宽测试或悄悄退到非原子写通过。

## 测试迁移/比对

| UE 现有测试 / 检索 | 输入 → 期望 | ZE 已有落点 / 差距 |
|---|---|---|
| [`FFileAppendTest`（`System::Core::Misc::FileAppend`）:95–125,158–175](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/FileTest.cpp#L95-L175>) | `CreateTempFilename` 默认承接普通短 temp；先创建 64 个零字节，再不 append 写 1 个字节；读回分别等于预期内容。 | [`atomic_write_replaces_existing_file_and_cleans_transaction_files`:138–149](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs#L138-L149>) 验证新字节及只留目标；[`windows_replace_failure_after_backup_restores_the_canonical_target`:215–231](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs#L215-L231>) 与 [`atomic_replace_failure_keeps_old_json_and_same_value_retry_commits_new_json`:599–631](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs#L599-L631>) 是独立的 ZE 故障对照。不是长 leaf 对等测试。 |
| **未定位到相同边界的 UE 原测试，源码推导，非迁移。** 检索 UE `Runtime/Core/Tests` 全目录、`Runtime/Core/Private/GenericPlatform/*Test*.cpp`、`Editor/EditorConfig/Private/Tests` 中 `CreateTempFilename`、长文件名/路径、保存和覆盖；对 `Engine/Source/**/**Test*.cpp` 检索 `WriteTempFileThenMove` / `SaveConfigFileWrapper`。 | `255` ASCII leaf 在目标卷可用，临时 leaf 应不依赖原长度增长；新建/覆盖都在目标原名完成。 | 已看同 owner 的 `atomic_file/tests`、`io/platform/tests/windows_path_encoding.rs`、Foundation `config_manager_tests`；已有整条 Windows 路径编码、普通替换及恢复，无 255-leaf 配置回归。`#130` 的[配置优先级回归:60–117,120 起](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/tests/unreal_equiv_r01.rs#L60-L120>) 不测名称。 |

查重：本地 PR 快照 `#135–145`（含 #136 流程退役）无同契约；合入基线的 [#130](https://github.com/He-Jiahui/ZirconEngine/pull/130) 涉配置 precedence/module availability/tasks，而非文件名；R01/R02 已有 review/migration 证据也未给出此路径边界。仅将 `zr_resource` 的通用 I/O 用 Foundation 见证，不接管 assets/project 主线。

## UE 只读源指纹（SHA-256）

| 源文件 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [Core.Build.cs:8–20](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs#L8-L20>) | `2bae34201d978e95d778674f5293b80683c8844ba8963184cf306c20ab06e558` |
| [ConfigCacheIni.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp>) | `4960815ec3d489ebfdd74643c6c24b0e76b41bc6264a3499db8fddc7c2105ae0` |
| [Paths.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/Paths.cpp>) | `57f7bb3a5894a9d5b12077986f48c15bd106020455b3671bb83061671eed9e13` |
| [FileManagerGeneric.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/FileManagerGeneric.cpp>) | `d40fd61f1a2121dfc671d7a2de89b0f0feb2ac2ba4c1733ef443776de5ae0caf` |
| [WindowsPlatformFile.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformFile.cpp>) | `b961e22f4713df6692246ca5fca3bb6d8c7d8eee51f6f15af911931a20f10165` |
| [FileTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/FileTest.cpp>) | `32423bef2cee04e38b032938490190bf3597ce0b281b0216e60850a1a62a418e` |


## 后续回归阶段：2026-11-17（测试前置尚未通过）

已在原 `zr_resource` lib harness 新增 6 个源码派生用例，未新建 runner、未改生产实现。[测试子模块](<https://github.com/He-Jiahui/ZirconEngine/blob/audit/ue-io-temp-name-20261117/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/long_names.rs>)、[原 harness 接线](<https://github.com/He-Jiahui/ZirconEngine/blob/audit/ue-io-temp-name-20261117/zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs>)只增加对应覆盖：

| 用例 | 期望/控制，尚未运行 |
|---|---|
| 排他新建 | 普通写读先证明卷能接受255-byte最终leaf，再删除本例已知目标，走 `atomic_write_new/commit_new`，精确原名发布且无旁产物 |
| 普通缺失目标 | 同样的合法性preflight，走 `atomic_write_with_fault(None)/commit` 的 Missing 分支，不能把前一条当此分支验收 |
| 已有目标替换 | 完整255-ASCII leaf保留，新字节发布、旧staging/backup清理；必须真实经过Windows平台原语 |
| 非ASCII | 5个crab emoji +230 ASCII +.json，255 UTF8 byte /245 UTF16单元，不能截断或改写原目标名 |
| Write fault | 必须实际达到既有 injected write错误，不接受提前命名错误；原字节与只剩目标的控制 |
| Windows ReplaceAfterBackup fault | 必须达到既有恢复文案、原字节和无残留，不把提前命名失败冒当正确恢复 |

每例沿用现有 `test_root`；FS preflight自身失败属于环境/fixture前置，不是原子写行为RED。测试新增/格式检查与源码反证不是已执行的引擎用例。

### 实际命令与失败分类

Windows x64/MSVC、Rust/Cargo 1.94.1；唯一选择包 `zr_resource`、关闭default features、空features、library tests，过滤 `io::atomic_file::`，test profile、jobs=1、test-threads=1。调用规范树的 local Cargo wrapper，`-RepoRoot`明确指向本问题登记工作树；target/build/Cargo home/cache/temporary均被物理路径校验限制在 `D:/cargo-targets/zircon-local/a/win32/6d2edd3f58711542/`。未选择workspace/all-features、未编译runtime/editor/plugin目标、未更新lockfile。

| 实际尝试 | 退出及原因 | 可宣称范围 |
|---|---|---|
| red-1 / offline | 101：新cache缺 `image` 索引，Cargo解析全root workspace metadata时停止 | 尚未编译/执行lib tests，不是RED |
| red-2 / online | 101：crates.io多次low-speed timeout，最终缺 `pulp-wasm-simd-flag` | 尚未编译/执行lib tests，不是RED |
| red-3 / offline | 101：只读Cargo info补齐上述单一元数据后，仍缺 `reborrow` 索引 | 尚未编译/执行lib tests，不是RED |

局部命令回执与完整stderr仅本机保存，未上传完整cache或凭证；可从 `D:/cargo-targets/zircon-local/agent-cargo-receipts/ze-ue-159-red-{1,2,3}/receipt.json` 对照确切argv、时间、日志指纹及实际根进程退出。该普通回执明确不是source closure/immutable seal、process-tree、compiler-artifact或产品验收证明。metadata探针成功也不是编译/测试PASS。

```powershell
$cargoArgs = @('test','-p','zr_resource','--no-default-features','--lib',
  'io::atomic_file::','--locked','--offline','--jobs','1','--','--test-threads','1')
# 先准备x64 MSVC与1.94.1 PATH；规范local-cargo -RepoRoot明确指定此问题工作树，
# BuildRoot为盘根D:/cargo-targets；每次新的ReceiptDirectory，保留已结束历史。
# online尝试仅去掉--offline，其他选择相同。所有真实回归仍待依赖前置满足后运行。
```

### 实施可行门（尚未批准生产修改）

UE的短前缀思想只解决预算，不能直接把完整leaf替换成裸hash后宣布完成：

- [Windows缺失目标恢复](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/recovery.rs#L31-L60>) 按原目标prefix匹配backup。若hash token仍复用旧grammar，短用户leaf可恰等于token导致串匹配；需要可判别namespace及scheme-aware源绑定，不扩大恢复授权。
- [durable discovery](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/transaction/recovery/discovery.rs#L91-L103>) 不只识别临时名，还以marker前 `.zrjournal` 后缀分类待清理orphan。不能给普通artifact伪造此后缀或让真正journal变unsupported，也不能把名称形状当任意文件的删除授权。合法32-byte tag的最坏journal leaf216再加staging44可达260；“所有journal永远不超限”不成立，必须有边界控制。
- [既有ReplaceFileW临时名观察](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/replace_file_meta_temp.rs#L3-L16>) 表明平台可能有自身的 `~RF...TMP`。自有名变短后，255 leaf替换仍须真实运行；不预先改平台原语、取消原子性或照搬UE异目录/先删后移。
- 保持同目录、完整实际target、排他创建、序号终止耗尽及既有短名/失败清理合同；先真实RED与可行评审，再最小修复、同输入GREEN，产品保存链仍单独验收。
