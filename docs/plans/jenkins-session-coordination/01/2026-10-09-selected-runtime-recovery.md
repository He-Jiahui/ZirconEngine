---
doc_type: implementation-record
related_code:
  - .jenkins/state/migration-baseline/resume_selected_execution_upgrade.py
  - .jenkins/state/migration-baseline/retire_failed_selected_upgrade_holder.py
tests:
  - immutable control-byte checks and native lifecycle observations
---

# 已选 Jenkins 版本的启动恢复

截至 2026-10-10，正式运行版本为 `0c1f8adbc3d675d524d508db80688e6ff3311aea055700024743575bca2e4099`，运行身份为 `start-1791615932752147100`。controller、Windows agent 和 79 个插件就绪；可见托盘右键、停止、启动及单请求重启均已实测并独立复查。新的六文件 Rust 工程完成两轮真实增量补丁、编译、测试发现、测试执行及权威验收，第二轮复用兼容池并重新测试；公开源码交接及日常入口同 ID 只读对账通过。实际原件见本文末尾。这些证据不表示产品整仓、M4 全覆盖或 M5–M8 全部通过；下文按阶段保留 `613`、`191` 的运行与失败历史。

## 已完成的实际操作

完整支持层回归收集 692 项，690 项通过，2 项因环境无法创建目录别名而跳过，没有失败或错误。原候选、配置及对应源码哈希未改变。正式受限预检已验证允许路径写入、控制配置拒绝写入、配置字节保持一致及宿主的实际 Job 关联。此记录不表示当前聊天采用了该受限 profile。

正式升级清单为 `.jenkins/state/deployment/usable-upgrade-613a0b5d47bcb59a-1791582787378554200.json`，SHA-256 为 `dcd76253be4fb9ed3c723b530e8020607ba82bbfb32bc442bdd2b87368fd420e`。升级已停止旧代 `start-1791561005966136300`，controller 与 agent 的原生证明均满足零活跃进程、子进程消失和输出流 EOF；supervisor、controller、agent 按精确出生时间观察均已离开。旧 keeper 也已按身份退出。

同一次升级选中了候选，但在重新核对终态时报告 `execution_upgrade_terminal_changed`。失败宿主 `34964` 未取得新的 `ownedGeneration`，没有该操作器签发的新运行启动回执。失败记录保留，未伪造运行接受，也未释放四项历史资源保护。随后读取相同原意图的终态差异为空；使用公开的原意图对账入口成功恢复到 `selected-awaiting-runtime-start`，没有重新选择或新建构建请求。

## 后续失败启动的核对

之后出现 `start-1791584364124307700`。目前没有足够事件原件证明它由哪个桌面调用者发起，因此不将其归属给升级宿主，也不将旧托盘有启动能力视为完整的来源证据。

该次 controller 完成初始化后，agent 的首次配置 POST 因默认五秒 API 超时失败。生命周期宿主已终止 controller，原生终止证明完整；`agentLaunchAttempted=false` 且没有 agent 身份或终止证明，表示 agent 实际尚未启动。supervisor `36484` 与 controller `3960` 的精确出生时间已观察为离开，官方终态检查通过。

实际只读证据为 `.jenkins/state/migration-baseline/failed-external-startup-terminal-1791585270380421300.json`，SHA-256 为 `03dada2e2139076eb906febda6d7a323f9049221d665b131c0baae1ae473b02e`。它保存 operation、host、selector、选择日志的原始字节、相应哈希、原意图和清单引用、原生离开观察及全部历史保护。它不授予运行、Cargo、历史保护释放或来源归属资格。

## 已完成的接续

接续沿用相同原意图和已选候选，没有重新选择。原失败管理宿主 `34964` 已在完整输入、精确 PID 出生时间及直属子进程身份核对后退出。完成证据为 `.jenkins/state/migration-baseline/failed-selected-holder-retirement-1791586512387582900-completed.json`，SHA-256 为 `7474d816e266ec7553b7f8368974f362fa36a27afba0ea6a3c8d2c0597e88551`；它没有将来源不明的失败启动归属给该宿主，也不释放历史保护。

新的接续执行者通过官方 `codex.exe sandbox -P zircon-strict --include-managed-config` 持续运行，保留真实操作器 `resume_selected_execution_upgrade.py` 的 argv 与代码 SHA。实际 keeper 为 `10148`，新运行身份为 `start-1791587008309259500`；supervisor 为 `34428`，controller 为 `32276`，agent 为 `39312`，三者的出生时间、命令身份、原生运行回执和在线健康均已核对。控制器零执行槽、Windows agent 一个执行槽。

不可变接续签发为 `.jenkins/state/deployment/history/execution-upgrade-selected-continuation-1e64bd5b07ebe1d4854afaad3187cf2a533bd9ba6c37ef56f1114aaa93005728.json`，SHA-256 为 `57946754974cef569673500e727dacccd84876b43249bfeeff3ab1b7de36589f`。实际启动回执为同目录 `execution-upgrade-runtime-started-75c5ec4e64b154da8f44fccfbd655ec692ff97795e3c18db389873c38a0bdb37.json`，SHA-256 为 `0b99e84cbd3b9f330098b6fd7a7cd4abcdfb964117a770f0a8f59fd717cc7773`。启动后的只读核对 `selected-continuation-started-verified-1791587401429532500.json` 返回成功；SHA-256 为 `1a65a68b5365c1be47e942a2e19c2b2d34e4f0df0f831c05901abf2a2066c77e`。该证据只关闭本次启动核对，后续合法请求写入不要求当前状态库永远保持启动时的完整快照。

桌面托盘已刷新为 `35960`，位于 `Default` 桌面，Explorer 已确认图标。实际 Version 4 右键消息能展开菜单并关闭；证据为 `.jenkins/state/tray/visible-ui-proof-1791587862702325700.json`。刷新与菜单测试没有停止服务。正常托盘停止／启动／重启的受限宿主继承仍由下一版代码与运行验收单独完成。

## 新 M4 请求的实际结果

描述符 `.jenkins/tmp/acceptance/m4-desktop-613-1791587458/descriptor.json` 只包含原始六文件 Rust 验收工程，显式选择真实 `F:\cargo-targets`，没有 Git 基线、直接 Cargo、提交或通知。`zircon-flow #34`（queue 1）实际接收了九项请求参数和十八项空默认值，原始 Jenkins API 数据及独立审计保存在 `.jenkins/state/migration-baseline/jenkins-build34-queue1-613-parameters.raw.json` 与 `.audit.json`；参数审计通过，构建结果为 FAILURE。

失败发生在补丁准备阶段，记录 `PermissionError`。协调器对账确认没有 source patch operation、workflow 或 execution dispatch，终止证明类型为 `no-execution-dispatch`，原源码 hash 保持不变。F 盘物理根合法，但已配置写入根 `F:\cargo-targets\zircon-local` 原本尚未创建；首次源码 preimage 物化需要创建该目录。实际正式受限探针记录了该目录的 WinError 5，证据 `actual-strict-build-grant-missing-F-1791588370.json` 的 SHA-256 为 `28463e6480efe986634b8e13a124bfe54e0f48a95bff19bd28c6f2e2dbcdbb1c`。

已在用户授权的现有物理写入根初始化空目录，没有扩展 profile、手工修改 ACL、复制源码对象或启动 Cargo。初始化记录为 `build-write-anchor-provisioning-1791588547270549400-completed.json`，SHA-256 为 `2cb55bbfca866c327c8e18f4298ca96a9ef19acc74445bf081ad8bb750b6aaba`。随后新官方受限宿主的文件写入通过，证据 `actual-strict-build-grant-existing-F-1791588619.json` 的 SHA-256 为 `335b5206f61a52c2bfe7465547521b8253e944eae3de6c66b8e10af7993c5c5c`。

第二份独立原始描述符 `m4-desktop-613-grant-1791588735/descriptor.json` 的 `zircon-flow #35`（queue 4）仍在相同阶段失败。其真实参数审计使用描述符的完整身份，九项参数和十八项空默认值通过；原始 API 字节 SHA-256 为 `8baeeebf408754641ba0491fdcf7e25c66bdb6efc3748c2b597144f545289bcb`，审计回执为 `jenkins-build35-queue4-613-grant-canonical-parameters.audit.json`，SHA-256 为 `725039b81d54ecb50aa5d128dd82cbefeb909f307bad776c32ca610f215ee876`。此前一次因调用方复制身份时遗漏字符而失败的审计保留为独立记录，不把它解释成 Jenkins 参数错误。

对现有 keeper、supervisor、controller、agent 的实际 primary token 与目录 DACL 只读观察显示：四者各有 120 个受限 SID，均没有匹配 F 盘目录的适用写入 ACE，而工程 `.jenkins` 的写入 ACE 匹配。进程出生时间、exe／argv 摘要、令牌标识和 DACL 首末均一致；证据 `live-build-grant-restricted-tokens-1791589468556778700.json` 的 SHA-256 为 `5aa27bd1547ca3f630cddd7a73b1ad4194d58f925e55a473dd47fdc21f0ac75d`。该证据证明现有 primary token／DACL 不匹配，不声称已执行 AccessCheck 或证明新部署成功。下一次受管切换须在目录已存在时由新的正式受限宿主签发原生进程；保留两次终态失败请求、全部前像及历史保护，不重放或伪造 M4 接受结果。

增量补丁验收显式使用真实 `F:\cargo-targets`，全局默认仍为 `D:\cargo-targets`。Jenkins 健康、插件安装、托盘右键和真实 compile/list/test 回执分别核验；单独的 Jenkins SUCCESS 或健康页面不关闭 M4 接受门槛。

## 191 版本的运行与补丁验收历史

新部署沿用工程内 Home、运行组件、agent、日志和控制状态，构建根配置没有自动换盘。升级清单为 `.jenkins/state/deployment/usable-upgrade-191f9aa4e455cb24-1791595837563356400.json`，SHA-256 为 `6b3f2b3cdc23c9b7305200335446732ab7e506466e471535f2e026985d6d12ab`。该版本完整支持层回归收集 741 项，739 项通过，2 项目录别名测试因环境跳过；证据 `patch-support-regression-1791592555622312300.json` 的 SHA-256 为 `9ad59643fe1c7e34b55a388040bff182c0c687b7f0bd89f25fff1370b18cf5c0`。

正式 keeper 为 `6140`，运行身份为 `start-1791596692121616400`，controller 为 `38116`，agent 为 `41400`。实际启动回执为 `.jenkins/state/deployment/history/execution-upgrade-runtime-started-5b741beaea2836b2631ad40083ed33395542a3ea1d0aa1ea882fbe3bcf30183d.json`，SHA-256 为 `f9b0e7f204bee3a1210dec7617120b2755a9085129db290a3178fcd3f125184c`。前代 `613` 的服务原生终止、相关进程离开及 keeper 退出均由本次升级核对；两次历史失败请求、外来任务与四项历史资源保护仍保留。

原始描述符 `.jenkins/tmp/acceptance/m4-191-desktop-1791596969/descriptor.json` 的 SHA-256 为 `e18ab759c4101b208c06260ed31d18f2314ea85ef04cca15c55daf65182dbb0d`。第一轮执行 `7bb67fb90ace50788337087555115966` 通过，源码身份为 `05a30854fcb6325a9b49aada2c380e54f343f7f823a5d995093767ff3e4bc2c5`，权威接受回执摘要为 `444d97eee8cfd706b76b9f6e295555c183e43c9219100ab47fb42f2f9ee3880b`。编译、测试发现和测试使用独立的原生 Job，完整终止证明、实际输出闭包和必需测试均经适配器核对。原始输出 `formal-patch-invocation-1791597398293775100.stdout.json` 的 SHA-256 为 `e5199a1d7721b680a025d58dd0b7f03d01a1bf9209cae77883e3e01ae114b968`。

同一描述符的第二轮单次请求在应用前因 `path_claim_conflict` 失败，对账记录 `foreign_claim_during_compensation`。原始输出 `formal-patch-invocation-1791597856888917200.stdout.json` 的 SHA-256 为 `e2f81d485e7315a16af94feb4ba67a6cda0570cb871366d8c10e38ff3e2da62f`。第一轮的已接受源码和引用保持原样，未重放失败请求或改写旧描述符。当时流程仅在 Git 提交后释放路径，尚缺少不提交时的受控源码交接；新版本的交接证据见后文，原第二轮失败结果继续保留。

托盘启停初始化实际拒绝了合法 keeper：Windows 命令行查询返回元组，而 guard 的候选值是列表，值和命令摘要虽一致，序列比较仍失败。最小修正与 16 项针对性控制已通过，补丁已应用到待部署源码，当前封存的运行版本保持原样；需要由正式新版本加载，不能在 root 中冒充 keeper 或补写启停描述符。

新托盘 `34276` 曾位于沙箱私有桌面，未取得 Explorer 图标。随后按精确 PID、出生时间和命令身份只替换该 UI 进程，服务保持运行；可见托盘 `2852` 位于 `WinSta0\\Default`，Explorer 图标已确认，实际 Version 4 右键菜单可展开并关闭。刷新证据为 `.jenkins/state/tray/python-formal-tray-refresh-1791599144023887600-completed.json`，SHA-256 为 `126cfd53a0f09ffe4a3cccba94b5ed95dff6355b35aa67728d945707a16b72c6`；菜单证据为 `.jenkins/state/tray/visible-ui-proof-1791599241124603900.json`。菜单检查未执行启停，完整托盘停止／启动／重启验收仍开放。

第一轮 Jenkins 的独立参数审计也已通过：queue 7 对应 `zircon-flow #36`，结果为 SUCCESS。完整九项请求参数和十八项空默认值匹配原描述符；原始 API 字节为 `.jenkins/state/migration-baseline/jenkins-m4-191-desktop-1791596969-build36-queue7-parameters.raw.json`，SHA-256 为 `09ca55d4afb55314085e703c28da77e60373a53b392321e8fa872c418009d4e6`；对应 `.audit.json` 的 SHA-256 为 `e8d8b9eb42af9de9272f07d20a2cf8432e2875df9a14748ad3fcbad94a0ce6e6`。八个已登记编译及测试输出文件的实际尺寸、hash 和物理 F 盘位置均经只读核对。

第二轮的独立参数审计确认 queue 45 对应 `zircon-flow #37`，结果为 FAILURE；九项参数与十八项空默认值匹配原描述符。原始 API 数据 SHA-256 为 `d76a85fc40981ca87463514423de0451c9558e9804a9c2333723871c6e25a2cd`，审计回执 SHA-256 为 `0525d280140829bd1f2449cfb65a6572add30606da3cc867a55a38bc29c12cb4`。该次失败仍发生在源码应用前，不产生新执行或验收回执。

## 修正版部署前的检查

托盘控制的两个参数校验入口均已修正，18 项定向控制通过。公开 `flow source-release` 支持在保留已接受源码、产物和消费者引用的前提下，按原请求授权及路径版本／hash 解除本任务的编辑占用；34 项定向控制通过。该操作区分原验收身份与当前封存操作器身份，不重新提交 Git 或构建，也不补偿失败请求。检查时在线的 `191` 版本尚未加载这些修复。

首次修复后完整回归收集 781 项，778 项通过、2 项跳过、1 项失败；源码前后身份稳定。证据 `.jenkins/state/migration-baseline/patch-support-regression-1791600822731638200.json` 的 SHA-256 为 `e76b8c08f2634f910936ed92ac3b50e4ada38f1689df7cb04e68ccac1e67d61e`。失败位于崩溃进程测试的目录清理。随后持有崩溃前精确 phase handle 的五次实测中，四次在外层 Job 已被标记完整时仍观察到子进程 `WAIT_TIMEOUT`；证据 `execution-guard-phase-handle-diagnosis-1791602687667995100.json` 的 SHA-256 为 `5424a5d80e587b186217f19b78104efdb7bb32d1429c451e57ae7578bfd8d488`。因此没有把失败解释为无关环境噪声或通过清理重试掩盖。

最小修复要求嵌套 phase 各自有完整身份、进程树终止及 EOF 证明；缺失时将外层 guard 降级为 `termination_unproven`，并同时阻断旧弱 abort 回执及直接资源释放入口。正常完成的 phase 保持原验收路径，崩溃且无法闭合的执行保留预约和 writer hold；跨 owner 的未知子树恢复仍是 M7 待验收项。74 项相关定向测试通过，证据 `native-guard-phase-proof-focused-1791603204144082300.json` 的 SHA-256 为 `2634983e03c62702c7c2f82969c139ec0f0c8f597a883e15babcf99b36e5a281`。第二次完整回归已启动，尚不能据此宣布部署或流水线验收通过。

第二次完整回归已通过：784 项收集，782 项通过，2 项目录别名测试因环境跳过，零失败／错误，源码前后身份一致。证据 `patch-support-regression-1791603326075578800.json` 的 SHA-256 为 `85d499fce0969e49b60a664c1442bc028958433a4a19576c8e2e8c8cdc5d3d0c`，原始日志 SHA-256 为 `30b611d3e1015d8ac3f7d6117d11ece1d0d89d33cc027a42171ae2be375e3238`。修正版封存驱动为 `0c1f8adbc3d675d524d508db80688e6ff3311aea055700024743575bca2e4099`；不可变准备记录的 `prepared-not-selected` 表示签发时状态，实际部署结果见后文。

对旧 `191` 的真实 keeper 只读调用其封存参数检查器，重现了合法原生命令因元组／列表表示不同而被拒绝；精确 keeper PID、出生时间、exe 和命令摘要首末一致，规范化后的值与摘要匹配。证据 `formal-keeper-live-native-argv-diagnosis-1791604678786085300.json` 的 SHA-256 为 `bca268f23e5aa984489bbdfc665b3e01ce6c086d2cbeaa6495646141572ca433`。该诊断没有调用生命周期构造器、写入 State 或创建启停描述符。

日常入口 `tools/dev/jenkins-validate.ps1 release-source` 已增量应用，34 项针对性控制通过。输入绑定原已接受请求、完整验收摘要及精确路径版本／hash，保留源码、消费者和产物引用；同一释放 ID 再调用只读对账原操作，不重新提交、取后续外来编辑的 pins 或重发未知交付。应用证据 `dev-release-source-canonical-application-1791605772640946000.json` 的 SHA-256 为 `d62b4e7170a6df97a7431d8b73fd25fb34b88a5d117354cb39f3f54cb03c9abd`。检查时 `191` 缺少正式 closeout 模块，命令在发送前阻断；实际源码交接及连续两轮编译随后由新选中版本验收。

## 0c 版本的实际部署与连续增量验收

正式升级清单为 `.jenkins/state/deployment/usable-upgrade-0c1f8adbc3d675d5-1791613220323415100.json`，SHA-256 为 `4d1e498a9397578f2062507171560ebcbd6ef960ee4106fd3fdc1f6f1edc521f`。实际 keeper 为 `31368`，通过官方 `zircon-strict` 宿主持有服务；没有扩展权限配置或更改控制配置字节。运行组件、Home、agent、日志、缓存、临时目录和控制状态保持在工程 `.jenkins` 下。编译根默认仍为真实 `D:\cargo-targets`，本次验收明确选择真实 `F:\cargo-targets`，没有自动换盘。

新 V4 描述符为 `.jenkins/tmp/acceptance/m4-0c-v4-1791614600/descriptor.json`，SHA-256 为 `18c9922ac2ebcc65278c8806328ce8be3e3c65ee23e8f5a3abe9962e395dbf27`；它只提交六文件原始 Rust 工程，不创建 Git 基线、适配器源码快照或直接执行 Cargo。两轮实际执行均绑定原运行身份 `start-1791614189691758800` 与封存 `0c`，不因随后托盘启停而改写历史身份。执行回执 `formal-v4-invocation-1791614948319901500-completed.json` 的 SHA-256 为 `5ff26b5ac3b7fc137cae7ef4a02826073592fc898cdf6ad27b625f3dd0c843b3`；原始输出 SHA-256 为 `a16ff3f08dcc810cb6a4b6c30df55e07be4ea1f665704962244833dbcc211faf`，exit 0，双轮 status 为 accepted。

第一轮执行为 `dee4c33435ec41ea9b9d772c0f8e148c`，接受回执摘要为 `9e674d325b9658c75941297ee2bd4f2e85de0c40a2f454a9f1614d8d5c09dfe4`；第二轮为 `faa04e277da127587560c547bb966d85`，接受回执摘要为 `d38672ea55b70ad8e9e66991f70249b9c22a47455f69aaf81dc09657dd4a8f0f`。两轮均核对 compile/list/test 的独立原生进程终态、子进程消失、零活跃进程、双输出流 EOF、实际物理输出闭包及两项必需测试。第二轮源码摘要不同，PreparationKey `dc11fbe0400a1f5fc04cfc4d6d9cefc7992a2cb8844673ccec603ed8105ab167` 与增量池保持一致，测试 Job 改变，默认重新执行测试。

原始 Jenkins API 对账确认第一轮 queue 48 对应 `zircon-flow #38`，第二轮 queue 72 对应 `#39`，均已终止为 SUCCESS。每轮完整九项请求参数及十八项空默认值与原描述符、原补丁引用及运行身份一致。`jenkins-m4-0c-v4-1791614600-build38-queue48-parameters.raw.json` 的 SHA-256 为 `717091e3ba8604c3c4ae9b29657519a59bddf4a1ba44296cde5cc257a4fdefa2`，对应 `.audit.json` 为 `fa17191b7118513516f249cbb37a4dc620d10f8fcb86e0197c8d770ba9f1f219`；第二轮原始字节 SHA-256 为 `6b718388438812e22976b83a0353e92b6c77817e71a1bb16d854948d38ca0846`，审计回执为 `3dbbbb17a8f26c6353adb918d4a201616bec9a985075faede8156b4b8ce7d483`。

两轮之间使用公开 `flow source-release` 按原已接受身份交接路径。第二轮验收后，日常入口再次完成原任务的源码交接，回执 `formal-v4-dev-second-source-release-1791616144962430500-completed.json` 的 SHA-256 为 `948ba7d70473be7d62b55cfc0015f57b4af34f5b85bce2a23036173b0224ea10`。同一释放 ID 的只读对账回执 `formal-v4-dev-source-release-readonly-reconcile-1791616256700278100-completed.json` 的 SHA-256 为 `45d8865209c83d808117f08fce0d6feafbf99a66c155457af18d3e796503c3fe`，reused 为 true，完整状态库前后摘要及源码字节不变；原验收、消费者和产物引用保留。前代 `191` 第一轮占用也经新发行者绑定原身份交接，原第二轮失败请求、外来 R4 任务和四项历史预约／writer 保护均保留。没有提交、推送、通知或 GC。

## 0c 可见托盘与正常启停

可见托盘为 `43180`，位于 `WinSta0\Default`，Explorer 确认图标，实际 Version 4 右键菜单可展开并关闭。UI 刷新回执为 `.jenkins/state/tray/next-formal-tray-1791614632828970600-completed.json`，SHA-256 为 `a295e401aaa94e7cd21df3d2a492e5927343de50b8b8d51c33d0b84770277b24`；刷新只替换 UI，未停止服务。

随后分别通过同一个已核验 UI 发送一次停止、启动和重启菜单命令，均观察原 keeper 请求及结果，没有复制副作用或将 Restart 拆成两个请求。停止核对 controller／agent 的完整原生终态与 supervisor 的精确进程离开，启动消费该停止的原终态，单请求重启再核对完整停止及新运行代。三份实际回执位于 `.jenkins/state/tray`：

| 操作 | 实际 completed 回执 | SHA-256 |
| --- | --- | --- |
| 停止 | `menu-m2-stop-start-1791614189691758800-completed.json` | `bfe4c64a70db56b31abf7f0be68e1f513d96f7e30ad98285f1a39d773d97858f` |
| 启动 | `menu-m2-start-start-1791614189691758800-completed.json` | `021e2877087ef3bd54c6e5148fdcde5461ca20a18d919fc67819b060e7232d27` |
| 单请求重启 | `menu-m2-restart-start-1791615733476423800-completed.json` | `554604d26747ff75d6849929580fa26507cb88667b310a87808f399e43daef2f` |

独立复查 `formal-m2-normal-lifecycle-independent-review-1791616558464923100.json` 的 SHA-256 为 `eb53c6a88775e8248aa8d2e9150c0ef78e950d8aad00ac6d8f4c1513bc39835e`，168 项检查及 37 个原件 hash 复核通过；纯协议重新推导 successor、双 journal 与 keeper 归属，并核对 strict creator／child Job、无 breakaway、唯一监听及四项历史保护。复查不新增运行或 Cargo 接受回执，也不把 supervisor 离开伪造成 EOF。

最终运行身份为 `start-1791615932752147100`，supervisor `38924`、controller `16660`、agent `37580`；keeper `31368`、可见托盘 `43180` 保留。正式 successor 为 `.jenkins/state/deployment/history/formal-lifecycle-successor-3adc1f029df88811a59c36d7ceacdc0dfc8efcca85bbb81b36573b62a52618fe.json`，相同 SHA-256；服务健康、接收请求且未处于 quiet-down。本地 admin 的密码认证实际返回 HTTP 200 且身份为已认证 admin，核对回执 `usable-local-admin-password-check-1791616424906804900.json` 的 SHA-256 为 `6d19626959d6cdd55f1615cc1a9578b362dab914fdd4da9fd8ef3bb58bf5852b`，不记录密码或 token。

以上关闭对应输入的运行、补丁及正常托盘启停核对；整仓、跨模块完整覆盖、三 Session 冲突／取消恢复、全部资源压力与 GC、配置回退及唯一入口切换仍按 M0–M8 各自门槛验收。历史失败和保护不因本次通过而删除或重放。
