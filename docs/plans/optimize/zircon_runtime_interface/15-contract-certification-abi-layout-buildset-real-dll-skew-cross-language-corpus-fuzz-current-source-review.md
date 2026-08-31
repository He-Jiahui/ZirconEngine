# `zircon_runtime_interface` Contract Certification / ABI Layout / BuildSet / Real DLL / Skew / Cross-Language / Corpus / Fuzz 当前源码复核

> report_id: `Interface15`  
> kind: `current-source-review`  
> canonical_source: [07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md](07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md)  
> related_runtime_contract: [08-runtime-dll-abi-ffi-version-handle-foreign-ownership-current-source-review.md](08-runtime-dll-abi-ffi-version-handle-foreign-ownership-current-source-review.md)  
> related_host_contract: [09-runtime-host-foreign-output-safe-owner-admission-budget-fuse-observability-current-source-review.md](09-runtime-host-foreign-output-safe-owner-admission-budget-fuse-observability-current-source-review.md)  
> baseline: 当前工作树 `f2df7ed2100a771881a3b7222b726789b0b40abd`（2026-08-29）；未提交与未跟踪源码纳入观察指纹，但不视为该提交已经交付的能力。  
> direct_certification_fingerprint: `f373ecd675e44cbf3260c7e444ac9a5ae99541cfbaf997c75b0ef57790406ca7`  
> focused_product_fingerprint: `a5a99c822b3769080b02eaffeb68516ec547d85776c4b9510abd137dd71d9efc`  
> deduplicated_current_source_fingerprint: `d49c5646d5801d8196e5f61c43ea172fe899a51a221cb6fb13546f5b1bc41a62`  
> reference_fingerprint: `a9d04f7a5d7b2f559a36792d57307fd45901cfb56ca2a289d4a356b05737c96`  
> status: review-only；不修改 production/test/Cargo/ABI，不运行 Cargo、真实 DLL、C/C++ consumer、unload/reload、child-process fault、sanitizer、fuzz、跨版本 artifact、跨平台或动态 benchmark。Tooling/Rust 迁移按用户要求排除；未查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Interface07 的核心判断仍成立，但不能再沿用“没有任何 InterfaceSpec、BuildSet 或真实产品 DLL 证据”的旧快照。当前工作树已经新增一个受构建脚本校验的 `interface_spec_v1.json`，生成 V8 family/version/entry symbol 与 required/optional slot-name catalog；新增 artifact manifest、SHA-256 artifact/host identity、InterfaceSpec digest、payload-schema digest、target architecture/OS/pointer-width/endianness、capability set 与派生 BuildSet ID；App 在 `Library::new` 前后校验 64 KiB sidecar、Runtime DLL digest、当前 Host digest和期望 BuildSet；构建脚本能为 staged DLL/Host 生成 sidecar；Windows MVP workflow 也开始从同一 source-bound BuildSet 构建并运行 staged F5 acceptance。这些都是应保留的工程底座。

它们仍没有组成发布级 ABI 认证：

1. 当前 InterfaceSpec 只有 7 个顶层字段与 slot 名称，没有字段类型、offset、size、align、calling convention、nullability、ownership、thread/reentrancy、budget、failure、plugin table、enum/constant 或 C header。Rust ABI table仍为手写，另有一份手写field-name常量，再由测试解析自身源码比较。
2. artifact manifest能绑定构建产物身份，却没有 PE/ELF/Mach-O export manifest、layout manifest、ABI diff、toolchain/CRT/LTO身份和资格结果receipt。按路径先hash再`Library::new`也没有形成handle-bound anti-replacement证明。
3. Windows staged F5是产品正向进展，不是Interface专用 external-host certification：没有逐required-slot真实DLL smoke/fault/owner-release映射，没有受控wrong/truncated/old/corrupt fixture family，没有unload/reload、stale callback/handle、child-process crash或C/C++ consumer。
4. 586个Interface测试覆盖大量同版本serde、layout、parser与DTO行为，但默认lane仍混入墙钟p99/throughput断言；7个Interface测试被ignore，跨层聚焦集还有17个ignore；real ABI correctness与benchmark仍可整体跳过。
5. `payload_schema_set_v1.json`只有family/version/encoding/serialization/status六项元数据，不是Schema Catalog；全crate没有proptest、quickcheck、fuzz target或versioned crash corpus，只有一处局部binary golden bytes用例。
6. 当前源码的source-shape guard已出现自相矛盾：`abi_safety_contracts.rs`仍要求build script包含V7 expected version，而实际build script已固定V8；`boundary.rs`仍禁止`[build-dependencies]`且依赖allowlist遗漏当前依赖，而Cargo已有`serde_json` build dependency。此类测试即使修到绿色，也仍只是源码形状证明，不是artifact ABI证明。
7. Godot的machine-readable interface/schema/header generation与历史C consumer、Unreal的BuildId/module manifest、Bevy独立compile-fail lane、Unity Graphics显式Editor/OS API validation与serialization compatibility都说明：声明、artifact身份、跨语言consumer、历史版本与suite taxonomy必须分层。Fyrox的same-toolchain Rust trait-object dylib只适合作为更低兼容目标，不能替代稳定C ABI资格。

本轮对 Interface07 的 1 个 P0、48 个 P1 与12个P2逐项重判：P0为 **1 Partial**；P1为 **26 Open / 22 Partial / 0 Closed**；P2为 **12 Open / 0 Partial / 0 Closed**。32项资格门为 **16 Fail / 16 Partial / 0 Pass**。本轮没有新增唯一P0/P1/P2；报告数增加1，canonical finding总数不重复增加。

## 2. 审查边界与证据

### 2.1 物理范围

| 选择集 | files / lines / bytes / test attrs / ignored | tracked / modified / untracked | fingerprint |
|---|---:|---:|---|
| Interface直接认证面 | **152 / 37,372 / 1,302,838 / 586 / 7** | **88 / 65 / 64** | `f373ecd675e44cbf3260c7e444ac9a5ae99541cfbaf997c75b0ef57790406ca7` |
| Runtime/Host/App/Editor/CI focused消费者 | **154 / 36,155 / 1,312,598 / 372 / 17** | **122 / 67 / 32** | `a5a99c822b3769080b02eaffeb68516ec547d85776c4b9510abd137dd71d9efc` |
| 去重当前源码 | **306 / 73,527 / 2,615,436 / 958 / 24** | **210 / 132 / 96** | `d49c5646d5801d8196e5f61c43ea172fe899a51a221cb6fb13546f5b1bc41a62` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics参考 | **18 / 17,503 / 613,618** | n/a | `a9d04f7a5d7b2f559a36792d57307fd45901cfb56ca2a289d4a356b05737c96` |

Interface直接选择包含全部125个带`#[test]`的Rust文件、Cargo/build/lib/version、`runtime_build_set`全量和`runtime_api/abi`全量。focused选择沿Runtime `dynamic_api`与owned-result V8、Host `foreign_output`、App `runtime_library`、Editor gateway session与real-runtime ABI、runtime manifest/staging脚本及测试、Runtime10 acceptance和三份CI workflow。两组当前没有物理重叠。

指纹算法为：规范化workspace相对路径排序，逐文件SHA-256，再以`path<TAB>sha256`和LF拼接后取SHA-256；未跟踪文件纳入当前观察指纹。它证明本文审查的工作树内容，不证明clean checkout、编译、测试或发布行为。

### 2.2 测试形状

| 范围 | 当前值 | 复核结论 |
|---|---:|---|
| Interface Rust文件 | **545** | crate规模已经不能靠少量中央测试文件代表。 |
| 带`#[test]`文件 / test attrs | **125 / 586** | 比Interface07的401项基线增加185项，但没有machine coverage receipt解释public contract增删。 |
| `src/tests` | **39 files / 15,062 lines / 260 tests** | 仍是中央复合suite；`contracts.rs`为2,962行/49 tests，`render_contracts.rs`为1,947行/36 tests。 |
| package-root integration | **3 files / 186 lines / 10 tests** | 只覆盖math、profiling和UI binding局部，不是独立SDK/ABI consumer。 |
| crate内被ignore | **7** | ignore理由存在，但没有required capability/owner/expiry/result manifest。 |
| `should_panic` | **2** | 仍由legacy UI hit-path builder承担外部拒绝语义。 |
| property/fuzz/corpus | **0 target** | `proptest`、`quickcheck`、fuzz target与crash corpus均不存在。 |

当前文本扫描还发现273处round-trip相关命中、499处`serde_json`、233处`.contains()`、94处`size_of`、12处`align_of`、60处`offset_of`。这些数字只用于揭示测试形状，不能按命中数推导覆盖率。

### 2.3 动态证据边界

本轮没有运行Cargo或产品进程。Windows workflow、staging脚本、测试属性和source-bound acceptance仅按源码审查；本文不声称当前dirty/untracked树能通过clean checkout，也不声称staged F5完成了全部required slot、unload或故障资格。

## 3. 当前可保留底座

1. 保留machine-readable InterfaceSpec family/version/entry symbol和required/optional slot partition，但把它升级为完整ABI AST，而不是继续扩充slot-name JSON。
2. 保留build-time exact-field、identifier、duplicate-slot和V8 metadata校验；生成物必须扩展到Rust declarations、C header、ABI/layout/symbol manifest、SDK docs和test inventory。
3. 保留artifact/host SHA-256、InterfaceSpec/payload digest、target model、capability与derived BuildSet ID，以及App pre/post-load validation。
4. 保留sidecar 64 KiB上限、deny-unknown manifest、host-artifact集合、expected capability和target fail-closed。
5. 保留source-bound staged Windows product build/F5 acceptance，但将其作为产品smoke消费者，不能把它复用为Interface certification唯一证据。
6. 保留opaque allocation、release owner、bounded foreign output、session lifecycle与已有concurrency/fuse tests，后续由生成的slot registry驱动真实DLL fault/release矩阵。
7. 保留局部binary golden、duplicate/depth/size/unknown-field测试，迁入Schema Catalog驱动的immutable corpus。
8. 保留release-only benchmark的显式ignore理由方向；默认正确性suite中的墙钟阈值必须移出。

## 4. Canonical P0 当前重判

| ID | 状态 | 当前源码证据 | 关闭条件 |
|---|---|---|---|
| RI-CERT-P0-001 | Partial | Windows MVP workflow开始从同一BuildSet构建、stage并运行产品F5；sidecar绑定Runtime/Host digest、InterfaceSpec、payload schema、target和capability，App在load前后校验。 | 独立external host对构建出的real DLL执行entry/symbol/layout、全部required slot、foreign release、close/quiesce、destroy/unload；old/wrong/truncated/corrupt/BuildSet-skew fixture fail-closed；结果绑定artifact/runner/toolchain并成为release required gate。 |

## 5. P1 旧条目当前重判

状态语义：`Closed`表示旧命题已被当前源码消除；`Partial`表示出现可迁移底座，但原工程合同仍不成立；`Open`表示旧命题仍直接成立。

### 5.1 InterfaceSpec、ABI manifest 与静态布局

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-CERT-P1-001 | Partial | 已有machine JSON spec和build-time slot catalog；它只描述family/version/symbol/slot name，不是carrier/type/layout/ownership/calling-convention IDL。 |
| RI-CERT-P1-002 | Open | `abi_safety_contracts`和`api_table`测试仍以`find`、行切分和字符串包含解析Rust源码。 |
| RI-CERT-P1-003 | Partial | V8 Runtime/Host slot进入spec partition；plugin tables、全部FFI carrier/enum/callback/public re-export没有统一枚举。 |
| RI-CERT-P1-004 | Open | field-count与slot-name guard仍不记录field type、pointer mutability、callback signature、offset或ownership。 |
| RI-CERT-P1-005 | Open | `Box<dyn`/`Rc<`/`Arc<dyn`/`impl Trait`文本blacklist仍是public FFI closure的主要守卫。 |
| RI-CERT-P1-006 | Open | 没有versioned ABI manifest与compatible/breaking diff artifact。 |
| RI-CERT-P1-007 | Open | 没有从最终PE/ELF/Mach-O提取并精确比较export/visibility的required gate。 |
| RI-CERT-P1-008 | Open | 没有由同一spec生成、发布并带static assert的C header。 |
| RI-CERT-P1-009 | Open | 没有MSVC/clang/gcc C/C++ consumer compile-link-run真实DLL资格。 |
| RI-CERT-P1-010 | Partial | manifest已有architecture/OS/pointer-width/endianness target model；layout断言仍是当前target literal，没有per-target snapshot。 |
| RI-CERT-P1-011 | Partial | Windows source-bound workflow提供一个实际构建组合；manifest没有完整Rust/compiler/linker/CRT/LTO identity，也没有声明支持矩阵。 |
| RI-CERT-P1-012 | Partial | target model显式记录pointer width和endianness；packing/natural alignment/scalar representation仍未进入manifest/header assert。 |
| RI-CERT-P1-013 | Open | 没有padding初始化、raw-byte identity禁用或跨工具动态检查。 |
| RI-CERT-P1-014 | Open | enum/constant/raw status/tag仍由零散手写断言维护，没有生成value registry与diff。 |
| RI-CERT-P1-015 | Partial | V8 slot partition形成单一名称清单；owner、type、requiredness条件、budget、release、fault和test mapping仍缺失。 |
| RI-CERT-P1-016 | Open | 文档phrase与源码needle guard仍被当作ABI version strategy证据，且当前V7/V8期望已经漂移。 |

### 5.2 Real DLL、进程、生命周期与故障

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-CERT-P1-017 | Partial | staged Windows产品路径开始正向加载真实DLL；App产品测试仍可因backend/env缺失ignore，也没有独立headless多target smoke。 |
| RI-CERT-P1-018 | Open | missing-symbol测试仍复用系统库，没有受控wrong symbol/version/truncated/oversized/host fixture family。 |
| RI-CERT-P1-019 | Open | 没有真实DLL session/allocation/callback/thread census归零后的unload/reload资格。 |
| RI-CERT-P1-020 | Open | 没有unload后stale handle/callback/releaser/table pointer的epoch-qualified拒绝测试。 |
| RI-CERT-P1-021 | Partial | BuildSet sidecar与expected identity能在load前拒绝mismatch；没有N/N-1 host/runtime binary、feature/toolchain/schema skew矩阵。 |
| RI-CERT-P1-022 | Open | 没有已发布header/manifest/table binary fixture与明确support window。 |
| RI-CERT-P1-023 | Open | panic wrapper仍主要由源码形状证明，没有逐slot优化构建panic injection。 |
| RI-CERT-P1-024 | Open | 没有child-process abort/SEH/signal/hang分类、dump与cleanup receipt。 |
| RI-CERT-P1-025 | Partial | null、misalignment、length、oversize和protocol fuse已有局部测试；没有guard-page、readonly output、dangling/fake callback adversarial process harness。 |
| RI-CERT-P1-026 | Open | 没有Interface边界适用的Miri/ASan/UBSan required lane与typed skip。 |
| RI-CERT-P1-027 | Open | callback destroy/unsubscribe/release/nested call/panic仍无逐slotreentrancy矩阵。 |
| RI-CERT-P1-028 | Partial | 已有双线程release、close/fuse和barrier型局部测试；没有loom/model state machine及real-DLL unload stress。 |
| RI-CERT-P1-029 | Open | `runtime_v8_release_performance_acceptance`与Host `foreign_output_decode_performance_acceptance`仍在普通`#[test]`中使用墙钟p99/throughput阈值。 |
| RI-CERT-P1-030 | Partial | DLL/Host/InterfaceSpec/payload/target能进入BuildSet sidecar；测试结果本身仍不携带selection、command、toolchain、runner和所有artifact digest receipt。 |

### 5.3 Schema、版本与生成式输入

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-CERT-P1-031 | Partial | same-version round-trip仍占主导；已有一处固定binary golden bytes与部分immutable fixture方向。 |
| RI-CERT-P1-032 | Partial | serialization存在单个局部golden测试；没有覆盖全部public wire family的valid/minimal/maximal/legacy/future/malformed corpus manifest。 |
| RI-CERT-P1-033 | Open | 没有保存N/N-1 reader/writer/host/runtime/plugin executable artifacts并运行双向矩阵。 |
| RI-CERT-P1-034 | Partial | 源码已有局部`SchemaId`与payload-schema digest；`payload_schema_set_v1.json`只是174-byte migration baseline，不可枚举wire type/owner/version/policy/budget/migration。 |
| RI-CERT-P1-035 | Partial | deny-unknown/default/missing-field存在零散测试；没有catalog-driven closed/open/extension-map政策。 |
| RI-CERT-P1-036 | Partial | duplicate、depth、Unicode/canonical parse在部分serialization/project域有覆盖；没有跨profile/UI/world/Hub/plugin family统一corpus。 |
| RI-CERT-P1-037 | Partial | shared payload/frame/foreign-output limit和若干边界测试已存在；没有每schema的limit-1/limit/limit+1 producer-to-consumer生成矩阵。 |
| RI-CERT-P1-038 | Open | crate与聚焦选择没有proptest、quickcheck、coverage-guided fuzz target或版本化crash corpus。 |
| RI-CERT-P1-039 | Open | 没有独立C++/C#/TypeScript JSON/DTO consumer验证canonical fixture与unknown policy。 |
| RI-CERT-P1-040 | Open | 没有跨architecture/endianness历史binary corpus与target-neutral/target-reject资格。 |
| RI-CERT-P1-041 | Open | migration fixture仍不绑定已发布producer/reader artifact与immutable support-window currentness。 |
| RI-CERT-P1-042 | Partial | Runtime owned-result与Host foreign-output已有少量真实producer/consumer语义测试；多数DTO仍只在同crate构造并round-trip。 |

### 5.4 Coverage、可维护性与资格声明

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-CERT-P1-043 | Partial | package-root integration从1文件增到3文件/10测试；仍不是只依赖发布API/header的独立SDK/ABI consumer。 |
| RI-CERT-P1-044 | Open | 586项测试没有PublicContractId到layout/schema/behavior/skew/fault/owner的machine映射。 |
| RI-CERT-P1-045 | Partial | 测试已分散到125个owner-near文件；`contracts.rs`和`render_contracts.rs`仍是大型跨域复合owner。 |
| RI-CERT-P1-046 | Open | round-trip/fixture/source parser helper仍跨文件重复，没有要求schema/version/policy/budget的统一WireHarness。 |
| RI-CERT-P1-047 | Partial | 部分benchmark已标记release-only ignore；source-shape、semantic和默认墙钟performance仍混在Cargo test发现面。 |
| RI-CERT-P1-048 | Partial | 24个聚焦ignore marker多有文字理由；没有required condition、owner、timeout、artifact、expiry与release criticality manifest。 |

## 6. P2 旧条目当前重判

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-CERT-P2-001 | Open | `runtime_10`仍出现在function-table inventory、version strategy与错误信息，和V8 public table命名混用。 |
| RI-CERT-P2-002 | Open | `abi_v3_layout_is_stable`包装测试仍存在，没有增加独立case覆盖。 |
| RI-CERT-P2-003 | Open | module行数budget仍是源码扫描中的裸治理阈值，不是named policy/waiver manifest。 |
| RI-CERT-P2-004 | Open | dependency与FFI blacklist仍可被alias、cfg、宏和格式变化绕过。 |
| RI-CERT-P2-005 | Open | `.contains()`在Interface Rust源码中仍有233处命中，部分wire/source guard没有结构化oracle。 |
| RI-CERT-P2-006 | Open | literal layout失败仍不统一打印target triple、pointer width、compiler和manifest revision。 |
| RI-CERT-P2-007 | Open | 两项`should_panic`仍承担UI hit-path legacy builder拒绝语义。 |
| RI-CERT-P2-008 | Open | current-target size与same-version round-trip仍出现`stable`命名，证据强度不足。 |
| RI-CERT-P2-009 | Open | 没有required编译的最小Rust host、C consumer、plugin callback和独立JSON SDK示例。 |
| RI-CERT-P2-010 | Open | 当前有326项测试位于`src/tests`之外的86个文件，仍无完整machine catalog分类。 |
| RI-CERT-P2-011 | Open | dependency allowlist已与Cargo/build-dependency漂移，仍没有owner/reason/scope/expiry waiver。 |
| RI-CERT-P2-012 | Open | Editor real-runtime ABI用例仍被ignore并同时承担正确性与性能证据。 |

## 7. 参考实现对照

| 参考 | 可验证的工程事实 | Zircon当前差距 | 不外推 |
|---|---|---|---|
| Unreal | `BuildVersion`记录BuildId、Changelist/CompatibleChangelist、branch/promoted；`ModuleManifest`把BuildId绑定到module filename map；Automation flags分application context、filter与priority。 | BuildSet有artifact digest底座，但无module/export/layout兼容manifest和分层资格taxonomy。 | 不复制Unreal全部BuildGraph/Automation层级。 |
| Godot | `gdextension_interface.json`与schema驱动header generator；compatibility test保留历史C extension consumer与API hash检查。 | Zircon spec只生成slot-name constants，无C header、历史C consumer、API hash/diff。 | 不把Godot API形状直接复制为Zircon ABI。 |
| Bevy | compile-fail是CI独立命令与目标，不会被普通workspace test自动代表。 | source-shape、semantic、artifact、fault、performance仍没有清晰selection/result边界。 | Bevy没有Zircon动态Runtime DLL同等边界。 |
| Fyrox | dylib依赖same-toolchain Rust trait object，并明确编译器版本风险；hot reload复制DLL后交换状态。 | 说明其兼容保证低于稳定C ABI，不能作为Zircon跨版本/跨语言闭环替代。 | 只借鉴reload隔离，不采用trait-object ABI。 |
| Unity Graphics | API validation job绑定package pack、Unity Editor版本与OS并上传日志/崩溃；serialization tests覆盖legacy/polymorphic mapping。 | Zircon资格结果未绑定完整runner/toolchain/artifact，schema corpus没有legacy independent consumer。 | 本地Graphics镜像不代表完整Unity native engine ABI。 |

## 8. 目标认证架构

### 8.1 单一契约来源

`InterfaceSpec`必须成为带稳定ContractId的完整AST，并至少生成：Rust FFI carrier与table、C header/static assert、symbol/layout/value manifest、SDK docs、slot owner/budget/fault/release registry和test inventory。手写Rust声明与手写slot-name mirror不能继续并存为两个truth source。

### 8.2 六层证据

| 层 | 必须回答的问题 | 主要owner |
|---|---|---|
| C1 Declaration | public carrier/type/value/symbol是否由同一spec生成且diff可审计？ | Interface |
| C2 Artifact | 最终DLL/Host/header/layout/export是否属于同一BuildSet和target/toolchain？ | Interface + Runtime + App |
| C3 Consumer | Rust external host与C/C++ consumer能否只通过发布surface完成握手和最小调用？ | App/Host |
| C4 Skew/Corpus | N/N-1、wrong build/schema/target和历史wire corpus是否按support window accept/reject？ | Interface + product consumers |
| C5 Fault/Lifecycle | panic/crash/hang/bad pointer/reentrancy/concurrency/unload是否隔离并产生typed receipt？ | Runtime + Host + App |
| C6 Release | correctness/performance结果是否绑定artifact、runner、selection、expiry并成为required admission？ | 产品发布控制面；本轮不扩展Tooling审查 |

## 9. 重构顺序

### M0 · Truth Freeze 与测试漂移清理

1. 冻结306文件选择、586项Interface基线、24个ignore和48+12 canonical finding映射。
2. 修复V7/V8 build-script expectation与dependency allowlist的当前源码矛盾，但不得把source-shape绿色当ABI关闭证据。
3. 为每个public table/carrier/schema/symbol分配ContractId、owner和required evidence class。

### M1 · 完整 InterfaceSpec 与生成式 ABI

1. 将field type/order/offset/align/nullability/ownership/calling convention、enum/constant、slot budget/failure/release写入spec AST。
2. 生成Rust声明、C header、layout/symbol/value manifest、docs和tests；删除手写field-name mirror与字符串parser权威地位。
3. 建立compatible/new-version/hard-cut/illegal-in-place ABI diff。

### M2 · Real DLL 与 Cross-Language fixture family

1. 从同一BuildSet产出Runtime DLL、external Rust host、C/C++ consumer、header与manifest。
2. 构建good、missing/wrong symbol、wrong version、truncated/oversized table、wrong BuildSet/target/schema和corrupt sidecar fixture。
3. 对每个required slot运行smoke、fault、output ownership/release与typed diagnostic。

### M3 · Version Skew 与 Schema Corpus

1. 建立Schema Catalog和每family的version、unknown/default、budget、migration、owner与codec。
2. 保存N/N-1 reader/writer/host/runtime/plugin artifacts及immutable valid/minimal/maximal/legacy/future/malformed corpus。
3. 增加至少一个独立非Rust JSON consumer和cross-arch binary policy资格。

### M4 · Fault、Concurrency、Sanitizer 与 Fuzz

1. child process隔离panic/abort/SEH/signal/hang、guard page、readonly/misaligned/dangling/fake callback。
2. 建立callback reentrancy matrix、loom/model小状态机和real-DLL load/use/close/unload stress。
3. 按适用域拆Miri/ASan/UBSan与coverage-guided fuzz；crash corpus自动回归。

### M5 · Suite Taxonomy 与 Release Admission

1. 拆architecture/source-shape、schema/unit、ABI artifact、cross-language、fault、performance suite和selection manifest。
2. 快速correctness不得因benchmark环境缺失被ignore；性能只在受控release artifact/hardware协议运行。
3. 所有结果绑定source、BuildSet、DLL/Host/header/layout/schema/corpus digest、target/toolchain、runner、command、selection、timeout与expiry。

## 10. 资格门当前状态

| Gate | 状态 | 当前证据缺口 |
|---|---|---|
| G01 全部public FFI surface进入spec | Partial | V8 Runtime/Host slot name已进入，carrier/plugin/value未进入。 |
| G02 Rust/C/docs/layout由同一spec生成 | Partial | 只生成Rust constants/slot lists。 |
| G03 Windows x64 MSVC Rust/C/C++ layout一致 | Fail | 无生成C header/consumer。 |
| G04 每个target有layout/symbol manifest和digest | Fail | 只有target identity，无layout/export manifest。 |
| G05 ABI diff分类compatible/new/hard-cut/illegal | Fail | 无diff artifact。 |
| G06 external host正向加载real DLL并握手 | Partial | staged产品F5存在，非独立Interface qualification。 |
| G07 每个required slot有real-DLL smoke/fault/release | Fail | 无全slot外部矩阵。 |
| G08 bad fixture在首调用前fail-closed | Partial | sidecar/BuildSet校验存在，fixture family不完整。 |
| G09 unload前owner census归零 | Fail | 无census/unload。 |
| G10 unload/reload后stale引用不可达 | Fail | 无reload epoch资格。 |
| G11 crash/hang有child-process classified result | Fail | 无进程隔离fault suite。 |
| G12 bad pointer/guard-page不污染主runner | Fail | 无guard-page harness。 |
| G13 Miri/ASan/UBSan适用lane通过 | Fail | 无required lane。 |
| G14 callback reentrancy matrix通过 | Fail | 无逐slot matrix。 |
| G15 call/close/fuse/callback/unload model+stress | Partial | 有局部并发/fuse，无model和unload stress。 |
| G16 每个wire family进入Schema Catalog | Partial | 有SchemaId片段与payload baseline，无catalog。 |
| G17 完整golden corpus及immutable digest | Partial | 只有局部binary golden。 |
| G18 N/N-1 artifact矩阵通过 | Fail | 无历史executables。 |
| G19 同版本双变更不能绕过golden diff | Partial | 单个golden能约束局部，不能覆盖全family。 |
| G20 adversarial parse edge横向覆盖 | Partial | duplicate/depth/Unicode等仅局部。 |
| G21 逐schema纵向预算边界通过 | Partial | 有共享limit和少量纵向测试，无catalog生成矩阵。 |
| G22 property/fuzz与crash corpus通过 | Fail | 无target/corpus。 |
| G23 独立非Rust JSON consumer通过 | Fail | 无独立consumer。 |
| G24 ContractId到五类测试映射无缺口 | Fail | 无machine mapping。 |
| G25 suite按证据类型分离 | Partial | 部分release benchmark隔离，默认lane仍混合。 |
| G26 ignored/manual有required manifest | Partial | 有文字理由，无owner/expiry/result manifest。 |
| G27 test discovery能解释401到586增删 | Partial | 本报告重取586基线，无自动currentness receipt。 |
| G28 资格结果绑定全部artifact身份 | Partial | BuildSet sidecar进展真实，result receipt缺失。 |
| G29 performance绑定硬件/workload/statistics | Partial | 有warmup/sample/p99输出，无hardware/build/置信身份。 |
| G30 与Unreal对照先证明场景等价 | Fail | 无等价协议和动态对照。 |
| G31 聚合器拒绝partial/stale/zero/timeout | Fail | Tooling本轮排除，所选源码无可接受证据。 |
| G32 不越权关闭Interface01-06 finding | Partial | 本报告保持canonical owner；测试系统没有machine防越权映射。 |

## 11. Owner 与排除边界

1. Interface拥有spec AST、generated carrier/header/manifest、Schema Catalog、corpus identity和ContractId/evidence mapping。
2. Runtime拥有真实producer、slot implementation、panic containment、allocation/callback/thread census和quiesce/unload readiness。
3. Host拥有foreign-output safe owner、budget/decode、release/fuse与external consumer oracle。
4. App拥有artifact discovery、pre/post-load validation、process isolation、session destroy和DLL unload产品路径。
5. Editor只消费已认证gateway，不以linked-symbol benchmark替代Runtime DLL发布资格。
6. Tooling聚合、Rust迁移和更大控制面按用户要求不在本轮review范围；未来C6必须消费而不能重新定义C1-C5证据。
7. Interface01/05及其当前复核08/09继续拥有ABI语义和Host safety finding；Interface15只刷新Interface07认证架构，不重复计数。

## 12. 当前状态

InterfaceSpec slot catalog、BuildSet artifact sidecar、target/digest/capability identity、App pre/post-load校验和Windows source-bound staged F5是可保留底座，足以把旧P0从Open重判为Partial；它们不足以关闭任何发布资格门。当前最短工程路径不是继续增加same-version unit test，而是先把spec升级为完整ABI AST，生成C/header/layout/export manifest，再建立external host + controlled fixture family + N/N-1 corpus；之后才有资格推进fault/unload/fuzz和release performance。

本轮只修改review、index与coverage，没有修改production/test/Cargo/ABI；没有运行Cargo、Runtime DLL、App/Editor、C/C++、unload/reload、fault、sanitizer、fuzz、cross-version、cross-platform或动态benchmark。当前dirty/untracked源码仅作为观察证据，所有实施前必须重新取selection fingerprint并在clean source上复验。
