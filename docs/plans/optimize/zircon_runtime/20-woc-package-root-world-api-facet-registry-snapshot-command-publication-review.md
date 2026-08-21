---
related_code:
  - examples/woc/scripts/woc_game/plugin.toml
  - examples/woc/scripts/woc_game/woc_game.zrp
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/identity.zr
  - examples/woc/scripts/woc_game/src/world_api
  - examples/woc/scripts/woc_game/src/world_api/card_minigame.zr
  - examples/woc/scripts/woc_game/src/social/card_duel_service.zr
  - examples/woc/scripts/woc_game/src/protocol/commands.zr
  - examples/woc/scripts/woc_game/src/protocol/command_payloads.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/bin
  - examples/woc/scripts/woc_game/bin/.zr_cli_manifest
  - examples/woc/scripts/woc_game/bin/main.zro
  - examples/woc/contracts/command_payloads.json
  - examples/woc/reference/current-head/world_api_catalog.json
  - examples/woc/reference/current-head/command_catalog.json
  - examples/woc/reference/current-head/command_payload_catalog.json
  - examples/woc/reference/current-head/command_payload_coverage.json
  - examples/woc/reference/current-head/card_duel_command_contract.json
  - examples/woc/reference/current-head/delta_from_7c10.json
  - examples/woc/reference/current-head/source_manifest.json
  - examples/woc/native/crates/woc_contract_codegen/Cargo.toml
  - examples/woc/native/crates/woc_contract_codegen/src/lib.rs
  - examples/woc/native/crates/woc_protocol/src/generated_commands.rs
  - examples/woc/tools/reference_inventory.mjs
  - examples/woc/tools/reference_delta.mjs
  - examples/woc/tools/card_duel_command_codegen.mjs
tests:
  - examples/woc/scripts/woc_game/src/world_api/card_minigame_test_main.zr
  - examples/woc/scripts/woc_game/woc_card_minigame_view_tests.zrp
  - examples/woc/native/crates/woc_contract_codegen/tests/reference_inventory.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/15-woc-social-identity-party-raid-chat-duel-arena-matchmaking-minigame-runtime-review.md
  - docs/plans/optimize/zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/WorldSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/godot/core/object/class_db.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/multiplayer_api.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 20 · WOC Package Root、World API Facet Registry、Snapshot、Command Publication 工程化差距

## 1. 结论

当前WOC已经拥有一份结构化的`world_api_catalog.json`：schema version为1，固定参考commit为`5ef9f7...`，声明28个facet与248个member，其中181个method、67个data member；ownership分布为186个simulation、56个service、6个presentation。catalog内部的facet数量、member数量、member→facet引用与ownership目前自洽。这份reference inventory适合作为重建范围输入，但它还不是World API schema、实现注册表、运行时capability或产品发布证明。

最直接的物理差距是：28个facet分别声明28个`woc_owner`路径，仓库只有`world_api/card_minigame.zr`一个路径存在，另外27个owner文件均不存在。唯一存在的Card Minigame文件也没有实现catalog要求的`cardMinigameInfo`、`joinCardDuelQueue`、`leaveCardDuelQueue`、`playCardInDuel`、`forfeitCardDuel`这5个公开成员；它实际导出3个DTO/view class和5个projection helper。四个command已经在ID 90..93的command catalog、payload contract和WorldState reducer链中出现，所以缺口不是“没有Card Duel规则”，而是没有把authoritative service安全地发布为typed world facade。

现有Rust validator只反序列化world member的`facet/name/kind/ownership_class`和facet的`name/ownership_class`。它会忽略JSON中的`signature`、`source_owner`、`woc_owner`与`member_count`，不会检查owner文件存在、声明签名、实现可达性、导出artifact、capability注册或catalog与BuildSet同代。它按裸member name做全局唯一性检查，而不是明确的`(facet, member)`标识。测试证明参考文件计数与hash没有漂移，却不能证明任何World API可以被client、UI、server或replay消费。

唯一projection还把`CardDuelService`、任意`pid`和调用者提供的`opponentName`作为public helper参数。它没有authenticated principal、session/world identity、snapshot generation或visibility capability；如果这类helper被直接发布，caller可选择任意player ID读取其手牌。`buildView`又把domain DTO与presentation state混在同一owner里，并把所有“非等待中”的卡标为playable，没有基于turn、match generation、command eligibility或server outcome的约束。这不是可跨客户端边界的安全snapshot。

根产品也没有发布该API。`main.zr`正常lifecycle不import `world_api/card_minigame`；只有独立test main引用它。对应`.zrp`声明`bin-card-minigame-view-tests`，但该binary目录不存在。根`bin/.zr_cli_manifest`登记6个module和18个zro/zri/AOT-C路径，实际只有6个zro存在，12个zri/AOT-C路径缺失；manifest还漏掉源代码已经使用的`protocol/command_payloads`、物理存在的`kernel/rng.zro`和World API模块。当前产物无法证明API与package schema、command、world state处于同一生成代际。

Runtime15继续拥有Card Duel的queue/match/rules/authority语义；Runtime19拥有command/payload/admission/outcome；Runtime12拥有package lifecycle、WorldState与save/restore；App03拥有VM/host/root product transaction；Tooling05拥有通用代码生成与artifact pipeline。本文只拥有World API schema/facet注册、principal-scoped immutable snapshot、typed command facade、publication generation与API artifact readiness。本轮登记 **5项P0、68项P1和16项P2**。

## 2. 审查边界与物理事实

### 2.1 文件级清单

| 文件/范围 | 行数 | bytes | 当前作用 |
|---|---:|---:|---|
| `src/main.zr` | 160 | 5,212 | package lifecycle与WorldState入口；正常路径不注册World API |
| `src/identity.zr` | 7 | 106 | package名常量 |
| `src/world_api/card_minigame.zr` | 174 | 5,702 | 唯一production World API owner；实际是DTO/view projection helper |
| `src/world_api/card_minigame_test_main.zr` | 79 | 3,414 | 唯一consumer；独立自测入口 |
| `plugin.toml` | 25 | 410 | `interp` execution mode；没有facet capability/export描述 |
| 两个`.zrp` | 12 | 238 | root与Card view test入口声明 |
| `bin/.zr_cli_manifest` | 59 | 2,151 | 6个module的本机绝对路径manifest |
| `world_api_catalog.json` | 2,436 | 85,354 | 28 facet / 248 member参考重建目录 |
| `woc_contract_codegen/src/lib.rs` | 699 | 20,851 | reference inventory validator；不生成World API |
| `reference_inventory.rs` | 205 | 7,255 | 7个inventory tests；不执行Zr实现/发布验证 |

`src/world_api`物理上只有两个`.zr`文件：一个production helper和一个test main。production根入口有29个`%import`表达式，但大部分只存在于`lifecycleSelfTest()`；normal activate/fixedTick/save/restore路径没有World API registry或projection publication。全仓库对`world_api/card_minigame`的唯一Zr consumer是test main。

### 2.2 Catalog 与物理实现差额

| 维度 | Catalog声明 | 当前物理事实 | 差额 |
|---|---:|---:|---:|
| facet | 28 | 1个owner路径存在 | 27缺失 |
| member | 248 | 0个按catalog签名实现/导出 | 248未证明 |
| Card Minigame member | 5 | 0个同名public surface | 5缺失 |
| Card command | 4 | command/payload/reducer已有 | 缺typed facade与receipt绑定 |
| ownership class | 248项字符串 | 无类型/模块/运行时强制 | 全部仅信息性 |

缺失的owner覆盖bank、chat、combat、cosmetics、daily rewards、deeds、delves、duel arena、dungeon finder、dungeons、entity roster、interaction、inventory、loot、mail、market、party、pet、professions、progression、quests、social graph、talents、targeting、telemetry、trade与Vale Cup。不能为满足文件数而生成27个空wrapper；facet只有在domain owner、snapshot projection、command port、权限、artifact和验证门全部materialized后才能宣告capability。

### 2.3 Validator 实际保证

`validate_reference_root()`当前对World API只保证：JSON可读；facet数等于28；facet name唯一；member裸name唯一；member引用已知facet；member ownership与facet ownership相同；总member/method/data计数等于248/181/67；reference source manifest身份有效。它不保证：

1. `member_count`等于facet实际member数。
2. `signature`可解析、稳定、唯一或与实现兼容。
3. `source_owner`与`woc_owner`路径存在或属于正确层级。
4. catalog member在Zr/Rust/native/client中生成或实现。
5. data与method使用正确的read/write、principal与authority语义。
6. ownership class形成依赖、链接、线程、VM或网络边界。
7. artifact manifest包含facet，或consumer能按BuildSet解析它。
8. current与old schema有兼容、tombstone、migration或协商规则。

因此测试通过最多表示“pinned reconstruction input没有变”，不能进入API readiness、产品功能完成率或运行时parity统计。

### 2.4 Root Artifact 事实

`bin`共有8个文件、1,382,792 bytes：manifest以及`main`、`identity`、`generated/contracts`、`kernel/rng`、`protocol/binary`、`protocol/commands`、`world/state`七个zro。manifest只登记其中六个module，漏登`kernel/rng`；每个登记module声明zro、zri与AOT C三个产物，所以18个路径中只有6个zro存在，12个zri/AOT C文件缺失。manifest保存的是当前工作机绝对路径，也不是可搬运的package-relative artifact index。

`main.zr::stateSchema()`已经import `protocol/command_payloads`并写入payload fingerprint，但manifest的main imports只有container、world/state、generated/contracts、protocol/commands、identity。World API从源码依赖、manifest module、zro集合和package identity中全部缺席。这里不把通用编译器/生成器问题重复记为Runtime P0；本文只要求World API readiness必须消费Tooling05产生的完整、同代、可验证artifact receipt。

### 2.5 Card Projection 安全与正确性

`infoFromService(state, pid, opponentName)`允许caller提供整个mutable service、任意pid与显示名。它循环读取该pid的hand、opponent、deck/discard、round与waiting state，然后返回包含私有手牌的DTO。没有证据证明调用者就是pid对应principal，也没有snapshot fence防止同一次构建跨越service mutation。

`matchInfo()`只拒绝零opponent、负count与超范围card value；不限制hand长度、count上界、字符串大小/规范化或总projection work。`copyHand()`和`infoFromService()`都无预算复制动态数组。`CardDuelViewModel.state`用1..4裸整数；`playable = !waitingOnOpponent`把命令资格简化成一个UI布尔值，无法表达不是当前turn、match已结束、card不在authoritative hand、command cooldown、generation过期或准入被拒绝。

## 3. 参考引擎给出的边界

- Unreal `UWorld`有独立World identity、context与lifecycle；`UWorldSubsystem`通过collection完成initialize/deinitialize并可按world/tick能力筛选。其可借鉴点不是复制UObject层次，而是facet/provider必须绑定明确world instance、生命周期和可查询注册表，不能靠模块名推断存在。
- Bevy `World`带`WorldId`与change tick，`Commands`通过deferred queue在明确边界应用。WOC需要把read snapshot与write command分开：query看到不可变generation，method只提交typed intent，不能持有并直接改service state。
- Fyrox plugin context把scene、resource、graphics、UI、serialization与lag等能力显式交给lifecycle callback。WOC facet也应收到最小capability context，而不是能import任意全局owner或接受整个mutable service。
- Godot `ClassDB`把类型/方法绑定集中注册，SceneTree和`MultiplayerAPI`把对象/scene path与network instance生命周期绑定，并对主线程操作设约束。WOC registry需要稳定FacetId/MemberId、provider generation、thread/VM domain与失效规则。
- Unity RenderGraph只提供跨域类比：resource handle仅在recording/execution generation中有效，imported resource也有显式registry与cleanup。World snapshot/member handle同样不能脱离发布代际长期缓存；这不是把render graph当成gameplay API参考。

## 4. 可保留基础

1. reference catalog是结构化JSON，已有pinned commit、source identity与delta材料。
2. 28个facet与248个member都有source owner、WOC owner、signature与ownership标注可供生成器消费。
3. catalog内部当前facet/member counts和ownership一致，适合建立更强validator。
4. Card projection没有暴露opponent hand，只复制查询player的一侧手牌。
5. Card helper会拒绝零opponent、负count、零/超上限card value。
6. 四个Card method已有command ID、payload contract与server-side reducer基础。
7. Card view test覆盖idle/unavailable/queued/in-match基础映射，可迁移为projection contract test。
8. package已具备lifecycle与schema查询入口，可扩展为facet registry/publication receipt owner。

这些基础不足以宣告World API，但可以保留为生成输入、domain projection原型和测试fixture。

## 5. P0：World API Publication 硬阻断

### WOC-WAPI-P0-001 · Catalog宣称28个facet，但27个owner不存在且capability没有fail-closed

reference inventory把28个facet和248个成员列为完整审查面，validator也固定这些计数；实际只有一个owner路径存在。系统没有`Declared/Generated/Implemented/Registered/Published/Ready`状态，也没有按产品角色/BuildSet裁剪facet。任何把catalog计数当功能完成率、启动capability或客户端可调用表的路径都会产生假能力。

建立generated `WorldApiSchemaRegistry`和materialization receipt。每个facet必须绑定稳定ID、schema/version/hash、owner module、provider、required capability、artifact digest和验证结果；缺owner/实现/依赖时facet必须从运行时capability中缺席并产生明确诊断，不能用空stub返回默认值。

### WOC-WAPI-P0-002 · 唯一Card owner与catalog公开合同0/5匹配

catalog要求1个data member与4个method，现有模块只导出DTO与projection helper；根产品也不注册它。command链虽然已有四个动作，却没有从world facade到typed payload、authenticated admission、command outcome的绑定。客户端无法从一份同代schema证明“能读什么、能调用什么、返回什么”。

从catalog与Runtime19 descriptor生成或验证`IWorldCardMinigame` facade：data member读取principal-scoped snapshot；method只构造typed command intent并返回submission/outcome handle；signature、command ID、payload schema、permission与provider必须一一对应。现有helper降级为内部projection实现，不得冒充公开facet。

### WOC-WAPI-P0-003 · Projection接受任意pid与mutable service，私有手牌没有principal/代际隔离

public `infoFromService`让caller选择任意pid读取held cards，且输入是可变service引用和外部字符串。当前只是测试模块消费不能消除设计风险；一旦按catalog直接发布，该API即形成横向读取私有状态的路径，也可能在多次getter之间读到不同tick事实。

公开query只接受由host构造的`PrincipalScopedWorldReadSnapshot`，内部绑定world/shard、principal/session、authority lease、tick、snapshot generation、BuildSet和visibility policy。projection一次性从immutable committed view生成，红action/隐私决策可审计；任意pid查询仅限有明确server/admin capability的独立接口。

### WOC-WAPI-P0-004 · Data snapshot、method submission与outcome没有共同publication generation

当前data helper直接拉service，method在另一个command/reducer体系，package schema又不含World API identity。没有保证UI看到generation N的手牌后提交的play command会携N的match/world generation，也没有保证outcome、下一份snapshot与command receipt可关联。跨tick、reload、restore或content install时可出现stale write与撕裂观察。

建立`WorldApiPublicationGeneration`：immutable read snapshot、facet registry、command descriptor set、permission policy、world/content/protocol BuildSet和outcome cursor同代原子发布。method submission携expected world/facet/entity generation与idempotency key；server返回accepted/rejected/committed/stale receipt，客户端只在publication fence后切换整代。

### WOC-WAPI-P0-005 · API artifact、入口与测试产物均未materialize，无法证明clean package可发布

normal package不import/register World API；Card test manifest目标目录缺失；根artifact manifest漏module、含12个不存在路径并使用本机绝对路径。即使源码helper正确，也没有证据表明fresh build可生成、打包、加载和查询该facet，更无法绑定catalog fingerprint。

World API readiness必须依赖Tooling05的原子artifact receipt，并新增API-specific gate：clean workspace生成schema/facade/projection bindings，package-relative manifest列全closure，loader注册facet，client查询capability，Card data/method端到端运行，所有artifact与BuildSet digest匹配。缺失/陈旧/不可搬运一律阻止Ready。

## 6. P1：Schema、Catalog 与 Codegen

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-001 | `schema_version: 1`没有演进规则 | 定义additive/breaking分类、major/minor、协商、迁移与retirement policy |
| WOC-WAPI-P1-002 | member identity是字符串拼接语义 | 生成稳定FacetId/MemberId并保留tombstone，禁止ID复用 |
| WOC-WAPI-P1-003 | validator按裸member name全局去重 | 明确按`(facet, member)`唯一，同时检测跨facet冲突策略 |
| WOC-WAPI-P1-004 | `signature`字段未反序列化 | 用结构化type AST解析参数、返回值、nullability、error与async形状 |
| WOC-WAPI-P1-005 | `member_count`未验证 | facet声明数必须等于实际member rows并进入identity hash |
| WOC-WAPI-P1-006 | `source_owner`/`woc_owner`被忽略 | 校验规范化相对路径、存在性、owner唯一性与layer规则 |
| WOC-WAPI-P1-007 | ownership只是enum label一致性 | ownership驱动allowed dependency、VM/thread/domain、publication与security policy |
| WOC-WAPI-P1-008 | data/method只用两个kind字符串 | schema表达query/command/event/stream、cache、consistency、side effect与receipt |
| WOC-WAPI-P1-009 | 没有member capability需求 | 每项声明role、feature、permission、provider、platform与availability条件 |
| WOC-WAPI-P1-010 | reference commit与runtime BuildSet脱节 | 生成WorldApiSchemaFingerprint并纳入package/handshake/save/replay identity |
| WOC-WAPI-P1-011 | delta只供审查，不形成兼容判断 | CI生成machine-readable compatibility report与required migration actions |
| WOC-WAPI-P1-012 | validator只读reference，不读实现 | 增加catalog→generated declaration→implementation→artifact四向一致性验证 |

## 7. P1：Package Root、Export 与 Artifact

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-013 | package root没有World API composition root | activate时构造registry/provider graph，deactivate按逆序注销并失效handle |
| WOC-WAPI-P1-014 | `plugin.toml`不声明facet capability | manifest列declared/materialized facet set、schema与required providers |
| WOC-WAPI-P1-015 | `woc_game.zrp`只有entry | package profile声明API schema、generated closure、tests与publish gates |
| WOC-WAPI-P1-016 | root schema不含World API fingerprint | package identity绑定facet/member/schema/provider/artifact digest |
| WOC-WAPI-P1-017 | normal lifecycle不触达API | startup registration与fixed-tick publication必须是产品调用图的一部分 |
| WOC-WAPI-P1-018 | manifest使用本机绝对路径 | artifact index改为package-relative URI并由install root安全解析 |
| WOC-WAPI-P1-019 | module list漏实际依赖/多出孤儿artifact | compiler生成closure，拒绝unregistered、missing与orphan artifact |
| WOC-WAPI-P1-020 | zro/zri/AOT能力混杂且缺项 | 每execution mode声明必需artifact set，缺一即不materialize |
| WOC-WAPI-P1-021 | test `.zrp`没有目标artifact | build graph必须产出并运行声明target，missing target为required failure |
| WOC-WAPI-P1-022 | 没有consumer link/reachability检查 | loader smoke test枚举facet并调用schema/query/command facade最小路径 |

## 8. P1：Facet Registry、Lifecycle 与 Capability

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-023 | 无per-world registry | `WorldApiRegistry`绑定WorldId、shard、instance generation与lifecycle |
| WOC-WAPI-P1-024 | facet通过import路径隐式发现 | provider显式注册FacetId、implementation、schema和capability receipt |
| WOC-WAPI-P1-025 | 无duplicate provider规则 | 同facet冲突fail-fast，或通过明确priority/variant selection解析 |
| WOC-WAPI-P1-026 | 无依赖顺序 | facet provider声明domain/service/content依赖并拓扑初始化/销毁 |
| WOC-WAPI-P1-027 | 无Ready/Degraded/Unavailable状态 | registry返回typed availability与稳定reason code，不返回假默认值 |
| WOC-WAPI-P1-028 | 无reload/restore失效语义 | provider generation变化使旧snapshot/handle确定失效并可诊断 |
| WOC-WAPI-P1-029 | 无角色裁剪 | client/server/editor/replay只materialize许可facet与method方向 |
| WOC-WAPI-P1-030 | 无thread/VM domain约束 | query、projection、command submit声明执行domain与handoff机制 |
| WOC-WAPI-P1-031 | 无feature/capability negotiation | handshake发布实际materialized set及每facet版本，不发送catalog全集 |
| WOC-WAPI-P1-032 | 无registry introspection | diagnostics可枚举owner、version、generation、dependencies与health，敏感项redact |

## 9. P1：Snapshot、Query、Privacy 与 Budget

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-033 | query直接接受mutable service | provider只读取tick commit后immutable domain snapshot |
| WOC-WAPI-P1-034 | caller可传任意pid | identity来自authenticated context；cross-principal query需显式capability |
| WOC-WAPI-P1-035 | world/shard/session未绑定 | snapshot token包含WorldId、shard、principal、session generation |
| WOC-WAPI-P1-036 | getter序列可能跨mutation | projection从单一generation view一次构建并携generation receipt |
| WOC-WAPI-P1-037 | opponentName由caller注入 | display identity从同代identity snapshot按visibility policy解析 |
| WOC-WAPI-P1-038 | 没有field-level visibility | schema标注public/self/party/opponent/admin及redaction/fallback |
| WOC-WAPI-P1-039 | DTO没有provenance | snapshot携schema、BuildSet、tick、provider generation与source owner |
| WOC-WAPI-P1-040 | dynamic array无上界 | schema声明max items/bytes；projection前reserve budget并截断或拒绝 |
| WOC-WAPI-P1-041 | string无长度/规范化 | display text执行UTF/normalization/size/localization key规则 |
| WOC-WAPI-P1-042 | 每次全量copy hand | immutable small-vector/arena或generation cache，基准证明成本 |
| WOC-WAPI-P1-043 | 无增量/dirty tracking | field/member change mask与subscription cursor由commit generation驱动 |
| WOC-WAPI-P1-044 | 无snapshot retention策略 | bounded generations、lease/expiry、memory accounting和slow consumer policy |

## 10. P1：Typed Command Facade、Admission 与 Outcome

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-045 | method签名未绑定command ID | generated descriptor一一映射FacetId/MemberId/CommandId |
| WOC-WAPI-P1-046 | method返回`void`掩盖异步authority | 返回submission receipt/operation handle，最终outcome独立可查询 |
| WOC-WAPI-P1-047 | play参数名与payload field漂移 | 结构化schema统一`cardValue/value`并生成双端codec |
| WOC-WAPI-P1-048 | facade没有principal/session | command envelope由host注入authenticated context，脚本不能伪造 |
| WOC-WAPI-P1-049 | facade没有expected generation | 写操作携world/match/facet/entity generation防止stale intent |
| WOC-WAPI-P1-050 | 无idempotency/sequence owner | command port分配sequence/idempotency key并暴露retry policy |
| WOC-WAPI-P1-051 | availability与admission分离 | snapshot给出的action eligibility绑定同代descriptor与reason code |
| WOC-WAPI-P1-052 | method没有permission metadata | generated facade在encode前做local eligibility，server仍权威复核 |
| WOC-WAPI-P1-053 | batch/atomicity不可见 | schema声明single/batch/transaction语义与partial outcome规则 |
| WOC-WAPI-P1-054 | command outcome不回流facet | committed/rejected/stale/duplicate outcome与下一snapshot cursor关联 |

## 11. P1：Card Projection、Presentation 与 Performance

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-055 | domain DTO与view model同owner | simulation projection与presentation adapter拆模块、单向依赖 |
| WOC-WAPI-P1-056 | view state用1..4裸整数 | generated enum含Unknown/Unavailable/Idle/Queued/InMatch/Terminal |
| WOC-WAPI-P1-057 | playable只看waiting flag | eligibility来自authoritative rules snapshot并带不可用reason |
| WOC-WAPI-P1-058 | snapshot无match identity | DTO加入opaque match ID/generation，不暴露可伪造内部对象 |
| WOC-WAPI-P1-059 | count只检查非负 | 校验deck/hand/discard/round组合不变量与配置上限 |
| WOC-WAPI-P1-060 | hand复制两遍且循环getter | service一次产出bounded self-view，projection避免N次动态lookup |
| WOC-WAPI-P1-061 | throw字符串是唯一projection错误 | stable projection error code、field path、generation与redacted source |
| WOC-WAPI-P1-062 | UI action不绑定outcome | view state消费pending submission、server receipt、retry/terminal状态 |

## 12. P1：验证、证据与可观测性

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-WAPI-P1-063 | inventory test不验证owner存在 | 加28/248 owner/member materialization与负向缺失测试 |
| WOC-WAPI-P1-064 | Card test只测helper映射 | 端到端覆盖registry→snapshot→facade→admission→outcome→next snapshot |
| WOC-WAPI-P1-065 | 没有privacy negative test | 证明principal A不能读取B手牌，admin能力单独审计 |
| WOC-WAPI-P1-066 | 没有代际/重载测试 | tick、restore、reload、content install时旧handle必须fail-closed |
| WOC-WAPI-P1-067 | 没有schema compatibility fixtures | additive/breaking/tombstone/signature变化进入golden与migration test |
| WOC-WAPI-P1-068 | 无API运行指标 | 记录facet availability、query cost/bytes、redaction、stale submit、outcome latency与drop |

## 13. P2：长期能力

| ID | 能力 | 目标 |
|---|---|---|
| WOC-WAPI-P2-001 | Typed client SDK | 从同一schema生成Zr/Rust/TS/C# query与command binding |
| WOC-WAPI-P2-002 | Schema browser | editor可浏览facet/member/owner/权限/版本/availability |
| WOC-WAPI-P2-003 | Query planner | 合并同tick projection、共享immutable views并限制重复work |
| WOC-WAPI-P2-004 | Subscription API | generation cursor、backpressure、resume与field mask |
| WOC-WAPI-P2-005 | Offline facade | 同schema支持local authority且保持outcome语义一致 |
| WOC-WAPI-P2-006 | Replay query | 按tick/generation只读查询历史facet，禁止写方法 |
| WOC-WAPI-P2-007 | Multi-world routing | 显式world/shard handle与迁移receipt，不使用进程全局current world |
| WOC-WAPI-P2-008 | Capability delegation | 短期、最小权限、可撤销的facet/member token |
| WOC-WAPI-P2-009 | Privacy audit trail | 敏感field access与redaction decision可采样审计 |
| WOC-WAPI-P2-010 | Projection cache | 按principal/facet/generation安全缓存并可证明失效 |
| WOC-WAPI-P2-011 | Cross-version adapter | 受控兼容层把旧client schema映射到当前publication |
| WOC-WAPI-P2-012 | API fuzzing | schema生成参数、权限、大小、序列与代际fuzzer |
| WOC-WAPI-P2-013 | Differential parity | 与参考实现逐facet比较snapshot与method outcomes |
| WOC-WAPI-P2-014 | Hot facet replacement | quiesce、双代并存、atomic switch与rollback receipt |
| WOC-WAPI-P2-015 | SLO profiles | 每facet query/submit延迟、allocation、bandwidth与availability预算 |
| WOC-WAPI-P2-016 | Deprecation telemetry | 统计旧member/client版本使用后再执行retirement gate |

## 14. 目标架构

```text
Reference Catalog + Domain Schemas + Runtime19 Command Descriptors
                         |
                         v
             WorldApiSchemaCompiler
                         |
       +-----------------+------------------+
       |                                    |
       v                                    v
Generated Facet/Member Registry       Typed Facade Bindings
       |                                    |
       +-----------------+------------------+
                         v
          Per-World WorldApiRegistry
                         |
          Authority Commit Generation N
                         |
        +----------------+----------------+
        |                                 |
        v                                 v
PrincipalScopedReadSnapshot N       WorldCommandPort N
        |                                 |
        v                                 v
Simulation Projection DTO        Admission/Outcome Receipt
        |                                 |
        +-------------- Publication Fence N/N+1
```

核心类型建议：

- `WorldApiSchemaRegistry`：稳定FacetId/MemberId、type AST、ownership、capability、compatibility与fingerprint。
- `WorldApiRegistry`：per-world provider graph、materialization状态、generation、dependencies和introspection。
- `PrincipalScopedWorldReadSnapshot`：immutable committed view、principal/visibility、tick/BuildSet与budget。
- `WorldCommandPort`：generated typed intent到Runtime19 admitted command/outcome的唯一桥。
- `WorldApiPublication`：registry、snapshot、command descriptors、permission与outcome cursor的原子同代发布。
- `WorldApiArtifactReceipt`：schema、implementation、module closure、loader smoke与consumer verification证明。

## 15. 分层重构里程碑

### M0 · Capability Truth 与 Fail-Closed Inventory

扩展validator读取signature/owner/member_count；生成28-facet materialization矩阵。运行时只宣告实际owner+implementation+artifact通过的facet，当前预期为0个公开ready facet而不是28个。

### M1 · World API Schema 与 Generated Registry

定义结构化type/member schema、稳定ID/tombstone和compatibility。把catalog、command descriptors、ownership与BuildSet生成成唯一registry source，消除手工字符串桥接。

### M2 · Per-World Registry 与 Lifecycle

package root构造per-world provider graph，显式initialize/ready/degraded/quiesce/deinitialize。reload/restore使旧handle失效，role handshake只发布materialized capabilities。

### M3 · Immutable Principal-Scoped Snapshot

从authoritative commit生成同代只读view；实现field visibility、redaction、大小/work budget和retention。先把Card helper改为内部provider并完成横向读取负测。

### M4 · Typed Command Facade 与 Outcome

生成Card四方法binding，注入principal/session/generation/idempotency，连接Runtime19 admission/outcome。UI eligibility、pending与terminal结果消费同一publication generation。

### M5 · Artifact 与 Product Reachability

通过Tooling05产出package-relative完整closure；root注册World API；补齐Card target并执行loader/consumer smoke。schema、zro/zri/AOT按execution profile完整且digest同代。

### M6 · 扩展 Facet 与 Parity

按domain owner成熟度逐个materialize其余27 facet；每个facet都需要schema、provider、privacy、command/query、artifact、negative tests和参考差分，不以空文件推进数量。

## 16. 必须通过的验收门

1. Catalog schema解析全部248 member签名，并验证28个facet的member_count、owner与ownership。
2. 缺owner、缺implementation、缺artifact或缺provider时facet不出现在Ready capability中。
3. Catalog、generated registry、Zr implementation与artifact export实现四向逐member一致。
4. World API fingerprint进入package schema、BuildSet、handshake、save与replay identity。
5. Registry绑定明确WorldId/shard/instance generation，并按依赖有序启停。
6. reload/restore/content切换后旧snapshot、member handle和command port确定失败。
7. 所有query从单一immutable commit generation构建，不接受mutable service public参数。
8. 普通principal不能指定任意pid读取private hand；跨principal/admin路径有独立capability和审计。
9. Snapshot field visibility、redaction、array/string/bytes/work budget均有负向测试。
10. `IWorldCardMinigame`的5个member与catalog签名、command descriptor和provider一一匹配。
11. 四个Card method提交typed envelope，携principal/session/world/match generation与idempotency。
12. 每次submission都有stable accepted/rejected/committed/stale/duplicate outcome并关联下一snapshot。
13. UI playable/availability来自同代authoritative eligibility，不由单一waiting flag猜测。
14. Package-relative artifact manifest列完整module closure，不含missing、orphan或本机绝对路径。
15. Card view test target真实生成并执行；normal root产品可枚举、query和submit该facet。
16. Clean workspace可生成schema/bindings/artifact并在不依赖reference checkout的运行目录加载。
17. Additive、breaking、tombstone、signature与ownership变化都有兼容fixture和CI决策。
18. Tick、reload、restore、disconnect、reconnect与content install交错测试不产生撕裂publication。
19. API指标能观察availability、query bytes/work、redaction、stale submit与outcome latency。
20. 其余27 facet只在各自domain、权限、artifact和端到端证据全部通过后逐个升级Ready。

## 17. 验证状态与禁止误判

本轮是静态源码、catalog、artifact与参考引擎审查，没有修改production代码。没有重跑此前未变化的WOC native compile失败与npm typed-contract失败；它们仍阻断动态产品验证。`reference_inventory`测试源码存在不等于本轮test pass，也不证明World API实现。

以下证据不得用于关闭本报告：

- `world_api_catalog.json`有248行entry，不代表248个member已实现。
- Rust validator接受catalog，不代表`woc_owner`存在或签名匹配。
- Card helper self-test可运行，不代表公开facet、权限和command outcome成立。
- command ID 90..93存在，不代表World API method facade已发布。
- `.zro`存在，不代表源/manifest/BuildSet同代或package closure完整。
- 为27个缺失owner创建空文件，不代表facet capability materialized。

关闭P0前必须提供M0–M5的可重放证据；完成整篇则还需M6逐facet审查，而不是用catalog总数一次性宣告完成。
