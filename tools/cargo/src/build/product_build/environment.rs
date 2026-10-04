//! 把受支持的 Windows MSVC 构建环境固定为可摘要的输入。
//! 产品构建入口在读取 Cargo 元数据与编译之前生成同一环境，并以清空继承环境的子进程运行；目录策略须由构建所有者预先准入。

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use super::{sha256_serialized, ProductBuildTarget, ProductReceiptError};

const WINDOWS_MSVC_ENVIRONMENT_POLICY: &str = "windows-msvc-v1";
const WINDOWS_MSVC_ENVIRONMENT_NAMES: &[&str] = &[
    "APPDATA",
    "CARGO_HOME",
    "CC",
    "CL",
    "CMAKE_GENERATOR",
    "COMSPEC",
    "CXX",
    "INCLUDE",
    "LIB",
    "LIBPATH",
    "LOCALAPPDATA",
    "NUMBER_OF_PROCESSORS",
    "OS",
    "PATH",
    "PATHEXT",
    "PROCESSOR_ARCHITECTURE",
    "PROGRAMDATA",
    "RUSTUP_HOME",
    "RUSTUP_TOOLCHAIN",
    "SYSTEMDRIVE",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "UCRTVERSION",
    "UNIVERSALCRTSDKDIR",
    "USERPROFILE",
    "VCINSTALLDIR",
    "VCTOOLSINSTALLDIR",
    "VCTOOLSVERSION",
    "VSCMD_ARG_HOST_ARCH",
    "VSCMD_ARG_TGT_ARCH",
    "VSCMD_VER",
    "WINDOWSSDKDIR",
    "WINDOWSSDKLIBVERSION",
    "WINDOWSSDKVERSION",
    "_CL_",
    "_LINK_",
];

/// 仅接受已识别的 MSVC 环境策略；返回子进程的完整环境及同内容摘要，目录需先由构建所有者准入。
pub(super) fn effective_build_environment(
    policy: &str,
    rustc_path: &Path,
    linker_path: Option<&Path>,
    target_directory: &Path,
    target: &ProductBuildTarget,
    trusted_host_build_set_id: &str,
) -> Result<(Vec<(String, String)>, String), ProductReceiptError> {
    let environment_names = if policy == WINDOWS_MSVC_ENVIRONMENT_POLICY
        && target.target_triple.ends_with("-windows-msvc")
    {
        WINDOWS_MSVC_ENVIRONMENT_NAMES
    } else {
        return Err(ProductReceiptError::new(format!(
            "unknown product build environment policy `{policy}` for target `{}`",
            target.target_triple
        )));
    };
    let mut captured = BTreeMap::<String, Option<String>>::new();
    for name in environment_names {
        let value = match env::var(name) {
            Ok(value) => Some(value),
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ProductReceiptError::new(format!(
                    "product build environment `{name}` is not Unicode"
                )));
            }
        };
        captured.insert((*name).to_string(), value);
    }

    insert_forced_environment(&mut captured, "RUSTC", path_text(rustc_path, "rustc")?)?;
    // BUG: [CR-TOOL-RUST-0001] 此处把请求目标目录交给 Cargo；上游仅检查绝对规范路径，未限制物理根为 D/E/F 盘的 cargo-targets，现行编译产物规则可被绕过。
    insert_forced_environment(
        &mut captured,
        "CARGO_TARGET_DIR",
        path_text(target_directory, "Cargo target directory")?,
    )?;
    insert_forced_environment(&mut captured, "CARGO_TERM_COLOR", "never".to_string())?;
    insert_forced_environment(
        &mut captured,
        "ZR_ZIRCON_TRUSTED_HOST_BUILD_SET_ID",
        trusted_host_build_set_id.to_owned(),
    )?;
    insert_forced_environment(
        &mut captured,
        "CARGO_ENCODED_RUSTFLAGS",
        target.rustflags.join("\u{1f}"),
    )?;
    if let Some(linker_path) = linker_path {
        let linker_key = cargo_linker_environment_key(&target.target_triple)?;
        insert_forced_environment(
            &mut captured,
            &linker_key,
            path_text(linker_path, "linker")?,
        )?;
    }

    let digest = sha256_serialized(&captured)?;
    let environment = captured
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| (name, value)))
        .collect();
    Ok((environment, digest))
}

fn cargo_linker_environment_key(target_triple: &str) -> Result<String, ProductReceiptError> {
    const PREFIX: &str = "CARGO_TARGET_";
    const SUFFIX: &str = "_LINKER";

    let mut linker_key = String::with_capacity(PREFIX.len() + target_triple.len() + SUFFIX.len());
    linker_key.push_str("CARGO_TARGET_");
    for byte in target_triple.bytes() {
        linker_key.push(match byte {
            b'a'..=b'z' => (byte - b'a' + b'A') as char,
            b'A'..=b'Z' | b'0'..=b'9' | b'_' => byte as char,
            b'-' => '_',
            _ => {
                return Err(ProductReceiptError::new(
                    "product build target triple cannot form a Cargo linker environment key",
                ));
            }
        });
    }
    linker_key.push_str("_LINKER");
    Ok(linker_key)
}

fn insert_forced_environment(
    environment: &mut BTreeMap<String, Option<String>>,
    name: &str,
    value: String,
) -> Result<(), ProductReceiptError> {
    if environment.insert(name.to_string(), Some(value)).is_some() {
        return Err(ProductReceiptError::new(format!(
            "product build environment policy must not override owned variable `{name}`"
        )));
    }
    Ok(())
}

fn path_text(path: &Path, label: &str) -> Result<String, ProductReceiptError> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        ProductReceiptError::new(format!("product build {label} path is not Unicode"))
    })
}

#[cfg(test)]
#[path = "environment/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "environment/tests/cases.rs"]
mod tests;
