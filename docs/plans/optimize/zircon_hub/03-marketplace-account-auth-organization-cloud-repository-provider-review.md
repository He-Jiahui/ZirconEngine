---
related_code:
  - zircon_hub/Cargo.toml
  - zircon_hub/package.json
  - zircon_hub/tauri.conf.json
  - zircon_hub/capabilities/default.json
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/overlays/UserMenuPopover.tsx
  - zircon_hub/web/src/pages/CatalogPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/TeamPage.tsx
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/web/src/tauri
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/install_receipt.rs
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/net/features/content_download/runtime/src/manager
  - zircon_runtime/src/core/framework/net/download.rs
  - tools/zircon_export/plugin_build_package.py
  - tools/zircon_export/plugin_build_signature.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/43-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Auth.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Commerce.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/UserFile.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/TitleFile.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineServices/Source/OnlineServicesInterface/Public/Online/Connectivity.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildPatchServicesModule.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildInstaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/BuildPatchManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Portal/Services/Public/IPortalService.h
  - dev/UnrealEngine/Engine/Source/Runtime/Portal/Services/Public/IPortalServiceLocator.h
  - dev/godot/editor/asset_library/asset_library_editor_plugin.h
  - dev/godot/editor/asset_library/asset_library_editor_plugin.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/Fyrox/project-manager/src/upgrade.rs
  - dev/bevy/Cargo.toml
  - dev/bevy/crates/bevy_internal/Cargo.toml
  - dev/bevy/crates/bevy_asset/Cargo.toml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/package.json
  - dev/Graphics/Packages/com.unity.shadergraph/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 03 · Marketplace / Account Auth / Organization / Cloud Repository Provider 工程化差距

## 1. 结论

Zircon Hub当前没有Marketplace、远程Account/Auth、Organization/RBAC或Cloud Repository provider。仓内也没有HTTP client、OAuth/OIDC/JWT、access/refresh token、OS keyring、TLS/certificate、entitlement、audit或WebSocket依赖与实现。远程同步、账号服务、云仓库、市场下载、邀请和权限都以`comingSoon`且`disabled: true`投影；这份真实性纪律应保留，不能因为页面、图标和类型已经存在就提前启用。

但产品表面已经出现一个需要在接远程服务前硬切的身份混淆：TopBar把本地Git `user.name`和首字母显示成用户头像与“我的账户”，Account点击又跳到Team页；Team的所谓members实际是首个Git仓库最近200条提交聚合出的最多8个authors。Git author、Git config identity、Hub登录主体、Organization member、Marketplace entitlement holder和Cloud authorization principal是六种不同身份，不能继续共用一个字符串投影。

Plugins页面也只是递归读取本地`plugin.toml`。其manifest字段多数optional、没有strict unknown-field、publisher/license/version dependency/artifact digest/signature/entitlement/install state，一个坏目录或manifest可用`?`中止整个catalog。Cloud页面则只读取本地package/install action history和本机目录；remote service tab明确禁用。现有package把项目树复制到时间戳目录，device install再复制一次并生成逐文件SHA-256 receipt；这是本地evidence底座，不是远程仓库、同步协议或可信package install。

Marketplace启用还受Plugins01的现有P0约束：native loader在project selection、enablement和trust验证前就可能`Library::new`并执行entry，当前hash/signature旁车没有产品消费者。远程下载会把本地目录风险升级为供应链远程代码执行面。因此本报告不新建第二套package resolver/install/trust：Plugin Package Service拥有package identity/lock/verify/install，Tooling09拥有engine release repository/update trust，Editor06拥有project activation/reload；Hub只消费这些服务并拥有账号、组织、浏览、授权、队列与恢复UX。

目标链为`Auth Provider + Secure Credential Broker -> Organization/RBAC -> signed Marketplace index/entitlement -> shared Package Service -> Hub operation receipt`，以及`Project Snapshot Manifest -> content-addressed encrypted blobs -> revision/CAS sync -> semantic conflict/recovery`。在这两条链完成前，远程入口继续Unavailable是正确产品行为。

## 2. 审查范围与证据

本轮冻结78个selected path，共29,314行、1,031,105 bytes、93个Rust test attributes、0 ignored、0个在途文件。路径排序后对每个文件取小写SHA-256，以`forward/slash/path|hash`、LF连接且末尾无LF形成manifest，当前工作树fingerprint为`a34523efa8734e38624614d28c2b4855c54a07201a467352d36d6fab7c2fcc40`。

| 子域 | 文件 / 行 / bytes | 本轮判定 |
|---|---:|---|
| Hub remote surfaces/contracts | 32 / 9,667 / 338,557 | E3逐TopBar/User Menu、Catalog/Cloud/Team、fallback、IPC/types、coming-soon、local catalog/Git/delivery及静态contracts；84个tests |
| Zircon package/download substrate | 19 / 2,458 / 80,781 | E3逐本地package/install/receipt、plugin manifest/package/signature producer与content download manager；9个tests |
| Unreal参考 | 14 / 5,541 / 239,991 | E2/E3按Auth、Commerce/Entitlement、User/Title File、Portal service与BuildPatch职责路由 |
| Godot参考 | 2 / 2,786 / 97,161 | E3按Asset Library query/version/license/download/SHA/install/offline/proxy路由 |
| Fyrox参考 | 4 / 2,279 / 78,308 | E2/E3按local project manager、Cargo dependency与upgrade路由 |
| Bevy参考 | 3 / 6,355 / 183,475 | E2仅确认Cargo workspace/package dependency，不外推Marketplace/Auth/Cloud |
| Unity Graphics参考 | 4 / 228 / 12,832 | E2仅确认render package manifest/version/dependency，不外推Unity Package Manager/账号/云服务 |
| 合计 | 78 / 29,314 / 1,031,105 | 93个test attributes、0 ignored、0个在途文件 |

本轮使用exact dependency/term absence scan：Hub production source对`reqwest/hyper/oauth/openid/jwt/access_token/refresh_token/keyring/credential/secret/websocket/tls/certificate/signature/entitlement/audit`均为0个实现命中；唯一`rbac`文本命中来自颜色token中的单词片段，不是权限代码。`Cargo.toml`只有serde/JSON/Tauri/error/TOML/runtime interface，前端只有React/MUI/Tauri API和构建依赖。

84个Hub test attributes多数验证source snippets、disabled reservations和本地流程，不是remote fake/provider/contract/integration tests。动态测试没有重跑：当前clean Hub source仍同时存在`persist_unchecked(&mut self)`定义和`persist_unchecked(None)`调用，Hub01已由managed Windows Cargo复现`E0061`编译阻断；重复同一lane不能验证本报告不存在的远程行为。

## 3. 当前实现事实

### 3.1 远程能力保持disabled，但Account表面仍混淆身份

`coming_soon_entries()`明确把`marketplace-download`、`remote-sync`、`account-service`、`cloud-repository`、`sign-out`、`team-invite`、`team-permissions`和`remote-collaboration`设为disabled。Cloud页只列本地Packages/Installs与三个reserved service；这比伪造云成功正确。

然而TopBar的`userName`直接取`state.team.identityName`，即`git config user.name`。User Menu显示“我的账户/个人资料和偏好”，Account route转到Team；sign-out虽disabled，但视觉上把本地profile包装为远程账号。必须在M0改名为Local Git Identity/Workspace Profile，或在无Auth provider时将Account明确Unavailable。

### 3.2 Team不是Organization

`discover_team_overview()`只找第一个Git root，执行`git config`和`git log --all --format=%an%x1f%ae -n 200`，按name/email计数后保留8个author。子进程没有deadline/process-tree cancellation；错误被降成empty。Team页又把authors显示为Team Members并展示email，没有organization ID、membership source、role、invitation、tenant、authorization或audit。

Git author history可以保留为Repository Contributors视图，但不能参与登录、RBAC、Marketplace entitlement或Cloud ACL。Organization provider必须是独立authority；source-control provider只能提供repo identity和contributor projection。

### 3.3 Plugins Catalog不是Marketplace

Hub local catalog递归项目`Plugins/plugins`和首个repo的`zircon_plugins`，跳过`.git/target`并读取`plugin.toml`。它能区分Project/Engine scope并投影display/category/maturity/module count，这是本地inventory基础。

但parser所有主要字段为`Option`或default，unknown field未拒绝；dependency/version/publisher/license/price/entitlement/artifact/target digest/signature/install/update/revocation均缺席。递归I/O或任一manifest parse错误会终止整个发现调用；ID重复只按manifest path去重，不建立package identity collision policy。UI的search/filter/pagination只对完整内存数组处理。

### 3.4 本地package与download不能直接升级为远程安装

`package_project()`递归复制除`.git/target`外的项目文件并写源绝对路径、时间和文件数；没有source revision、allowlist、secret scan、per-file digest或immutable artifact identity。device install复制整个package到新目录，失败清理owned directory，随后receipt记录每文件SHA-256并构造`file://` chunk URLs。这些正向原语可复用，但没有签名publisher、lockfile、dependency transaction、atomic activation/update/rollback或license receipt。

content download manager验证Range、长度与chunk hash，但Tooling09已确认resume state仅在内存、production可接受development/non-TLS policy、没有signed release metadata和最终原子安装。Marketplace只能给它签名metadata、allowlisted URL和expected digests，不能把URL直接交给loader。

### 3.5 Cloud页没有Cloud Repository语义

Cloud的package readiness由本地action history推导，service rows全部disabled。没有remote repository ID、snapshot/revision、object manifest、upload/download、CAS、conflict、encryption、quota、retention、offline journal或recovery。Runtime User Cloud/SaveGame、Tooling release repository、Editor source control、Editor43 live collaboration与Hub project backup/sync是不同domain，禁止以一个“Cloud”按钮混合。

### 3.6 安全边界尚未为远程内容准备

Hub Tauri capability当前只允许main window基础操作，值得保留；但`tauri.conf.json`的CSP为`null`。一旦Catalog显示remote description/image/link，当前WebView没有生产CSP、remote origin allowlist、content sanitization或navigation/download隔离。Hub config、完整snapshot、action history和diagnostics也没有secret-bearing field policy；在secure credential broker完成前不得把token放入现有serde DTO。

## 4. 参考源码的可用边界

Unreal `Auth.h`明确区分NotLoggedIn、UsingLocalProfile、LoggedInReducedFunctionality与LoggedIn，Login携platform user、credential type/token和scopes；Logout可销毁persistent auth，并有login status、pending token expiration和account attribute change事件。`Commerce.h`把offer、checkout、transaction和entitlement分开，操作都绑定local account。`UserFile/TitleFile`证明account-qualified异步文件枚举/读写的最低接口，但没有revision/CAS/merge，不能作为高级项目同步的充分参考。

Unreal BuildPatch提供manifest、installer、progress/error/pause/cancel和installation lifecycle，generic release/install责任已由Tooling09拥有。Portal service的locator/provider边界可借鉴service discovery与provider分离，但不能证明Epic Launcher后端、组织、支付或Marketplace合规实现。

Godot Asset Library是真实公共catalog下限：repository选择、API request types、category/license/search/sort/page、asset detail、多release/version/changelog、download progress/retry、SHA-256验证、installer preview，以及offline/proxy/ETag image cache。它没有Zircon第三方native code的强trust问题，也不提供账号、entitlement或项目Cloud repository，故只能参考browse/download UX与失败闭环。

Fyrox Project Manager通过Cargo dependency/source做本地upgrade；Bevy镜像只含Cargo workspace/package；Unity Graphics镜像只含本地render package manifests。三者都不构成Marketplace/Auth/Cloud完成证据，本报告只将其作为“不要把包声明误当包服务”的反例边界。

## 5. 目标架构与Owner

```text
AuthProvider -> SecureCredentialBroker -> AccountSession
                         |
                Organization/RBAC/Audit
                         |
Signed Marketplace Index + Offer/Entitlement
                         |
        Shared Plugin Package Service
     resolve -> lock -> acquire -> verify
     -> stage -> install -> activate/rollback

Project Snapshot Source -> Manifest/CAS -> Encrypted Blob Store
     -> revision compare -> upload/download -> conflict plan
     -> staged local apply -> receipt/recovery
```

| Owner | 拥有 | 不拥有 |
|---|---|---|
| Hub03 | account session UX、organization selection、Marketplace browse/acquire orchestration、Cloud project snapshot/sync UX、provider health/receipts | 密码学、native loader、Editor activation、engine release repository |
| Editor26 Online Provider | provider-neutral account handle、environment、secure credential lease基础合同 | Hub organization/marketplace/cloud project policy |
| Plugins01 Package Service | package identity/version/dependency/lock、artifact trust、install/update/rollback/uninstall | Marketplace offer、用户entitlement和Hub页面 |
| Editor06 Plugin Manager | project enable/disable、permission prompt、reload/restart、runtime diagnostics | 下载/购买、组织和云存储 |
| Tooling09 Release | engine release manifest/repository/signing/promotion/update trust | 第三方Marketplace catalog和project sync |
| Editor27 Source Control | repository/workspace/revision/changelist/diff/submit | Organization membership和blind file sync |
| Editor43 Collaboration | live multi-user transaction/activity/lock/presence | durable project backup、package distribution |
| Editor24 Runtime Cloud Storage | SaveGame/slot/platform-cloud semantics | Hub project source/artifact repository |

## 6. P0：远程入口启用前必须封闭

### P0-01 无provider时继续fail-close

Marketplace/Auth/Organization/Cloud action ID、route、command和capability在真实provider factory、version negotiation、health与terminal receipt存在前不得加入production。当前disabled reservations保留；fallback/demo也不能启用远程动作。

### P0-02 Local Git Identity与Remote Account硬分域

TopBar/User Menu/Team立即停止把Git author包装成“我的账户”或organization member。定义LocalProfile、GitIdentity、OnlineAccount、OrganizationMember四种typed projection；无Auth provider时Account明确Unavailable，Git信息仅留在Repository Contributors。

### P0-03 Marketplace不得绕过Plugin trust/admission P0

在Plugins01完成selection/trust/signature验证先于`Library::new`、disabled package零代码执行、package lock和隔离策略前，远程package即使下载/哈希成功也不得安装或加载。Hub不直接复制到扫描目录。

### P0-04 Secret必须由安全凭据owner托管

禁止access/refresh token、authorization code、client secret、cookie、device credential或encryption key进入`hub.toml`、完整Hub snapshot、action history、CLI、日志或crash bundle。先建立OS vault-backed credential broker、短期lease、redaction和revocation。

### P0-05 Cloud sync不得复用递归复制或last-writer-wins

没有versioned snapshot manifest、remote revision/CAS、staging、conflict artifact、atomic local apply和crash recovery前，Remote Sync保持disabled。本地package/action history不能作为同步source或完成receipt。

## 7. P1：工程级远程服务主链

### 7.1 Account、Auth与凭据

### P1-01 建立qualified account identity

账号handle至少包含provider、environment/issuer、subject和generation，display name/email/avatar只是attributes。禁止把Git email、字符串用户名或本机profile当authorization key。

### P1-02 区分Local Profile与Online Login状态

状态至少包括NoProfile、LocalProfile、SigningIn、OnlineReduced、Online、Refreshing、Expired、Revoked、Offline和Failed。UI、capability与操作授权由状态机投影，不由头像是否非空判断。

### P1-03 建立Auth provider registry

provider注册factory、provider ID、environments、protocol/schema versions、login methods、scope catalog、credential policy和owner lease。缺provider或版本不兼容时保持Unavailable，不自动选择首个实现。

### P1-04 隔离Development/Staging/Production环境

issuer、client ID、redirect URI、API endpoint、audience、scope、certificate pin/policy和account namespace按environment配置并受签名policy约束。生产UI不能接受自由URL或development credential。

### P1-05 使用标准授权流与外部浏览器broker

桌面interactive login采用Authorization Code + PKCE、设备流或受支持平台broker，校验state/nonce/redirect ownership并限制callback lifetime。Hub不收集服务密码，也不在WebView中承载不受信登录页。

### P1-06 建立OS-backed SecureCredentialBroker

Windows Credential Manager、macOS Keychain和Linux Secret Service由platform adapter封装；Hub只持opaque credential lease ID。vault不可用、锁定或权限拒绝有明确degraded/offline策略。

### P1-07 管理token生命周期与rotation

access/refresh token记录issuer/audience/scopes/expiry/key generation但不暴露bytes；刷新有single-flight、deadline、backoff和generation fencing。旧refresh结果不能覆盖新账号session。

### P1-08 建立可恢复AccountSession状态机

login、callback、refresh、switch、logout、provider disconnect和app restart均有operation ID与terminal receipt。session恢复先验证provider/credential/expiry，不凭本地“已登录”布尔值恢复。

### P1-09 支持账号link、switch与多profile

local platform profile、remote account和external identity通过typed link flow关联；切换前关闭旧lease和敏感query cache。跨账号catalog/entitlement/cloud数据严格按account generation分区。

### P1-10 实现scope与consent最小化

Marketplace browse、acquire、organization read、cloud read/write和publisher操作使用不同scope；UI解释新增scope并记录用户decision。后台不得借登录时一次性索取所有权限。

### P1-11 定义offline与cached profile策略

离线可显示带`stale_at/expires_at/source`的非敏感profile、已安装package和上次验证entitlement；禁止离线执行需要服务器authorization的purchase/invite/share。过期状态不显示Ready。

### P1-12 完成logout、revoke与本机清除

区分sign out、destroy persistent credential、revoke provider grant和remove local profile；每项显示影响范围。失败可重试并保留revocation intent，不能只清UI状态而留下refresh token。

### P1-13 建立全链secret redaction

DTO、serde debug、tracing、diagnostic、action history、support bundle、panic和HTTP错误统一使用sensitive field metadata。测试注入canary secrets并证明所有输出、文件和telemetry零泄漏。

### P1-14 提供deterministic fake Auth provider

fake覆盖success、MFA/continuation、consent、timeout、invalid state、token expiry、refresh race、revocation、offline和account switch；真实provider contract tests复用同一状态/receipt suite。

### 7.2 Organization、Membership与RBAC

### P1-15 建立Organization/Workspace/Project stable identity

organization、workspace和cloud project各有provider-qualified ID、display name、revision和lifecycle state。路径、repo URL和项目名只作metadata，rename不改变ACL目标。

### P1-16 将Git Contributors从Organization Members拆开

Team页分为Repository Contributors与Organization Members两个provider-backed view；前者明确source为Git history，后者携membership ID/status/role。任何一方缺失不伪造另一方。

### P1-17 定义versioned role/permission schema

权限使用stable action/resource IDs和deny-by-default policy，角色只是permission set模板；owner/admin/member/viewer等display role不能替代最终决策。schema变化有compatibility和migration。

### P1-18 服务端执行resource-action authorization

Marketplace acquire/publish、Cloud read/write/delete/share、Organization invite/role change均由服务端按principal、tenant、resource revision授权。前端disable只改善UX，不是安全边界。

### P1-19 建立Invitation状态机

invite拥有stable ID、organization、target identity、role、issuer、expiry和accepted/revoked/expired状态；重复发送、接受竞态和账号不匹配按CAS处理。email不是membership identity。

### P1-20 支持Group、Service Account与Automation Principal

CI/publisher/upload不复用个人refresh token；service principal使用最小scope、短期credential、rotation和owner。group membership expansion有cycle/budget和审计。

### P1-21 强制tenant隔离

所有cache key、cursor、operation、blob namespace、audit和diagnostic包含tenant/org ID；切换organization清空或分区view。测试证明相同package/project name不会跨tenant泄露。

### P1-22 建立immutable Audit Event

login/revoke、membership/role、entitlement、publish/yank、cloud share/delete/restore和policy denial产生actor/subject/resource/action/outcome/request/correlation/time记录。客户端日志不替代服务端审计。

### P1-23 定义PII最小化与数据生命周期

Git email、account email、IP、organization roster和audit按用途、consent、retention、export/delete policy处理。普通Hub snapshot默认不携完整email，support bundle需显式redaction preview。

### P1-24 处理membership revoke与离线cache

membership/role cache有TTL、revision和revocation event；高风险写操作在线复核。离线过期cache只能展示，不能继续cloud write或获取新package entitlement。

### 7.3 Marketplace Catalog、Publisher与Entitlement

### P1-25 建立Marketplace provider/repository registry

每个repository声明ID、base origin、environment、index/trust roots、supported package kinds、auth requirement和policy version。first-party、enterprise mirror和community source不能混成无来源数组。

### P1-26 使用签名、版本化Catalog Snapshot

index page/cursor绑定snapshot revision/digest、generated/expires time和signature chain；分页期间snapshot稳定。rollback、freeze、mix-and-match、过期或未知key必须fail closed并可切换已验证offline snapshot。

### P1-27 分离Package、Release、Offer与Artifact identity

Package ID标识产品，Release含version与compatibility，Offer表达价格/可见性，Artifact表达target/package kind/digest。UI row ID、manifest path或display name不得兼任四层identity。

### P1-28 建立Publisher identity与verification

publisher拥有stable ID、verified domains/organization、signing keys、status和security contact；package namespace ownership、防仿冒和transfer有审计流程。UI明确显示publisher与trust tier。

### P1-29 完成严格Marketplace metadata schema

metadata包含name/summary/description/category/tags/media、license、privacy/security URL、support、source、maturity、capabilities、permissions、targets、size和release notes。未知字段、预算超限和危险markup按schema拒绝或隔离。

### P1-30 建立独立version与engine compatibility

每个package release使用SemVer或明确version scheme，声明engine Build Set/API/ABI/schema/target range与deprecation。禁止继续用统一`0.1.0`和宽泛engine range推导兼容。

### P1-31 建立可求解dependency contract

dependency包含package ID、version requirement、source/repository、feature/capability、target、optional/conflict和artifact constraints。solver输出完整解释、lock和unsat core，结果不依赖catalog返回顺序。

### P1-32 建立platform/architecture/package-kind variants

同一release可有source、data、native、WASM/ZrVM及Windows/Linux/macOS/arch变体，每个变体单独digest/signature/size/permission。无当前target artifact时UI明确Unsupported。

### P1-33 分离Offer、License与Entitlement

免费、付费、订阅、seat、organization license与private package通过provider policy表达；entitlement绑定account/org/package/quantity/expiry和source transaction。Hub不自行计算价格、税或授权结论。

### P1-34 提供server-side搜索、过滤、排序与cursor

query有normalized text、category/tag/license/publisher/trust/target/compat/price filters、stable sort、cursor、total/freshness和deadline。10万package不能先下载全量数组再在React过滤。

### P1-35 隔离remote description、image与link

description使用安全受限markup；image由受控fetch/cache解码并限制格式、尺寸、像素与重定向；外链经allowlist/确认由系统浏览器打开。remote content不能进入Tauri privileged origin。

### P1-36 建立Review、Rating、Report与Moderation边界

review与rating由已验证主体/entitlement policy约束，举报、恶意包隔离、yank、security advisory和appeal有stable state/audit。Hub只投影provider结果，不在客户端聚合可信分数。

### P1-37 合并Local Inventory与Remote Catalog时保留provenance

同一package显示installed version/source/digest、available releases、lock/entitlement/update/trust状态；local path package不能因ID相同冒充remote publisher。collision必须阻断或显式选择source。

### P1-38 建立bounded cache与offline snapshot

catalog metadata/media按repository+snapshot+locale缓存，有size/TTL/GC和checksum；离线只使用完整已验证snapshot。partial page、失败image和过期entitlement不会污染Ready状态。

### 7.4 Package Acquire、Install与Activation

### P1-39 生成project/package lockfile

resolve结果记录package/release/repository/artifact digest、publisher/signing identity、target、dependency closure、permissions和solver version。另一台机器离线解析到相同closure，未锁变体不能静默替换。

### P1-40 先验证signed payload manifest再触达代码

Hub acquire只接受Package Service返回的verified manifest与artifact handles；schema、compatibility、entitlement、signature/revocation和hash验证必须发生在解压、probe或`Library::new`前，并防TOCTOU。

### P1-41 使用content-addressed artifact与独立trust receipt

artifact以digest寻址，repository拒绝同digest异bytes；publisher signature、platform signing/notary、malware/static policy和provenance分别产生receipt。普通SHA-256只证明完整性，不证明授权发布者。

### P1-42 建立生产级bounded resumable download

下载强制TLS/host allowlist、redirect policy、Range validation、chunk digest、deadline、retry/backoff、bandwidth/proxy和有界并发；resume journal持久化且只复用回读通过chunk。取消释放lease并保留可回收partial。

### P1-43 实现staging与atomic install

解包在唯一staging目录，防path traversal/symlink/hardlink/bomb并校验文件manifest；flush完成后原子发布immutable version slot。失败或崩溃不改变installed/current state。

### P1-44 以dependency closure为安装transaction

prepare阶段冻结完整closure、磁盘/权限/冲突和expected lock revision；所有包验证完成后一次commit。任一包失败不产生部分lock、部分目录或混合generation。

### P1-45 建立permission、trust tier与用户decision

package声明filesystem/network/process/editor/runtime/native等能力，Hub显示增量权限和publisher/trust来源；decision绑定package release/digest。未知第三方native默认隔离/禁用，不能套first-party policy。

### P1-46 将native隔离设为Marketplace前置合同

高风险package在child process、WASM/ZrVM或受控host运行，声明CPU/memory/I/O/callback budget和crash policy。仅“已签名”不能获得Editor主进程任意代码权限。

### P1-47 将activation委托Editor06

Hub安装只改变package inventory/lock candidate；项目enable/disable、dependency admission、restart/reload、state migration和diagnostic由Editor Plugin Manager执行并返回receipt。Hub不直接编辑运行中Editor目录。

### P1-48 实现versioned update与rollback

update保留旧immutable slot与lock，先验证兼容/migration/permission delta，再atomic switch；activation health失败自动回退。downgrade/rollback检查project data migration不可逆性。

### P1-49 建立repair、uninstall与owned resource inventory

installed record逐文件/系统资源记录owner和digest；repair只恢复损坏项，uninstall只删除owned resource并检查依赖/项目引用。shared cache通过reference count/lease GC，不能递归删除猜测目录。

### P1-50 完成preflight、progress与terminal receipt

检查entitlement、compatibility、disk、path、process lock、network和policy；进度区分resolve/download/verify/stage/install/activate。receipt含operation/source/snapshot/lock/artifact/attempt/outcome/recovery，UI只在commit后显示Installed。

### 7.5 Cloud Project Repository与Sync

### P1-51 先拆分Cloud service classes

至少区分Project Snapshot/Backup、Source Control remote、Build/Package Artifact、Runtime SaveGame Cloud和Live Collaboration。Cloud页按provider/capability展示，不用一个Remote Sync动作替代所有语义。

### P1-52 建立versioned ProjectSnapshotManifest

manifest记录project ID、base/parent revision、source revision、engine/lock/schema、entries、ignore policy、created actor/time和content digest。绝对本机路径、mtime或文件数不能作为远程identity。

### P1-53 使用content-addressed blob/chunk store

文件按策略分块、压缩、digest寻址并去重；manifest create-only，blob上传幂等。大文件、稀疏文件、权限/链接和不支持类型有显式policy与预算。

### P1-54 以remote revision与CAS更新head

push提交`expected_base_revision`，服务器只在head匹配时发布新revision；pull也记录observed head。stale写返回conflict artifact，不以最后完成请求覆盖更新内容。

### P1-55 建立local change journal与稳定scan

watcher事件只是hint，snapshot以一致性scan、ignore policy和re-read validation构建；变化中的文件重试/隔离。journal记录dirty paths、base revision和upload state，重启可恢复。

### P1-56 实现明确Sync状态机

Idle、Scanning、Comparing、Uploading、Publishing、Downloading、Staging、Applying、Conflicted、Completed/Failed/Canceled均有generation与receipt。并发sync、账号/org/project切换和shutdown按lease终止。

### P1-57 产生typed conflict artifact

冲突区分both-modified、delete/modify、rename、binary、schema/provider unavailable、permission和remote deleted；携base/local/remote revisions及安全预览。不得只返回“sync failed”。

### P1-58 委托VCS与Editor semantic merge

Git/source project使用Editor27 provider执行branch/diff/merge；scene/asset语义冲突可调用Editor42/43的typed diff/merge primitive。Hub只协调plan和结果，不自写文本三方merge或覆盖binary。

### P1-59 建立offline operation queue

离线只排可安全重放的snapshot/upload intent，记录base revision、account/org/project generation、deadline和user confirmation。恢复在线先重新授权/比较head，绝不盲目replay stale delete/share。

### P1-60 定义传输、静态与端到端加密

TLS为底线；服务端加密、tenant key、可选client-side project key分别声明。key由credential/KMS lease管理，rotation、recovery、revocation和丢失后果有产品流程，不能塞进project文件。

### P1-61 实现quota、retention与GC

provider返回logical/physical bytes、object/revision limits、retention、trash和billing/plan source；上传前预检。blob GC只清理所有manifest/retention/legal-hold均无引用的对象。

### P1-62 建立share/ACL与download authorization

project/revision/artifact分享使用typed ACL或短期scoped link，包含audience、expiry、permissions和revocation；URL本身不是永久权限。每次download按principal/tenant/resource验证并审计。

### P1-63 使用staged atomic local apply

pull先下载并验证完整manifest/blob，在project外staging；生成apply/backup plan并检查本地dirty/locks。atomic或journaled replace失败可恢复原project，不能逐文件覆盖正在编辑的workspace。

### P1-64 排除secret、cache与生成物

ignore policy默认排除`.git` credential、Hub/token配置、build/target/DDC、crash dump和机器私有文件；规则版本化且在上传前显示敏感扫描报告。unknown secret命中默认阻断。

### 7.6 Hub Protocol、UX与Operations

### P1-65 扩展versioned IPC而非完整snapshot轮询

为auth/org/catalog/install/sync定义typed request/result/event schema、protocol revision、operation ID、cursor和resync；secret永不进入WebView。长任务发布delta，不在全局mutex内做network/I/O。

### P1-66 重构账号、Team、Catalog与Cloud信息架构

TopBar明确Local/Online状态和active org；Account、Repository Contributors、Organization Members分开。Catalog分Local/Marketplace，Cloud按Snapshots/Sync/Artifacts/Services显示真实provider state。

### P1-67 投影provider health与freshness

每个provider显示Unavailable/Authenticating/Ready/Degraded/Offline/Stale/RateLimited、endpoint environment、last success和retry。在一个服务失败时其他本地工作流继续可用。

### P1-68 建立可取消Operation Center

login/acquire/install/sync/share均有独立row、phase/progress、bytes/rate/ETA、cancel/retry、diagnostic和artifact；连续任务不覆盖全局单槽。terminal记录可按retention查询和导出。

### P1-69 统一rate limit、retry与circuit breaker

解析server retry-after和idempotency key；query/read与mutation采用不同retry policy。连续失败打开per-provider circuit，manual retry和恢复事件可见，避免重启风暴。

### P1-70 建立端到端observation与redaction

client/provider/server使用correlation/request/operation/account-generation/org/resource IDs关联，但日志默认哈希/省略PII和secret。SLO覆盖login、catalog、download/install、sync与audit delivery。

### P1-71 为remote content启用CSP与origin sandbox

production CSP非空；connect/img/navigation origin按provider policy授权，禁止inline/eval和remote script。Marketplace HTML/media在unprivileged renderer或安全codec处理，不能调用Tauri commands。

### P1-72 建立CI、contract、fault、security与scale矩阵

加入generated Rust/TS codecs、fake providers、real staging contract、OAuth callback、vault、RBAC/tenant、signed index、package attack corpus、sync conflicts、crash/recovery、offline/rate-limit和10万package/TB project基准；真实Tauri窗口为release gate。

## 8. P2：主链稳定后的增强项

### P2-01 Payment、Tax与退款合规

如引入付费市场，由外部合规commerce provider处理payment method、税、invoice、refund、chargeback和地区限制；Hub不保存卡数据。

### P2-02 Publisher Portal与Release Workflow

提供namespace申请、package upload、validation preview、staged rollout、yank、security advisory和support analytics，所有动作受organization RBAC和audit约束。

### P2-03 自动安全分析与Moderation

在隔离worker运行malware、secret、license、native import、behavior和content policy分析，结果可解释、可复核、可申诉。

### P2-04 Enterprise SSO、MFA、SCIM与Policy

扩展OIDC/SAML federation、MFA step-up、SCIM lifecycle和conditional access；不能把enterprise policy硬编码进普通账号状态机。

### P2-05 KMS、BYOK与客户托管密钥

为企业Cloud Repository支持tenant key、BYOK/HSM、rotation、dual-control、recovery和cryptographic deletion。

### P2-06 多区域复制与灾备

定义region residency、replication lag、RPO/RTO、failover、read-after-write和legal hold；通过真实区域故障演练验收。

### P2-07 Air-gapped Mirror与离线License

支持签名catalog/package bundle、离线trust root、entitlement lease和单向promotion；mirror import仍验证来源、expiry与revocation snapshot。

### P2-08 CDN、Delta与Peer Cache

在content-addressed完整下载正确后增加delta、multi-CDN、LAN peer cache和带宽调度，最终bytes仍以target manifest验证。

### P2-09 Cloud Review与Live Collaboration联动

将snapshot/revision与Editor43 session checkpoint、review annotation和branch关联；durable repository与实时transaction仍保持独立owner。

### P2-10 Enterprise Package Policy

支持organization allow/deny/pin、approved publisher、license/security threshold、mirror-only和emergency revoke，并提供dry-run影响报告。

### P2-11 Privacy-preserving Recommendation与Analytics

推荐/使用分析必须opt-in、最小化、可删除、可解释且不影响基本catalog；不得上传项目内容或插件使用细节作为默认行为。

### P2-12 跨引擎与服务基准

以相同catalog规模、package体积、网络故障和project数据集测量Godot/Unreal可见基线及Zircon正确性、延迟、吞吐、内存和恢复；没有实测不声称领先。

## 9. 关键产品合同

### 9.1 Account与Authorization

| 合同 | 必要字段 | 禁止替代 |
|---|---|---|
| `AccountHandle` | provider/environment/subject/generation | Git name/email、display name |
| `AccountSessionSnapshot` | state、scopes、expiry、active org、provider health、credential lease ref | `logged_in: bool`、token bytes |
| `OrganizationMembership` | org/member IDs、status、role/policy revision、freshness | commit author row |
| `AuthorizationDecision` | principal/resource/action/policy revision/outcome/reason/correlation | disabled button |
| `CredentialLease` | opaque ID、kind、scope、expiry/generation、owner | TOML/JSON secret string |

Hub的本地profile在未登录时仍可打开项目、构建和使用已安装的离线内容；任何需要远程授权的动作必须消费online account + organization + fresh service decision。这样既不强迫本地引擎依赖云服务，也不允许local identity越权。

### 9.2 Marketplace与Package Service

```text
CatalogSnapshot(revision, expiry, signature)
  -> Offer/Entitlement check
  -> PackageRelease + target Artifact
  -> deterministic dependency resolve
  -> ProjectPackageLock(expected revision)
  -> acquire/verify/stage transaction
  -> PackageService commit
  -> Editor06 activation/restart receipt
```

Marketplace拥有“可发现/可获取什么”，Package Service拥有“解析并安全安装什么”，Editor拥有“当前项目实际启用什么”。三者共享package/release/artifact identity，但不能共享状态authority。免费包可跳过purchase，不可跳过publisher/trust/license/compatibility。

### 9.3 Cloud Snapshot与Sync

`ProjectSnapshotManifest`只引用content-addressed blobs并记录base/head revision、engine/package lock、ignore policy和source provenance。push先上传missing blobs，再以CAS发布manifest/head；pull先下载/验证到project外staging，再生成local apply plan。conflict不改变remote head或local workspace，用户或domain merger解决后产生新revision。

### 9.4 状态真实性矩阵

| UI状态 | 最低真实依据 |
|---|---|
| Signed Out / Local Profile | 无active credential lease，local profile source明确 |
| Online | provider验证且token未过期，account generation当前 |
| Organization Member | fresh membership revision与服务端authorization capability |
| Available | signed catalog snapshot中存在compatible release |
| Owned/Entitled | account/org-qualified entitlement receipt仍有效 |
| Downloaded | artifact bytes与signed manifest digest一致 |
| Installed | Package Service transaction committed，inventory/lock一致 |
| Enabled | Editor06 activation receipt对应当前project/package generation |
| Synced | local snapshot digest与observed remote head一致且无uncommitted change |
| Conflict | typed base/local/remote artifact已持久化，未做覆盖 |

## 10. 里程碑

| 里程碑 | 交付与退出条件 |
|---|---|
| M0 | 真实性硬切：Local Git/Account分域，所有remote capability继续fail-close，Marketplace受Plugin trust P0阻断，secret字段禁入现有DTO |
| M1 | Auth/Credential：provider registry、environment、PKCE/device flow、OS vault、session/refresh/logout/redaction与fake provider通过 |
| M2 | Organization/RBAC：qualified org/member、role/policy、server authorization、invite、tenant isolation、audit/PII通过 |
| M3 | Marketplace Catalog：signed snapshot、package/release/offer/artifact、publisher、metadata、query/cache与remote content sandbox通过 |
| M4 | Resolve/Lock/Trust：dependency solver、target variants、entitlement、project lock、signature/revocation与permission plan通过 |
| M5 | Package Transaction：bounded download、safe unpack、atomic install/update/rollback/repair/uninstall和Editor activation handoff通过 |
| M6 | Hub UX/Protocol：typed IPC delta、Account/Org/Catalog/Cloud IA、provider health、Operation Center、offline/degraded状态通过 |
| M7 | Cloud Repository：snapshot manifest、CAS blob、head revision、quota/retention/encryption/share authorization通过 |
| M8 | Sync/Conflict：local journal、state machine、typed conflict、VCS/semantic merge、offline queue、atomic apply/recovery通过 |
| M9 | Operations/Security：rate limit/circuit、SLO/correlation、audit retention、CSP/origin、incident/revoke/support bundle通过 |
| M10 | Scale/Fault/Cross-platform：10万package、TB project、long offline、crash/network/vault/provider faults及Windows/Linux/macOS通过 |
| M11 | 硬切与资格：删除Git-as-account和fallback remote fixture、默认provider装配、文档/CI/runbook/benchmark/release gates闭合 |

依赖顺序为M0 -> M1 -> M2/M3 -> M4 -> M5/M6 -> M7 -> M8 -> M9 -> M10 -> M11。M1前不得显示Online；M2前不得开放organization write；M4前不得下载可执行包；M5前不得显示Installed；M7/M8前不得显示Synced。

## 11. 产品资格门

1. **G01** 无Auth/Marketplace/Cloud provider、协议不兼容或backend失败时所有remote action保持Unavailable，本地项目/构建仍可用且不显示假Ready。
2. **G02** TopBar、Account、Team在未登录时只显示Local Profile/Git Contributors，绝不把Git author称为online account或organization member。
3. **G03** login state/nonce/PKCE verifier/redirect URI错误、重放、超时和并发callback全部fail closed且无credential落盘泄漏。
4. **G04** OS vault锁定、不可用、权限拒绝和进程崩溃有明确恢复；Hub config/snapshot/history/log/support bundle零token/secret。
5. **G05** refresh、account switch、logout与revoke竞态由generation fencing解决，旧结果不能恢复已退出账号。
6. **G06** scope/consent增量可解释，拒绝新增scope不影响已有本地能力；敏感操作需要fresh/step-up authorization。
7. **G07** organization A的principal、cursor、cache、blob和audit在相同名字/ID输入下不能读写organization B数据。
8. **G08** role/member/invite change按expected policy revision提交，stale mutation零写入；服务端deny不能被客户端enable绕过。
9. **G09** membership撤销和账号logout在定义SLO内使Marketplace/Cloud lease失效，离线cache不能继续高风险写入。
10. **G10** login、role、invite、entitlement、publish/yank、cloud write/share/delete均产生不可变可检索audit且secret/PII按policy脱敏。
11. **G11** catalog snapshot、page、media manifest任一字节修改、rollback、freeze、过期、未知/revoked key或mix-and-match均被拒绝。
12. **G12** package/release/offer/artifact/publisher identity在rename/localization/pagination后稳定，local ID collision不冒充remote package。
13. **G13** 10万package catalog的server query/cursor/filter/sort结果稳定，无全量前端加载，输入/滚动/内存满足预算。
14. **G14** free/paid/org/private package的entitlement、expiry、seat和offline policy有完整matrix，Hub不自行伪造Owned。
15. **G15** 相同snapshot、project/target和policy重复resolve得到相同closure/lock digest；unsat输出可解释最小冲突链。
16. **G16** package在selection/trust/signature/compat/permission完成前零code execution；disabled package在任何目录中均不`Library::new`。
17. **G17** path traversal、symlink/hardlink、zip bomb、duplicate path、case collision、bad signature和malware fixture均在staging隔离失败。
18. **G18** 下载中断/重启只复用已回读验证chunk，TLS/host/redirect/range/digest错误不进入verified artifact cache。
19. **G19** dependency transaction任一download/verify/install fault后installed inventory、project lock和current generation保持旧状态。
20. **G20** update activation crash/health failure自动rollback；repair/uninstall不删除非owned或仍被其他package/project引用的文件。
21. **G21** permission/trust增量由用户确认并绑定release digest；未知第三方native不能进入Editor主进程故障域。
22. **G22** Hub install与Editor enable/reload/restart receipt关联准确，Installed、Enabled和Running三种状态永不混用。
23. **G23** snapshot manifest canonical、重复构建digest一致，absolute path、secret、build/cache和机器私有文件不进入remote content。
24. **G24** 两客户端以同base并发push时只有一个CAS成功，另一个获得typed conflict且local/remote内容均未覆盖。
25. **G25** rename/delete/binary/scene/schema/provider-unavailable conflict经VCS或semantic merger解决后可重放、可审计且digest稳定。
26. **G26** pull在download、verify、stage、apply、flush和process crash任一fault点都能恢复原workspace或完成journal，无半应用。
27. **G27** TLS、at-rest/optional client-side encryption、key rotation/revoke/recovery通过；key bytes不进入project或Hub WebView。
28. **G28** quota、disk、retention、trash、GC和legal hold测试证明无引用丢失、误删shared blob或无限存储增长。
29. **G29** 长时离线queue恢复时重新验证account/org/head/policy，stale delete/share/write不被盲目执行。
30. **G30** remote markup/image/link通过CSP、origin allowlist、sanitizer/decoder budget和system-browser policy，不能调用未授权Tauri command。
31. **G31** fake/staging provider端到端覆盖login、org、browse、entitle、resolve、install、enable、snapshot、conflict、recover、logout/revoke和audit。
32. **G32** 与Unreal/Godot/Fyrox/Bevy/Unity Graphics可见能力及服务目标在相同规模/故障下公开测量correctness、latency、throughput、memory、storage和recovery；仅实测达标才可声称领先。

## 12. 验证说明

本轮是review-only，没有修改production Hub、Runtime、Plugin、Tooling、Editor代码或tests，也没有连接远程账号、Marketplace或Cloud service。当前代码不存在这些provider，动态mock成功不能增加真实证据；已知Hub Rust `persist_unchecked(None)`编译P0仍存在，因此没有重复同一managed Cargo失败lane，也没有运行真实Tauri截图或远程security测试。

本报告静态验证要求：78个selected path存在且无重复/在途；fingerprint匹配；P0/P1/P2分别为5/72/12；M0-M11连续；资格门为32；frontmatter、Hub索引、根索引与coverage链接无断链；Markdown为LF、无trailing whitespace、BOM或占位标记。实施阶段必须先恢复Hub build，再执行Auth/vault/RBAC/tenant、signed catalog/package attacks、transaction fault、Cloud CAS/conflict/recovery、CSP、跨平台、scale和真实staging provider资格。

## 13. 审查决策

1. 保留所有remote `comingSoon` disabled状态，直到对应里程碑和资格门真实通过。
2. 本地Git identity与contributors保留，但从Account/Organization语义硬切出去。
3. Hub local plugin catalog保留为inventory adapter；Remote Marketplace建立独立signed provider，不把递归scanner接网络。
4. Package identity/solver/lock/trust/install/update/rollback由Plugins01共享Package Service拥有；Hub不复制实现。
5. Engine update/release repository继续由Tooling09拥有，Marketplace不得复用未分域的engine channel。
6. Project Cloud Snapshot、VCS、Live Collaboration和Runtime SaveGame Cloud保持四个owner；Hub只提供统一入口与协调receipt。
7. 远程内容进入前先启用CSP/origin/sanitization，远程代码进入前先关闭native pre-admission执行P0。
8. 性能目标通过signed cursor snapshot、bounded cache/download、content-addressed dedup、incremental scan和实测取得，不通过省略授权、签名、冲突或恢复取得。
