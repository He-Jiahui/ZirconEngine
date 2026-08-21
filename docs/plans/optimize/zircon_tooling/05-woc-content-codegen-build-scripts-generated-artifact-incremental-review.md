---
related_code:
  - .gitignore
  - examples/woc/zircon-project.toml
  - examples/woc/tools/package.json
  - examples/woc/tools/package-lock.json
  - examples/woc/tools/reference_inventory.mjs
  - examples/woc/tools/command_payload_source_extract.mjs
  - examples/woc/tools/command_payload_codegen.mjs
  - examples/woc/tools/m4_ability_zr_codegen.mjs
  - examples/woc/tools/m5_content_zr_codegen.mjs
  - examples/woc/contracts
  - examples/woc/reference/current-head
  - examples/woc/scripts/woc_game
  - examples/woc/native/crates/woc_contract_codegen
  - zircon_app/build.rs
  - zircon_editor/build.rs
  - zircon_hub/build.rs
  - tools/session_tray/build.rs
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/physics/runtime/build.rs
  - zircon_runtime/build.rs
tests:
  - examples/woc/tools/command_payload_pet_contract_test.mjs
  - examples/woc/native/crates/woc_contract_codegen/tests/contract_generation.rs
  - examples/woc/native/crates/woc_contract_codegen/tests/reference_inventory.rs
  - zircon_runtime/tests/frameworks_03_profile_feature_presets.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/04-reflection-derive-script-host-macros-schema-codegen-review.md
  - docs/engine-architecture/generated-code-boundary.md
reference_engines:
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_asset/src/processor
  - dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.UHT
  - dev/godot/methods.py
  - dev/godot/core/core_builders.py
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderGenerator
  - dev/Fyrox/fyrox-resource/src/graph.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 05 · WOC 内容 Codegen、Cargo Build Script、生成产物与增量编译工程化差距

## 1. 结论

WOC 工具链不是几个临时示例脚本。受版本控制的 `examples/woc` 已有 1,967 个文件、73,610,362 bytes，其中 `tools` 有 386 个直接 `.mjs`、127 个 codegen、44 个 source extractor；ZrVM 包有 707 个 `.zr`，本轮识别出 106 个带生成标记的 `.zr`，合计 84,087 行、3,367,241 bytes。`reference_inventory` 会固定上游 commit、保存 SHA-256，`woc_contract_codegen` 也有 typed manifest、稳定 numeric ID、reserved ID、长度/finite-number 校验和 Rust/ZrVM fingerprint projection。这些是可保留的正确方向。

但它仍不是工程级内容编译系统。仓库根 `.gitignore` 用 `examples/*` 忽略整个 WOC，历史文件只是已追踪例外；已提交的 `package.json` 当前引用两个只存在于本机、被该规则忽略的脚本，clean clone 的对应命令必然缺文件。386 个受控脚本只有 188 个被任一 package script 引用，198 个完全没有入口；默认 `check` 只运行 28 个脚本，没有 CI consumer，并且本轮真实执行在第 7 步因硬编码的 148 与当前 157 个 typed contract 不一致而失败，余下 21 步被 `&&` 短路。

生成发布也没有事务边界。130 个受控脚本直接调用 `writeFileSync`，0 个使用 `renameSync`；64 个脚本有至少两个常见 emit helper call site，`reference_inventory` 顺序发布 8 个目录文档，`command_payload_codegen` 和 native `woc_contract_codegen` 都顺序发布 Rust/ZrVM 两份投影。进程崩溃、磁盘写满、并发 generator 或第二个输出失败均可留下 mixed generation，而产物没有统一 receipt、generation ID、stale-output cleanup 或恢复日志。

运行时数据形状进一步暴露了引擎缺口。106 个生成 ZrVM 文件含 56,242 个 `if`，多数是数据查找级联，不能简单等同于 56,242 条手写业务规则；但 `rankAtLevel`、effect/field selection 等选择语义也被生成进源代码，且插件明确使用 `execution_mode = "interp"`。这说明 ZrVM 尚缺紧凑只读数据表、稳定 data-handle、索引/枚举访问与编译期常量池，内容被迫膨胀成解释执行的源代码。

7 个真实 crate-root Cargo build script 中，`zircon_runtime/build.rs` 的 typed/strict profile preset 是当前最佳基线，但其 746 行实现仍复制 builtin/profile authority并通过解析 `OUT_DIR/.../build/...` 猜自定义 profile。`zircon_editor/build.rs` 则自行用 raw TOML 建立另一份 plugin catalog，只读取第一项 editor module并静默丢弃非字符串 capability。`zircon_app/build.rs` 更直接使用 build-host 的 `cfg!(windows, target_env="msvc")` 判断 target，在交叉编译时会发错或漏发 `/STACK` linker arg。

本轮记录 6 个 P0、48 个 P1 和 10 个 P2。没有修改 WOC、Rust、Cargo、生成物或 CI；只新增审查记录并更新索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 规模 | 本轮状态 |
|---|---:|---|
| WOC 全 tracked set | 1,967 文件 / 73,610,362 bytes | E1 inventory；tools、contract、reference、生成物和native codegen纵向E2-E3 |
| WOC tools | 390 tracked文件，其中386个直接`.mjs` | E2全量分类；127 codegen / 44 source extract / 18 contract test / 109 static guard / 78 state-or-source check，类别可重叠 |
| codegen脚本 | 127文件 / 21,799行 / 1,154,916 bytes | E2；最大脚本`command_payload_codegen.mjs`为2,039行 |
| generated ZrVM | 106文件 / 84,087行 / 3,367,241 bytes | E2-E3；读取最大产物、生成器、产品import与执行配置 |
| native contract codegen | 8个Rust source/test文件，加manifest | E3：manifest、validation、fingerprint、projection、CLI与测试逐文件读取 |
| Cargo crate-root build scripts | 7文件 / 1,019 physical lines | E3：输入、target环境、验证、emit、rerun和consumer边界逐文件读取 |

WOC tracked set 的 `git status --short -- examples/woc` 为 0 项；基于 Git index record 的内容指纹为 `2f18a4ce8c1bca40f48d0c8b57f1f97f33bcc2e389202bb6a0d9368717fd3600`。7 个 build script 同样为 clean，指纹为 `d0ffc5b2c5cf86e45988dd5f9ba908709bf36c7b10107aa110505eab437b827a`。实施前必须重取指纹。

本机 `examples/woc/tools` 另有 103 个被 `.gitignore:128` 隐藏的 `.mjs`，它们不计入上述产品能力。只发现其中两个 basename 被 tracked WOC 文件引用，引用方均为 `package.json`：`weapon_skin_contract_codegen.mjs` 与 `wos163_weapon_skin_runtime_static_guard.mjs`。这两个文件在本机存在不能修复 clean-clone 缺失。

### 2.2 动态验证

在 `examples/woc/tools` 执行：

```powershell
npm run check
```

Node 为 22.13.1，npm 为 11.1.0。命令耗时 80.945 秒、exit 1。前 6 项成功检查了 7 个 reference catalog、158 个 client command site、157 个 typed payload contract、1,070 个 trace symbol和165个command；第 7 项 `command_payload_pet_contract_test.mjs:67` 随后失败：`coverage.totals.typed_contract_commands` 实际 157，测试仍要求148。默认28步中的后21步未运行。该结果只能证明当前默认门禁为红，不能证明未执行的生成物是否漂移。

`.github` 下未找到 `examples/woc`、`woc_contract_codegen` 或 WOC npm package 的引用。没有运行全部 304 个 package scripts，也没有把本机103个ignored脚本当作验证输入。本轮未运行native Cargo workspace；其源码测试只作为静态证据，不声明当前动态通过。

文档验证中，4个本轮文件的`git diff --check`通过，仅报告仓库既有LF/CRLF提示。plan-output audit仍有2个旧child-record违规，docs convention path gate仍有670个旧违规；两者对本报告的命中均为0。本轮没有把历史基线写成新报告失败，也没有据此宣称全仓文档门通过。

### 2.3 关键静态量化

| 信号 | 结果 | 含义 |
|---|---:|---|
| package script entries | 304 | 扁平命令面已超出人工维护可靠范围 |
| 任一package script可达的tracked `.mjs` | 188 / 386 | 198个脚本没有入口 |
| 默认`generate` | 11个脚本 | 不能生成全部权威产物 |
| 默认`check` | 28个脚本 | 不能证明全图无漂移 |
| `writeFileSync` | 130文件 / 130 call | 每个脚本复制直接writer helper |
| `renameSync` | 0 | 没有原子替换惯例 |
| 两个以上常见emit call site | 64文件 / 130 call sites | 多产物发布是常态而非例外 |
| `execFileSync` | 291文件 / 336 call | 重复Git/子进程访问成为默认数据层 |
| `spawnSync` | 47文件 / 48 call | 无统一进程池、取消或budget |
| TypeScript `createSourceFile` | 20文件 / 22 call | source extraction大多不是typed AST |
| `process.argv` | 136文件 / 146 hit | CLI schema和诊断分散 |
| `createHash` | 135文件 / 161 call | 局部fingerprint很多，但没有统一依赖图/cache key |
| generated ZrVM `if` | 56,242 hits / 50 files | 数据被编译为巨大解释器分支 |

### 2.4 参考边界

- Bevy `AssetProcessor`保存asset source hash与每个process dependency的`full_hash`，只在输入实际变化时重处理；`ProcessorTransactionLog`用write-ahead log记录begin/end，启动时发现未完成事务会重处理相关资产。它直接证明内容图、依赖hash与crash recovery必须是一套系统，而不是每个generator各有`--check`。
- Unreal UHT把parse/type/export集中在独立程序，通过`IUhtExportFactory.CommitOutput`提交生成文件，并计算header body hash与package combined body hash。这里借鉴typed graph、集中提交和definition identity，不把C++ header model照搬到WOC。
- Godot由SCons target显式调用集中 `generated_wrapper`，统一generated marker、newline、header guard与输出buffer。该wrapper仍直接覆盖目标文件，所以只作为“共享生成基础设施”参考，不作为原子发布完成态。
- Unity Graphics `CSharpToHLSL`从typed attributes收集并排序generator，任一type失败会阻止该文件成功生成，产物带统一不可手改标记。它同样直接写文件，故只用于typed IR和whole-file failure语义参考。
- Fyrox有显式resource dependency graph，但本地源码没有给出与Bevy AssetProcessor等价的生成事务/增量content compiler；本报告不把它未提供的能力推断为已存在。

## 3. 当前 P0

### TOOL-CODEGEN-P0-001 · WOC 不能从 clean clone 完整重现

根 `.gitignore:128` 的 `examples/*` 会隐藏WOC后续新增文件。当前tracked `package.json:281-282/310` 已引用 `weapon_skin_contract_codegen.mjs` 和 `wos163_weapon_skin_runtime_static_guard.mjs`，但两者都不在Git index，只因本机ignored文件存在而看似可用。clean clone运行相应named script必然得到module-not-found，新文件也可长期不出现在普通`git status`。

立即修复必须先为WOC建立显式negation/局部ignore规则，盘点并分类103个ignored脚本，只提交产品需要的两项及其真实依赖；然后在临时clone/`git archive`中执行入口发现与门禁。禁止用`git add -f`继续维持不可见工程。

### TOOL-CODEGEN-P0-002 · 唯一默认门禁当前失败且短路大部分检查

`npm run check`在第7/28步因157与硬编码148不一致失败，后21步没有执行。前6项成功日志仍会出现在输出中，若只看尾部或统计成功消息可能误判为大部分合同有效。当前不能用该命令支持“WOC generated contracts are current”的任何完成声明。

先修复权威计数来源和测试期望，改为调度器执行全部独立节点并聚合失败；只有全图完成后才能发布一个总exit code。每个节点必须报告started/completed/cache-hit/duration/output hash。

### TOOL-CODEGEN-P0-003 · 没有可证明完备的 producer/check 图或 CI consumer

386个tracked `.mjs` 只有188个进入任一package script，198个无入口；其中包括9个codegen、43个source extractor和1个contract test。默认generate只跑11个，默认check只跑28个；`.github`完全没有WOC lane。当前不存在“所有checked-in产物都被其唯一producer重新生成并检查”的证明。

需要一份machine-readable `ContentBuildGraph`：每个node声明owner、tool kind、inputs、outputs、dependencies、platform、check/generate action与retirement状态。CI从图生成全量/changed-set lane，未分类脚本、无producer产物、重复output owner和悬空node都必须阻断。

### TOOL-CODEGEN-P0-004 · 多产物发布没有事务，允许 mixed generation 进入产品

130个脚本直接写目标，0个采用rename；64个生成器有多个emit call site。`reference_inventory`顺序写8份catalog，JS `command_payload_codegen`与Rust `woc_contract_codegen`都先后写两种语言投影。任一中断可使Rust、ZrVM、JSON和manifest来自不同generation，而runtime会直接import这些文件。

必须引入staging directory、每个输出的content hash、完整output manifest、fsync/close、全部验证后的一次commit与write-ahead receipt。平台不支持目录原子替换时，使用generation directory加单个current-pointer/manifest切换；启动与CI要恢复或拒绝incomplete transaction。

### TOOL-CODEGEN-P0-005 · 上游 source parity 没有单一 typed IR authority

291个脚本自行执行Git/子进程，只有20个使用TypeScript AST；大量extractor通过`match/indexOf/slice`重复解释源码。2,039行 `command_payload_codegen` 又手工维护command/payload分类并同时生成两种语言。相同上游事实存在于source extractor、JSON、JS常量、Rust fixed counts、ZrVM生成器和测试期望中，当前157/148漂移正是该authority分裂的动态证据。

需要一个固定commit只读source service和typed `WocSourceIr`。TypeScript compiler一次解析project graph，Git object reader一次materialize/pin source，所有catalog/contract/test projection只消费validated IR。generator不得再重新用字符串切片解释同一语义。

### TOOL-CODEGEN-P0-006 · `zircon_app/build.rs` 用 host cfg 决定 target linker 行为

`cfg!(all(windows, target_env = "msvc"))`描述的是build script自身的编译host，不是Cargo target。Windows host交叉编译Linux会错误发出MSVC `/STACK`，Linux host交叉编译Windows MSVC则漏发。产品host的link contract因此不具备cross-target correctness。

必须读取 `CARGO_CFG_TARGET_OS` / `CARGO_CFG_TARGET_ENV`，对支持矩阵做显式match，并用至少 Windows-host→Linux-target 与 Linux-host→Windows-target 的build-plan测试锁定emit结果。link argument owner还应进入产品target descriptor，而不是散落单crate常量。

## 4. 内容图、入口与可观测性差距

### TOOL-CODEGEN-P1-001 · 304个script仍是扁平手写命令表

默认check本身是1,351字符的`&&`链，named generate/check又逐项复制。依赖、并行性、互斥output、retry与changed-set都不能由字符串可靠推导。应由typed graph生成CLI aliases，`package.json`只保留稳定入口。

### TOOL-CODEGEN-P1-002 · 默认generate不是全量生成语义

`generate`只运行11个核心脚本，却使用无qualifier的产品级名称。应拆成`generate:core`与真正的`generate:all`，release/CI只能接受图计算出的权威全量或有证明的changed closure。

### TOOL-CODEGEN-P1-003 · 198个不可达脚本没有library/archive/retired分类

不可达不必全部执行，但当前无法区分helper、人工工具、过期milestone和漏接producer。每个文件必须属于graph node、shared library、fixture、migration archive或retired tombstone；unclassified count固定为0。

### TOOL-CODEGEN-P1-004 · 没有唯一output owner与冲突检测

脚本在代码内部拼接目标路径，调度面不知道两个node是否会覆盖同一JSON/Zr/Rust文件。graph装载时必须canonicalize output并拒绝重复owner、父子目录重叠和source/output alias。

### TOOL-CODEGEN-P1-005 · 没有 stale-output 删除合同

generator减少分片或重命名输出时，旧文件仍可被package/import扫描。每次成功generation必须携完整output set，并只在ownership确认后回收上代多余产物。

### TOOL-CODEGEN-P1-006 · 80秒默认检查没有阶段可观测性

运行期间长期没有stdout，无法判断卡在Git读取、AST、hash还是测试。统一runner应发结构化progress event、node timing、cache key、input/output count和慢节点排名，而不是依赖各脚本偶尔打印摘要。

### TOOL-CODEGEN-P1-007 · `&&`只报告第一个失败

当前157/148错误隐藏了其后21项状态，修一次才能知道下一处。独立check应继续运行并聚合diagnostic；只有真实依赖失败的下游才标记blocked。

### TOOL-CODEGEN-P1-008 · CLI schema分散在136个脚本

`process.argv.includes`与自写option reader没有统一unknown-option、required value、path、mutual exclusion或help contract。应使用共享typed CLI前端，并让generate/check/dry-run/output/source revision成为统一字段。

### TOOL-CODEGEN-P1-009 · Node/toolchain身份没有进入产物

package-lock只固定极小依赖集，package没有`engines`，产物也不记录Node/TypeScript/generator version。graph receipt必须包含tool binary/hash、runtime版本和schema revision；CI拒绝未批准toolchain。

### TOOL-CODEGEN-P1-010 · 诊断没有稳定code与source span合同

多数失败只是throw/assert自由文本，无法由Editor/CI聚合或长期豁免。typed diagnostic至少包含code、severity、node、source identity/span、input hash、output和repair hint。

## 5. Source Extraction、Schema 与依赖图差距

### TOOL-CODEGEN-P1-011 · 291个脚本重复启动Git/子进程

相同commit/tree/blob被多次查询，cold check成本和失败面随脚本数线性放大。需要共享Git object session、batch cat-file、bounded worker pool与content-addressed blob cache。

### TOOL-CODEGEN-P1-012 · 大多数TypeScript extraction不是AST语义

386个脚本中只有20个调用`ts.createSourceFile`；regex/slice对format、comment、alias、re-export、generic和computed syntax脆弱。所有语义提取必须基于compiler Program/TypeChecker；文本扫描只允许明确的literal fixture并有format-perturbation test。

### TOOL-CODEGEN-P1-013 · 没有统一固定source snapshot owner

虽然current commit被写入多个catalog，脚本仍各自决定工作树、commit、历史模式和路径。runner应先解析一个immutable `SourceSnapshotId`，所有node只能通过snapshot service读取，禁止混用live worktree与Git object。

### TOOL-CODEGEN-P1-014 · 相同计数在JS、Rust、文档和测试中重复

`woc_contract_codegen::lib`硬编码commands/world members/tests/GLB等expected count，JS inventory和README又保留同类数字，pet test再硬编码typed contract count。计数应由catalog schema与baseline policy生成，手写代码只声明允许的兼容变化规则。

### TOOL-CODEGEN-P1-015 · extractor输出和generator输入没有版本化typed boundary

JSON常有`schema_version`，但没有共享schema registry、canonical codec或compatibility diff。应让每个IR schema有stable ID/version/definition hash，producer与consumer在graph加载时协商并拒绝未知breaking revision。

### TOOL-CODEGEN-P1-016 · 局部SHA-256没有形成增量cache key

135个脚本使用`createHash`，主要写入catalog/产物fingerprint；runner仍不知道“tool + transitive inputs + options + target”是否未变。需要统一node key和dependency full hash，命中时复用verified outputs而不是重跑脚本。

### TOOL-CODEGEN-P1-017 · dependency discovery发生在执行期间且不回写图

source extractor动态遍历Git tree/import，却没有可供下次invalidating的dependency receipt。首次执行可发现依赖，但必须把resolved dependency set写入receipt并参与后续changed closure。

### TOOL-CODEGEN-P1-018 · current/historical/rebaseline policy分散

`reference_inventory`同时暴露`--historical`、`--rebaseline`、`--check`和自由`--output`，package甚至组合`--rebaseline --check`。source baseline promotion应是独立受审命令，普通check只能读取已批准baseline，不能共享可变更语义的flag组合。

### TOOL-CODEGEN-P1-019 · 路径没有统一canonical/containment策略

扫描未发现`realpathSync`，各脚本直接`join/resolve`输入输出。graph层必须解析symlink、case和drive identity，约束source/output roots，防止不同文本路径命中同一文件或越过project边界。

### TOOL-CODEGEN-P1-020 · 子进程、buffer和并行资源没有全局budget

个别脚本自行设置`maxBuffer`，整体没有并发、内存、stdout、timeout和取消策略。runner必须提供process lease、cooperative cancellation与per-node/global resource budget。

## 6. 产物发布、provenance 与恢复差距

### TOOL-CODEGEN-P1-021 · 没有跨输出generation ID

即使每个文件内容单独通过check，也不能证明Rust/ZrVM/JSON来自同一次IR。所有projection header或sidecar必须携相同generation ID、IR fingerprint和producer receipt hash。

### TOOL-CODEGEN-P1-022 · check按单文件精确比较，不验证集合完整性

`writeOrCheck`能发现已知目标漂移，但不知道少一个目标、多一个陈旧目标或错误owner。check必须验证manifest中的完整集合、mode、permission、hash和absence rules。

### TOOL-CODEGEN-P1-023 · 没有并发writer lease

两个npm命令、Editor后台import或CI shard可同时覆盖同一产物。需要按graph/output set获取跨进程lease，冲突时等待、取消或明确失败，不能last-writer-wins。

### TOOL-CODEGEN-P1-024 · 没有 write-if-changed

JS、native codegen和三个生成型build script均直接写，即使内容相同也更新时间戳。publisher应先比较hash/bytes，未变化时保留inode/mtime，以保护Cargo和Editor增量链。

### TOOL-CODEGEN-P1-025 · generated marker格式不统一

产物使用完整tool path、仅tool basename、`woc_contract_codegen`或泛化generated文本。仓内已有统一first-line marker guard思路；WOC应采用machine-readable producer ID、schema、generation和do-not-edit格式。

### TOOL-CODEGEN-P1-026 · 多数产物不记录完整输入与tool identity

source commit或catalog SHA存在于部分文件，但没有统一transitive input hash、options、tool binary/version、platform和timestamp policy。receipt必须成为可重放和审计的最小证据。

### TOOL-CODEGEN-P1-027 · 没有失败恢复或incomplete transaction quarantine

当前失败只留下半写文件和进程exit。启动时应扫描transaction log/staging generation，丢弃未commit代或重新调度，不允许runtime/CI消费未完成输出。

## 7. Generated ZrVM 与运行时数据模型差距

### TOOL-CODEGEN-P1-028 · 缺少引擎级紧凑只读内容表

`m4_ability_catalog.zr`把117个ID映射展开成连续`if`，其他产物对index/field/byte重复相同形状。应由engine提供versioned immutable table/blob、typed column、string pool、hash/index与VM只读handle，生成物保留数据而非控制流。

### TOOL-CODEGEN-P1-029 · 解释执行放大生成分支成本

`woc_game/plugin.toml`明确`execution_mode = "interp"`。在没有JIT/AOT证明前，56,242个分支会增加parse/compile、instruction dispatch与I-cache压力。必须建立package load、compile、lookup和tick microbenchmark，再以数据表迁移后的预算作为门禁。

### TOOL-CODEGEN-P1-030 · 生成代码承担rank/effect选择语义

`rankAtLevel`、metric override和effect field dispatch不是纯常量表；改变生成器会改变runtime选择语义，违反“生成物可替换而不改变语义”的leaf boundary。规则应由手写interpreter/selector拥有，生成物只提供rank/effect rows。

### TOOL-CODEGEN-P1-031 · 字符串被展开为逐byte accessor

generator为ID生成`Utf8Length`与每个byte的分支，说明VM缺稳定string/blob view ABI。需要只读slice/string handle、bounded UTF-8 validation与zero/low-copy host bridge。

### TOOL-CODEGEN-P1-032 · 大生成函数没有compiler complexity budget

最大单文件13,070行，部分函数包含数千分支；没有AST node、function instruction、compile time、bytecode size或max branch budget。compiler必须显式拒绝/分区超预算artifact并给出producer诊断。

### TOOL-CODEGEN-P1-033 · 内容分片按里程碑/生成器历史而非runtime locality

`m3/m4/m5/m8`命名和多个catalog重叠反映实施阶段，不是加载域、hot set、zone或feature chunk。产物应按runtime access/locality/streaming ownership分区，milestone只留在计划记录。

### TOOL-CODEGEN-P1-034 · package schema fingerprint未覆盖全部generated content

`stateSchema()`只组合contract、command catalog与payload fingerprint；ability/content/encounter/talent等大量生成表没有一个aggregate content set identity。save/replication/hot reload必须绑定完整content manifest hash并定义兼容策略。

## 8. Native `woc_contract_codegen` 差距

### TOOL-CODEGEN-P1-035 · typed native codegen未成为统一pipeline owner

该crate拥有本轮最完整的schema validation，却不在WOC npm入口或CI中；JS generators继续维护平行contract逻辑。应把它提升为graph node/共享schema library，JS只保留必要的upstream adapter或全部收敛到同一IR。

### TOOL-CODEGEN-P1-036 · CLI是三个位置参数加可选末尾`--check`

参数顺序脆弱，没有named output、manifest receipt、format selection、diagnostic mode或unknown option结构。应使用workspace统一CLI DTO并提供`generate/check/describe`子命令。

### TOOL-CODEGEN-P1-037 · Rust/ZrVM projection仍顺序直接写

binary在`write_projection(rust)`成功后才写ZrVM，第二次失败即留下mixed pair；`fs::create_dir_all + fs::write`没有temporary sibling或rollback。必须接入统一publisher。

### TOOL-CODEGEN-P1-038 · fingerprint只证明manifest，不证明producer/toolchain

`GeneratedProjections`的fingerprint来自canonical manifest，这一点应保留；但相同manifest在generator变更后可生成不同代码而fingerprint不变。需要另有projection definition hash和tool identity。

### TOOL-CODEGEN-P1-039 · 输出没有集合manifest与stale cleanup

CLI只知道两条路径，不产生output-set receipt，也不能在projection拆分/重命名后清理旧文件。应让emitter返回logical artifacts，由publisher决定布局与retirement。

### TOOL-CODEGEN-P1-040 · reference baseline计数仍手工编译进Rust

`EXPECTED_SOURCE_FILES/COMMANDS/TEST_CASES/...`使正常上游升级必须同时改多个常量。baseline policy应读取受审snapshot manifest并生成compatibility diff，Rust validator只验证结构与批准的变化集。

## 9. Cargo Build Script 与增量边界差距

### TOOL-CODEGEN-P1-041 · Editor build script建立了另一份不完整plugin schema authority

`zircon_editor/build.rs:108-156`直接parse `toml::Table`，只抽取id/display/category/第一项editor module/crate/capabilities，绕过plugin SDK的版本、platform、engine range、module uniqueness和capability policy。必须调用canonical manifest parser/validator crate，build script只消费validated catalog DTO。

### TOOL-CODEGEN-P1-042 · Editor catalog会静默吞掉错误字段

只使用`find`取得第一项editor module；capability array通过`filter_map(as_str)`静默丢弃非string；重复package ID只排序不拒绝。应拒绝未知/错误类型、重复ID、多editor module冲突，并报告manifest source span。

### TOOL-CODEGEN-P1-043 · Runtime profile build script复制权威清单

TOML已有12个builtin module与6个profile，build.rs又硬编码同一顺序/variant/feature期待值。严格比对能发现漂移但也形成双authority。应把stable ID/variant映射放入共享typed schema，Rust enum和preset projection从同一source生成，validation检查policy而非复制整表。

### TOOL-CODEGEN-P1-044 · Runtime通过Cargo内部目录形状猜`profiling` profile

`active_profile_dir`在`OUT_DIR` components中找最后一个`build`并取前一段。这依赖Cargo target目录布局且没有正式协议。应由managed build request显式传入受控env/config，或把profile policy放到上层validator，build script只验证Cargo公开env。

### TOOL-CODEGEN-P1-045 · 生成型build script没有共享write-if-changed/receipt helper

Runtime写两份preset Rust，Editor写两份manifest，均直接`fs::write`。应建立小型build-codegen library：validated IR、deterministic render、compare-before-write、producer header和聚焦测试；Cargo OUT_DIR artifact不必进入全局内容事务，但仍应避免无意义rewrite。

### TOOL-CODEGEN-P1-046 · Editor asset rerun范围过宽且icon inventory手工复制

build script递归对`assets`所有目录/文件发`rerun-if-changed`，任意无关Editor资源变化都可触发整个crate build；两种gizmo icon又在Rust数组中手工列举。应有独立asset manifest/node，只把真实consumer依赖及manifest hash接入Cargo。

### TOOL-CODEGEN-P1-047 · Navigation native source枚举不确定且invalidating过宽

`fs::read_dir`结果未排序即加入`cc::Build`，不同filesystem可改变object/archive order；同时递归vendor root被整体watch。应固定source manifest或排序canonical path，记录vendor revision/compiler/flags/target ABI，并追踪精确输入集合。

### TOOL-CODEGEN-P1-048 · Physics native链接策略把未知target当GNU `stdc++`

Jolt非MSVC且非Apple时一律链接`stdc++`，Android、wasm、musl和未来target没有显式admission；build script又未发rerun directive，Cargo退回package-wide invalidation。应由target capability matrix选择supported runtime/linker，未知target明确失败或禁用backend，并对每个支持target做link-plan test。

## 10. 次要完整性与维护差距

### TOOL-CODEGEN-P2-001 · WOC README 已成为71KB手写状态账本

大量WOS逐版本行为说明难以与generated manifest保持一致。应生成简洁capability/migration table，README保留定位和权威链接。

### TOOL-CODEGEN-P2-002 · 没有machine-readable工具覆盖率报告

198不可达、28默认check等只能靠临时脚本计算。graph validator应稳定输出node/tool/output/CI coverage摘要并允许baseline diff。

### TOOL-CODEGEN-P2-003 · 默认门禁没有wall-time与慢节点预算

80.945秒只完成前7项仍无性能告警。建立cold/warm/changed-one-node预算和历史趋势，防止工具链随脚本数退化。

### TOOL-CODEGEN-P2-004 · generator命名暴露阶段而非责任

大量`m3/m4/m5/m8/wosNN`难以从名称判断schema owner、输入和输出。保留迁移alias，但新graph ID应使用domain + artifact + version。

### TOOL-CODEGEN-P2-005 · newline、format和排序惯例重复实现

每个脚本自行`JSON.stringify`、尾换行、array sort和string escape。共享canonical codecs可减少无意义diff和平台差异。

### TOOL-CODEGEN-P2-006 · build script日志没有生成摘要

Cargo输出看不到input count、unchanged/written、artifact hash或耗时。debug/CI模式应提供结构化摘要，普通build保持安静。

### TOOL-CODEGEN-P2-007 · 8 MiB Editor stack reserve没有验证依据

常量有名称但没有call-depth、crash、platform或profile证据。保留临时值时应链接测量记录，并以target descriptor policy管理。

### TOOL-CODEGEN-P2-008 · 两个Tauri build script只有opaque one-liner

`tauri_build::build()`本身合理，但本仓没有围绕实际config/resource变化的contract test或artifact摘要。升级Tauri或增加resource时应由package-level build test覆盖，不要在脚本内复制Tauri逻辑。

### TOOL-CODEGEN-P2-009 · generated source缺少统一source map

错误通常只能定位巨大`.zr`产物行，无法返回上游TS/JSON row。IR/emitter应输出compact source map，compiler/runtime diagnostic携origin identity。

### TOOL-CODEGEN-P2-010 · 没有面向Editor的内容编译状态模型

当前npm stdout无法支持Editor展示queued/running/cache-hit/failed/stale。统一runner应发布versioned event DTO，后续Editor只消费状态而不另建调度authority。

## 11. 目标架构

```text
Pinned SourceSnapshot / Authored Content / Build Target
                         |
                         v
                 ContentBuildGraph
       node id / owner / inputs / outputs / deps / budgets
                         |
        +----------------+----------------+
        |                                 |
        v                                 v
 TypeScript/Git Source Adapter      Native/Cargo Adapter
        |                                 |
        +------------> Typed IR <---------+
                         |
                 Shared Validators
        identity / schema / compatibility / policy
                         |
          deterministic leaf emitters
        JSON / binary table / Rust / ZrVM adapter
                         |
                Transactional Publisher
    staging / hashes / output manifest / journal / commit
                         |
                  Artifact Receipt
       generation / tool / inputs / outputs / source map
                         |
          CI, Editor, Runtime, Incremental Cache
```

关键owner规则：

1. `ContentBuildGraph`拥有发现、依赖、调度、cache与check completeness；npm/Cargo/Editor只是入口。
2. source adapter拥有外部语法解析，但不能拥有gameplay/runtime规则。
3. typed IR与validator拥有schema、identity和兼容策略；emitter不能重新解释业务语义。
4. generated artifact只允许data/table/manifest/schema/thin adapter；rank/effect selection由手写runtime owner解释数据。
5. publisher拥有所有mutating write、lease、atomicity、receipt、stale cleanup与recovery；generator不得直接写产品路径。
6. Cargo build script只做target admission和OUT_DIR leaf projection，复杂parser/validator/render进入普通可测试library。

## 12. 分层重构路线

### M0 · 恢复可复现基线

- 修正`examples/*` ignore策略，审计103个ignored工具文件，提交或删除当前两个tracked入口依赖；增加clean-clone gate。
- 从单一coverage artifact生成pet等测试期望，修复当前157/148失败；默认check改为all-node aggregation。
- 建立临时`tool-inventory.toml`，让386个脚本全部分类，CI拒绝unclassified、missing file和duplicate output。
- 修复`zircon_app/build.rs` host/target判断并加cross-target emit test。

### M1 · 建立统一 ContentBuildGraph 与 runner

- 定义node/input/output/dependency/tool/budget/diagnostic/receipt DTO和stable schema ID。
- 从现有package scripts导入aliases，但由graph生成`generate/check/changed`计划。
- 接入structured progress、failure aggregation、process lease、cancellation和cold/warm timing。

### M2 · 原子发布与增量cache

- 所有JS/Rust generator改为返回artifact bytes，不直接写目标。
- 实现staging、write-if-changed、hash、source map、output manifest、journal与generation commit。
- 用tool + options + source snapshot + transitive input full hash形成cache key；实现stale cleanup和crash recovery fault injection。

### M3 · 收敛 source adapter 与 typed IR

- 建立单一Git object/source snapshot service和TypeScript Program/TypeChecker session。
- 先迁command/payload/reference inventory，再迁ability/content/scene/contract extractor。
- 删除重复字符串parser、fixed count和JS/Rust schema表；compatibility diff成为baseline promotion的唯一入口。

### M4 · 引入紧凑运行时内容数据

- 为ZrVM提供immutable table/blob、string pool、typed column、index和bounded view ABI。
- 把rank/effect/field选择迁到手写runtime accessor；generated ZrVM只保留薄handle/table adapter。
- 对106份现有产物逐域迁移，建立compile time、bytecode size、lookup latency、package load与memory预算。

### M5 · Cargo build-codegen治理

- 抽出canonical plugin manifest/profile schema/build-codegen helper。
- Editor catalog只消费validated plugin DTO；runtime preset由共享schema生成enum/rows，移除`OUT_DIR`目录猜测。
- Navigation固定source manifest/排序与ABI receipt；Physics使用显式target matrix；所有OUT_DIR emit采用write-if-changed。

### M6 · 产品接入与长期验证

- CI运行clean clone、graph completeness、determinism、fault injection、changed-set与cross-platform path/case测试。
- Editor消费runner event DTO，展示内容状态但不拥有第二调度器。
- release receipt绑定Build Set、content generation、plugin/schema/runtime fingerprint；runtime拒绝mixed/incompatible generation。

## 13. 验收门

1. 从clean clone执行WOC bootstrap、`check:all`和所有named authoritative入口，不依赖ignored/untracked文件。
2. 所有工具文件分类完成，`unclassified = 0`；所有tracked generated artifact恰有一个producer，duplicate/missing owner均为0。
3. 默认门禁执行全图或有证明的changed closure，失败仍完成所有独立节点并聚合诊断。
4. 当前157/148漂移修复后，命令/payload计数只从一个typed catalog派生，不再手工复制。
5. 任意generator在第N个output写入前后故障，旧generation仍完整可读，新generation不可见且下次可恢复。
6. 并发运行两个重叠生成计划不会互相覆盖、死锁或产生mixed receipt。
7. unchanged生成不改变目标mtime；单输入变化只重跑transitive dependents。
8. Windows/Linux两个独立root、不同目录枚举顺序生成的artifact bytes与manifest hash完全一致。
9. 每个receipt包含source snapshot、tool identity、IR/schema hash、inputs、outputs、generation和source map。
10. TypeScript语义extractor使用Program/TypeChecker；format/comment/re-export变化不改变语义结果。
11. ZrVM generated artifact不拥有rank/effect/gameplay选择规则，只携数据与薄adapter，generated-code boundary gate保持clear。
12. 迁移后的ability/content lookup不再依赖线性`if`级联，并达到明确的load/compile/lookup/memory预算。
13. runtime state/hot reload/replication绑定完整content set fingerprint，兼容与迁移决策可审计。
14. `woc_contract_codegen`进入统一graph与CI，Rust/ZrVM projection共享generation并原子发布。
15. Editor plugin catalog由canonical manifest validator生成，错误capability、多editor module、重复ID和未知schema均阻断。
16. Runtime profile generation不复制整份module/profile authority，也不解析Cargo内部OUT_DIR布局。
17. Navigation source order在所有filesystem稳定，native receipt记录vendor/compiler/flags/target ABI。
18. Physics Jolt只在显式支持target链接，未知target有确定诊断而非默认`stdc++`。
19. `zircon_app` cross-target build-plan测试证明link arg只按target OS/env发出。
20. graph runner提供结构化progress、stable diagnostic code、cancel和resource budget，80秒静默检查不再出现。
21. CI保存cold/warm/changed-one-node性能趋势，超过预算阻断或要求显式批准。
22. implementation完成后重取两个scope fingerprint，并回归WOC、Cargo build-codegen、Editor catalog、runtime profile、plugin/runtime content fingerprint consumer。

## 14. 不应保留的临时方案

- 不再把新的WOC工具靠`git add -f`塞进被`examples/*`隐藏的目录。
- 不再通过给默认`check`末尾继续追加`&& node ...`宣称覆盖增长。
- 不再让generator直接顺序覆盖Rust/ZrVM/JSON多份产品文件。
- 不再为每种内容生成更大的ZrVM `if`查找函数来替代数据表能力。
- 不再用新的手工expected count、source slice或regex parser修补authority漂移。
- 不再让Editor/build.rs/CLI各自解析plugin TOML子集。
- 不再用build-host `cfg!`、Cargo内部OUT_DIR形状或未知target fallback决定产品target行为。

本报告只完成review和重构设计。M0之前不得把当前WOC默认check、generated contract完整性、clean-clone可重现性或跨target build script正确性写成已完成能力。
