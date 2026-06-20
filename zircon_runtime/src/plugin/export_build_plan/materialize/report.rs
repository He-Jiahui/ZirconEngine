use std::fs;
use std::path::Path;

use super::super::native_dynamic_package_plan::{
    native_dynamic_package_report_template, NativeDynamicPackageExportPlan,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
};

pub(super) fn write_native_dynamic_package_report(
    destination: &Path,
    package: &NativeDynamicPackageExportPlan,
) -> Result<(), std::io::Error> {
    fs::write(
        destination.join(NATIVE_DYNAMIC_PACKAGE_REPORT_FILE),
        native_dynamic_package_report_template(package),
    )
}
