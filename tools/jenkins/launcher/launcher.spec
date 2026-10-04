# PyInstaller spec for ZirconEngine Jenkins Launcher
# 打包命令（在仓库根目录执行，需要 PyInstaller >= 5.0）:
#   pip install pyinstaller
#   pyinstaller tools/jenkins/launcher/launcher.spec
#
# 产物: dist/zircon-jenkins.exe  (~5-8 MB, 单文件, 无 Python 依赖)
# 运行时通过 subprocess 调用 .jenkins/runtime/python/pythonw.exe，
# 不打包任何 tools/jenkins 代码。

from pathlib import Path

# SPEC 变量由 PyInstaller 注入，指向本 spec 文件的绝对路径。
REPO_ROOT = Path(SPEC).resolve().parent.parent.parent  # launcher/ → jenkins/ → tools/ → repo/

a = Analysis(
    [str(REPO_ROOT / "tools" / "jenkins" / "launcher" / "__main__.py")],
    pathex=[str(REPO_ROOT / "tools" / "jenkins" / "launcher")],
    binaries=[],
    datas=[],
    hiddenimports=[],       # 纯标准库，无隐式导入
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # 明确排除仓库内所有 tools 代码
        "tools",
        "tools.jenkins",
    ],
    noarchive=False,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="zircon-jenkins",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,          # UPX 可选；设为 True 需要 upx 在 PATH
    runtime_tmpdir=None,
    console=True,       # True = 有控制台窗口，可看到启动日志
                        # 改为 False 则静默运行（不显示窗口）
    onefile=True,
)
