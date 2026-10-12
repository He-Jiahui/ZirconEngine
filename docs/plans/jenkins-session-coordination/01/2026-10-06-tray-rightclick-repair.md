---
related_code:
  - tools/jenkins/tray/formal_app.py
  - tools/jenkins/tray/tests/test_formal_commands.py
plan_sources:
  - "user: Jenkins tray 右键菜单无法打开，需要修复"
  - docs/plans/jenkins-session-coordination/01-single-host-jenkins-coordination.md
tests:
  - "zircon-strict: tools.jenkins.tray.tests.test_formal_commands + test_menu，9 项通过"
  - "Windows 正式托盘：两个 version-4 回调均显示真实九项菜单"
doc_type: runtime-repair-record
---

# 2026-10-06 正式 Jenkins 托盘右键修复

右键菜单已恢复。Windows Shell notification version 4 将事件放在 lParam 的低 16 位、图标 ID 放在高 16 位；正式托盘以前直接比较整个参数，图标 ID 为 1 时无法识别右键事件。formal_app.py 现在先取低 16 位，再处理右键、上下文菜单及双击。依据：[Microsoft NOTIFYICONDATAW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw)。

9 项托盘命令和菜单单元测试通过；修复源码 SHA-256 为 bfd58a4c2b7f3bd24625e05d3cac2e5514b5f9d6da49267e3bd77461e4a35a5a，测试源码为 3b3b8d06f88c999109a7b076a062fafb21f785b710644291d58a27c45f821a33。其他预先存在的修改保持原样。

首次运行验证刷新了托盘 UI，未通过托盘停止或重启 Jenkins。当次 UI PID 8540，创建时间 134357014980031364，窗口句柄 62657958，图标注册成功。验证向精确归属的隐藏窗口提交 (1 << 16) | WM_RBUTTONUP 和 (1 << 16) | WM_CONTEXTMENU，分别观察到 Windows #32768 原生菜单，读取九个命令 ID 与文字，然后用 WM_CANCELMODE 关闭。没有选择服务命令，lastCommand 保持为空，服务运行记录字节未改变。

本次启动使用 zircon-strict 与 elevated Windows sandbox，并仅对该命令设置 windows.sandbox_private_desktop=false，以便托盘在交互桌面显示；文件权限清单和全局／工程配置未修改。主进程显式传入已有物理 Codex 数据目录，避免 runner 误用默认用户目录。兼容选项依据：[OpenAI Windows sandbox](https://learn.chatgpt.com/docs/windows/windows-sandbox)。最初两个 UI 启动尝试及其失败日志保留，不作为通过证据。

原始运行回执位于 .jenkins/state/migration-baseline/tray-rightclick-runtime-1791227912828796100.json；验证脚本为同目录的 verify_tray_rightclick_runtime_v3.py。该回执仅接受托盘事件与菜单本身。当次 Jenkins 处于 controller-only：服务地址 http://127.0.0.1:18080/，operationId 为 start-1791221221971009300，构建 agent 暂停。旧 selector、历史资源保护和 runtimeAcceptance=false 保持。此修复没有关闭 M2 的完整 agent／部署验收，也没有关闭 M5–M8。

## 当前进程复核

2026-10-06 复核发现，首次回执中的托盘 PID 8540 已退出；原 controller PID 5300 与 lifecycle host PID 12148 也已退出，18080 登录端点不可达。旧 running 状态文件未作为当前可用证据。

已仅启动修复后的托盘，当前 UI PID 11660、窗口句柄 8913138，图标注册成功。两个 version-4 回调再次分别弹出九项原生菜单并正常关闭；验证没有选择菜单命令，也没有改变 deployment-operation.json 的字节。验证进程结束后再观察，托盘仍存活且创建时间和可执行文件匹配。

当前回执为 .jenkins/state/migration-baseline/tray-rightclick-current-1791262835290697200.json，scope 为 formal-tray-rightclick-current-runtime，status 为 accepted。serviceReady=false 如实记录服务退出；此回执没有颁发控制器或构建验收。恢复服务仍须核对精确进程及原生终止证据，历史 execution holds 保持。

## 13:09 UTC 再次运行复核

此前记录的 UI PID 11660 已退出。本次只启动正式托盘 UI，当前 PID 21828、创建时间 134357656458153129、窗口句柄 25564928，图标注册成功。验证程序退出后另一次进程核验确认该实例仍存活，可执行文件和创建时间匹配。

两种 packed version-4 回调再次弹出真实 Windows 九项菜单并正常关闭；没有选择菜单命令，deployment-operation.json 字节保持不变。新回执为 .jenkins/state/migration-baseline/tray-rightclick-current-1791292064144374000.json，status=accepted。服务仍离线，serviceReady=false；此菜单验收不代表控制器、agent 或构建可用。正式服务恢复按独立的生命周期和原生终止证据门槛推进。
