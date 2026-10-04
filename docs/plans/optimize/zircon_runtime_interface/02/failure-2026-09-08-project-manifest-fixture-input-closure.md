---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: project-manifest-fixture-input-closure
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/project/tests/manifest_summary.rs
tests:
  - 'cargo test -p zircon_runtime_interface --no-default-features --locked --lib project::tests::manifest_summary::'
  - python -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_declared_build_and_test_dependency_roots_have_distinct_scopes
---

# Interface02: shared project manifest fixture input closure

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：完整接口库受管动态验收。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 交接原因：项目 schema golden corpus 属于 Interface02；包元数据须声明其测试数据依赖。

## 失败现象与复现证据

Windows 作业 `893e5568e6364fd9944fd431c04c6ffa` 实际执行 legacy-v1、shared-v2/v3
及 future-version 三个 manifest summary 测试，均因 fixture 文件不存在失败。
输入 `interface-cache-generation-3159-20260908`，manifest
`e275ded38d2cd913d9bac17ee5ec8fdcff1f9e17e75306de307a82d6b5c38038`；
完整结果 747 passed、16 failed、101 ignored，日志为 `results/interface-library-3159.log`。
测试的 `CARGO_MANIFEST_DIR/../tests/fixtures/serialization/project-manifest` 是当前规范
路径，原工作树包含 v1/v2/v3/future/invalid 五份 TOML，冻结输入却未包含这些文件。

## 最低共享层根因

Cargo 输入闭包已支持 `[package.metadata.zircon.validation]` 下的
`test-dependency-roots`，但接口包未声明运行时读取的仓库根 fixture。正确修复为声明
`["../tests/fixtures/serialization/project-manifest"]`；不改测试路径或 golden bytes。

## 架构修复验收

- 正式输入规划通过包声明自动封入五份 fixture，hash 与来源快照一致。
- 三个原始失败及全部 manifest summary 测试实际通过，旧版本迁移、future 拒绝和
  invalid engine requirement 断言全部保留。
- 现有 build/test dependency-roots 分层回归通过，编译闭包不无条件扩大到全仓库。
- 独立审查 C0/I0/M0、正式验证绑定、failure return 与 closeout 完成。

## 禁止临时方案

- 不将 fixture 内容硬编码到测试，不使用当前工作目录或绝对 E 盘源路径兜底。
- 不在已封存输入内修改文件，不擅自取得活动 owner 的 Cargo.toml。
- 手工封入已归属 fixture 的定向验收仅证明动态根因，不替代元数据修复。

## 修复结果与回传

Open state: `source-repaired_managed-validation-pending`。
Session `failure-roll-01a07160-interface02`，transfer
`f784db39fe9f465d88cbd51764be20bf`，前置快照 3161、定向输入快照 3162。
五份 fixture 与 HEAD 无内容语义差异，已完整阅读并封存精确 bytes，未编辑。
临时验证输入推导只允许这五个明确路径，并要求 current/attribution/ObjectStore
与快照一致；不会从活动工作树任意复制文件。

`zircon_runtime_interface/Cargo.toml` 当前 hash
`10dc9e5aaab96bad9fc56c84746c3e9bd0d31988fdc5faacfaba2d92225131e9`，
由活动 Session `astra-full-domain-20260905`（Astra01，resolving_failure）拥有。
所需三行元数据改动经 `git apply --check` 后提交延迟补丁 155；协调器立即返回
`applied`，将新 hash `b5e30acb07e3e73019945a9bd855b59b4d7a4ca70d9c952971a1063af5669fa4`
归属到 Interface02。请求 `05aa04987d8c491a9a189f98b852372a` 保存原始 base hash
与补丁 object `5fe5417d9a477ac1492b8b01e46c533b88e0cfbd2afd06228666b08aba926fe8`。
后续以该源码和精确五份 fixture 验证自动闭包；当前仍未取得动态结果。

元数据源码快照 3167 已冻结。现有 coordinator build/test dependency-roots 分层测试
1/1 通过；直接用 TOML parser 读取当前 package metadata，并通过现有输入规划 owner
解析声明、枚举受管资源，得到的五份文件及 hash 与快照 3161 完全一致。该检查证明
包声明可被现有闭包 owner 消费，不替代随后的受管 Cargo 动态回归。

补丁服务已转移 attribution，却没有将 Cargo.toml 加入 Session write_scope。补充
`session register` 请求 `d1e0b5c3dfcd476cae9881af82b1cf2f` 终态拒绝
`session_write_scope_immutable`；自转移 preview `dffb7b3b098c48f5b62fba80f6dbfc28`
也因 `path_already_owned_by_target` 拒绝。没有修改协调器数据库或强行转交其他 Session；
这一正式提交范围阻断待 Tooling01 修复后闭环，源码与归属证据保持原样。
未 failure return、未 commit、未发送企微。

受管 Windows locked/static 作业 `971de0c483af45aaabfb499a48e2807e` 已实际通过
全部八个 manifest_summary 测试，原始 v1/v2/v3/future/invalid 三个失败转绿。
输入 `interface-project-contracts-red-3167-20260908`，manifest
`4a1f09d59338ae525a09dba4becf2c02fa7e4ac64135a0cce1ad7bc7c040f3e3`；
完整结果 751 passed、12 failed、101 ignored，日志为 `results/interface-library-red-3167.log`。
同源码后继作业 `dd18d653fe3c494c91fbe57f5959580a` 保持八项通过。
关联 [Tooling01 patch 范围回传](fixed-2026-09-08-patch-attribution-write-scope-drift.md)
拥有历史 patch 155 的 scope 恢复；本项在独立审查和正式提交条件补齐前保持 open。

Independent review in the existing task "优化协调器验证效率" completed on
2026-09-08 with C0/I0/M0. Report:
`.codex/tmp/interface02-runtime09-3173-review-20260908-result.txt`.
All ten effective Interface02 input paths matched their attributed snapshots.
Patch155 scope recovery completed through reviewed coordinator integration
`695e4e58987a1ac82111ac700f4e23b1ba52d74e` and ownership apply
`c3b2d8a27ec04657842535908f2b0b53`. The original manifest hash is unchanged and
the same Interface02 Session now owns its write scope. Fresh formal automatic
fixture closure remains pending; explicit-input fixture passes do not replace it.


## 2026-10-03 独立 Windows 验证终态与下层编译阻塞

当前执行采用用户确认的独立 Windows 证据流程，原协调器票据及历史归属保持为历史证据。本条仍为 **open**；未生成 fixed 回传、commit SHA 或通知结果。

- 自动闭包捕获实际得到 1,216 个文件、5 个由真实包 metadata 发现的 fixture、17 个内嵌模板、2 个本地编译包及 9 个外部 metadata 输入；源快照前后及 D 盘副本均一致。实际捕获回执 SHA256 为 `314f079d5f0b868071eb142df3362313c354172bc37db1321c78771b4fe12aa4`。
- 实际执行 `cargo +1.94.1 check -p zircon_runtime_interface --no-default-features --locked`，工具附加合规 D 盘 `--target-dir`，完整 package check 退出 0；执行前后输入守卫无差异，独立部分接收 C/I/M=0/0/0。
- 原 `project::tests::manifest_summary::` 八项过滤已实际提交 Cargo；lib test 编译在 `zircon_runtime_interface/src/tests/boundary.rs:189` 报 E0277：`needle` 是 `&str`，`REVIEWED_COMPILE_TIME_INPUTS.iter()` 的闭包变量 `reviewed_needle` 是 `&&str`。Cargo 退出 101，八项测试实际执行 **0**。Python roots 回归因失败停止尚未启动。
- 本次 Root 执行 ID 为 `4454`，终态退出 1；整批 `passed=false` / `allFixtureStagesPassed=false`。六份保存的输入守卫无漂移，末尾输入核对通过；Root 与下层子进程全部终态，pending/unknown 均为 0。整批失败终态 SHA256 为 `bf32f0c037a257393543a0ba51ad638dd4b24b5c1bc37aabc4fa7e9ef6425500`。
- [失败回执接收](../../../../../.codex/tmp/failure-roll-20261003-interface02-complete-failed-run-reception-r1/compactreview.json) SHA256 `ee60b8ee3e64e260356071208eb8c6009e453613b9e544b47a82f5a6bc57dd5a` 的 C/I/M=0/0/0 仅确认上述失败证据准确，未接受本条完整验收。
- [下层根因记录](../../../../../.codex/tmp/failure-roll-20261003-interface02-boundary-test-compilation-diagnosis-r1/terminal.json) 和 [私有一字节补丁](../../../../../.codex/tmp/failure-roll-20261003-interface02-boundary-test-compilation-diagnosis-r1/one-token.patch) 将比较改为 `needle == *reviewed_needle`。候选仅插入 `*`，独立静态审查 C/I/M=0/0/0；移除此字节可完整恢复现行业务与注释字节。当前源和冻结副本均为诊断记录绑定的 `039739…` / 19,926 字节，候选后像为 `429dd165818cba0e83b8c6b3461ff534e084405b3586a5999ec921028050f0d5` / 19,927 字节。
- 该助手属于预先存在的业务增量；当前实际业务写入者和采用边界尚未确认。旧审计记录的零注释增量未证明当前业务归属。补丁保持私有，Main、冻结 D 副本、原请求及原失败回执均未修改。

下一步先协调该精确源范围，再执行现有下层 `reviewed_boundary_exceptions_stay_path_scoped` 回归，随后完成原 package check、全部八项 summary 和原 Python roots 回归及前后完整守卫。独立 Python 下层验证将使用另一个明确限定范围的新请求；其结果不替代本批次失败或完整向上验收。完整 origin Interface library、其他 Runtime/SDK 与 Jenkins 验收仍分别开放。

元数据首个 tests 命令仅补单引号，使其 YAML 值为字符串；命令内容、生命周期、日期、根因及全部原历史正文保持原样。


## 2026-10-03：观察层修复与独立 Python 下层的源码阻断

原整批因 Rust 编译 101 而停止，原 Python1 未启动。Root 随后单独验证原 Python 用例；这些独立尝试不续接旧整批，也不替代 Rust8 或向上验收：

| 尝试 | 实际结果 | 目标用例数 |
| --- | --- | --- |
| lower-run-r1 | admission 的运行文件映射不匹配，退出 1；没有创建子进程 | 0 |
| lower-run-r2 | 私有观察器在加载原模块时记录 TypeError，退出 1；三个自有子进程（两个工具探针和一个观察器）均终态，源码守卫通过 | 0 |
| observer-repair-run-r1 | 新观察层执行前的 currentOriginal 守卫不匹配，退出 1；在 fresh environment 前停止，未创建子进程 | 0 |

lower-run-r2 的实际错误为 `function() argument 'code' must be code, not str`。静态源码链显示，旧私有观察层把 `subprocess.Popen` 类换成函数，原模块的 `unittest.mock` 导入会经 Windows asyncio 定义 `class Popen(subprocess.Popen)`。旧回执没有 traceback，因此这里只记录源码链与保存错误一致。新版私有观察层保留类继承，并在 `__new__` 拒绝直接及普通派生类的未观察构造，已通过独立实施审查 C/I/M 0/0/0；原用例、11 个导入来源、真实 Git 子进程及所有源码守卫保持。

Root 的私有 primitive 实际执行 3/3，失败、错误和跳过均为 0：直接构造与普通派生类构造在 initializer 前被拒绝，真实 Windows 标准库 mock/asyncio 继承链成功导入，Popen 最终还原。此结果仅验证观察层，未执行目标 Python1、Rust、Cargo 或 VM，不能作为本 lifecycle 的动态通过。

新尝试的 before-fresh-environment 和 after-original-python-closeout 保存观察一致：1216 个文件无新增或删除，仅 `zircon_runtime_interface/src/ui/design_tokens.rs`、`ui/design_tokens/cascade_registry.rs` 和 `ui/design_tokens/chrome.rs` 的哈希及文件身份不同，总大小增加 128 bytes。其余 12 类非文件元数据和 D 冻结副本不变。三份新业务后像的作者、adopter 及改动性质没有由这些保存观察证明；不得据此替换或采用。其他八组保存数据完成且无错误，但工具镜像集合为空、环境为 null 是 PRE-PROBE 状态，不能计作工具或环境动态通过。

`src/tests/boundary.rs` 在这两份保存观察中仍为 19926 bytes 的旧编译失败前像，未安装一个 `*` 的私有候选。一次限定父会话日志核查已证明历史 Main 安装者为 Hub R3 parent Root `01a0f033-ce80-77e2-86f0-eed5bfbb5926`，实际发布调用于 2026-10-03 08:47:54 UTC 退出 0；当前可接收路由及此次一字节修复交接尚未证明。历史安装实证不授予当前业务采用或修复权限。

本项挂起，保留原始请求、冻结副本和全部失败回执。后续先核对业务归属和现行输入，再冻结新的精确快照，完成下层 boundary 回归及 package check / Rust8 / 原 Python1 的最小完整批次。当前状态仍为 open；没有 failure return、canonical fixed 文件、提交 SHA 或通知。

当前证据（SHA-256，均保留旧版本）：

- 新观察层实施审查：`.codex/tmp/failure-roll-20261003-interface02-python-observer-repair-independent-review-r1/review.json`，`502809fcc8ebe04f29ee9eec74e4243914d2fce53316c247429c8c37bdd93e18`。
- 私有 primitive：`D:/cargo-targets/zircon-local/windows/failure-roll-20261003-interface02-observer-primitive-run-r1/terminal.json`，`e5096d16d222e67309d019cbdf73cdcfbceacaca5668082814f7cbb474ea6cba`。
- 原用例执行前阻断：`D:/cargo-targets/zircon-local/windows/failure-roll-20261003-interface02-lower-python-observer-repair-run-r1/terminal.json`，`307eb2231268ff68098caff13f639c576526faed5ba7ab1fcbaaefbdd00d0e33`。
- 保存差异诊断：`.codex/tmp/failure-roll-20261003-interface02-observer-repair-guard-drift-reception-r1/terminal-r2.json`，`fdad5d744e3388ab0cef27e089adbfb62937ce88449fc8e9ed10ccabc18e73c3`；对应证据审查 `review-r2.json`，`fa5b8777d4e7c07fb503d0339310931eda66e6acbc93a4f82db6507cb54c6788`，C/I/M 0/0/0 仅表示保存证据一致。
- 历史 Main 安装核查：`.codex/tmp/failure-roll-20261003-interface02-boundary-current-writer-routing-ro-r2/terminal.json`，`3369fbe56f7e04bd85975d189b8f63e37189305589c63d0d2f85acc98735d208`。
