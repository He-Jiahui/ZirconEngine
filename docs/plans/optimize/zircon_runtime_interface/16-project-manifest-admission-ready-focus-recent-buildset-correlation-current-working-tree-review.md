# `zircon_runtime_interface` Project Manifest / Admission / Ready / Focus / Recent / BuildSet Correlation 当前工作树复核

> report_id: `Interface16`  
> kind: `current-working-tree-review`  
> refreshes: [14-project-manifest-session-admission-hub-launch-focus-recent-cross-process-contract-current-source-review.md](14-project-manifest-session-admission-hub-launch-focus-recent-cross-process-contract-current-source-review.md)  
> canonical_contract_owner: [06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md](06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md)  
> canonical_product_owner: [../zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md](../zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md)  
> certification_owner: [15-contract-certification-abi-layout-buildset-real-dll-skew-cross-language-corpus-fuzz-current-source-review.md](15-contract-certification-abi-layout-buildset-real-dll-skew-cross-language-corpus-fuzz-current-source-review.md)  
> observed_head: `ca3ac3cc6ad218d04a5cd469447cea2452441321`（2026-08-31）；当前工作树有并发修改，本文指纹包含相对 `HEAD` 的 staged/unstaged additions 与 modifications，但不把 source presence 视为 clean-checkout 交付。  
> direct_contract_fingerprint: `2b6ca1fc0f4f90790a15674162431de09429427e0fbbfc75db6d8cc02b3de7e0`  
> focused_consumer_fingerprint: `47804023bc9604e9be99d201a27cd55babbafa50392f9bb54c5f5319fb07e30f`  
> deduplicated_current_source_fingerprint: `057afacf0eedebf4111d41d2c0307eefa8fa8eb50c7a5264482da3ff586a5e51`  
> reference_fingerprint: `56cfbf4f8b1d97df3bf0c38ceba0cc1fc8830e18d9e011a58b37ce6e3abc1539`  
> status: review-only；不修改 production/test/Cargo/ABI，不运行 Cargo、Hub、Editor、真实双进程、fault、ACL、fuzz、scale、soak 或动态 benchmark。Tooling 与 Rust 迁移按用户要求排除；不查询、轮询、等待或实时跟踪协调器。

## 1. 审查结论

Interface14 的工程判断仍成立，当前工作树又补强了 manifest complexity admission、BuildSet artifact validation、session ledger ready phase 等局部底座，但它们仍是分散事实，不是同一个跨进程事务。

当前最危险的错误不是“没有类型”，而是多个名字强于实际证明：

1. Hub 的 `ProjectValidation::Valid` 只证明 summary parser 接受文件；它没有证明 full manifest、迁移策略、provider、plugin/script、trust 或 BuildSet 可被当前 Editor 安全打开。
2. Editor 的 `SessionAdmissionRequest` 文档写成“authenticated”，但其 principal 合同明确只是 provenance，不是 authentication claim。
3. session record 的 `Ready` 先于 activation ledger `Session` commit；因此跨进程观察者可以在本地事务尚未提交时得到“可聚焦”结论。
4. `HubEditorReadyReceiptV1::after_first_present()` 一次性制造五个 milestone 全集；Hub 最终只验证 child PID，没有验证 operation、ProjectIdentity、manifest digest、BuildSet、request hash 或 deadline。
5. BuildSet 已在 App 的 Runtime artifact preflight 和 session record 中分别出现，但 composition receipt、project preflight receipt、session commit 与 Hub Ready 没有被一个 correlation envelope 绑定。
6. focus 有真实 generation/sequence/request/deadline/ack 进展，但 public derived `Deserialize` 仍可绕过 constructor；malformed/mismatch claim 会停留在 private filename，Hub 还能自行创建 Editor inbox。
7. recent store 的 CAS、tombstone、deadline、quarantine 与 durability 是可保留实现；但其 identity 仍是 lossy lexical path，并且 filesystem lock、repair、atomic replace、环境变量 root 等业务 owner 已错误进入 Interface crate。

因此，当前系统不能把 `summary parsed -> child spawned -> session record Ready -> first-present mailbox -> recent updated`解释为同一项目启动事务已提交。Interface16 不新增唯一 P0/P1/P2：Editor268/51/172 继续拥有五项项目生命周期 P0；Interface06 的 56 项 P1 与 14 项 P2仍是 canonical 合同账目；Interface15 继续拥有 BuildSet/ABI/corpus/real-DLL 认证。状态保持 **P1 20 Open / 25 Partial / 11 Closed，P2 8 Open / 6 Partial / 0 Closed，36 门 14 Fail / 14 Partial / 8 Pass**。

## 2. 审查边界与证据等级

### 2.1 当前工作树冻结范围

| 选择集 | files / lines / bytes / test attrs / ignored | HEAD-delta paths / untracked | fingerprint |
|---|---:|---:|---|
| Interface project/hub protocol/BuildSet 直接合同 | **91 / 6,859 / 233,318 / 82 / 1** | **79 / 0** | `2b6ca1fc0f4f90790a15674162431de09429427e0fbbfc75db6d8cc02b3de7e0` |
| App/Runtime/Editor/Hub focused consumers | **141 / 24,831 / 908,032 / 282 / 8** | **103 / 10** | `47804023bc9604e9be99d201a27cd55babbafa50392f9bb54c5f5319fb07e30f` |
| 去重当前源码 | **232 / 31,690 / 1,141,350 / 364 / 9** | **182 / 10** | `057afacf0eedebf4111d41d2c0307eefa8fa8eb50c7a5264482da3ff586a5e51` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考 | **18 / 17,200 / 640,148 / 19 / 0** | n/a | `56cfbf4f8b1d97df3bf0c38ceba0cc1fc8830e18d9e011a58b37ce6e3abc1539` |

直接合同选择包括 `hub_protocol` 全量、manifest summary/admission/migration、activation operation ID、engine compatibility、session record/codec、canonical descriptor、manifest digest、ProjectGuid/ProjectIdentity、ProjectLaunchIntent 和 Runtime BuildSet 全量。consumer 选择沿 App launch/runtime loader，Runtime full manifest，Editor project authority/preflight/session guard/当前 `project_session_effect_ledger`/Hub link/startup/retained host，以及 Hub launch/handshake/focus/recent action 全链。审查期间Editor ledger目录从暂存后删除的 `activation_ledger` hard-cut为当前owner路径，本文已按新路径重新冻结统计与指纹。

指纹算法为：相对路径小写并排序，逐文件 SHA-256，再以 `path<TAB>hash` 和 LF 拼接后取 SHA-256。`HEAD-delta` 表示路径内容相对 observed HEAD 不同，包含已暂存新增文件；它不是“已提交”。

### 2.2 动态证据边界

本轮没有运行 Cargo，原因不是把静态审查当成测试通过，而是当前任务明确是 review-only，且 232 个选择文件中 182 个相对 HEAD 已改变、10个仍未跟踪。两次全局 `audit_runtime_structure.py --json` 尝试均超时，没有结果；该门记录为 unavailable，不记作 Pass，也不继续等待阻塞。本文只对定点源码链和本地参考源码给出 E3 静态结论。

### 2.3 本轮逐段追踪的实际链

| 阶段 | 当前 producer / consumer | 已证明 | 未证明 |
|---|---|---|---|
| Hub list probe | `zircon_hub/projects/validation.rs:29-31` | summary parser 接受当前 bytes | full manifest、migration、trust、BuildSet、provider 可用 |
| Launch issue | `zircon_hub/process/editor_launch.rs:14-15,87-126` | intent JSON、Hub session、protocol 被传入 child | 单一 request digest、deadline、capability 与 selected BuildSet |
| Runtime load | App `runtime_library` + artifact manifest | staged artifact 与 host expectation 有独立校验 | 该校验结果与项目 request/admission/Ready 同一事务绑定 |
| Editor preflight | `zircon_editor/core/project` | bounded read、digest、ProjectIdentity、migration/compatibility typed facts | trust/authorization、完整 provider/plugin/script admission |
| Session claim | Interface record + Editor `SessionGuard` | OS lease、operation、principal provenance、BuildSet、lifecycle、heartbeat | ProjectIdentity、process creation identity、随机 admission epoch、ledger digest |
| Activation commit | `editor_manager_project_session.rs:795-861` | rollback/recovery 分支与 ledger 存在 | session record Ready 与 ledger Session 的原子共同 commit |
| Hub Ready | `retained_host/app.rs:369-390` -> Hub wait | focus binding 后、first present 才写 mailbox | request-bound逐 milestone evidence 与 child terminal supervision |
| Existing focus | Hub publisher -> Editor claim -> native focus -> ack | generation/sequence/request/deadline 与 native-owner ack | owner-created namespace、replay set、malformed cleanup、ACL |
| Recent projection | Interface store + Hub/Editor callers | bounded registry、CAS、tombstone、quarantine、durable replace | ProjectId identity、namespace/retention/security、正确 crate owner |

## 3. Interface 直接合同逐域复核

### 3.1 Manifest summary 不是项目可打开证明

可保留底座：

- `manifest_summary/limits.rs` 已定义 4 MiB、4,096 asset roots、32 nesting depth、16,384 table entries 与 65,536 array items；`admission.rs` 以迭代 traversal 计算 depth/item budget。
- `parse.rs:25-68` 会校验 project name、default scene、engine requirement、current-format ProjectGuid、asset roots、settings 与 library version shape。
- `ProjectIdentity` 已组合 canonical descriptor、ProjectGuid 与 manifest digest，Editor preflight 会在 admission 前重验 digest。

仍不合格：

- `summary.rs:9-17` 的字段公开并 derived `Deserialize`；任何 recent/fixture/consumer 都能绕过 parser invariants。
- `parse.rs:66-78` 验证后丢弃 asset roots、settings 与 library version；返回值没有 `validated_sections/deferred_sections/ignored_sections/reader_policy/migration_steps`。
- Hub 在 `validation.rs:29` 与 `recent_project.rs:42` 先 `fs::read` 整个文件，之后才进入 4 MiB parser cap；恶意或损坏文件仍可导致 cap 前分配。
- 复杂度有 container 预算，但没有统一 per-key/per-string/path bytes、decoded heap 或 duplicate-key policy receipt。
- `CanonicalDescriptorIdentity` wire 仍承载 `PathBuf`；它证明 lexical absolute/no-dot-segment，不证明 filesystem object identity、opened-root capability 或跨 OS 编码稳定。

路由：`RI-PROJ-P1-001..007` 与 `RI-PROJ-P1-013`。其中 P1-006 仍为 Partial，但当前证据应更新为“container complexity cap 已存在，Hub read-all 与 string/path/heap budget 未闭合”。

### 3.2 Launch intent 只是请求意图，不是 admission grant

`ProjectLaunchIntent` 的 private fields、custom deserialize、schema version、operation ID、source、Normal/Safe/Recovery profile 是正确方向。但 V1 仍只携带 target `PathBuf` 与请求来源；没有：

- expected `ProjectIdentity` / manifest digest；
- selected/required `ZrRuntimeBuildSetId` 与 capability set；
- issuer identity、request nonce/hash、attempt、deadline/cancel token；
- trust decision、migration approval、recovery evidence；
- durable idempotency/dedup record。

Hub 又把 `--hub-session` 与 `--hub-protocol` 作为 intent JSON 旁路参数传递。operation ID 由随机 origin UUID、进程内 atomic sequence 和 nonce 组成，可避免常见碰撞，但它本身不提供 durable exactly-once 或 restart replay 语义。

路由：`RI-PROJ-P1-008..011`、`RI-PROJ-P1-022`、`RI-PROJ-P1-024..025`、`RI-PROJ-P1-049`。

### 3.3 Session admission record 缺少决定性绑定

`ProjectSessionAdmissionRecordV1` 当前含 process ID、instance ID、principal、BuildSet ID、operation ID、lifecycle、checked epoch、optional session generation 与 wall-clock heartbeat。lifecycle 与 generation 关系有验证，codec 拒绝 duplicate/unknown key，production heartbeat 也真实运行。

但 `record.rs:6` 明确 principal 是 provenance，不是 authentication；Editor `session_guard/admission.rs:5` 却把三字段 request 称为 authenticated。该命名会诱导上层把 Normal profile 当作已授权。

记录中仍没有：

- ProjectIdentity / manifest digest / preflight receipt digest；
- process creation time/token，因而 PID reuse 仍未排除；
- 随机 admission epoch；新 claim 从 checked epoch 1 开始，不能独立抵抗 ABA；
- activation ledger generation/digest/commit marker；
- monotonic heartbeat sequence、producer clock 与 observer time；
- terminal Closed/cleanup receipt。

`from_persisted` 还没有统一拒绝 PID 0、heartbeat 0 或无界 instance ID；session/Ready/focus/path 的 instance grammar 与 length 约束不同。手写 key=value codec 接收无界 `&str`，没有 source byte/line/key/value cap 或 checksum。

路由：`RI-PROJ-P1-015..021`、`RI-PROJ-P2-003..005`。trust/authorization 产品阻断仍归 Editor268，不创建 Interface P0 副本。

### 3.4 Ready receipt 把标签集合误作阶段证据

`HubEditorReadyReceiptV1` 使用 private fields 与 validated deserialize，且 Editor 在 focus binding 完成并等待 first present 后才发布，这两点应保留。

但 `ready_receipt.rs:13-17` 定义的五个 milestone 在 `after_first_present()` 中一次性构造；没有每阶段 producer、evidence digest、time、generation 或 disposition。receipt 只有 PID、instance、session generation 和 milestone set，不携带 operation、ProjectIdentity、manifest digest、BuildSet、launch session/request hash、deadline。

Hub `editor_launch_actions.rs:556-582` 对 Ready 的产品校验只比较 `receipt.editor_process_id() == child_process_id`。PID 一致不能证明：

- mailbox 属于本次 request，而不是 stale/replayed compatible mailbox；
- child 使用 Hub 选中的 BuildSet；
- ProjectIdentity 在 preflight 后未漂移；
- session ledger 已提交；
- first present 对应本次 project generation；
- child 超时后已终止、取消或进入明确 terminal state。

mailbox DTO 仍是 public fields + derived Deserialize；Hub read 使用无界 `fs::read`，固定 250 ms poll/10 s timeout，没有 claim/ack/remove/retention，也不在等待期间检查 child exit/cancel。

路由：`RI-PROJ-P1-012`、`RI-PROJ-P1-023..028`。

### 3.5 Focus 有真实闭环，但 namespace 与错误恢复未工程化

可保留底座：request 路径含 target instance、generation、sequence、UUID request ID；请求和 ack 各有 4 KiB 后置检查；Editor rename claim；deadline/stale/full 有 typed disposition；只有 native `Focused(true)` 才发布 `Focused` ack；bridge 容量 32。

仍不合格：

- focus signal/ack 的字段公开并 derived `Deserialize`；signal constructor validation可被 wire 绕过，ack没有等价 validate。
- Hub cleanup 在 cap 前 `fs::read`，`read_dir(...).filter_map(Result::ok)` 会忽略部分目录错误。
- Hub atomic writer 在 `publish.rs:137` 自行 `create_dir_all`；publisher可制造“看似已绑定”的 inbox。
- Editor `focus_signal.rs:72` 已明确 malformed/instance-mismatched request 会留在 private claimed name；oversize/parse/mismatch 的早退也阻断当前 batch 后续请求。
- ack mismatch 返回错误时文件不会被消费，可能持续 poison；writer epoch、durable consumed set、duplicate/replay disposition仍缺失。
- owner-only ACL、symlink/reparse、parent identity pinning、namespace quota/retention没有合同。

路由：`RI-PROJ-P1-029..035`、`RI-PROJ-P1-047..050`。

### 3.6 Recent store 的实现质量高于它的边界质量

当前 store 有 256 KiB read cap、revision/CAS、logical clock、8 entries/64 tombstones、bounded lease deadline/cancel/try-now、corruption disposition、quarantine/rebuild、file sync、Unix parent sync 与 Windows `ReplaceFileW(...WRITE_THROUGH)`。Editor 把 recent writeback 放到 project commit 后，失败只产生 deferred diagnostic。这些行为应迁移保留。

但 V1 的核心语义仍不合格：

- `HubRecentProjectV1` 的 summary/path/time 字段公开并 direct deserialize；`validate()` 只拒绝空 path。
- identity 使用 `Path::to_string_lossy()`、slash normalization 与 Windows-looking lowercase；这不是 physical identity，也不是 ProjectGuid/ProjectIdentity。
- storage root 从 HOME/USERPROFILE 推导，缺失时可落到相对 cwd `.zircon`；没有 product/channel/profile/BuildSet/test-instance namespace。
- corrupt quarantine 没有 retention/quota/last-known-good；temp/claim/ack/private files也没有统一 scavenger owner。
- filesystem lock、OS mutex/flock、deadline policy、repair、quarantine、atomic replace和环境变量 root 全部位于 `zircon_runtime_interface`。这违反“Interface 只拥有 schema、validated value、bounded codec”的边界。

路由：`RI-PROJ-P1-036..048`、`RI-PROJ-P2-006..008`、`RI-PROJ-P2-013..014`。

### 3.7 BuildSet 已可校验，但没有进入项目启动 commit

`ZrRuntimeArtifactManifestV1::validate_against` 会校验 derived BuildSet、artifact/target identity、host artifacts 和 required capabilities；App 在动态库加载前后使用 expectation，session record 也携带 BuildSet ID。这是重要底座。

然而：

- artifact identity、target、manifest 多个 DTO 仍 public fields + derived Deserialize，正确性依赖 consumer 记得调用 `validate_against`。
- host artifact/capability/feature/string 数量与长度没有统一 decode budget。
- `ZrRuntimeModuleCompositionReceiptV1` 只有 schema/catalog generation/source fingerprint/target/module profile/session profile/composition hash，未绑定 ProjectIdentity、operation、admission、session generation 或 Ready request。
- Hub source engine resolution与 App loaded artifact expectation 是两条并行链；Hub不能验证 Ready child 实际使用了所选 BuildSet。
- session record 仅存 BuildSet ID，无法证明其来自哪个 artifact validation receipt，也无法把 runtime unload/recovery关联回 launch request。

BuildSet 本身的完整 ABI/real-DLL/skew/corpus 资格继续由 Interface15 负责；本报告只记录它没有进入 project transaction correlation，路由到 `RI-PROJ-P1-022/049/052`。

## 4. 跨 crate 消费链的具体错误窗口

### 4.1 Hub summary 与 child supervision

`zircon_hub/src/projects/validation.rs:29-31` 和 `recent_project.rs:42-43` 都在 parser 前读完整 manifest。launch action spawn 后丢弃 `Child` owner，只保留 PID；`wait_for_project_editor_ready()` 固定等待 mailbox，无法同时检测 child exit、请求取消或超时后的终止/收割。成功后又重新读取 manifest写 recent，未复用 Editor preflight identity receipt。

目标应是 Hub 持有 `LaunchAttempt`：包含 child handle、request digest、deadline/cancel、selected BuildSet expectation、mailbox claim、terminal outcome 与 reap receipt。PID 只能是诊断字段，不能是唯一绑定。

### 4.2 Editor admission commit 顺序错误

`zircon_editor/src/ui/host/editor_manager_project_session.rs:792-858` 的顺序为：

1. `guard.commit_ready()` 把跨进程 session record 写成 Ready；
2. `ledger.commit(ProjectSessionEffect::Session)`；
3. `ledger.begin_ready()`；
4. 最后才把 guard 安装到 manager slot 并启动 heartbeat（859-885）。

因此至少存在两个观察窗口：Ready record早于ledger commit；ledger commit又早于manager正式接管guard。失败分支会尝试 `RecoveryRequired` 并保留 guard，这是有价值的恢复底座，但不能撤销其他进程已经观察到的 Ready。

该问题继续由 Editor268 的生命周期 P0拥有。Interface 的责任是提供一个不可拆分误用的 `ProjectAdmissionCommitV2` schema/codec，而不是再加一个可选字段。

### 4.3 First-present 之后仍有 close 绕过

`retained_host/app.rs:369-390` 正确地把 Ready mailbox 延迟到 first present；但主循环之后多个 `return Err` 位于 `commit_project_close()`（521）之前。close/rollback/recovery的产品状态机仍可能被 host error shortcut 绕过。该问题属于 Editor268，不重复为 Interface finding。

### 4.4 当前源码存在已知测试集成红灯

`ProjectManifestSummary` 新增 `project_guid` 后，两个同 crate test fixture 的 struct literal仍缺该字段：

- `zircon_runtime_interface/src/hub_protocol/tests.rs:417-422`
- `zircon_runtime_interface/src/hub_protocol/recent_projects/store.rs:821-826`

Rust struct literal在字段缺失时无法编译，因此这是可静态确认的 source integration blocker；本轮没有运行 Cargo。它已由 `zircon_editor/51/failure-2026-08-25-engine-compatibility-caret-range-exhaustiveness.md` 记录，Interface16 只链接现状，不新建重复 failure/finding。

## 5. 本地参考源码对照

| 参考 | 定点证据 | 对 Zircon 的约束 |
|---|---|---|
| Unreal ProjectDescriptor | `ProjectDescriptor.h:25,45,76,85,88,91` 显式 file version、engine association、modules、plugins、target platforms | summary display、descriptor truth、engine selection与upgrade decision必须分层 |
| Unreal ProjectManager | `IProjectManager.h:88,99,163,248` 分离 current project、load、query status、save；`ProjectManager.cpp:185` 单独计算requires update | `parse ok`不能命名为完整Valid；status必须带可解释 disposition |
| Unreal Project Browser | `SProjectBrowser.cpp:771-775,932` 比较project engine identifier并进入显式open流程 | selected engine/build与project compatibility应在spawn前形成决定 |
| Godot Project List | `project_list.cpp:77-103,289-368` 区分older/newer major/minor、unknown version、unsupported feature、missing | Hub必须展示和传递typed compatibility，不得压成bool Valid |
| Fyrox Project Manager | `manager.rs:79-81,837-881,1182` manager持有command queue与`Child`并轮询process；同时显示engine version/upgrade | Hub launch owner应持有child到terminal/reap，升级是显式workflow |
| Bevy App | `app.rs:232-240,274,294,1529` plugin状态分Adding/Ready/Finished/Cleaned，Ready由所有plugin共同满足 | Ready必须是多个owner evidence的组合结果，不能由单constructor制造全集 |
| Unity Graphics migration | `IVersionable.cs:7-18` 与 `MigrationDescription.cs:18-24,42-50` 将version与ordered migration steps绑定 | manifest migration receipt必须列完整步骤、目标version与writeback/rollback disposition |

Graphics 本地镜像不含 Unity Hub 跨进程实现，因此 launch/focus/recent比较标记为 N/A，不据此虚构参考能力。上述参考支持工程分层，不证明 Zircon 当前性能、稳定性或表现已经达到或超过 Unreal。

## 6. Canonical finding 重判与去重

### 6.1 Interface06 P1 状态

本轮逐项回放后没有状态类别变化：

| family | Open | Partial | Closed | 本轮证据变化 |
|---|---|---|---|---|
| P1-001..007 manifest/identity/migration | 3 | 4 | 0 | complexity container budgets是真实新增/增强证据；仍缺Hub bounded reader和section receipt |
| P1-008..014 version/capability/wire | 5 | 2 | 0 | BuildSet validation更强，但未进入launch capability negotiation |
| P1-015..021 session/liveness | 2 | 3 | 2 | ledger `begin_ready` 存在；record Ready仍先发生，process/admission identity未补齐 |
| P1-022..028 launch/Ready/mailbox | 2 | 5 | 0 | first-present timing保留；request-bound receipt与child supervision仍缺 |
| P1-029..035 focus | 2 | 1 | 4 | native focus ack保留；public wire、inbox ownership、claim cleanup仍缺 |
| P1-036..042 recent conflict | 1 | 1 | 5 | CAS/tombstone/quarantine底座保留；summary/path identity未变 |
| P1-043..049 owner/storage/lineage | 1 | 6 | 0 | store越界与BuildSet correlation断裂均仍在 |
| P1-050..056 security/qualification | 3 | 4 | 0 | 没有ACL、dynamic matrix、N/N-1；统一correlation仍缺 |
| **合计** | **20** | **25** | **11** | 无新增唯一finding |

### 6.2 Interface06 P2 状态

保持 **8 Open / 6 Partial / 0 Closed**。重点仍是 schema V1散落、instance ID validator分裂、free-string error、multiple temp/claim naming、wire DTO/validated value混合、environment storage fallback与Interface非业务owner守卫缺失。

### 6.3 P0 与 failure owner 路由

| 现象 | canonical owner | Interface16处理 |
|---|---|---|
| record Ready早于ledger Session commit | Editor268 / Editor51/172 | 提供不可拆分commit contract，不重复P0 |
| Normal无trust/authorization | Editor268 | 修正文档语义并提供typed admission receipt，不拥有产品决策 |
| operation无durable dedup/replay | Editor268 + Interface06 | 提供identity/schema；durable owner在Hub/Editor service |
| host错误出口绕过close | Editor268 | 仅要求terminal receipt schema |
| 两个summary fixture缺`project_guid` | Editor51 failure | 记录source blocker，不重复failure |
| BuildSet真实DLL/skew/corpus不完整 | Interface15 | 本文只审project transaction binding |

## 7. 目标合同与 owner

| 合同 | schema必须携带 | 行为owner |
|---|---|---|
| `ProjectDescriptorProbeV2` | locator、source digest/size、reader policy、validated/deferred/ignored sections、migration chain、budget receipt | Runtime manifest service |
| `ProjectLaunchRequestV2` | operation、attempt、expected ProjectIdentity、selected/required BuildSet、capabilities、profile、deadline、request digest | Hub issuer；App只解析/转交 |
| `ProjectAdmissionDecisionV2` | trust、engine/BuildSet/provider compatibility、migration/recovery decision、preflight digest、expiry | Editor project authority |
| `ProjectAdmissionCommitV2` | request digest、ProjectIdentity、BuildSet validation receipt、process creation identity、random admission epoch、ledger digest、session generation | Editor session owner |
| `EditorStartupProgressV2` | stage owner、stage generation、evidence digest、monotonic time、terminal/cancel/retry disposition | Editor host；Interface只定义bounded codec |
| `EditorStartupReceiptV2` | commit digest覆盖所有progress、window/first-present/interactive facts与launch request | Editor publisher；Hub验证/claim/ack |
| `FocusInboxBindingV2` | owner-created namespace capability、target generation、writer epoch、quota、expiry、ACL disposition | Editor window owner |
| `FocusRequest/AckV2` | writer epoch、sequence、deadline、dedup/replay disposition、cleanup receipt | Hub + Editor |
| `RecentProjectOperationV2` | stable ProjectId、writer/revision、upsert/tombstone、bounded transaction、repair/durability | Hub recent service；store迁出Interface |
| `CrossProcessCorrelationV1` | operation/request/admission/session/BuildSet/project IDs与privacy/retention | shared diagnostics schema + owner services |

硬边界：Interface只保留 schema、validated values、bounded codecs与compatibility dispositions；不得继续拥有 filesystem root解析、OS lock、deadline循环、quarantine、atomic replace或业务重试。

## 8. 重构顺序

### M0 · 冻结真相与修复source integration

- 修复已登记的两个 `project_guid` fixture并取得 focused compile evidence；不扩大为production重构。
- 生成 protocol/schema/owner inventory，列出每个 public deserialize入口、预算、版本支持窗和consumer。
- 将 `Valid` 改为 `PartialProbeAccepted` 等不超出事实的typed状态；修正“authenticated principal”错误文档。

### M1 · Manifest probe与admission decision

- 分离 wire document、validated summary、full preflight receipt与product admission decision。
- Hub改为metadata + bounded reader，不得在parser cap前全量分配。
- 输出validated/deferred/ignored/unsupported sections、migration steps、reader/budget receipt。

### M2 · 单一launch request与BuildSet binding

- 合并intent、Hub session、protocol、capabilities、selected BuildSet、deadline为request envelope。
- App Runtime artifact validation生成digest receipt；Editor admission必须消费并绑定该receipt。
- operation进入durable attempt/dedup store，restart/retry产生typed replay disposition。

### M3 · 原子admission commit

- 引入random admission epoch与OS process creation identity。
- 将ProjectIdentity、BuildSet receipt、ledger digest、session generation写入单一commit marker。
- 先commit ledger与owner state，再原子发布外部Ready；失败只能发布terminal RecoveryRequired receipt。

### M4 · Startup progress与mailbox transaction

- 每个阶段由实际owner签发evidence，最终receipt覆盖所有stage digest。
- Hub保留child handle，支持cancel/deadline/exit/reap；mailbox实行bounded read、claim、ack、remove、retention。
- stale/replay/mismatch必须产生可清理的typed terminal disposition。

### M5 · Focus namespace与recent owner迁移

- Editor先发布inbox binding capability，Hub不得创建目标目录。
- private claim、ack、temp、malformed进入统一bounded scavenger/quarantine。
- recent store整体迁到Hub/shared host service，以ProjectId替换lossy path key并保留现有CAS/durability。

### M6 · Security、fault与observability

- owner-only ACL、symlink/reparse/parent replacement fail-closed；定义字段敏感级别。
- write/flush/rename/read/claim/ack/cleanup逐crash point建立可见状态、repair与terminal receipt。
- 统一operation/request/admission/session/BuildSet correlation metrics与retention。

### M7 · Compatibility与发布资格

- N/N-1/N+1 Hub/Editor/Runtime artifact matrix、immutable golden corpus、unknown field/upgrade/downgrade policy。
- 双Hub/双Editor、PID reuse、kill/suspend、publisher race、replay、permission、network filesystem动态矩阵。
- corpus/fuzz/property/scale/soak evidence绑定source fingerprint、BuildSet、platform与test artifact；通过前不得宣称Unreal等价或更优。

## 9. 36 项产品资格门

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01 | Partial | 232个当前文件与指纹已冻结；182个路径相对HEAD不同且10个未跟踪，无自动drift/clean-checkout gate。 |
| G02 | Pass | 18个本地参考文件有适用边界；Graphics Hub明确N/A。 |
| G03 | Fail | summary/mailbox/focus/ack/recent/BuildSet多处public derived deserialize可形成未validated值。 |
| G04 | Fail | partial probe不列validated/deferred/ignored/unsupported sections。 |
| G05 | Partial | Editor preflight绑定digest/ProjectIdentity；Hub launch与Ready不绑定同一receipt。 |
| G06 | Partial | manifest已有bytes/depth/table/array/root cap；Hub cap前read-all及string/path/heap预算未闭合。 |
| G07 | Partial | migration types存在；完整step/lossy/backup/writeback receipt缺失。 |
| G08 | Partial | directional semver与BuildSet artifact validation存在；trust/provider/product decision未组合。 |
| G09 | Fail | mailbox/focus/recent共享exact V1，无独立schema family/support window。 |
| G10 | Fail | 无真实N/N-1/N+1 Hub/Editor/Runtime binary matrix。 |
| G11 | Partial | operation/BuildSet/session/generation分散存在；无共同request/commit digest。 |
| G12 | Partial | first-present与五项标签存在；不是逐阶段owner evidence。 |
| G13 | Fail | timeout/cancel无terminal receipt，Hub不持有child到reap。 |
| G14 | Partial | focus有claim/ack；launch mailbox与malformed cleanup不确定。 |
| G15 | Fail | OS lease probe与record read不是同一epoch snapshot。 |
| G16 | Fail | PID/instance无OS process creation identity。 |
| G17 | Partial | heartbeat真实运行；无monotonic sequence、clock/skew/suspend disposition。 |
| G18 | Pass | Hub不把Claimed/Activating/Closing分类为Ready。 |
| G19 | Partial | mailbox在first present后发布；session record仍在ledger commit前Ready。 |
| G20 | Pass | focus按generation/sequence/request ID排队。 |
| G21 | Pass | Hub只在精确request-bound native Focused ack后报告FocusedExisting。 |
| G22 | Partial | expired/stale/full有typed ack；malformed/mismatch/duplicate/replay未统一terminal。 |
| G23 | Fail | private malformed claim无bounded quarantine/cleanup owner。 |
| G24 | Partial | recent有revision/CAS/tombstone；仍无ProjectId/writer identity。 |
| G25 | Pass | recent lease支持deadline/cancel/nonblocking。 |
| G26 | Partial | corrupt/oversize可quarantine/rebuild；无last-known-good与retention。 |
| G27 | Pass | recent失败不回滚成功project activation。 |
| G28 | Pass | Hub/Editor当前共享同一recent transaction实现；owner crate仍错误。 |
| G29 | Fail | storage namespace未隔离product/profile/BuildSet/test instance。 |
| G30 | Fail | ACL、symlink/reparse、parent identity无fail-closed contract。 |
| G31 | Partial | Ready/failure部分redact；缺统一sensitivity policy。 |
| G32 | Fail | 无write/flush/rename/read/claim/ack/cleanup crash matrix。 |
| G33 | Fail | 无双进程、PID reuse、kill、suspend、race、replay动态测试。 |
| G34 | Fail | 无arbitrary bytes/deep JSON/duplicate key/Unicode/path/oversize fuzz。 |
| G35 | Fail | 无统一correlation telemetry与launch/session cleanup census。 |
| G36 | Pass | 报告、索引、coverage、链接、状态计数与diff静态检查已完成。 |

合计：**14 Fail / 14 Partial / 8 Pass**。

## 10. 实施验收必须产生的证据

1. clean checkout能编译Interface/Hub/App/Editor focused targets，并验证两个stale fixture已关闭。
2. golden corpus覆盖每个schema family的current、old supported、future unknown、malformed、oversize与duplicate-key输入。
3. 真实Hub启动真实Editor，证明request digest、ProjectIdentity、BuildSet、admission epoch、session generation与Ready receipt一致。
4. fault injection覆盖ledger commit前后、record publish、mailbox rename、first present、focus claim/ack与recent flush/replace。
5. child exit/timeout/cancel均有terminal outcome与reap receipt，无残留process/mailbox/lease。
6. PID reuse、双Hub、双Editor、restart replay与stale mailbox不能错误focus或复活recent entry。
7. Windows/Linux的ACL、symlink/reparse、directory replacement与durability声明分别验证。
8. scale/soak记录manifest parse heap、startup latency、focus latency、lease contention、quarantine增长和cleanup backlog。

## 11. 最终审查决策

当前代码可以保留 manifest complexity admission、ProjectIdentity/digest、typed intent/profile、BuildSet artifact validation、OS lease/lifecycle/heartbeat、first-present mailbox、native focus ack以及recent CAS/tombstone/durable replacement。

当前代码不能继续声称：summary可解析等于项目Valid；principal provenance等于authenticated；session record Ready等于activation事务提交；固定milestone集合等于逐阶段证据；PID一致等于本次request/BuildSet；lossy path等于ProjectId；Interface内filesystem store等于稳定公共合同。

后续实现应从 M0/M2/M3 开始：先关闭source integration与命名超承诺，再建立共同request/BuildSet/admission commit。不要继续在V1各DTO上追加可选字段来掩盖缺少共同identity、commit marker与terminal receipt的根问题。
