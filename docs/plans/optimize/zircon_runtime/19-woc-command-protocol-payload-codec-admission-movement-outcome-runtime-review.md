---
related_code:
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/protocol
  - examples/woc/scripts/woc_game/src/protocol/binary.zr
  - examples/woc/scripts/woc_game/src/protocol/commands.zr
  - examples/woc/scripts/woc_game/src/protocol/command_payloads.zr
  - examples/woc/scripts/woc_game/src/protocol/movement_input.zr
  - examples/woc/scripts/woc_game/src/protocol/movement_input_test_main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/bin
  - examples/woc/scripts/woc_game/bin/protocol/binary.zro
  - examples/woc/scripts/woc_game/bin/protocol/commands.zro
  - examples/woc/contracts/woc.contracts.json
  - examples/woc/contracts/command_payloads.json
  - examples/woc/reference/command_catalog.json
  - examples/woc/reference/current-head/command_catalog.json
  - examples/woc/reference/current-head/command_payload_catalog.json
  - examples/woc/reference/current-head/command_payload_coverage.json
  - examples/woc/native/crates/woc_protocol
  - examples/woc/native/crates/woc_protocol/Cargo.toml
  - examples/woc/native/crates/woc_protocol/src/lib.rs
  - examples/woc/native/crates/woc_protocol/src/contracts.rs
  - examples/woc/native/crates/woc_protocol/src/generated.rs
  - examples/woc/native/crates/woc_protocol/src/generated_commands.rs
  - examples/woc/native/crates/woc_protocol/src/generated_command_payloads.rs
  - examples/woc/native/crates/woc_protocol/src/codec.rs
  - examples/woc/native/crates/woc_protocol/src/payload.rs
  - examples/woc/native/crates/woc_protocol/src/command_payload.rs
  - examples/woc/native/crates/woc_protocol/src/command_value.rs
  - examples/woc/native/crates/woc_protocol/src/movement_input.rs
  - examples/woc/native/crates/woc_protocol/src/error.rs
  - examples/woc/native/apps/woc_client/src/input/intent.rs
  - examples/woc/native/apps/woc_client/src/presentation/movement/tick_input.rs
  - examples/woc/native/apps/woc_client/src/presentation/frame_driver.rs
  - examples/woc/native/apps/woc_server
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
tests:
  - examples/woc/scripts/woc_game/woc_m8_movement_input_tests.zrp
  - examples/woc/native/crates/woc_protocol/tests/command_payloads.rs
  - examples/woc/native/crates/woc_protocol/tests/command_value.rs
  - examples/woc/native/crates/woc_protocol/tests/movement_input.rs
  - examples/woc/native/crates/woc_protocol/tests/protocol.rs
  - examples/woc/native/apps/woc_client/tests/input/intent.rs
  - examples/woc/native/apps/woc_client/tests/presentation/frame_driver.rs
  - examples/woc/native/apps/woc_server/tests/fixed_tick_driver.rs
  - examples/woc/native/plugins/woc_runtime/tests/transaction.rs
  - examples/woc/tools/package.json
  - examples/woc/tools/command_codegen.mjs
  - examples/woc/tools/command_payload_codegen.mjs
  - examples/woc/tools/command_payload_coverage_codegen.mjs
  - examples/woc/tools/command_payload_bank_contract_test.mjs
plan_sources:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md
  - docs/plans/optimize/zircon_runtime/15-woc-social-identity-party-raid-chat-duel-arena-matchmaking-minigame-runtime-review.md
  - docs/plans/optimize/zircon_runtime/16-woc-instance-dungeon-delve-pet-companion-lockout-reset-collision-runtime-review.md
  - docs/plans/optimize/zircon_runtime/17-woc-world-terrain-collision-locomotion-spawn-spatial-targeting-runtime-review.md
  - docs/plans/optimize/zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/Serialization/NetSerializer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/DataStream/DataStream.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/godot/core/io/marshalls.h
  - dev/godot/core/io/packet_peer.h
  - dev/godot/scene/main/multiplayer_api.h
  - dev/bevy/crates/bevy_ecs/src/message/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 19 · WOC Command Protocol、Payload Codec、Admission、Movement 与 Outcome Runtime 工程化差距

## 1. 结论

WOC 已经有一套数量可观的命令协议实现，但它还不是一个唯一、不可绕过、可版本协商的工程级 authority。当前 current-head command catalog 有 165 个连续 ID，其中 156 个 `client_send`、9 个 `dispatch_only`；typed payload contract 只登记 157 项，恰好遗漏 8 个 dispatch-only ID：76、106、107、108、109、110、111、113。脚本 `commands.zr` 用 `id < 165` 判断 known，`command_payloads.zr`只提供 kind/min/max/fixed length与局部 contract test；真正的 WorldState 仍在 68,730 行单文件中用长 `if/else` 手工解释 payload并直接变更候选世界。

native侧 `validate_command_payload` 对61种payload kind做了长度、UTF-8、枚举、finite/range与部分cross-field检查，不能将其评价为“完全没有typed validation”。问题在于这套校验不是权威入口：`Command::decode_payload`、`Command::decode_from`和`FixedTickInput::decode_payload`只验证command ID已知与单payload不超过64 KiB，不调用command-specific validator；`Command::encode_into`也允许外部直接构造的未校验`Command`进入host。只有`woc_client::ClientCommandMapper`先显式校验再要求`client_send`。同一wire payload因此会因调用路径不同而得到不同资格，server/host decode不是client mapper的安全对偶。

协议身份也发生分裂。native frame固定携`WOC1`、protocol version、32-byte core contract fingerprint、kind与payload length；它不携完整command catalog SHA、command payload SHA、WOS schema、Runtime18的ContentBuildSet或兼容矩阵。脚本`commands.fingerprintHex(...)`只返回完整SHA-256的16 hex字符前缀，还要求一组无协议语义的Fibonacci marker；`main.stateSchema()`把该64-bit前缀、payload digest和`WOS113`拼成字符串，而native identity仍声明`WOS83`。这不足以证明join、restore、replay、hot reload或client/server使用同一命令代际。

movement路径存在确定的顺序错误。native `MovementInputRelay`按actor保留ACK高水位，但明确“apply every valid packet”；较低或重复sequence仍会覆盖当前flags/facing，只是acknowledgement保持max。relay key仅是`(entity id, generation)`，没有world/session/connection/principal，也没有remove/clear生命周期接口。脚本`movement_input.zr`则只有57行scalar helper，并明确说明在container ABI完成前不接完整batch；产品fixed-tick实际走`binary.zr`解出的12组并行Array，未使用这个模块。

成本上限不是工程预算。core contract允许单frame 128 MiB、committed state 64 MiB、每tick 4,096 commands、65,536 movement frames；native和Zr两端会复制frame、state、payload或parallel columns。Zr `ByteReader`构造时复制源，`readBytes`再复制字段，command payload又被展平；`readF64LeAt`为每次读取重建reader并从头skip，手写`powerOfTwo`最坏循环约1,074次。现有bound能阻止无限长度，却没有证明在最大合法输入下CPU、allocation、latency、VM instruction、GC和fault行为合格。

资格链当前不可执行。`woc_protocol`物理有39个production `.rs`和4个test文件，但10个被`.gitignore`的源文件被`lib.rs`直接声明，clean clone缺源码；当前物理树仍有6个已记录编译错误，76个Rust tests未运行。脚本movement manifest存在但其`bin-m8-movement-input-tests`目录缺失，主bin也只有`binary.zro`和`commands.zro`，没有`command_payloads.zro`或`movement_input.zro`。默认npm check又在旧148项预期与当前157项typed contract冲突处提前停止。

本轮登记 **6项P0、68项P1和16项P2**。Runtime08E继续拥有通用transport、replication、connection security与网络driver；Runtime12拥有WorldState/schedule/save transaction；Runtime13–17拥有各domain reducer语义；Runtime18拥有ContentBuildSet；App03拥有product role、VM transaction和clean-clone产品闭环；Tooling05/10拥有generator与test scheduling。本文唯一拥有command schema/codec、decode→validate→admit→dispatch入口、movement input transport state与per-command outcome合同。

## 2. 审查范围与事实基线

### 2.1 物理清单

| 范围 | 生产文件 | 生产行数 | bytes | 测试/证据 |
|---|---:|---:|---:|---|
| Zr `src/protocol`非test-main | 4 | 2,643 | 109,929 | 1个7行test main |
| native `woc_protocol/src`物理树 | 39 | 11,713 | 378,644 | crate内另有13个`#[cfg(test)]` |
| native `woc_protocol/tests` | - | - | - | 4文件、3,432行、113,411 bytes、76个`#[test]` |
| `world/state.zr`协议消费者 | 1 | 68,730 | 3,298,740 | 同文件内大量contract test函数 |
| 主脚本protocol artifacts | 2 | - | 583,749 | `binary.zro`、`commands.zro`；另外两项缺失 |

Zr四个非测试文件分别为`binary.zr` 578行、`command_payloads.zr` 1,645行、`commands.zr` 363行、`movement_input.zr` 57行。前三者进入主产品调用图；movement helper只由独立test main消费。`world/state.zr`对`command_payloads`有66处动态import，并把protocol parsing、domain admission、dispatch和mutation交织在一个类中。

### 2.2 Catalog、Contract 与 Coverage

| 项目 | 当前事实 | 工程含义 |
|---|---|---|
| command catalog | 165项，ID 0..164连续 | 连续数组lookup可O(1)，但连续性本身不是版本合同 |
| command direction | 156 client-send / 9 dispatch-only | direction只在client mapper显式检查，host decode不区分 |
| command facets | 21种非空facet，35项无facet | facet不是完整owner/capability/admission metadata |
| payload contract | 157项、61 kinds | 8个dispatch命令无typed contract；1个dispatch命令为空payload |
| payload size | 97 fixed / 60 variable / 47 empty | 最大值仍以全局64 KiB或局部大上限为主，没有work预算 |
| generated lookup | command按数组index；payload按157项线性`iter().find` | hot admission每command多一次线性descriptor search |
| script command identity | 完整SHA仅保留16 hex前缀 | collision、诊断与兼容表达能力被无理由削弱 |

`command_payloads.json`的schema version为60，source commit为`5ef9f7...`，并绑定完整command catalog SHA。旧reference catalog只有156项，而current-head为165项；这证明catalog确实演进过，也证明“ID仍连续”不能替代兼容规则、reserved/tombstone和双端协商。

### 2.3 实际入口拓扑

```text
ClientGameplayIntent
  -> encode_intent
  -> validate_command_payload + require_client_send
  -> public Command { raw Vec<u8> }
  -> WocTransactionalRuntime::tick
  -> FixedTickInput::encode_payload (known ID + 64 KiB only)
  -> VM fixedTick
  -> Zr binary decode (known ID + 64 KiB only)
  -> WorldState::applyCommands (kind/min/max/fixed length only)
  -> hand-written domain parsing and mutation
  -> batch success OR throw/whole candidate rollback
  -> WorldSnapshot (events currently no typed catalog/outcome contract)
```

外部caller也可跳过`ClientGameplayIntent`直接构造public `Command`。server tests与runtime transaction正是以`Vec<Command>`为边界；没有一个opaque `ValidatedCommand`或`AdmittedCommand`阻止未校验payload越过层级。`VmTickError::RejectedCommand { index, reason }`只是VM错误载体，不是每项command receipt；脚本throw会把整tick视为fault，无法表达accepted/rejected/duplicate/stale/unauthorized/deferred的混合batch结果。

### 2.4 Wire、Copy 与 Budget

native outer frame头44 bytes，payload decode必定`to_vec()`；FixedTick又把command payload、committed state、offline bootstrap读成新`Vec`。transaction在每tick先clone committed state，再encode完整输入；VM输出又decode完整state。脚本binary进一步把command拆成ID/actor/generation/sequence/offset/length/bytes列，把movement拆为actor/generation/sequence、7 flags、presence与facing等并行Array。

这些结构有canonical order、长度和finite checks，是可保留的基础；但128 MiB合法frame和65,536 movement frames同时存在时，内存峰值远大于wire size，且在admission前已发生多轮allocation/copy。工程化重构应先建立view/arena/borrowed decode、aggregate work budget和真实最大合法输入基准，而不是只调小常量掩盖模型问题。

### 2.5 静态矛盾与不可执行证据

| 证据 | 结果 |
|---|---|
| native source completeness | 39个physical source中仅29个tracked；10个required modules被`examples/*`规则忽略 |
| native compile | 当前物理树仍有6个编译错误；其中generated enum缺`WeaponSkinChange`但descriptor/validator引用它 |
| Rust tests | 4集成文件76 tests，因compile失败0执行 |
| Zr artifacts | movement test binary目录缺失；主bin缺payload/movement artifacts |
| npm default check | 旧脚本期待148项，当前typed contract为157项，后续21步未执行 |
| product server | `woc_server`只有测试展示fixed-tick driver消费；没有完成的authoritative网络产品证据 |

本轮不重跑未变化的失败命令，也不把source-shape check、test数量或生成文件存在记成runtime pass。

## 3. P0 阻断

| ID | 差距 | 当前证据 | 必须重构为 |
|---|---|---|---|
| PROTOCOL-P0-001 | 协议源码与artifact不是clean-clone可重建产品 | 10个`lib.rs`必需模块被ignore；native仍6 compile errors；两个Zr artifact缺失 | tracked/generated ownership唯一化；clean clone先生成再编译，产物绑定source/tool/schema并原子发布 |
| PROTOCOL-P0-002 | 没有唯一不可绕过的typed decode→validate→admit入口 | native Command/FixedTick decode只查known ID+64 KiB；public raw Command可直构；Zr只查长度后手工解析 | opaque `DecodedCommand`→`ValidatedCommand`→`AdmittedCommand` typestate；所有client/server/replay/bot/VM路径共用registry，raw command不可dispatch |
| PROTOCOL-P0-003 | 命令、payload、world、content与wire身份不能证明同代 | 165/157集合分裂；script只用64-bit SHA前缀；WOS83/113；frame不携完整组合身份 | `ProtocolBuildIdentity`完整绑定core schema、catalog、payload、event/outcome、world、content、schedule和producer；handshake/restore/replay fail-closed |
| PROTOCOL-P0-004 | 准入、authority、dispatch与结果回执被WorldState长分支和batch throw合并 | 68K行state手工解析；client-only direction不在host验证；任一throw回滚整batch | 独立AdmissionRuntime验证principal/session/world/actor/sequence/capability/budget；typed dispatch；每command durable outcome与明确batch atomicity |
| PROTOCOL-P0-005 | movement顺序与生命周期可接受旧包覆盖新输入 | relay对lower/duplicate sequence仍Applied并覆盖flags；key无world/session；无remove/clear；Zr helper未接产品 | connection/world scoped relay；严格newer/duplicate/stale disposition、ack/NAK、despawn/disconnect teardown、loss/reorder/reconnect测试 |
| PROTOCOL-P0-006 | 最大合法输入成本与资格链均未证明 | 128 MiB frame、64 MiB state、4,096 commands、65,536 movement；多轮copy；Rust/npm/Zr lanes均阻断 | admission前aggregate bytes/items/work/depth预算；borrowed/arena codec；真实上限profile/fuzz/soak；required lane无compile/missing/not-run假绿 |

## 4. P1 工程化差距

### 4.1 Schema、Catalog 与 Compatibility

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-001 | 165个ID只有name/kind/facet | descriptor补owner、payload type/version、direction、authority、reliability、ordering、atomicity、rate class、outcome/event contract |
| PROTOCOL-P1-002 | 35个command无facet | 所有command必须显式归domain或protocol-internal；unowned command阻断catalog发布 |
| PROTOCOL-P1-003 | 8个dispatch-only无typed payload contract | dispatch command同样进入唯一schema；若无wire payload则显式`Empty`，不能靠coverage例外 |
| PROTOCOL-P1-004 | `targetNearest`空payload是唯一已映射dispatch项但无调用角色说明 | 明确server-generated、client-visible、replayable与actor authority，或从wire catalog移到internal operation |
| PROTOCOL-P1-005 | ID连续性被当作known判定 | generated registry验证ID、reserved range、tombstone和alias；unknown/newer ID按协商策略处理 |
| PROTOCOL-P1-006 | command rename/move/delete无生命周期字段 | descriptor记录introduced/deprecated/removed版本和migration；ID永不静默复用 |
| PROTOCOL-P1-007 | script command fingerprint截断到64-bit | 全链保留完整256-bit digest；若热路径需短ID，短ID仅作索引并始终由完整identity确认 |
| PROTOCOL-P1-008 | fingerprint API依赖Fibonacci marker | 删除无语义marker；读取identity不应靠magic call shape，兼容条件由typed manifest表达 |
| PROTOCOL-P1-009 | native core fingerprint不包含command/payload catalogs | outer frame或session handshake绑定完整组合identity；每frame可用negotiated generation ID避免重复大header |
| PROTOCOL-P1-010 | WOS83与WOS113可同时被标为current | 一个build只能发布一个world schema generation；旧代以migration reader存在，不进入current identity |
| PROTOCOL-P1-011 | payload schema version 60没有兼容矩阵 | 每kind/command记录wire version、forward/backward policy、default/unknown-field规则与golden bytes |
| PROTOCOL-P1-012 | reference、current-head、generated、contract没有单一发布receipt | codegen产出canonical ProtocolBuild manifest，校验四者digest与集合完全一致后才可install |

### 4.2 Typed Codec 与 Validation

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-013 | `Command`公开raw `Vec<u8>`且字段可任意构造 | raw wire type保持私有；构造只能经typed encoder或trusted decoded typestate |
| PROTOCOL-P1-014 | encode只查known ID和全局max | encode必须调用同一command validator并验证direction/actor/sequence基本不变量 |
| PROTOCOL-P1-015 | decode不调用`validate_command_payload` | registry decode完成length、shape、semantic和trailing-byte检查后才返回typed value |
| PROTOCOL-P1-016 | Zr只暴露numeric kind与长度 | 生成typed payload DTO/visitor或prepared field view，不把语义解析留给WorldState分支 |
| PROTOCOL-P1-017 | native 61-kind match与39个手写模块并存 | schema生成descriptor/dispatch table；handwritten validator通过registered hook扩展且completeness静态证明 |
| PROTOCOL-P1-018 | generated enum已与descriptor drift | generator原子生成enum、descriptor、Rust/Zr codecs、tests与manifest；编译前做exhaustiveness check |
| PROTOCOL-P1-019 | payload descriptor lookup线性扫描157项 | 连续ID使用checked direct table；稀疏代使用perfect hash/sorted binary lookup并基准 |
| PROTOCOL-P1-020 | fixed-size基础kind只检查length | 对slot/index/count/signed value/EntityRef按command-specific range、nonzero、generation与unit验证 |
| PROTOCOL-P1-021 | UTF-8规则分散在helper与domain modules | schema统一byte/codepoint/UTF-16 limits、normalization、control character、locale与canonical encoding策略 |
| PROTOCOL-P1-022 | f64只在部分kind检查finite | 所有float字段生成finite/range/unit/quantization规则；位置、角度与比例不共享裸f64政策 |
| PROTOCOL-P1-023 | trailing-byte规则依赖各validator主动调用 | codec框架默认要求exact consumption；只有显式versioned extension区允许剩余字段 |
| PROTOCOL-P1-024 | `CommandValue`是第二套无人消费的通用tree codec | 明确迁为schema IR/tool fixture或删除；不得与product payload形成双authority |
| PROTOCOL-P1-025 | Event仅有任意u16 ID与raw payload | 建立event catalog、typed codec、sequence/ordering/delivery/visibility/retention schema |
| PROTOCOL-P1-026 | ProtocolError大量把domain context压成string | typed error携stage、command、field/path、offset、expected/actual、source generation，展示层再本地化 |

### 4.3 Admission、Authority、Ordering 与 Outcome

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-027 | direction只由client mapper检查 | server/host在admission重新验证ClientSend/DispatchOnly/Internal，禁止信任producer |
| PROTOCOL-P1-028 | command没有connection/session/principal identity | transport context以不可伪造capability传给admission；wire actor不能代替认证principal |
| PROTOCOL-P1-029 | actor authority只查world中player kind | 显式principal→controlled entity lease、world generation、ownership/possession和role policy |
| PROTOCOL-P1-030 | sequence只在WorldState按actor比较 | sequence window在mutation前处理wrap、duplicate、gap、replay、reconnect与per-channel ordering |
| PROTOCOL-P1-031 | sequence 0在native Command层可进入 | catalog/admission统一规定initial value与exhaustion；client mapper和server validator共用规则 |
| PROTOCOL-P1-032 | mixed actor batch的atomicity只靠注释 | batch schema声明AllOrNothing/PerCommand/OrderedGroups，host按合同提交并生成receipt |
| PROTOCOL-P1-033 | unsupported reducer在VM中throw | capability negotiation先拒绝未安装handler；运行中返回typed Unsupported而非session fault |
| PROTOCOL-P1-034 | invalid single command可暂停/fault整个runtime | malformed/unauthorized/rate-limited属于command outcome；只有协议完整性或VM invariant才升级session fault |
| PROTOCOL-P1-035 | temporal suppression等domain策略直接夹在dispatch ladder | admission policy、domain precondition和reducer分层；domain报告拥有具体规则，协议层只拥有调用合同 |
| PROTOCOL-P1-036 | command没有deadline/target tick | 明确issued/target/expiry tick、late policy与clock domain；不可用wall-time替代simulation order |
| PROTOCOL-P1-037 | 没有per-command cost class | descriptor声明base/size-dependent work、allocation、fanout、rate class；aggregate budget在decode前预留 |
| PROTOCOL-P1-038 | 没有idempotency/dedup key | 对重试型事务定义request ID、dedup retention和same-key/different-payload冲突政策 |
| PROTOCOL-P1-039 | `VmTickError::RejectedCommand`只有index+string | outcome携command identity、sequence、status、reason code、retryability、authoritative tick和correlation ID |
| PROTOCOL-P1-040 | WorldSnapshot events不能证明command处理结果 | 单独CommandOutcome stream或typed events覆盖accepted/rejected/deferred/duplicate；client reconciliation按receipt推进 |

### 4.4 Framing、Movement 与 Transport Handoff

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-041 | outer Frame与NetworkEnvelope形成嵌套双header | 定义每层职责与唯一顺序/ack owner，禁止两个互不关联的version/kind/length模型 |
| PROTOCOL-P1-042 | frame无session/build generation | handshake生成negotiated protocol/session IDs；frame引用它们并在reconnect/rollover时失效 |
| PROTOCOL-P1-043 | frame无integrity/auth/compression metadata | 由Runtime08E transport提供认证、完整性、压缩与anti-replay；protocol明确authenticated context是admission前置 |
| PROTOCOL-P1-044 | decode_frame总复制payload | 提供bounded borrowed frame view；跨async/lifetime时由显式arena/owned promotion控制 |
| PROTOCOL-P1-045 | Reader对每个bytes字段分配Vec | 支持slice/subreader和零拷贝validated view；只有需长期持有的字段复制 |
| PROTOCOL-P1-046 | FixedTick同时承载state、commands、movement、bootstrap | 分离session bootstrap、state checkpoint与per-tick input；用明确transaction composition而非巨型opaque payload |
| PROTOCOL-P1-047 | 65,536 movement/tick与simulation对象规模无关 | 按connected actors、packet bytes、work和per-principal rate收紧aggregate admission；异常fanout typed拒绝 |
| PROTOCOL-P1-048 | relay接受lower/duplicate sequence并覆盖flags | 只应用严格newer；duplicate回ACK不重放，older/stale返回明确disposition且不变更retained input |
| PROTOCOL-P1-049 | relay key缺world/session/connection | key包含authoritative world generation与control lease；connection teardown撤销所有输入 |
| PROTOCOL-P1-050 | relay无despawn/remove/reset API | entity despawn、world unload、role transfer、teleport/reset、disconnect均有幂等清理路径 |
| PROTOCOL-P1-051 | stale阈值固定15 tick且只清held flags | threshold进入versioned movement profile；loss/latency/reconnect下明确facing、jump edge、held/impulse状态 |
| PROTOCOL-P1-052 | Zr scalar movement helper未接产品 | 要么接入共享generated codec/admission，要么删除并由native-authoritative adapter拥有；测试不能代表产品集成 |

### 4.5 Performance、Memory 与 Failure Containment

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-053 | 128 MiB frame只受byte max约束 | 另加commands/events/fields/nesting/string/codepoint/work/allocation/time预算并在allocation前检查 |
| PROTOCOL-P1-054 | transaction每tickclone完整64 MiB committed state | 使用generation-qualified immutable snapshot/COW/page delta或VM-owned handle，避免固定全量往返 |
| PROTOCOL-P1-055 | Zr command payload展平后再按offset解析 | decoded batch使用validated slices/typed views；offset table只作wire index，不反复复制 |
| PROTOCOL-P1-056 | movement使用十余个parallel dynamic Array | 采用AoS/SoA均可，但必须一次分配、明确layout/alignment/cache合同并防列长度失配 |
| PROTOCOL-P1-057 | `readF64LeAt`每字段复制/skip | 提供checked random-access primitive或单次cursor decode；不得按offset重建全buffer reader |
| PROTOCOL-P1-058 | `powerOfTwo`最坏循环约1,074次/float | VM提供经过验证的bitcast/IEEE codec或有界table/intrinsic；用合法极值输入测instruction成本 |
| PROTOCOL-P1-059 | error string与payload copy可放大恶意输入成本 | error detail限长、结构化且不回显secret/raw payload；失败路径同样受allocation/log rate预算 |
| PROTOCOL-P1-060 | 没有协议级perf corpus | 建立small/typical/max-valid/adversarial corpus，报告decode/validate/admit/dispatch的p50/p95/p99、alloc、RSS与VM instructions |

### 4.6 Build、Evidence 与 Migration

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P1-061 | 10个required Rust源文件未tracked | 修正ignore/生成策略；clean clone inventory必须与`mod`声明完全一致 |
| PROTOCOL-P1-062 | generator可生成互相不编译的enum/descriptor/modules | 单次IR驱动所有target，临时目录生成→compile/check→digest→atomic promote |
| PROTOCOL-P1-063 | 148/157硬编码散落在检查脚本 | 从manifest读取expected set；任何count变化输出added/removed/changed diff，不维护手写数字 |
| PROTOCOL-P1-064 | default npm lane首错后不知后续21步状态 | runner输出每step planned/ran/pass/fail/blocked/skip及原因，整体不可把blocked当pass |
| PROTOCOL-P1-065 | movement manifest存在而binary缺失 | required test artifact由build graph声明producer与digest；missing在执行前即fail |
| PROTOCOL-P1-066 | Rust测试只证明physical tree行为 | 增加clean clone/codegen/compile lane、cross-language golden/fuzz、product host和network loss/reorder lane |
| PROTOCOL-P1-067 | catalog evolution无migration fixture | 保存历史build identities与golden packets；测试supported upgrade、explicit reject、tombstone和rollback |
| PROTOCOL-P1-068 | finding修复可能只补一处validator调用 | acceptance从wire ingress跨native/Zr/WorldState/outcome全链证明，禁止source-shape字符串检查作为唯一证据 |

## 5. P2 完整性与可维护性

| ID | 差距 | 重构要求 |
|---|---|---|
| PROTOCOL-P2-001 | command命名混用camelCase/snake_case/缩写 | canonical wire name、display name与legacy alias分开生成 |
| PROTOCOL-P2-002 | numeric payload kind泄漏到脚本业务 | 生成named enum/typed descriptor，调试输出稳定名称 |
| PROTOCOL-P2-003 | descriptor缺文档链接与owner contact | manifest记录schema source、owner模块、runbook和deprecation说明 |
| PROTOCOL-P2-004 | magic marker参数污染public API | 删除marker，错误注入使用专用test seam |
| PROTOCOL-P2-005 | protocol constants散落Rust/Zr/JSON | canonical IR生成语言投影并做roundtrip digest |
| PROTOCOL-P2-006 | large validator match难定位覆盖 | 按kind family模块化，registry保持唯一入口和exhaustive table |
| PROTOCOL-P2-007 | error display与stable reason code未分离 | stable machine code、structured fields与localized presentation分层 |
| PROTOCOL-P2-008 | tests重复手写Command fixture | typed fixture builder从catalog生成valid baseline，再局部mutate测试失败 |
| PROTOCOL-P2-009 | 没有packet pretty-printer/redaction | 开发工具显示版本、字段、offset与outcome，默认隐藏chat/account/telemetry内容 |
| PROTOCOL-P2-010 | 缺schema diff可读报告 | CI产出added/removed/renamed/size/direction/compatibility diff |
| PROTOCOL-P2-011 | 缺Wireshark/trace等离线decode版本策略 | decoder按BuildIdentity选择schema，未知代只显示header/hex且不猜测 |
| PROTOCOL-P2-012 | movement dispositions缺指标聚合 | 统计applied/duplicate/stale/unauthorized/rate-limited与ack lag并绑定world/session generation |
| PROTOCOL-P2-013 | command latency没有阶段分解 | trace decode/validate/admit/queue/dispatch/commit/outcome各阶段并共享correlation ID |
| PROTOCOL-P2-014 | payload大小没有分位数观测 | 按command/kind记录bounded histogram，避免高基数name/payload标签 |
| PROTOCOL-P2-015 | protocol文档与generated source分离 | 从IR生成wire table、compat matrix与golden示例，手写文档只解释策略 |
| PROTOCOL-P2-016 | 没有协议ownership lint | lint禁止domain直接解析raw bytes、禁止public raw Command构造、禁止绕过AdmissionRuntime |

## 6. 目标架构与唯一 Owner

```text
ProtocolSchemaIR
  -> ProtocolBuildManifest
  -> generated Rust/Zr descriptors + typed codecs + golden corpus

AuthenticatedTransportContext + BorrowedFrameView
  -> CommandBatchDecoder
  -> CommandCodecRegistry
  -> ValidatedCommandBatch
  -> CommandAdmissionRuntime
       principal/control lease
       build/world generation
       direction/capability
       sequence/dedup/deadline
       aggregate budget
  -> AdmittedCommandBatch
  -> CommandDispatchRegistry
  -> domain reducers (Runtime13-17)
  -> CommandOutcomeJournal
  -> client reconciliation / replay / diagnostics

MovementFrameCodec
  -> MovementInputRuntime(world/session/connection scoped)
  -> typed dispositions + ACK/NAK
  -> locomotion consumer (Runtime17)
```

| Owner | 唯一职责 | 不得拥有 |
|---|---|---|
| `ProtocolSchemaRegistry` | 安装/协商完整BuildIdentity与schema generation | generator scheduling、domain reducer |
| `CommandCodecRegistry` | typed encode/decode/validate与descriptor lookup | principal authority、world mutation |
| `CommandBatchDecoder` | bounded borrowed framing、canonical batch与aggregate parse budget | gameplay policy |
| `CommandAdmissionRuntime` | principal、control lease、direction、sequence、deadline、rate/work、capability准入 | domain效果实现 |
| `CommandDispatchRegistry` | typed handler registration、generation lease与dispatch topology | catalog生成、网络transport |
| `CommandOutcomeJournal` | per-command terminal disposition、correlation、retention/replay | UI toast或domain event语义 |
| `MovementInputRuntime` | scoped sequence/ack/stale/teardown与retained input | position、collision、speed、combat |
| `ProtocolEvidenceRunner` | cross-language golden/fuzz/load/compat product qualification | 修改expected结果以制造pass |

## 7. 参考引擎约束

| 参考 | 本轮核对的结构事实 | 对WOC的约束 | 不外推 |
|---|---|---|---|
| Unreal Iris `FNetSerializer` | serializer显式version/config/traits，分Serialize/Deserialize/Delta/Quantize/Dequantize/Validate及dynamic-state生命周期 | schema/validation/version/lifetime必须是第一等合同，不能只有raw bytes加长分支 | 不宣称复制Iris API即可获得Unreal性能 |
| Unreal Iris `UDataStream` | BeginWrite/WriteData/ReadData与per-write record；delivery有ACK/NAK/close回调 | send record、delivery disposition与retirement必须可追踪，outcome不能只有batch throw | 不把transport ACK等同gameplay command成功 |
| Godot marshalls/PacketPeer/MultiplayerAPI | decode有结构/深度约束；packet有大小/lifetime；multiplayer显式peer identity、authority mode与object configuration | bounded decode、peer/principal、authority和object/world配置要在dispatch前交接 | 不复制Godot Variant或RPC模型作为唯一wire格式 |
| Bevy ECS message | message ID world-scoped且reader有独立cursor；源码明确它不是network protocol | 本地dispatch cursor与网络sequence应分层且带world scope | 不把Bevy message直接冒充认证网络协议 |
| Fyrox Visitor | versioned tree、typed fields与稳定版本规则 | save/asset migration可借鉴typed versioning，但command需要更严格实时admission/outcome | 不用Visitor替代低延迟命令codec |
| Unity RenderGraph registry | handle带version/write count，resource有create/import/release lifecycle | 只借鉴generation-qualified handle和显式retirement的结构约束 | Unity Graphics镜像不提供完整multiplayer参考 |

共同点不是某个引擎“类更多”，而是wire value、version、owner、identity、validation、delivery、lifetime和failure disposition彼此可区分。Zircon可以选更紧凑的布局和更高效的codec，但不能通过省略准入、回执或兼容语义声称性能领先。

## 8. 重构里程碑

### M0 · Truth Freeze 与 Source Completeness

- 冻结165/157/9/8集合、WOS83/113和current digests，生成machine-readable差异；
- 修正tracked/generated ownership，使clean clone可生成并编译`woc_protocol`；
- 所有missing/not-run/blocked lane进入receipt，不修改production行为。

### M1 · Canonical Schema 与 Build Identity

- 以单一IR生成command/event/outcome/payload descriptors、Rust/Zr codecs和完整ProtocolBuildIdentity；
- 引入reserved/tombstone/version/compatibility规则；
- handshake、save、replay和trace绑定Runtime18 ContentBuildSet与Runtime12 world/schedule identity。

### M2 · Typed Codec Hard Cutover

- raw `Command`迁为私有wire view，所有构造和decode必须生成Validated typestate；
- native/Zr共享golden bytes、negative corpus和exact-consumption规则；
- 删除或收编未消费`CommandValue`与scalar movement第二authority。

### M3 · Admission、Dispatch 与 Outcome

- 建立principal/control lease、direction、sequence/dedup/deadline、capability与aggregate budget准入；
- WorldState只接typed admitted commands，domain reducer迁出raw payload parsing；
- per-command outcome替代index+string和普通command导致的session fault。

### M4 · Movement、Frame 与 Lifetime

- relay按world/session/connection/entity generation隔离，严格newer并具teardown；
- fixed tick拆分bootstrap/checkpoint/input职责；
- borrowed/arena codec减少frame/state/payload多轮copy。

### M5 · Compatibility、Migration 与 Product Integration

- 完成client/server/bot/replay统一入口和supported-version矩阵；
- historical packet/build fixture验证upgrade/reject/rollback；
- App03真实offline/client/server host接入相同receipt与diagnostics。

### M6 · 性能、故障与竞争资格

- 在真实small/typical/max/adversarial corpus上测native与Zr各阶段；
- fuzz、loss/reorder/duplicate/reconnect、OOM、deadline、malformed与soak同时通过；
- 只有correctness与failure gates先通过，才能对比Unreal等引擎的协议CPU/内存/延迟。

## 9. 验收门

| Gate | 验收内容 |
|---|---|
| PROTOCOL-G01 | clean clone生成/编译全部Rust/Zr protocol sources，required source/artifact 0 missing、0 ignored |
| PROTOCOL-G02 | command、payload、event、outcome catalog集合闭包；每个ID有owner/direction/schema/version/handler或显式unsupported |
| PROTOCOL-G03 | Rust/Zr/JSON/manifest完整SHA与BuildIdentity逐字节一致，不使用截断digest作为authority |
| PROTOCOL-G04 | 所有ingress只能产出Validated/Admitted typestate；public raw command无法dispatch |
| PROTOCOL-G05 | 165个command逐项valid/invalid/trailing/size/direction golden，8个现有unmapped项不再例外 |
| PROTOCOL-G06 | malformed、unauthorized、unsupported、duplicate、stale与rate-limit产生typed outcome，不误报session fault |
| PROTOCOL-G07 | sequence wrap/gap/replay/reconnect与batch atomicity由明确矩阵和property tests证明 |
| PROTOCOL-G08 | movement lower/duplicate不覆盖新输入；despawn/world unload/disconnect后0 retained state |
| PROTOCOL-G09 | handshake/save/replay拒绝不兼容core/catalog/payload/world/content/schedule组合，并能说明差异 |
| PROTOCOL-G10 | max-valid与malformed输入在allocation前受aggregate bytes/items/work/depth/deadline预算 |
| PROTOCOL-G11 | borrowed/arena codec的copy/allocation峰值有基准，64 MiB state不再每tick固定多轮clone |
| PROTOCOL-G12 | event/outcome有typed catalog、ordering、digest、retention和client reconciliation验证 |
| PROTOCOL-G13 | cross-language golden、differential、fuzz与historical migration corpus全部绑定source/build identity |
| PROTOCOL-G14 | client、server、bot、replay、offline host均通过同一codec/admission/dispatch/outcome入口 |
| PROTOCOL-G15 | required Rust/npm/Zr/product tests全部实际运行；compile/missing/blocked/skip不能汇总为pass |
| PROTOCOL-G16 | correctness、fault、soak、RSS/allocation、native/Zr CPU/VM instruction与p50/p95/p99报告通过后才允许竞争结论 |

## 10. 边界与依赖

| 相邻报告 | 该报告拥有 | 本文消费/交付 |
|---|---|---|
| Runtime08E Network | connection、transport、replication、security、congestion | 接收authenticated transport context；返回frame/delivery requirements，不重做NetDriver |
| Runtime12 WOC Kernel | fixed schedule、candidate/commit、save/world schema | 交付admitted batch与outcome；不拥有world serialization |
| Runtime13-17 Domains | combat/progression/social/instance/world reducer语义 | 提供typed handler接口；不复制具体玩法finding |
| Runtime18 Content | ContentBuildSet、catalog install/query generation | ProtocolBuildIdentity引用content root；不生成内容 |
| App03 WOC Product | role、process、VM transaction、client/server host | host必须走唯一入口并展示receipt；本文不拥有窗口/进程 |
| Tooling05 | source extract、codegen build graph、artifact publish | 提供canonical schema/manifest要求；不拥有脚本调度实现 |
| Tooling10 | test inventory、runner、result completeness | 提供protocol required lanes/gates；不拥有全仓测试平台 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zr protocol物理扫描 | review_complete | 2026-08-16 | 5文件含test main；非测试2,643行/109,929 bytes，movement helper产品断线 |
| native protocol物理扫描 | review_complete | 2026-08-16 | 39 production source、4 integration tests；10 required sources ignored |
| catalog/payload集合核对 | review_complete | 2026-08-16 | 165 commands、157 payloads、8 unmapped dispatch IDs、61 payload kinds |
| ingress/admission拓扑核对 | review_complete | 2026-08-16 | client mapper校验；Command/FixedTick decode绕过command-specific validator |
| movement与成本核对 | review_complete | 2026-08-16 | lower sequence仍覆盖；128 MiB/64 MiB/4,096/65,536 bounds与多轮copy |
| 动态验证 | blocked_by_existing_build_failures | 2026-08-16 | native 6 compile errors；npm 148/157冲突；movement binary缺失；0 Rust tests运行 |
| Production重构 | pending | - | 本篇仅review与refactor plan，未修改production/tests/manifests/artifacts |
