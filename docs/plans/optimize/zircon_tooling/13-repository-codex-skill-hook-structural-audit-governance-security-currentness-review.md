---
related_code:
  - .codex/config.toml
  - .codex/hooks.json
  - .codex/hooks/pre_tool_use_cargo_guard.py
  - .codex/hooks/zircon_session_sync.py
  - .codex/skills/project-skills-index/SKILL.md
  - .codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md
  - .codex/skills/project-skills-index/scripts/list-skill-tree.ps1
  - .codex/skills/project-skills-index/scripts/list-skill-tree.sh
  - .codex/skills/zircon-dev/SKILL.md
  - .codex/skills/zircon-dev/agents/openai.yaml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/workflow/SKILL.md
  - .codex/skills/zircon-dev/workflow/testing/mod-rs-map.md
  - .codex/skills/zircon-engineering/SKILL.md
  - .codex/skills/zircon-project-skills/SKILL.md
  - .codex/skills/zircon-project-skills/development-conventions.md
  - .codex/skills/zircon-project-skills/milestone-first-workflow-policy.md
  - .codex/skills/zircon-project-skills/cross-session-coordination/SKILL.md
  - .codex/skills/zircon-project-skills/zr-module-boundary-discipline/SKILL.md
  - .codex/skills/zircon-project-skills/zr-module-boundary-discipline/references/binding-rs-anti-pattern.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
  - tools/check-conventions.ps1
  - tools/check_conventions.py
  - tools/install-codex-session-hook.ps1
  - tools/session_coordinator/codex_sync/hook.py
  - .github/workflows/ci.yml
tests:
  - tools/session_coordinator/tests/test_cargo_guard.py
  - tools/tests/codex-session-hook.Tests.ps1
  - tools/tests/test_editorui10_test_file_budget_contract.py
  - tools/tests/test_hard_cutover_migration_smells.py
  - tools/tests/test_non_network_server_naming.py
  - tools/tests/test_runtime_api_boundary.py
  - tools/tests/test_runtime_asset_pipeline_audit.py
  - tools/tests/test_runtime_ecs_kernel_data_audit.py
  - tools/tests/test_runtime_input_stack_audit.py
  - tools/tests/test_runtime_job_system_audit.py
  - tools/tests/test_runtime_module_family_boundary.py
  - tools/tests/test_runtime_performance_hotpath_boundary.py
  - tools/tests/test_runtime_schedule_frame_loop_audit.py
  - tools/tests/test_runtime_script_binding_audit.py
  - tools/tests/test_runtime_tech_stack_boundary.py
  - tools/tests/test_runtime_ui_architecture_boundary.py
plan_sources:
  - docs/plans/milestone-validation-policy.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BgGraphBuilder.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BgNodeExecutor.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/TempStorage.cs
  - dev/UnrealEngine/Engine/Source/Developer/AutomationController/Private/AutomationReport.cpp
  - dev/bevy/tools/ci/src/ci.rs
  - dev/bevy/tools/ci/src/commands/mod.rs
  - dev/bevy/tools/ci/src/commands/test.rs
  - dev/Fyrox/.github/workflows/ci.yml
  - dev/godot/tests/test_main.cpp
  - dev/godot/tests/test_macros.h
  - dev/Graphics/.yamato/postprocessing-win-dx12.yml
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 13 · Repository Codex Skill、Hook 与结构审计治理、安全、时效工程化差距

## 1. 结论

`.codex`不是普通说明目录，而是会改变代理权限、命令执行、Session同步、验证入口、架构决策和完成判定的仓库本地控制面。本轮对243个tracked文件、34,889行、1,559,270 bytes完成物理清单和分簇审查；其inventory fingerprint为`b5f4012081bc37375d7e3c8f451b16c6f07bdcf417474f694fe6bea5189cf178`。其中239个文件属于skills，111个是可执行Python/PowerShell/Shell脚本，`zr-runtime-interface-convergence`单个skill已增长到91文件、15,552行、643,380 bytes。

当前控制面有大量值得保留的工程基础：Windows managed validator、Session Coordinator hook、目录化skill、reference-engine routing、结构审计规则和外部Python合同测试都已存在；96个tracked Python文件可通过AST解析，12个PowerShell文件无parser error，3个Shell脚本通过`bash -n`，21份agent YAML和`hooks.json`也可解析。问题不是“完全没有治理”，而是治理定义、执行、结果与安全权限尚未形成同一个可证明系统。

最严重的安全边界是tracked `.codex/config.toml`把`approval_policy`固定为`never`、把`sandbox_mode`固定为`danger-full-access`。仓库内容因此试图替用户或执行环境授予最高权限；与此同时Cargo guard与五个Session lifecycle hook都显式fail-open。前者只接受`tool_name == Bash`的payload并用命令字符串正则判断，后者捕获任意异常后返回0。规则声称Cargo和跨Session协调是强制约束，执行层却允许命名、别名、解释器、payload或依赖异常静默旁路。

第二个硬阻断是工程规范双真源。`.codex/skills/zircon-project-skills/development-conventions.md`与CI读取的`docs/plans/zircon_runtime/frameworks/development-conventions.md`同为169行却有不同hash，并已在三个关键MUST规则分叉：`KernelError`/`CoreError`、旧`ZrByteSlice/ZrOwnedByteBuffer` ABI/当前不可复制`ZrOwnedResultV2 + opaque allocation id + table release`、九个D/E/F受管根/普通共享`CARGO_TARGET_DIR`。代理读取第一份，`tools/check_conventions.py`只验证第二份，因此旧ABI和旧构建政策可被代理当作权威，而CI对另一份文档保持绿色。

第三个硬阻断是结构审计没有门语义。Runtime aggregate实测114.6秒、输出673,171字符JSON，明确报告`runtime_naming_boundary=blocked`、多个`migration-debt-present`、missing source/anchor和native surface差异，却固定返回0；Editor aggregate实测8.6秒、报告30项migration debt，也固定返回0。19个外部结构测试模块运行50个用例耗时180.597秒，当前12个失败，外层命令在182.1秒超时；CI只运行dependency governance与两个convention runner测试，不运行aggregate或这19个模块。故“运行了audit”与“通过工程门”目前没有可机读关系。

目录发现也已漂移。实际有11个顶层skill目录和53个`SKILL.md`，缓存catalog的浅树漏掉6个动画/设计顶层skill，只列10个summary；12个嵌套`SKILL.md`没有frontmatter，21个skill有agent YAML、32个没有，仓库没有manifest说明哪些只是章节、哪些是可独立触发的skill。catalog还声称`zircon-engineering`带static contract validator，但该目录只有一个tracked `SKILL.md`和空的physical `scripts/`目录。

本篇不重复Tooling01的Cargo/toolchain通用实现、Tooling06的Coordinator数据库/lease/process/Git协议、Tooling10的全语言Test Service，也不重复Runtime/Editor专题自身的production差距。本篇只拥有repo-local AI/automation trust boundary、skill registry/currentness、hook enforcement、structural audit engine与CI接线。本轮登记 **6项P0、72项P1和16项P2**。

## 2. 审查边界与物理清单

### 2.1 Tracked控制面

| 子域 | 文件 | 行数 | bytes | inventory fingerprint / 当前角色 |
|---|---:|---:|---:|---|
| `.codex/config.toml` | 1 | 7 | 121 | repo-local权限与feature开关；当前授予never/danger-full-access |
| `.codex/hooks.json` | 1 | 80 | 3,201 | 5类Session sync + 1类PreToolUse Cargo guard |
| `.codex/hooks` | 2 | 181 | 5,856 | command guard与Coordinator sync adapter；`bb872d...71cbac9` |
| project-skills-index | 10 | 329 | 17,108 | filesystem listing、cache与scaffold；`a3d4ac...73ee7` |
| superpowers | 32 | 3,467 | 117,224 | generic plan/TDD/debug/review/delegation workflow |
| zircon-dev | 15 | 4,314 | 190,235 | Windows validator、workflow、Cargo target policy；`b7daf6...8f57a6` |
| zircon-engineering | 1 | 39 | 3,378 | MVP gate和specialist routing；无tracked validator |
| zircon-project-skills | 172 | 24,616 | 1,110,987 | repo architecture/execution/validation/coordination rules |
| runtime-interface-convergence子树 | 91 | 15,552 | 643,380 | 2 aggregates、87 audit scripts、refs与UI metadata；`0c65da...54c06` |
| 其他视觉/动画顶层skill | 9 | 1,856 | 111,160 | UI/motion glossary、audit与standards |
| **合计** | **243** | **34,889** | **1,559,270** | **inventory SHA-256 `b5f401...cf178`** |

`.codex/plans`、`.codex/sessions`、`.codex/state`、`.codex/validation`和Python `__pycache__`受根`/.codex/*` ignore影响，不属于上述tracked source。本机存在大量历史plan、coordinator state和validation artifact；它们只作为Tooling06/07拥有的local evidence/state观察，不作为本篇source truth或clean-clone能力证据。

### 2.2 Skill与脚本形状

| 项目 | 实测 | 工程含义 |
|---|---:|---|
| 顶层skill目录 | 11 | 都有根`SKILL.md` |
| 全部`SKILL.md` | 53 | 12个嵌套文件无name/description frontmatter |
| `agents/openai.yaml` | 21 | YAML可解析；没有schema/version/optional-role registry |
| executable script | 111 | 96 Python、12 PowerShell、3 Shell；24,632行/1,006,207 bytes |
| colocated test-named script | 4 | validate-matrix、closeout、handoff三类；不代表其余107个无外部测试 |
| runtime audit外部consumer | 19 Python module | 放在`tools/tests`，没有统一suite manifest或CI selection |
| catalog summary | 10 | 不覆盖11个顶层目录或53个skill入口 |
| catalog更新时间 | 2026-08-03 | 无source tree digest、generator version或CI currentness gate |

111个脚本的语法清洁度可保留，但语法可解析不等于规则正确、结果可作为gate或clean clone可重现。ignored目录内还存在一份231行的旧`.codex/skills/zircon-dev/scripts/WindowsPathResolver.psm1`，与tracked `tools/WindowsPathResolver.psm1`的980行实现hash不同；正式validator已导入tracked tools owner，这份隐藏副本仍会误导本机维护者和ad-hoc调用。

### 2.3 动态审计证据

| 命令/检查 | 耗时/规模 | 结果 |
|---|---:|---|
| Runtime aggregate `--json` | 114.6s / 673,171 chars | exit 0；gate blocked/debt，多个missing与count mismatch |
| Editor aggregate `--json` | 8.6s / 6,159 chars | exit 0；30项debt、28个oversized tests、1 duplicate tree、1 UI owner violation |
| 19个结构test module | 180.597s / 50 tests | 38 pass / 12 fail；外层182.1s timeout |
| Cargo guard unit tests | 0.086s / 6 tests | 6 pass；测试明确把non-Bash payload视为allow |
| closeout evidence test | 3.231s / 1 test | error：固定地址端口冲突`WinError 10048` |
| Python AST | 96 files | 0 error |
| PowerShell parser | 12 files | 0 error |
| Shell `bash -n` | 3 files | 0 error |
| YAML/JSON parser | 21 YAML + 1 JSON | 0 error |

12个结构失败不是同一个重复根因：包括5个未分类hard-cutover smell、Runtime API新增`frame_demand/highlight_set/session`未同步、70个未分类editor命名位置、Input/Job/Module Family/Performance guard漂移、render legacy debt、Runtime06 inventory/anchor漂移以及UI新增`text_artifact`未同步。这些结果证明审计确实能发现漂移，也证明当前没有required lane消费它们。

## 3. 参考引擎约束

- Unreal BuildGraph把graph、node executor、typed task、temp storage和artifact/report task分开；AutomationReport保存稳定test path、tags、enabled/filter/exclusion与结果树。Zircon不必复制C#或Horde，但policy、action、execution、result和artifact不能混在自由文本/anchor regex里。
- Bevy把CI写成独立typed command tool，format/clippy/compile/test/integration/doc/example/bench各自有命令owner；命令失败通过进程结果传播，`test`还显式覆盖workspace lib/bin/tests与bench smoke。Zircon aggregate不能在blocked时仍返回0。
- Fyrox在Linux/Windows/macOS执行workspace all-target/all-feature build/test，并把format、clippy、docs、WASM、PC/Android/WASM template生成分成required jobs。repo-local开发规则和CI环境差异必须由同一matrix声明，而不是两份Markdown各说一套。
- Godot test runner按tag初始化和释放Display/Audio/Navigation/Physics/Editor singleton，真实runner拥有环境生命周期。结构规则若需要runtime/editor语义，应调用typed owner或fixture，不能只查字符串存在。
- Unity Graphics Yamato job固定agent/GPU/API/colorspace/suite/test project/timeout/dependency，并发布logs/test-results/player artifact。审计结果必须带source/tool/rule inventory/环境/artifact identity，不能只打印一份无schema的大JSON。

这些参考实现共同表明：工程治理也需要owner、version、action graph、typed result、失败退出、artifact/currentness和最小权限；“有很多脚本”和“代理被告知必须遵守”不能替代执行证明。

## 4. 可保留的正确基础

1. Repo skill已按parent/leaf/reference/script逐步拆分，绝大多数入口有明确trigger与渐进式读取规则。
2. project skill tree listing脚本能从真实filesystem列出11个顶层目录和frontmatter summary，不依赖cache才能发现新入口。
3. Windows validator、Coordinator、path resolver和closeout/failure handoff已有独立实现与部分fixture测试。
4. Cargo guard不记录完整命令文本，denial log只保存session、相对cwd、subcommand和reason，降低secret泄漏面。
5. Runtime audit把boundary、inventory和Markdown renderer拆成模块，并为19个规则族提供外部Python测试基础。
6. Runtime/Editor aggregate都支持JSON输出，后续可迁到versioned FindingSet，而不必解析人类Markdown。
7. CI已有convention runner入口、toolchain pin与workspace build/test基础，可增加快速governance lane。
8. reference routing明确区分Unreal/Fyrox/Bevy/Godot/Unity Graphics适用域，避免只参照一个引擎。
9. 本轮语法检查证明tracked控制面没有基础parse blocker，可直接从语义、接线和安全边界开始收敛。

## 5. P0：控制面可信性硬阻断

### CODEX-CONTROL-P0-001 · 仓库内容自行授予never-approval与danger-full-access

tracked `.codex/config.toml`把安全权限作为项目默认值提交。打开、切换或审查不可信revision时，仓库内容不应拥有扩大文件系统、网络和命令执行权限的authority。安全权限必须由用户/组织/runtime policy在repo外决定；仓库只能声明需要哪些能力，并在缺少能力时降级或拒绝具体operation。

### CODEX-CONTROL-P0-002 · Agent规范与CI规范在ABI和构建政策上形成双真源

两份169行`development-conventions.md`分别被agent与CI消费，hash不同且三条MUST规则冲突。尤其PL-4会把跨DLL输出所有权导向已经过时的carrier，而当前Interface01/RuntimeHost05要求不可复制owner、opaque allocation与table-level release。必须选定单一machine-readable RuleRegistry，由skill展示和CI gate从同一生成物投影；双份正文不能继续独立编辑。

### CODEX-CONTROL-P0-003 · Aggregate明确blocked仍固定exit 0

Runtime与Editor aggregate在所有分支末尾返回0，JSON/Markdown中的`blocked`或`migration-debt-present`不影响进程状态。任何CI、agent或人工脚本只检查exit code都会把红门当成功。必须定义rule severity与gate policy，required blocker返回非零，同时保留`--report-only`显式模式；不能靠调用方再解析几十种字段名猜测失败。

### CODEX-CONTROL-P0-004 · 当前12个结构合同失败未进入required CI

19个外部module的50项test当前有12项失败，CI只运行另外3个Python module与`tools/check_conventions.py`。这使Runtime API、UI、render naming、hotpath、job/input/module family等已知结构漂移长期不影响main资格。先恢复当前red baseline的owner/expected-currentness，再把完整suite按快速/慢速lane接入；不得删除失败测试或放宽expected list制造绿色。

### CODEX-CONTROL-P0-005 · Cargo guard是可旁路、fail-open的字符串过滤器

guard仅在event/tool恰为`PreToolUse/Bash`时工作，测试明确允许其他tool name；它只对command string做Cargo和D/E/F literal正则，alias、变量、包装器、新shell tool、payload缺字段和内部异常都放行。若受管target/lease是强制安全约束，必须在真正创建Cargo process或分配artifact root的typed owner执行，hook只能作早期UX提示。

### CODEX-CONTROL-P0-006 · Session lifecycle hook吞掉所有同步故障

五类sync入口统一`except Exception: return 0`，除Stop外没有degraded output；5秒timeout、Python import、spool、Coordinator unavailable或protocol drift均可静默发生。此时技能仍要求按Session/lease/failure graph协调，使用者却不知道控制面已失效。需要typed health/admission、bounded retry与durable degraded receipt；只允许明确声明的read-only工作在degraded模式继续。

## 6. P1：Trust、配置与Hook协议

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-001 | config无schema/version | 增加project capability schema，只声明需求，不授予权限 |
| CODEX-CONTROL-P1-002 | 安全设置与feature flag同文件 | user security policy、repo feature request、session grant三层分离 |
| CODEX-CONTROL-P1-003 | hooks.json无protocol/version | 每个event声明payload/result schema与兼容范围 |
| CODEX-CONTROL-P1-004 | hook依赖`py -3`/`python3`当前环境 | 发布受支持Python范围、dependency lock与preflight |
| CODEX-CONTROL-P1-005 | 3/5秒timeout没有timeout receipt | 输出typed timeout/degraded reason并记录attempt identity |
| CODEX-CONTROL-P1-006 | hooks.json与installer复制完整定义 | installer从单一manifest生成并验证，而非复制字符串 |
| CODEX-CONTROL-P1-007 | installer按exact managed definition删除 | 使用owner ID/schema/version迁移，保留未知用户hook |
| CODEX-CONTROL-P1-008 | guard只识别Bash tool name | shell/tool surface由host capability枚举，不硬编码一个名称 |
| CODEX-CONTROL-P1-009 | guard解析命令文本而非process plan | 在ProcessSpec/CargoAction层验证program、args、cwd、env和lease |
| CODEX-CONTROL-P1-010 | managed root只匹配命令内D/E/F literal | 对resolved final path和lease receipt验证，不信任显示字符串 |
| CODEX-CONTROL-P1-011 | denial log无rotation/retention | 设size/time quota、atomic append、corruption recovery和redaction gate |
| CODEX-CONTROL-P1-012 | hook没有metrics/health query | 发布event count、latency、timeout、bypass、last-success与schema mismatch |

## 7. P1：Skill Registry、路由与规范真源

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-013 | 53个SKILL没有统一registry | 建立SkillManifest：ID/version/owner/trigger/platform/dependency/conflict/test |
| CODEX-CONTROL-P1-014 | 12个嵌套SKILL无frontmatter | 明确标为chapter或补齐独立skill metadata，禁止同名文件双语义 |
| CODEX-CONTROL-P1-015 | 21有agent YAML、32没有 | registry声明UI metadata optional/required及缺失原因 |
| CODEX-CONTROL-P1-016 | YAML只有display/description/prompt | 加schema version、skill ID、compatible host、required capability |
| CODEX-CONTROL-P1-017 | catalog浅树漏6个顶层skill | catalog必须从filesystem生成并在diff中fail stale |
| CODEX-CONTROL-P1-018 | catalog只写10个summary | 要么完整索引53入口，要么明确只索引top-level并移除混合child摘要 |
| CODEX-CONTROL-P1-019 | catalog只有手写Updated日期 | 记录tree digest、generator version、source revision |
| CODEX-CONTROL-P1-020 | list脚本输出不与cache比较 | 增加`--check`，Windows/Linux输出规范化后字节一致 |
| CODEX-CONTROL-P1-021 | zircon-engineering被描述有validator但不存在 | 落地tracked validator或删除虚假能力声明 |
| CODEX-CONTROL-P1-022 | skill依赖和优先级只写在 prose | 建立DAG与override规则，检测cycle/conflict/不可满足platform |
| CODEX-CONTROL-P1-023 | generic superpowers与repo规则冲突靠人工记忆 | machine resolve branch/worktree/delegation/TDD/validation precedence |
| CODEX-CONTROL-P1-024 | agent prompt硬编码main/no feature branch | 根据当前组织policy和session branch capability投影，不在prompt冻结 |

## 8. P1：规范内容、路径与Currentness

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-025 | 双convention只有人工复制 | 单一RuleRegistry生成docs、skill摘要和CI inputs |
| CODEX-CONTROL-P1-026 | Rule行没有generation/effective version | 每条规则有owner、introduced/superseded、applies-to selector |
| CODEX-CONTROL-P1-027 | PL-4 agent侧仍是旧foreign buffer合同 | 由Interface ABI schema生成，不允许skill手写carrier |
| CODEX-CONTROL-P1-028 | WF-3两份构建政策冲突 | 统一本地/CI/Windows/WSL例外和lease要求 |
| CODEX-CONTROL-P1-029 | RT-3错误类型名称漂移 | 从实际public API/architecture manifest生成锚点 |
| CODEX-CONTROL-P1-030 | `mod-rs-map.md`把不存在目标树写成现状 | 区分current inventory与target architecture，附迁移状态 |
| CODEX-CONTROL-P1-031 | binding anti-pattern仍以已删除单文件为当前主语 | 标为dated case study并链接现有`ui/binding/mod.rs` owner |
| CODEX-CONTROL-P1-032 | docs维护示例路径与真实tree混写 | sample使用显式`example://`或fixture，不进入path currentness |
| CODEX-CONTROL-P1-033 | `.codex/plans/.sessions`兼容fallback无代际 | 由Coordinator导出typed compatibility view并标记expiry |
| CODEX-CONTROL-P1-034 | reference engine只记录路径不记录revision | 路由结果绑定subtree revision/license/scope snapshot |
| CODEX-CONTROL-P1-035 | 规则没有source drift invalidation | owner/API/path digest变化自动把规则标成needs-review |
| CODEX-CONTROL-P1-036 | 没有全skill dead-link/currentness gate | 对root path、relative resource、command、example分别typed验证 |

## 9. P1：Structural Audit Engine

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-037 | 91文件/15,552行仍是单skill内私有系统 | 升格为owned tooling package与稳定CLI/API |
| CODEX-CONTROL-P1-038 | boundary与30个Markdown renderer并行增长 | rule输出统一Finding，renderer只消费schema |
| CODEX-CONTROL-P1-039 | 大量规则用substring anchor | Rust/Cargo/TOML/Markdown使用parser/AST/schema query |
| CODEX-CONTROL-P1-040 | expected file/module/count写死在Python | 从owner manifest和typed inventory生成expected set |
| CODEX-CONTROL-P1-041 | workspace members用regex解析Cargo.toml | 使用Cargo metadata或TOML parser，处理workspace inheritance/exclude/glob |
| CODEX-CONTROL-P1-042 | 同一文件在多个规则重复read/split | 一次构建immutable SourceSnapshot并共享index |
| CODEX-CONTROL-P1-043 | aggregate 115秒无阶段耗时 | 记录rule scan time、files/bytes/cache hit与预算超限 |
| CODEX-CONTROL-P1-044 | 673KB JSON无分页/压缩/artifact owner | summary与detail artifact分离，detail content-addressed |
| CODEX-CONTROL-P1-045 | aggregate JSON无schema version | FindingSet/RuleSet/SourceSnapshot全部versioned |
| CODEX-CONTROL-P1-046 | 结果无source/build/tool fingerprint | 每次run绑定tree、dirty digest、Python/tool/rule inventory |
| CODEX-CONTROL-P1-047 | 各boundary字段名任意 | 统一severity/status/owner/location/evidence/remediation/suppression |
| CODEX-CONTROL-P1-048 | 没有rule-level stable ID | ID独立于文件名和renderer，rename不丢历史趋势 |

## 10. P1：规则质量、范围与结果语义

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-049 | `blocked`/debt/risk/missing/count需调用方猜测 | RuleResult统一pass/fail/error/skipped/waived |
| CODEX-CONTROL-P1-050 | 无severity到exit policy映射 | required P0/P1 fail，advisory独立展示，report-only显式开关 |
| CODEX-CONTROL-P1-051 | exact module count会惩罚合法新增 | 检查owner registration/schema coverage，不冻结总数 |
| CODEX-CONTROL-P1-052 | source snippet存在即可满足行为anchor | 行为合同必须由compile/test/query证明，文本只作diagnostic |
| CODEX-CONTROL-P1-053 | audit读取历史plan/acceptance字符串 | 历史文档退出current gate，改读owner manifest/receipt |
| CODEX-CONTROL-P1-054 | Runtime/Editor/Tool/Plugin范围混在一个scan | rule声明scope selector与canonical owner，避免跨域误报 |
| CODEX-CONTROL-P1-055 | Runtime aggregate很深、Editor只一类boundary | 用统一rule registry逐域补齐，不接受不对称“全量”命名 |
| CODEX-CONTROL-P1-056 | 无rule dependency order | inventory/schema先行，derived rule只消费已成功lower layer |
| CODEX-CONTROL-P1-057 | 无baseline diff | 输出new/fixed/persisting/regressed与first-seen generation |
| CODEX-CONTROL-P1-058 | 无suppression schema | waiver必须有owner/reason/expiry/scope/source generation |
| CODEX-CONTROL-P1-059 | parse/read异常可能变成进程trace或静默缺失 | typed infrastructure error且required scan fail-closed |
| CODEX-CONTROL-P1-060 | 无SARIF/JUnit/annotation adapter | 同一FindingSet投影CLI、CI annotation、dashboard和plan evidence |

## 11. P1：测试、CI与可重现执行

| ID | 当前差距 | 重构要求 |
|---|---|---|
| CODEX-CONTROL-P1-061 | 111脚本只有4个colocated test-named文件 | manifest登记每个executable的unit/integration/smoke owner |
| CODEX-CONTROL-P1-062 | 19个外部audit tests靠路径扫描发现 | 建立显式governance TestPlan和selection receipt |
| CODEX-CONTROL-P1-063 | 50 test耗时181秒 | 共享snapshot、fixture unit与repo integration分层，设时间预算 |
| CODEX-CONTROL-P1-064 | 当前12 fail无quarantine/owner | 每项绑定owner和failure generation；required仍红，不静默忽略 |
| CODEX-CONTROL-P1-065 | outer timeout与suite完成相撞 | runner保留partial result、graceful terminate和artifact flush |
| CODEX-CONTROL-P1-066 | closeout test固定端口冲突 | 绑定port 0/ephemeral endpoint并完全隔离并行实例 |
| CODEX-CONTROL-P1-067 | Python/PS/Shell/YAML语法检查不在CI | 增加快速跨平台control-plane lint lane |
| CODEX-CONTROL-P1-068 | Pester/Python/YAML依赖未统一锁定 | 发布tool environment lock与bootstrap receipt |
| CODEX-CONTROL-P1-069 | PowerShell/Shell listing parity未测试 | 同fixture比较排序、Unicode、symlink、empty tree和error exit |
| CODEX-CONTROL-P1-070 | ignored旧PathResolver副本与tracked owner不同 | 删除/隔离本机副本，clean workspace gate拒绝source shadow |
| CODEX-CONTROL-P1-071 | Python运行产生大量ignored pycache | 设置统一cache root/cleanup policy，不污染skill source tree |
| CODEX-CONTROL-P1-072 | CI不发布governance result artifact | 保存summary/detail、rule inventory、source fingerprint与retention |

## 12. P2：可维护性与开发体验

| ID | 改进项 |
|---|---|
| CODEX-CONTROL-P2-001 | 为SkillRegistry生成按domain、trigger、platform和owner可查询的索引 |
| CODEX-CONTROL-P2-002 | catalog只保留生成区，人工说明放稳定前后区，避免全文件重写 |
| CODEX-CONTROL-P2-003 | agent YAML display name/description统一长度、语言和标点规范 |
| CODEX-CONTROL-P2-004 | 为每个skill显示last-reviewed、compatible host和known limitation |
| CODEX-CONTROL-P2-005 | hook错误给出短error code和诊断路径，不回显payload正文 |
| CODEX-CONTROL-P2-006 | guard denial展示正确managed command的结构化替代建议 |
| CODEX-CONTROL-P2-007 | audit CLI支持`--list-rules`、`--only`、`--changed-since`和`--explain` |
| CODEX-CONTROL-P2-008 | summary按owner/severity排序，detail不在终端倾倒数十万字符 |
| CODEX-CONTROL-P2-009 | rule fixture使用最小虚拟tree，测试失败不打印整份大型source |
| CODEX-CONTROL-P2-010 | current/target/example/archive路径在文档显示上使用不同标签 |
| CODEX-CONTROL-P2-011 | 统一Windows与POSIX显示路径，内部identity使用规范化repo-relative path |
| CODEX-CONTROL-P2-012 | 为stale catalog、missing frontmatter和shadow file提供一键只读doctor |
| CODEX-CONTROL-P2-013 | CI annotation聚合同一rule重复位置，保留完整artifact链接 |
| CODEX-CONTROL-P2-014 | 给slow audit输出阶段progress，避免115秒无反馈 |
| CODEX-CONTROL-P2-015 | 历史rule rename保留alias/tombstone，不让trend dashboard断代 |
| CODEX-CONTROL-P2-016 | 控制面文档统一术语：policy、rule、gate、test、receipt、finding各有唯一含义 |

## 13. 目标架构

### 13.1 Repo Control Plane Manifest

建立单一versioned manifest，至少包含：

- `SkillManifest`：stable ID、version、owner、trigger、platform、dependencies、conflicts、resources、test plan；
- `RuleManifest`：rule ID、severity、scope selector、source owner、applies-to generation、implementation query、waiver policy；
- `HookManifest`：event/payload/result schema、timeout、degraded policy、required host capability；
- `ToolEnvironment`：Python/PowerShell/Shell版本与依赖digest；
- `ControlPlaneBuildId`：repo source、manifest、tool、generated projection的组合digest。

Skill Markdown、agent YAML、catalog、hooks.json和CI selection都是manifest projection，不再各自成为source truth。

### 13.2 Security与Admission

权限所有权必须是：

```text
User / Organization Security Policy
  -> Session Capability Grant
    -> Repo Operation Requirement
      -> Typed Process / File / Network Admission
        -> Execution Receipt
```

Repo不得上调sandbox或approval。Cargo lease、artifact root、Session mutation和external message分别在真正owner处admit；hook只做预检查和UX。Coordinator unavailable时发布`Healthy/Degraded/Unavailable`，operation按风险选择fail-close或read-only继续，不能吞异常后伪装正常。

### 13.3 Structural Audit Service

```text
SourceSnapshot
  -> Cargo/TOML/Rust/Markdown/File-System Index
    -> Rule DAG
      -> FindingSet
        -> Exit Policy
        -> SARIF/JUnit/CLI Summary
        -> Detail Artifact / Trend Store
```

一次scan读取每个文件，parser结果按snapshot digest缓存；rule不直接生成Markdown，也不读取历史acceptance作为current truth。Finding包含rule、severity、owner、location、evidence、first/last generation、remediation和waiver。新增合法module只要求owner/schema coverage，不要求手工修改总数常量。

## 14. 重构里程碑

### M0 · Security与Truth Freeze

- 从tracked config移除权限授予，只保留capability request；
- 指定唯一development convention owner，冻结双真源继续编辑；
- 记录当前12个结构失败，不删除或放宽测试；
- 给aggregate增加显式`--report-only`，默认required debt非零退出。

### M1 · Registry与生成投影

- 落地Skill/Rule/Hook/ToolEnvironment manifest和schema validator；
- 从filesystem/manifest生成catalog与agent metadata；
- 解决12个无frontmatter文件的chapter/skill身份；
- CI验证生成物无漂移、所有resource path存在。

### M2 · Hook与Admission重构

- Session hook使用typed health/result和bounded retry；
- Cargo policy下沉到ProcessSpec/lease owner；
- 覆盖所有shell/process surface及resolved path；
- 增加unavailable、timeout、schema mismatch、parallel session fault tests。

### M3 · Audit Snapshot与Finding Schema

- 统一SourceSnapshot、parser indexes和Rule DAG；
- 迁移Runtime/Editor boundary到stable Finding；
- 增加source/tool/rule fingerprint、timing、baseline diff和waiver；
- CLI支持规则选择与detail artifact。

### M4 · Test与CI接线

- 先用fixture unit覆盖规则语义，再跑repo integration；
- 修复当前12个失败的实际owner或同步合法manifest；
- quick lane执行schema/catalog/hook/unit，full lane执行repo audit；
- 输出JUnit/SARIF/summary/detail，timeout保留partial receipt。

### M5 · Currentness与运营

- source drift自动失效skill/rule review状态；
- reference engine subtree绑定revision和用途；
- 建立rule latency、flake、waiver expiry、finding trend dashboard；
- 清除ignored shadow source与skill tree cache污染。

## 15. 验收门

| Gate | 验收内容 |
|---|---|
| G1 | clean clone不从repo配置获得更高sandbox/approval权限 |
| G2 | Skill/Rule/Hook manifest schema、ID与dependency DAG有效且无环 |
| G3 | 53个现有SKILL全部被明确分类为独立skill或chapter |
| G4 | catalog由manifest/filesystem生成，11个顶层目录与全部入口无漂移 |
| G5 | agent YAML、skill Markdown、CI rules由同一generation投影 |
| G6 | development convention只剩一个canonical owner，ABI/错误/build政策无双值 |
| G7 | hook payload/schema mismatch、timeout、Coordinator down产生typed degraded result |
| G8 | 任意shell/alias/wrapper不能绕过受管Cargo process admission |
| G9 | Runtime/Editor aggregate遇required finding返回非零，report-only必须显式 |
| G10 | SourceSnapshot绑定tree/dirty/tool/rule digest并被所有Finding引用 |
| G11 | 合法新增module由owner registration通过，不需修改脆弱总数 |
| G12 | 历史plan/acceptance文本不再作为current结构测试输入 |
| G13 | 结构规则fixture unit在Windows/Linux一致，语法和schema lane低于30秒 |
| G14 | full audit共享scan/cache，在预算内完成并输出阶段timing |
| G15 | 当前50项结构test全部执行且0 fail/0 omitted；timeout保留partial result |
| G16 | CI发布source-bound JUnit/SARIF/summary/detail artifact并执行waiver expiry |

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| `.codex` tracked physical inventory | review_complete | 2026-08-16 | 243文件、34,889行、1,559,270 bytes；inventory `b5f401...cf178` |
| skill/catalog/schema审查 | review_complete | 2026-08-16 | 11顶层目录、53 SKILL、12缺frontmatter、21 agent YAML、catalog 10 summary |
| hook/config安全审查 | review_complete | 2026-08-16 | never/danger-full-access；Cargo与Session sync均fail-open |
| aggregate动态审查 | review_complete | 2026-08-16 | Runtime 114.6s/673,171 chars exit0；Editor 8.6s/30 debt exit0 |
| 结构test current baseline | review_complete | 2026-08-16 | 50 tests / 180.597s / 12 failures；CI未选择 |
| control-plane syntax baseline | review_complete | 2026-08-16 | 96 Python、12 PS、3 Shell、21 YAML、1 JSON均可解析 |
| Production/tool implementation | pending | - | 本篇只新增review，不修改config、hook、skill、audit、tests或CI |
