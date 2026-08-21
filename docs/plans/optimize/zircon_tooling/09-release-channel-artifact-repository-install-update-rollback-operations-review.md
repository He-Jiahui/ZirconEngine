---
related_code:
  - .github/workflows/ci.yml
  - .github/workflows/mvp-editor-windows.yml
  - .github/workflows/profile-feature-contract.yml
  - Cargo.toml
  - zircon_plugins/Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_hub/package.json
  - zircon_hub/tauri.conf.json
  - zircon_hub/hub.toml
  - zircon_hub/src/build/runner.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/source_engine_paths.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/engines/validation.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/install_receipt.rs
  - zircon_hub/src/projects/local_paths.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - tools/mvp/MvpStagingRelease.psm1
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/install-codex-session-hook.ps1
  - tools/install-session-coordinator-task.ps1
  - tools/install-session-tray-startup.ps1
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/plugin_build_signature.py
  - tools/zircon_export/plugin_build_package.py
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/transport.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/attempts.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/bitmap.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/manifest.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/progress.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/resume.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/state.rs
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
tests:
  - tools/tests/mvp-staging-release.Tests.ps1
  - tools/zircon_export/tests/test_native_dynamic_build_signing.py
  - tools/zircon_export/tests/test_native_dynamic_signing_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_signing_notarization.py
  - zircon_hub/tests/project_source_engine_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildPatchServicesModule.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildInstaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/BuildPatchManifest.h
  - dev/godot/editor/project_manager/engine_update_label.cpp
  - dev/godot/editor/export/export_template_manager.cpp
  - dev/bevy/.github/workflows/post-release.yml
  - dev/bevy/_release-content/README.md
  - dev/Fyrox/project-manager/src/upgrade.rs
  - dev/Graphics/.yamato/wrench/promotion-jobs.yml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/CHANGELOG.md
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09 · Release Channel、Artifact Repository、Install/Update/Rollback 运维工程化差距

## 1. 结论

ZirconEngine 目前没有产品级发行系统。仓库没有Git tag，三个GitHub Actions workflow只响应branch push/pull request或手工MVP验证；唯一`upload-artifact`上传的是保留7天的Windows MVP evidence，而非可安装、可验证、可推广的引擎分发体。根workspace、plugin workspace、Hub Rust crate、Hub前端和Session Tray虽然都写着`0.1.0`，但没有单一version authority、发行候选、channel、不可变release manifest、artifact repository、promotion ledger或支持周期。

Hub的`SourceEngineInstall`只保存source/output path和最近8次本地构建历史，ID是规范化路径的64位FNV。`validate_source_engine()`只确认目录、`Cargo.toml`中的`zircon_runtime` member与`tools/zircon_build.py`存在；build进程退出0后便把可变output目录显示为staged payload。Tauri虽然配置了NSIS bundle，却没有updater plugin、endpoint、公钥或updater artifact。UI文案甚至明确说明remote update service在local v1未启用。因此“Hub可构建源码”不能算“Hub能安装、更新或回滚可信引擎版本”。

当前也没有产品信任链。NativeDynamic export可以按配置执行外部sign/notarize命令，并记录执行前后SHA-256；这是可保留的执行审计。随后生成的`<plugin>.sig`却只是包含文件hash和signing audit字段的TOML旁车，且全仓没有runtime/editor/Hub消费者验证它。产品host、Hub NSIS、engine distribution、release manifest、SBOM/provenance均没有独立可验证签名、证书/时间戳身份、key rotation或revocation。文件内容hash只能证明字节一致，不能证明发布者授权。

本地项目包delivery也不是更新器。`install_package_to_device()`递归复制目录、拒绝既有目标并在失败时删除新目录；receipt逐文件记录SHA-256是正向基础，但manifest用本机安装路径作为resource identity并生成`file://` chunk URL。它没有base/target Build Set、版本兼容、delta、磁盘空间预检、A/B slot、atomic current pointer、安装journal、启动health gate或rollback。content download插件会验证Range、长度和chunk hash，但所有resume bytes只在内存，HTTP固定使用允许非TLS的development policy，也没有签名release metadata或最终原子落盘。

因此本轮给出三个P0：没有可信Release Manifest与promotion authority；没有产品签名/验证信任根；没有原子install/update/rollback状态机。在三者完成前，任何“release ready”“installer ready”“auto update ready”状态都必须fatal，不得用本地release profile、目录存在、进程exit 0、`.sig`旁车或MVP `release probe`代替。

本轮记录3个P0、54个P1和10个P2。未修改生产Rust、Python、PowerShell、workflow、安装目录或Hub状态，只新增审查与索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 范围 | 本轮深度 |
|---|---:|---|
| CI与版本身份 | 3个workflow、根/plugin workspace、Hub/Tray版本声明、Git tag | E3：trigger、artifact、retention、version authority与absence proof |
| Hub Engine/Build | source engine、path identity、registry、validation、build completion、settings/state | E3：发现到staged状态的纵向读取 |
| 安装与更新 | Hub project device install/receipt、Tauri bundle、3个本地工具installer | E3：复制、receipt、cutover/rollback seed与产品边界 |
| 签名与供应链 | NativeDynamic signer/notarizer、plugin hash sidecar与消费者搜索 | E3：命令、audit、hash publication与trust gap |
| 下载与迁移 | runtime net descriptor、content download manager、asset migration transaction | E3：security、resume、integrity、publication；迁移仅复用边界 |
| reference engines | Unreal BuildPatchServices、Godot update/template manager、Bevy release content、Fyrox upgrade、Unity Graphics promotion | E3责任对照，不虚构参考仓库不存在的能力 |

本轮选定47条Git index record，共13,765行、539,986 bytes，Git-index fingerprint为`7c4bb6f21f1bd004ba9b41caa92100a2ba618b003ef0ace0f9f27739323c07cf`。该集合覆盖本报告的主要producer/consumer链，不表示Hub全部web页面、export全部平台实现或runtime网络算法均由本报告重新拥有。

### 2.2 动态与定量证据

| 检查 | 结果 | 可支持结论 |
|---|---|---|
| Git发行历史 | 0个tag；HEAD `52072b2049be5a357cd43e70c64b32c9d1d9e15c` | 当前没有可审计semver/tag promotion历史 |
| workflow扫描 | 3个workflow；0个tag/release/publish trigger；1处7天MVP evidence upload | CI不生产或发布引擎distribution |
| updater/trust owner搜索 | 无Tauri updater、TUF、Sigstore/cosign、minisign、in-toto或release action | 当前无产品更新metadata与签名信任root |
| `.sig`消费者扫描 | 只有export producer/tests；无runtime/editor/Hub读取者 | hash sidecar不是admission control |
| NativeDynamic聚焦测试 | 26 passed / 3.695秒 | 外部sign/notarize执行、platform gate、失败清理与hash审计内部自洽；不验证证书或发布者 |
| MVP release probe | 1个PowerShell合同通过 / 约7秒 | staged目录可完成rename/restore且边界检查有效；它只证明目录handle释放 |

Hub Rust测试本轮未重复运行：Hub 01已经在Windows managed Cargo中复现当前tracked source的`persist_unchecked(None)`/零参数定义`E0061`编译阻断。该已知P0与本报告的发行架构缺口正交，不能通过重复失败增加证据。

### 2.3 正向基线

- Hub device receipt逐文件记录path、bytes与SHA-256，并在复制失败时清理本次新建的owned directory；可演化为安装transaction的payload inventory。
- content download验证Range边界、`Content-Range`、body length和每chunk内容hash；这些是artifact transport的必要底座。
- asset migration已有read-only preflight、dry-run、durable intent journal、staging/flush/commit/recovery与typed issue；它应成为engine upgrade后的project migration gate，而不是重写一套弱迁移器。
- Coordinator task installer已有`preparing/active/rolled_back` cutover record、health check、legacy enable/disable与显式rollback；可抽取事务模式，但不能直接充当产品installer。
- NativeDynamic signing在最终hash manifest前执行外部命令，记录before/after hash、exit/stdout/stderr，并在失败时清理staged payload；应保留审计形状并接入真实trust verification。
- Hub把source build历史限定为最近8条并持久化status/profile/jobs/output/log/command；可作为开发者本地构建历史，不能改名冒充release ledger。

### 2.4 参考边界

- Unreal `BuildPatchServices`以manifest为中心提供installer factory、staging/cloud/backup目录、已安装版本注册、chunk校验/打包/差异、安装状态、pause/cancel、verify/repair、progress/error/statistics与prerequisite。Zircon应学习责任分层与可恢复状态，不照搬其具体chunk格式。
- Godot的Engine Update Label只做版本发现并打开官方下载页，没有假装内置自更新；Export Template Manager又只向官方版本提供mirror下载，并按完整引擎版本隔离模板、支持缺失文件repair。该边界证明“检查更新”“下载组件”“自更新”应是不同能力。
- Bevy用`_release-content`维护本周期release notes/migration guides，post-release workflow从Cargo metadata读取版本、做`-dev`格式sanity check、统一workspace bump并创建PR；它不是binary updater，但提供可审计版本推进与迁移内容门。
- Fyrox project manager读取项目manifest中的Fyrox dependency，允许specific/local/nightly来源并调用template upgrade；它解决项目依赖升级，不代表完整引擎二进制分发。
- Unity Graphics仓库为每个package维护semver与CHANGELOG，Wrench promotion job显式依赖package pack和跨Editor/OS validation，并区分dry-run/publish。它是package promotion参考，不是Unity Hub updater源码。

## 3. 当前P0

### TOOL-RELEASE-P0-001 · 没有可信Release Manifest、候选状态与promotion authority

仓库没有tag、release workflow、immutable candidate或channel catalog。`release`在Hub只表示Cargo优化profile；MVP `release`只表示通过rename确认目录未被进程占用；export report中的成功也未绑定最终distribution。任一本地source checkout只要目录形状正确且build exit 0，就能在Hub中显示staged payload。没有一个authoritative object把source tree、Build Set、toolchain/dependency lock、target/platform、host/plugins/cooked content、symbols、SBOM、tests、签名和promotion decision绑定为不可变发行身份。

必须建立`ReleaseManifest`与`ReleaseProvider`：candidate只能由通过既有Build Set验真的immutable artifact集合创建；状态至少为Built、Verified、Signed、Candidate、Promoted、Revoked，转换由policy engine与append-only ledger拥有。channel只是指向已签名manifest digest的受控pointer，不复制或重建artifact。任何缺source/dependency/toolchain/target/test/signature receipt的candidate不得promotion。

### TOOL-RELEASE-P0-002 · 产品分发没有加密签名、验证器和可轮换信任根

`native_signing.py`只执行调用方指定的任意命令并记录退出码/hash；它不记录证书链、subject、timestamp authority、notary ticket或验证结果。`plugin_build_signature.py`生成的`.sig`是未签名TOML，内容可与payload一起被攻击者重写，且没有consumer。Hub NSIS、engine host、plugin catalog、release manifest和update metadata都没有公开key、threshold、rotation/revocation或offline root。

必须把“平台code signing”“Zircon release metadata signing”“内容hash”拆成三个typed receipt。离线root授权在线targets/channel keys，客户端内置最小root并支持版本化rotation/revocation；Windows/macOS等平台签名由独立verifier回读certificate、timestamp/notary结果。Hub在任何解压、加载或执行前先验证metadata chain、artifact digest、target/platform与rollback counter；`.sig`重命名为hash/audit manifest，除非它真正承载可验证签名。

### TOOL-RELEASE-P0-003 · 没有原子Install/Update/Rollback状态机和启动健康闭环

Hub source build直接写可变output；device install只允许新目录并递归copy；Tauri没有updater；三个PowerShell installer分别管理repo hook、scheduled task或HKCU Run。当前没有安装inventory、current/previous slot、operation lease、磁盘预检、staged verify、atomic switch、post-switch launch health、crash-loop detection、rollback journal或uninstall ownership。进程中断、磁盘满、杀毒软件锁文件、旧进程仍持有DLL或新版本无法启动时都没有统一恢复语义。

必须建立单一`InstallService`和durable operation journal。下载只写content-addressed staging；完整验证后物化新version slot；current pointer切换必须原子且可恢复；首次启动通过version/build handshake、ready/first-frame与bounded health window后才commit。失败或crash-loop自动回到previous known-good slot，并保留诊断、失败candidate与人工retry/repair入口。in-place覆盖可执行文件必须禁止。

## 4. Release Identity、Channel 与 Compatibility 差距

### TOOL-RELEASE-P1-001 · 没有单一release domain owner

CI、export、Hub、plugin build、MVP staging和本地installer各自使用release术语。建立独立domain crate/tooling service，拥有ReleaseId、Manifest、Channel、Candidate、Promotion、Revocation与InstallPlan；各域只提交typed receipt。

### TOOL-RELEASE-P1-002 · `0.1.0`由多个文件手工重复声明

根/plugin workspace、Hub Tauri、Hub npm与Tray没有生成/检查关系。定义一个version authority并生成各consumer projection；CI必须拒绝version、ABI、SDK、bundle metadata或release manifest不一致。

### TOOL-RELEASE-P1-003 · 没有tag与source release identity

0个tag意味着不能从版本反查source tree和release decision。release record绑定annotated/signed tag或等价immutable source revision、tree digest、submodule/reference revision与dirty=false proof。

### TOOL-RELEASE-P1-004 · 没有stable/beta/nightly/dev channel语义

channel需要明确定义更新频率、允许的prerelease、兼容保证、telemetry ring、retention与rollback窗口；不能由branch名或用户手输路径隐式决定。

### TOOL-RELEASE-P1-005 · Release没有绑定Tooling 03的Build Set

promotion必须消费同一Build Set ID和stage receipts，不能在publish job重新猜测输入。Release Manifest引用immutable output digests，不引用可变workspace/output path。

### TOOL-RELEASE-P1-006 · Cargo `release` profile被误用为产品成熟度

优化级别只是一项build policy。引入Development、Test、Profile、Shipping等product configuration，明确assert/log/telemetry/debug symbol/console/plugin/signing政策；channel再约束允许的configuration。

### TOOL-RELEASE-P1-007 · 没有engine/plugin/project兼容tuple

定义engine distribution ID、runtime ABI、plugin SDK/API、asset/scene schema、editor protocol、Hub minimum version和platform capability tuple。安装与打开project前必须计算兼容结果和迁移计划。

### TOOL-RELEASE-P1-008 · 没有支持周期与撤回政策

声明每channel支持的版本窗口、关键安全修复、minimum supported version、end-of-support与forced update条件；客户端需要可解释的unsupported/revoked状态。

### TOOL-RELEASE-P1-009 · 没有release notes与migration guide gate

仓库只有资产迁移文档，没有产品CHANGELOG/release-note authority。对用户可见行为、API/ABI/schema变化要求结构化change fragment，在candidate阶段生成release notes、migration guide和known issues。

### TOOL-RELEASE-P1-010 · Release source policy不限制dirty、untracked或本机依赖

candidate producer必须证明clean immutable tree、locked dependencies、toolchain image与declared secret-free environment；本地开发build可以保留，但只能标记UntrustedLocal且永不promotion。

## 5. Artifact Repository 与 Promotion 差距

### TOOL-RELEASE-P1-011 · CI上传的是短期evidence而非distribution

唯一artifact name固定为MVP evidence且retention 7天。建立按ReleaseId/Build Set/target分类的distribution、symbols、SBOM、provenance、tests和logs对象；evidence不能与可执行payload混为一类。

### TOOL-RELEASE-P1-012 · 没有不可变artifact repository

发布对象必须以强digest寻址、create-only、server-side integrity、namespace ACL和retention lock保存；logical name只能是manifest索引，禁止覆盖同version bytes。

### TOOL-RELEASE-P1-013 · 没有candidate到channel的promotion事务

promotion不能复制并重新压缩文件。对已验证manifest digest执行policy check和原子channel pointer更新，记录actor、reason、previous pointer与rollback token。

### TOOL-RELEASE-P1-014 · 没有完整distribution catalog

catalog需列出Hub、Editor、runtime/player、tooling、plugin SDK、export templates、symbols与platform prerequisites的精确版本和digest；partial target必须显式unsupported。

### TOOL-RELEASE-P1-015 · 没有base/target patch graph

Tooling 03的delta仍不是installer patch。repository维护有界delta graph、base/target manifest digest、full fallback、patch chain上限与apply cost，客户端下载前选择最安全可用路径。

### TOOL-RELEASE-P1-016 · 没有release retention、legal hold与GC

channel rollback窗口内的manifests/artifacts必须pin；symbols和provenance至少覆盖支持期；GC只删除没有channel、install inventory、legal hold或active download lease引用的对象。

### TOOL-RELEASE-P1-017 · 没有mirror/CDN一致性与故障隔离

mirror list必须由签名metadata提供，客户端按health/region选择并始终验证digest。origin、CDN和mirror污染要可quarantine，不能把任意URL写进可信manifest。

### TOOL-RELEASE-P1-018 · Symbols/source map未与发行身份绑定

Tooling 07已确认无symbol service。每个binary debug ID对应不可变symbol/source map artifact和访问策略；crash report用ReleaseId+debug ID定位，不靠文件名或本机path。

### TOOL-RELEASE-P1-019 · SBOM、license与vulnerability结果不参与promotion

生成SPDX/CycloneDX、第三方notice、dependency provenance和scanner result并绑定manifest。policy明确可接受severity、exception owner/expiry与重扫规则；不能只在报告中列hash。

## 6. Signing、Attestation 与 Supply-chain 差距

### TOOL-RELEASE-P1-020 · `.sig`命名把hash audit误导成cryptographic signature

当前TOML没有signature bytes、algorithm、key ID或signed payload。立即改名为`.artifact-audit.toml`或实现真实detached signature，并在schema中拒绝两者互换。

### TOOL-RELEASE-P1-021 · 没有consumer-side signature verification

producer report再完整也不能替代独立verifier。Hub、installer、runtime native loader和offline inspect tool应复用同一验证库，并以fail-closed typed error拒绝unknown key、expired metadata、wrong target和digest mismatch。

### TOOL-RELEASE-P1-022 · 外部sign命令没有工具与身份证明

receipt应记录signer executable digest/version、key/certificate ID、subject、issuer、algorithm、timestamp与verification result；命令行和环境不得泄露secret，成功退出不等于签名有效。

### TOOL-RELEASE-P1-023 · 没有key custody与权限分离

定义offline root、online release/channel、platform signing和emergency revocation keys；使用受审计KMS/HSM或平台vault，CI短期凭据最小权限，build worker不得获得root key。

### TOOL-RELEASE-P1-024 · 没有timestamp/notary真实性和离线验真政策

Windows Authenticode timestamp、macOS notarization ticket等必须由verifier回读并进入receipt。明确离线、证书过期、服务暂时不可用与历史签名验证规则。

### TOOL-RELEASE-P1-025 · Hub NSIS bundle没有平台签名gate

`bundle.targets=["nsis"]`只生成格式。Windows installer、Hub executable、DLL和uninstaller必须签名并验证architecture、publisher、timestamp、SmartScreen相关metadata和安装权限模型。

### TOOL-RELEASE-P1-026 · 更新metadata没有rollback/freeze protection

签名链需包含递增metadata version、expiry、consistent snapshot与minimum accepted counter；客户端持久化最高可信版本，拒绝旧channel metadata、无限期冻结和mix-and-match。

### TOOL-RELEASE-P1-027 · 构建provenance没有独立attestation

对source tree、workflow identity、runner image、toolchain、dependency lock、commands和outputs生成可验证attestation；签名者只签已通过policy的attestation digest，不能信任任意本机JSON。

### TOOL-RELEASE-P1-028 · signing日志与失败artifact没有secret/redaction policy

当前捕获完整stdout/stderr和command。建立字段级secret分类、redaction、访问控制、保留期与incident export；任何token/key path不得进入用户可下载release report。

## 7. Installer、Update 与 Rollback 差距

### TOOL-RELEASE-P1-029 · 没有单一`InstallService`

Hub delivery、Tauri bundle、Coordinator task、Tray startup和Codex hook各自安装。产品installer只管理engine distribution/Hub/SDK与系统prerequisite；developer tooling installers保留独立scope但复用transaction primitives。

### TOOL-RELEASE-P1-030 · Hub把process exit 0直接投影为staged engine

`BuildExecutionReport::succeeded()`后立即记录成功路径。新增distribution validator：读取Release/Build Set manifest，重算files/digests，验证host architecture/dependencies/signatures并运行隔离smoke，再允许candidate状态。

### TOOL-RELEASE-P1-031 · SourceEngine ID是路径FNV而非发行身份

路径可继续作为local checkout locator，但engine install identity必须是ReleaseId/manifest digest。FNV64碰撞或移动目录不能改变distribution identity，也不能覆盖另一安装记录。

### TOOL-RELEASE-P1-032 · Source validation只检查三个路径形状

本地build admission还应检查Git revision/dirty state、workspace/lock/toolchain compatibility、build tool hash、target capability与output ownership；结果记录为UntrustedLocal receipt，不升级成signed release。

### TOOL-RELEASE-P1-033 · `SourceEngineInstall`没有version/channel/build/slot字段

拆分`SourceCheckout`与`InstalledDistribution`。后者至少含ReleaseId、channel、manifest digest、install root、slot、state、installed/verified/last launched时间、health与previous known-good。

### TOOL-RELEASE-P1-034 · 可变output目录允许旧新字节混合

每次build/install写unique attempt/staging root并create-only；验证通过后以manifest digest命名version slot。不得复用`output_dir/ZirconEngine`作为current release事实。

### TOOL-RELEASE-P1-035 · Device install无法升级既有目标

拒绝existing dir避免覆盖是好的fail-safe，但没有upgrade path。基于当前receipt计算计划，把新slot物化后切换；失败保留旧slot，repair以manifest为authority而非盲目recopy。

### TOOL-RELEASE-P1-036 · Install receipt绑定本机path和`file://` URL

receipt的resource ID/download ID来自install path，不可跨机器或作为release identity。改为ReleaseId/Build Set/target manifest digest；本机path只作为install location，URL只存在于可替换transport plan。

### TOOL-RELEASE-P1-037 · 没有prerequisite与系统集成inventory

声明VC runtime、GPU/runtime minimum、WebView、driver/OS、file association、shortcut、registry/service/firewall等prerequisite及owner。install/uninstall必须幂等记录并只删除自己拥有的资源。

### TOOL-RELEASE-P1-038 · 没有A/B slot与atomic current pointer

使用versioned immutable slots与小型atomic current marker；running process继续使用旧slot，新进程读取新pointer。Windows locked DLL不能触发in-place覆盖或半更新。

### TOOL-RELEASE-P1-039 · 没有统一journal、recovery与post-switch health

复用asset migration和Coordinator cutover的durable phase思想，覆盖Acquire、Download、Verify、Materialize、Preflight、Switch、Health、Commit/Cleanup/Rollback。重启必须从journal确定恢复，不根据目录猜测成功。

## 8. Hub、Project Migration 与 Version UX 差距

### TOOL-RELEASE-P1-040 · Tauri没有updater集成和配置

Cargo无updater plugin，config无endpoint/pubkey/updater artifacts。即使未来采用Tauri updater，也只能更新Hub自身；engine distributions仍由Zircon InstallService独立管理并共享trust root/policy。

### TOOL-RELEASE-P1-041 · Hub明确把update标为reserved状态

`ui_text.rs`说明remote update service未启用。UI必须保持真实disabled/degraded语义，直到provider、trust、download、install与rollback端到端可用，不能只接按钮和静态版本号。

### TOOL-RELEASE-P1-042 · 没有check/download/install/restart分离状态

定义Idle、Checking、Available、Downloading、Verifying、ReadyToInstall、SwitchPending、RestartRequired、HealthChecking、Committed、RollbackAvailable/Failed；每态有取消、重试与恢复规则。

### TOOL-RELEASE-P1-043 · Project未保存期望engine compatibility

project binding当前指向path-derived engine ID。manifest应声明tested engine range、exact lock可选项、plugin set与schema epoch；Hub在打开前选择兼容安装或给出可审计迁移计划。

### TOOL-RELEASE-P1-044 · 现有asset migration未接入engine upgrade

不要重写迁移算法；由upgrade planner在副本/branch或事务scope中调用既有dry-run，收集issue、changed files、estimated work和backup policy。engine install成功不等于project已迁移。

### TOOL-RELEASE-P1-045 · 没有用户可确认的migration preflight

展示breaking changes、plugin incompatibility、asset/scene steps、disk/time estimate、不可逆项与rollback边界。自动批量迁移必须生成machine-readable report并保留原project可恢复入口。

### TOOL-RELEASE-P1-046 · 没有跨版本schema migration matrix

登记每个from/to engine epoch可用的migration chain、minimum tool version、lossless/losing、online/offline与测试fixture。缺链时拒绝打开写入，不让最新reader尝试猜测。

### TOOL-RELEASE-P1-047 · 没有side-by-side版本与项目隔离

工程级引擎需要同时保留stable、preview和project-pinned版本。Hub按distribution identity启动对应Editor/Runtime，cache、plugins、settings与symbols按兼容边界隔离。

### TOOL-RELEASE-P1-048 · 没有更新后启动失败/crash-loop自动判定

Hub/Editor握手应上报ReleaseId、Build Set、phase与ready；连续早退、ABI mismatch或初始化fatal触发candidate unhealthy并自动回旧slot。单纯进程创建成功不能commit update。

## 9. Download、Operations 与 Fleet 差距

### TOOL-RELEASE-P1-049 · Content download固定使用development security policy

`NetSecurityPolicy::development()`允许非TLS且不pin。发布transport必须显式production policy、允许host、TLS required、可选pin/root与redirect限制；development transport永不允许承载可执行release artifact。

### TOOL-RELEASE-P1-050 · Download Manifest缺release签名与整体约束

当前只有download/resource/chunks/mirrors，没有manifest signature、issuer/expiry、target、total/root hash、chunk non-overlap/order或artifact type。下载前验证signed release descriptor，完成后再验证完整artifact digest。

### TOOL-RELEASE-P1-051 · Resume只保存在进程内存

partial chunks、bitmap和attempt state都在HashMap。把partial bytes写入unique staging，journal记录verified ranges/chunks；重启回读时逐chunk重验，取消/expiry按policy清理。

### TOOL-RELEASE-P1-052 · 没有更新SLO与可观测性

记录check latency、download bytes/rate/retries、mirror health、verify/install/switch/health duration、failure code、rollback rate与版本分布；高基数identity进入trace/receipt，不直接成为metric label。

### TOOL-RELEASE-P1-053 · 没有kill switch、revocation与安全响应入口

运营方需要签名revocation、channel freeze、bad release block、minimum safe version与emergency rollback。客户端离线时应用缓存metadata的expiry与本地deny list，且不得静默继续安装已撤回candidate。

### TOOL-RELEASE-P1-054 · 没有ring/canary与逐级推广

定义internal、canary、preview、stable等ring，按明确cohort推进同一manifest；promotion gate消费crash/performance/install health阈值，可暂停或回退pointer。不得为不同ring重建“同版本”不同字节。

## 10. P2 治理与体验差距

### TOOL-RELEASE-P2-001 · `release`术语在仓内多义

建立词汇表区分Cargo release profile、directory handle released、Release Candidate、Promoted Release与cargo job lease release，并重命名`MvpStagingRelease`为unlock/handle probe语义。

### TOOL-RELEASE-P2-002 · 没有`zircon release inspect`只读入口

展示manifest、channel、Build Set、targets、digests、signatures、SBOM、tests、promotion和revocation，支持离线验真与JSON输出。

### TOOL-RELEASE-P2-003 · 没有update plan dry-run

在下载前显示full/delta选择、预计bytes、临时/最终磁盘、停机/重启、迁移、prerequisite、rollback保留与不兼容原因。

### TOOL-RELEASE-P2-004 · 没有channel pin/skip/snooze policy

允许项目或管理员pin受支持版本、延期非强制更新和跳过单个bad candidate；安全撤回/minimum version按签名policy覆盖本地偏好。

### TOOL-RELEASE-P2-005 · 更新诊断没有稳定code和remediation

所有网络、metadata、hash、disk、permission、process、health、migration与rollback失败使用稳定typed code、bounded context和可本地化remediation ID。

### TOOL-RELEASE-P2-006 · 没有带宽、代理与计量网络控制

提供限速、暂停、schedule window、system proxy、企业CA、metered network/battery policy和后台优先级；策略写入download receipt。

### TOOL-RELEASE-P2-007 · 没有artifact/component选择与repair UX

用户应能查看必选/可选components、占用空间、symbols/templates/SDK，并按manifest执行verify/repair；不能让目录复制成为唯一恢复手段。

### TOOL-RELEASE-P2-008 · 没有安装历史与回滚原因视图

Hub展示每次operation的from/to ReleaseId、channel、bytes、duration、result、health与rollback reason；敏感日志由权限控制，不直接塞进UI snapshot。

### TOOL-RELEASE-P2-009 · 没有release日历与支持状态机器可读输出

发布节奏、EOL、minimum version和maintenance window应来自signed metadata/服务API，而非手工网页说明；CLI/Hub消费同一模型。

### TOOL-RELEASE-P2-010 · 没有灾备演练与恢复手册

定期演练repository/mirror故障、坏签名、key compromise、坏release、磁盘满、断电switch与metadata expiry；产出RTO/RPO和可执行runbook。

## 11. 目标架构

```text
Build Set + validation + symbols/SBOM/provenance
                     |
                     v
             Release Candidate Builder
                     |
          immutable artifacts + manifest digest
                     |
        Sign/Verify + Promotion Policy + Ledger
                     |
              signed channel pointer
                     |
        Hub ReleaseProvider / metadata client
                     |
      Download -> Verify -> Materialize new slot
                     |
       Atomic Switch -> Launch Health -> Commit
                     |
                failure -> Rollback
```

核心协议：

1. `ReleaseManifest`：ReleaseId、Build Set、version、channel eligibility、target/component artifacts、compatibility、symbols/SBOM/provenance/test receipts、minimum versions与signatures。
2. `PromotionRecord`：candidate manifest digest、policy inputs/results、actor/workflow identity、previous/new channel pointer、time、reason与rollback token。
3. `InstalledDistribution`：manifest digest、slot、install state、verified time、last health、previous known-good与owned system resources。
4. `InstallOperationJournal`：phase、attempt、source/target、staging/slot paths、verified digests、switch marker、health deadline与recovery disposition。
5. `UpdatePolicy`：channel/ring、pin/deferral、minimum safe/revoked versions、network/power/time window、retention与telemetry consent。

## 12. 分层重构路线

### M0 · 禁止伪发行状态并统一术语

1. 把Cargo profile、MVP handle probe与产品Release状态分开命名。
2. 所有现有UI/CLI将source build标记`UntrustedLocalBuild`，不得显示installed/promoted。
3. `.sig`在实现真实签名前改名为artifact audit；shipping gate对无trust receipt fatal。
4. 冻结version authority、compatibility tuple与Release Manifest schema ADR。

### M1 · Release Identity、Repository 与 Inspect

1. 复用Tooling 03 Build Set，生成immutable candidate manifest和component catalog。
2. 建立本地filesystem repository prototype：content-addressed、create-only、digest re-read、retention引用。
3. 实现`zircon release inspect/verify`与golden malformed/unknown-field/budget fixtures。
4. 建立version projection/check、change fragment、release note/migration guide gate。

### M2 · Trust Root、Signing 与 Attestation

1. 定义offline root/online targets/channel/platform key角色、rotation、expiry与revocation。
2. 接入真实平台signer并在独立进程/库中回读验证证书、timestamp/notary。
3. 生成SBOM、license、provenance attestation并绑定manifest。
4. Hub/installer/plugin admission共享fail-closed verifier与离线fixture。

### M3 · Transactional Install 与 Side-by-side

1. 建立InstalledDistribution inventory、unique staging、immutable version slots和atomic current pointer。
2. 实现Acquire到Commit/Rollback的durable journal与restart recovery。
3. 加disk/prerequisite/process lock preflight、owned resource registry、repair/uninstall。
4. 把现有项目device receipt和Coordinator cutover模式收敛到共享transaction primitives。

### M4 · Hub Update Provider 与 Project Migration

1. Hub接入签名channel metadata，分开check/download/install/restart状态。
2. 支持engine side-by-side、project pin与compatibility resolver。
3. 调用既有asset migration dry-run/apply，生成upgrade plan与可恢复project副本/transaction。
4. 首次启动通过ReleaseId/Build Set handshake和ready/first-frame health后commit。

### M5 · Delta、Mirror 与运维控制面

1. 实现base/target manifest约束的块级delta、full fallback与chain cap。
2. durable resume、mirror health/quarantine、代理/限速/计量网络与带宽policy。
3. channel/ring promotion、kill switch、revocation、rollout pause与telemetry gate。
4. symbols/crash service、install/update dashboards、retention/GC与incident runbook。

### M6 · 跨平台、破坏注入与规模验收

1. Windows NSIS、macOS bundle/notary、Linux package/archive分别执行真实install/update/rollback matrix。
2. 注入断网、坏mirror、坏签名、磁盘满、断电、locked DLL、进程崩溃和journal损坏。
3. 执行N-2/N-1/current/preview side-by-side与project/plugin/schema迁移矩阵。
4. canary到stable逐级推广同一manifest，以crash/performance/install SLO自动暂停或回滚。

## 13. 验收门

1. 同一source tree、Build Set和target重复生成字节一致的Release Manifest digest。
2. dirty/untracked或未锁dependency的本地build只能成为`UntrustedLocalBuild`。
3. 根/plugin/Hub/Tauri/npm/Tray版本projection由单一authority生成并由CI校验。
4. 每个release version可反查immutable source tree、toolchain、dependency lock和promotion record。
5. channel pointer只引用已签名manifest digest，更新是原子且保留previous pointer。
6. 修改manifest、artifact、SBOM、signature、key ID或target任一字节均被离线verifier拒绝。
7. unknown/expired/revoked key、metadata rollback、freeze与mix-and-match均fail closed。
8. Windows installer/Hub/executable/DLL的publisher、timestamp、architecture由独立verifier回读通过。
9. promotion前强制消费Build Set tests、symbols、SBOM、license、provenance和security policy结果。
10. artifact repository拒绝覆盖同digest/name的不同bytes，mirror始终以manifest digest验真。
11. 下载中断并重启进程后只复用已回读验证chunk，partial corruption会局部重取。
12. production artifact下载拒绝HTTP、未允许host和development security policy。
13. 磁盘满、权限失败或进程终止不会改变current slot，也不会删除known-good。
14. 新slot完整物化和验证前，任何启动入口仍解析到旧slot。
15. atomic switch后Hub/Editor握手返回正确ReleaseId/Build Set，health通过才commit。
16. 新版本连续早退或ready超时会自动回滚，重启后journal仍能完成恢复。
17. locked DLL/旧进程运行时更新不做in-place overwrite，旧进程可继续退出。
18. verify/repair能从manifest定位单文件损坏并恢复，不盲目重装全部内容。
19. stable、preview和project-pinned版本可side-by-side，cache/plugin/settings按compatibility隔离。
20. engine upgrade前生成完整project/plugin/asset/scene migration dry-run；缺迁移链时不写project。
21. N-2到current及current到previous rollback的project fixture保留可恢复原始数据。
22. delta apply后的target manifest逐文件digest等于full install，失败自动使用full fallback。
23. canary同一manifest达到定义的install/crash/performance阈值后才promotion到stable。
24. repository outage、key compromise、bad release和metadata expiry演练满足已声明RTO/RPO并留下审计记录。

## 14. Ownership 与实施约束

- Tooling拥有Release Manifest构建、repository/promotion/signing/attestation CLI与CI policy；不重新拥有Cook/Pack算法。
- Hub拥有用户update workflow、installed inventory、operation orchestration、side-by-side选择和恢复UX；不自行实现密码学或迁移算法。
- Platform层拥有OS installer、atomic switch、process/lock/prerequisite、code-sign verifier与system integration adapter。
- Runtime/asset拥有既有migration执行与schema chain；upgrade planner只调用typed dry-run/apply接口。
- Network/content download拥有transport、Range/retry/resume；ReleaseProvider提供签名metadata、URL allowlist与expected digests。
- Security/release operations拥有keys、channel policy、promotion/revocation、audit retention与incident response，build worker不得同时拥有root签名权限。
- 第一阶段只做协议、local repository与fail-closed verifier；没有可信manifest和transaction前，不接远程自动更新按钮。

## 15. 与既有报告的边界

- Tooling 03拥有preset、Build/Cook/Pack、Product Host、PlatformBundle与Build Set生产真实性；本报告从“已有immutable distributable”开始，拥有repository、promotion、install/update/rollback。
- Tooling 01拥有workflow/toolchain/dependency pinning通用风险；本报告只要求其结果进入release provenance和promotion gate。
- Tooling 06拥有Coordinator本机控制与自身installer；本报告只复用其cutover journal模式，不重复列token-free控制面等P0。
- Tooling 07拥有benchmark、crash与symbols service；本报告定义release必须绑定并消费这些artifact。
- Tooling 08拥有DDC/CAS/remote execution；release repository是不可变、长期、信任受控的产品分发存储，不能直接拿可逐出的DDC充当。
- Hub 01拥有当前build/process/persistence/device copy具体bug，Hub 02拥有web shell/coming-soon/provider UX；本报告只定义release/update domain合同。
- Runtime 08E拥有通用network/content download实现；本报告只约束可执行artifact transport必须使用production security和signed release metadata。
- Plugins 01拥有native plugin loader/ABI/catalog admission；本报告只定义发行签名与release compatibility输入。
