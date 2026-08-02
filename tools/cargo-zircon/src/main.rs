use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cargo_zircon::plugin::check::check_plugin_workspace_with_artifact_root;
use cargo_zircon::plugin::manifest_sync::{synchronize_workspace_manifests, SyncMode, SyncOutcome};
use cargo_zircon::plugin::scaffold::{scaffold_plugin, NewPluginOptions, PluginKind};
use cargo_zircon::plugin::validate::{validate_native_artifact, validate_plugin_manifest};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("cargo zircon: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(mut arguments: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if arguments.first().is_some_and(|value| value == "zircon") {
        arguments.remove(0);
    }
    if arguments.first().map(String::as_str) != Some("plugin") {
        return Err(usage_error().into());
    }
    arguments.remove(0);
    let command = arguments.first().cloned().ok_or_else(usage_error)?;
    arguments.remove(0);
    match command.as_str() {
        "sync-manifest" => run_manifest_sync(arguments, SyncMode::Write),
        "check-manifest" => run_manifest_sync(arguments, SyncMode::Check),
        "new" => run_new(arguments),
        "check" => run_check(arguments),
        "validate" => run_validate(arguments),
        _ => Err(usage_error().into()),
    }
}

fn run_manifest_sync(
    arguments: Vec<String>,
    mode: SyncMode,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut repo_root = env::current_dir()?;
    let mut selector = None;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--root" {
            let value = arguments.get(index + 1).ok_or_else(usage_error)?;
            repo_root = PathBuf::from(value);
            index += 2;
            continue;
        }
        if selector.is_some() {
            return Err(usage_error().into());
        }
        selector = Some(arguments[index].as_str());
        index += 1;
    }

    let entries = synchronize_workspace_manifests(&repo_root, selector, mode)?;
    let mut drift = false;
    for entry in entries {
        println!("{}: {:?}", entry.package_id, entry.outcome);
        drift |= entry.outcome == SyncOutcome::Drift;
    }
    Ok(if drift {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_new(arguments: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut repo_root = env::current_dir()?;
    let mut id = None;
    let mut kind = None;
    let mut native = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--root" => {
                repo_root = PathBuf::from(arguments.get(index + 1).ok_or_else(usage_error)?);
                index += 2;
            }
            "--kind" => {
                let value = arguments.get(index + 1).ok_or_else(usage_error)?;
                kind = PluginKind::parse(value);
                if kind.is_none() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unknown plugin kind `{value}`; use importer, system, or editor"),
                    )
                    .into());
                }
                index += 2;
            }
            "--native" => {
                native = true;
                index += 1;
            }
            value if id.is_none() => {
                id = Some(value.to_string());
                index += 1;
            }
            _ => return Err(usage_error().into()),
        }
    }
    let id = id.ok_or_else(usage_error)?;
    let kind = kind.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "plugin new requires --kind importer|system|editor",
        )
    })?;
    let report = scaffold_plugin(&NewPluginOptions {
        repo_root: &repo_root,
        id: &id,
        kind,
        native,
    })?;
    println!(
        "{}: created {} files",
        report.package_id,
        report.created_paths.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn run_check(arguments: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let (repo_root, artifact_root) = parse_check_paths(arguments)?;
    let report = check_plugin_workspace_with_artifact_root(&repo_root, artifact_root.as_deref())?;
    for diagnostic in &report.diagnostics {
        eprintln!(
            "{}: {}\nhint: {}",
            diagnostic.code, diagnostic.message, diagnostic.hint
        );
    }
    if report.diagnostics.is_empty() {
        println!("checked {} plugin manifests", report.checked_manifests);
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(4))
    }
}

fn run_validate(arguments: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut path = None;
    let mut artifact = None;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--artifact" {
            artifact = Some(PathBuf::from(
                arguments.get(index + 1).ok_or_else(usage_error)?,
            ));
            index += 2;
        } else if path.is_none() {
            path = Some(PathBuf::from(&arguments[index]));
            index += 1;
        } else {
            return Err(usage_error().into());
        }
    }
    let path = path.ok_or_else(usage_error)?;
    let (manifest_path, package_root) = if path.is_dir() {
        (path.join("plugin.toml"), path)
    } else {
        let package_root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        (path, package_root)
    };
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let mut diagnostics = validate_plugin_manifest(&manifest_text, Some(&package_root));
    if let Some(artifact) = artifact {
        diagnostics.extend(validate_native_artifact(&manifest_text, &artifact));
    }
    for diagnostic in &diagnostics {
        eprintln!(
            "{}: {}\nhint: {}",
            diagnostic.code, diagnostic.message, diagnostic.hint
        );
    }
    if diagnostics.is_empty() {
        println!("{}: valid", manifest_path.display());
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(4))
    }
}

fn parse_check_paths(
    arguments: Vec<String>,
) -> Result<(PathBuf, Option<PathBuf>), Box<dyn std::error::Error>> {
    let mut repo_root = env::current_dir()?;
    let mut artifact_root = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--root" => {
                repo_root = PathBuf::from(arguments.get(index + 1).ok_or_else(usage_error)?);
                index += 2;
            }
            "--artifact-root" => {
                artifact_root = Some(PathBuf::from(
                    arguments.get(index + 1).ok_or_else(usage_error)?,
                ));
                index += 2;
            }
            _ => return Err(usage_error().into()),
        }
    }
    Ok((repo_root, artifact_root))
}

fn usage() -> String {
    "usage: cargo zircon plugin <new|check|validate|sync-manifest|check-manifest> [OPTIONS]"
        .to_string()
}

fn usage_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, usage())
}
