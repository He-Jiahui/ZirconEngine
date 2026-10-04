//! 每个复制的原生包附带导出 ABI 报告，供产物检查与人工排查对应具体包。
use std::fs;
use std::path::Path;

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_report_template, NativeDynamicPackageExportPlan,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
};

/// 原生包文件复制完成后写 ABI 报告，使包目录自描述；输出位置须与加载清单一致。
pub(super) fn write_native_dynamic_package_report(
    destination: &Path,
    package: &NativeDynamicPackageExportPlan,
) -> Result<(), std::io::Error> {
    fs::write(
        destination.join(NATIVE_DYNAMIC_PACKAGE_REPORT_FILE),
        native_dynamic_package_report_template(package),
    )
}
