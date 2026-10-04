export function catalogCopy(language: string) {
  const pair = (en: string, zh: string) => language === "Chinese" ? zh : en;
  return {
    local: pair("Local library", "本地库"), service: pair("Service catalog", "服务目录"),
    search: pair("Search packages", "搜索包"), searchAction: pair("Search", "搜索"),
    organization: pair("Organization", "组织"), chooseOrganization: pair("Select organization", "选择组织"),
    noReleases: pair("No releases found", "未找到发布版本"), release: pair("Release", "发布版本"),
    publisher: pair("Publisher", "发布者"), version: pair("Version", "版本"), size: pair("Download size", "下载大小"),
    license: pair("License", "许可"), reviewLicense: pair("Review license", "查看许可"),
    acceptLicense: pair("Accept license", "接受许可"), accepted: pair("Licensed", "已授权"),
    licenseRequired: pair("License required", "需要许可"), licensePending: pair("License status pending", "许可状态待确认"),
    adminRequired: pair("An organization owner or admin must accept this license.", "此许可需要由组织所有者或管理员接受。"),
    licenseConsent: pair("I accept this license for the selected organization.", "我代表所选组织接受此许可。"),
    licenseMore: pair("Load more licenses", "加载更多许可"),
    install: pair("Install", "安装"), installed: pair("Installed", "已安装"), notInstalled: pair("Not installed", "未安装"),
    inventory: pair("Installation inventory", "安装清单"), inventoryPending: pair("Installation status pending", "安装状态待确认"),
    inventoryEmpty: pair("No installed packages", "暂无已安装包"), inventoryRevision: pair("Inventory revision", "安装清单修订号"),
    installTarget: pair("Install target", "安装目标"), editorHostTarget: pair("Editor host", "编辑器宿主"), clientRuntimeTarget: pair("Client runtime", "客户端运行时"),
    packageTooLarge: pair("This package exceeds the 16 MiB installation limit.", "此包超过 16 MiB 安装大小限制。"),
    packageId: pair("Package ID", "包 ID"), digest: pair("Artifact digest", "产物摘要"),
    searchTooLong: pair("Search exceeds 256 bytes", "搜索内容超过 256 字节"),
  };
}
