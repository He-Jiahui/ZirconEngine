use super::*;
use std::process::Command;

pub(crate) struct DllFixture {
    pub(crate) root: PathBuf,
    pub(crate) main: PathBuf,
    pub(crate) marker: PathBuf,
    main_digest: NativePluginArtifactDigest,
    pub(crate) dependency: NativePluginArtifactDependency,
}

impl DllFixture {
    pub(crate) fn build() -> Self {
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let root = std::env::temp_dir().join(format!("zircon-native-admission-fixture-{suffix}"));
        std::fs::create_dir(&root).unwrap();
        let native_root = root.join("native");
        std::fs::create_dir(&native_root).unwrap();
        let dependency_name = format!("zircon_dependency_{suffix}");
        let dependency_source = root.join("dependency.c");
        std::fs::write(
            &dependency_source,
            "__declspec(dllexport) int fixture_dependency(void) { return 42; }",
        )
        .unwrap();
        let dependency_path = native_root.join(format!("{dependency_name}.dll"));
        compile_dll(&dependency_source, &dependency_path, &[]);
        let marker = root.join("main-entry-ran");
        use std::os::windows::ffi::OsStrExt;
        let marker_units = marker
            .as_os_str()
            .encode_wide()
            .chain([0])
            .map(|unit| unit.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            r#"
#include <windows.h>
__declspec(dllimport) int fixture_dependency(void);
__declspec(dllexport) int fixture_value(void) {{ return fixture_dependency(); }}
BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID reserved) {{
    if (reason == DLL_PROCESS_ATTACH) {{
        const WCHAR marker[] = {{{marker_units}}};
        HANDLE file = CreateFileW(marker, GENERIC_WRITE, FILE_SHARE_READ, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
        if (file != INVALID_HANDLE_VALUE) CloseHandle(file);
        return fixture_dependency() == 42;
    }}
    return TRUE;
}}
"#
        );
        let main_source = root.join("main.c");
        std::fs::write(&main_source, source).unwrap();
        let main = native_root.join(format!("zircon_main_{suffix}.dll"));
        compile_dll(
            &main_source,
            &main,
            &[dependency_path.with_extension("lib")],
        );
        std::fs::write(
            root.join("plugin.toml"),
            format!(
                r#"id = "fixture"
version = "0.1.0"
display_name = "Native admission fixture"
supported_targets = ["client_runtime"]
supported_platforms = ["windows"]
[distribution]
forms = ["dist"]
abi_version = 3
engine_compat = ">=0.1, <0.2"
dist_crate = "zircon_main_{suffix}"
[[modules]]
name = "fixture.runtime"
kind = "runtime"
crate_name = "zircon_main_{suffix}"
"#
            ),
        )
        .unwrap();
        Self {
            main_digest: NativePluginArtifactDigest::capture(&main).unwrap(),
            dependency: NativePluginArtifactDependency {
                file_name: dependency_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                digest: NativePluginArtifactDigest::capture(&dependency_path).unwrap(),
            },
            root,
            main,
            marker,
        }
    }

    fn stage(&self) -> Result<NativePluginArtifactStaging, NativePluginArtifactAdmissionError> {
        NativePluginArtifactStaging::prepare(
            "fixture",
            &self.main,
            &self.main_digest,
            std::slice::from_ref(&self.dependency),
        )
    }
}

impl Drop for DllFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn compile_dll(source: &Path, output: &Path, imports: &[PathBuf]) {
    compile_dll_with_definition(source, output, imports, None);
}

fn compile_dll_with_definition(
    source: &Path,
    output: &Path,
    imports: &[PathBuf],
    definition: Option<&Path>,
) {
    let definition = definition.map(|path| format!("/DEF:{}", path.display()));
    let result = Command::new("cl.exe")
        .current_dir(source.parent().unwrap())
        .args(["/nologo", "/LD", "/MT", "/Od"])
        .arg(source)
        .arg(format!("/Fe:{}", output.display()))
        .arg("/link")
        .arg("/INCREMENTAL:NO")
        .arg(format!(
            "/IMPLIB:{}",
            output.with_extension("lib").display()
        ))
        .args(imports)
        .args(definition)
        .output()
        .expect("Windows native fixture requires the validator MSVC environment");
    assert!(
        result.status.success(),
        "DLL fixture compile failed: {} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn forwarded_export_dependency_is_rejected_without_executing_entry() {
    let mut fixture = DllFixture::build();
    let source = fixture.root.join("forwarded.c");
    std::fs::write(&source, "int fixture_not_exported(void) { return 42; }").unwrap();
    let definition = fixture.root.join("forwarded.def");
    let rogue_name = format!("zircon_forwarded_{}", uuid::Uuid::new_v4().simple());
    std::fs::write(
        &definition,
        format!("EXPORTS\nfixture_dependency={rogue_name}.fixture_value\n"),
    )
    .unwrap();
    let dependency_path = fixture
        .main
        .parent()
        .unwrap()
        .join(&fixture.dependency.file_name);
    compile_dll_with_definition(&source, &dependency_path, &[], Some(&definition));
    fixture.dependency.digest = NativePluginArtifactDigest::capture(&dependency_path).unwrap();
    // A real executable DLL at the forwarded name must never be considered by admission.
    std::fs::copy(
        &fixture.main,
        fixture
            .main
            .parent()
            .unwrap()
            .join(format!("{rogue_name}.dll")),
    )
    .unwrap();
    let pe_bytes = std::fs::read(&dependency_path).unwrap();
    let pe = goblin::pe::PE::parse(&pe_bytes).unwrap();
    assert!(pe.exports.iter().any(|export| export.reexport.is_some()));
    assert!(!pe
        .libraries
        .iter()
        .any(|name| name.starts_with(&rogue_name)));
    let error = fixture.stage().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("forwarded native package exports"),
        "{error}"
    );
    assert!(!fixture.marker.exists());
}

#[test]
fn authenticated_closure_loads_and_retains_immutable_images() {
    let fixture = DllFixture::build();
    let staging = fixture.stage().unwrap();
    assert!(!fixture.marker.exists());
    assert_ne!(staging.library_path(), fixture.main);
    assert!(OpenOptions::new()
        .write(true)
        .open(staging.library_path())
        .is_err());
    let library = unsafe {
        libloading::os::windows::Library::load_with_flags(staging.library_path(), 0x100 | 0x800)
    }
    .unwrap();
    assert!(
        fixture.marker.exists(),
        "real DLL attach must execute after successful admission"
    );
    let value =
        unsafe { library.get::<unsafe extern "C" fn() -> i32>(b"fixture_value\0") }.unwrap();
    assert_eq!(unsafe { value() }, 42);
    drop(library);
    let staged_root = staging.root.clone();
    drop(staging);
    assert!(!staged_root.exists());
}

#[test]
fn product_build_authority_embeds_original_closure_and_admits_real_dll() {
    use crate::core::framework::{platform::RuntimeTargetMode, project::ExportTargetPlatform};
    use crate::plugin::native::{
        NativePluginArtifactAuthority, NativePluginArtifactTarget, NativePluginCandidate,
    };
    use crate::plugin::{PluginModuleKind, PluginPackageManifest};
    let fixture = DllFixture::build();
    let unrelated_sibling = fixture
        .main
        .parent()
        .unwrap()
        .join("zircon_unrelated_sibling.dll");
    std::fs::copy(&fixture.main, &unrelated_sibling).unwrap();
    let manifest_path = fixture.root.join("plugin.toml");
    let expectations = NativePluginArtifactAuthority::capture_trusted_build_package(
        "fixture",
        &manifest_path,
        "product-build-fixture",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
    )
    .unwrap();
    assert_eq!(expectations.len(), 1);
    assert_eq!(
        expectations[0].dependencies,
        vec![fixture.dependency.clone()]
    );
    let baked = Box::leak(
        serde_json::to_string(&expectations)
            .unwrap()
            .into_boxed_str(),
    );
    let authority = NativePluginArtifactAuthority::from_embedded_build_json(baked).unwrap();
    let manifest: PluginPackageManifest =
        toml::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let candidate = NativePluginCandidate {
        plugin_id: "fixture".into(),
        package_manifest: manifest,
        manifest_path,
        library_path: fixture.main.clone(),
    };
    let admission = authority
        .admit(&candidate, &fixture.main, &[PluginModuleKind::Runtime])
        .unwrap();
    assert!(!fixture.marker.exists());
    let library = unsafe {
        libloading::os::windows::Library::load_with_flags(
            admission.admitted_library_path(),
            0x100 | 0x800,
        )
    }
    .unwrap();
    assert!(fixture.marker.exists());
    drop(library);
    drop(admission);
}

#[test]
fn tampered_main_or_dependency_never_executes_dll_entry() {
    let fixture = DllFixture::build();
    let dependency = fixture
        .main
        .parent()
        .unwrap()
        .join(&fixture.dependency.file_name);
    let original = std::fs::read(&dependency).unwrap();
    std::fs::write(&dependency, b"tampered dependency").unwrap();
    assert!(fixture.stage().is_err());
    assert!(!fixture.marker.exists());
    std::fs::write(dependency, original).unwrap();
    std::fs::write(&fixture.main, b"tampered main").unwrap();
    assert!(fixture.stage().is_err());
    assert!(!fixture.marker.exists());
}

#[test]
fn unlisted_sibling_dependency_is_rejected_before_dll_entry() {
    let fixture = DllFixture::build();
    let result =
        NativePluginArtifactStaging::prepare("fixture", &fixture.main, &fixture.main_digest, &[]);
    assert!(
        result.is_err(),
        "source directory DLL search must not authorize an unsigned dependency"
    );
    assert!(!fixture.marker.exists());
}

#[test]
fn already_loaded_spoof_dependency_is_rejected_before_main_entry() {
    let fixture = DllFixture::build();
    let spoof_root = fixture.root.join("spoof");
    std::fs::create_dir(&spoof_root).unwrap();
    let source = spoof_root.join("spoof.c");
    std::fs::write(
        &source,
        "__declspec(dllexport) int fixture_dependency(void) { return 7; }",
    )
    .unwrap();
    let spoof_path = spoof_root.join(&fixture.dependency.file_name);
    compile_dll(&source, &spoof_path, &[]);
    let spoof =
        unsafe { libloading::os::windows::Library::load_with_flags(&spoof_path, 0x100 | 0x800) }
            .unwrap();
    assert!(
        fixture.stage().is_err(),
        "Windows loaded-module cache must not substitute an unauthenticated DLL"
    );
    assert!(!fixture.marker.exists());
    drop(spoof);
}

#[test]
fn retained_admitted_generation_can_supply_the_same_verified_dependency() {
    let fixture = DllFixture::build();
    let retained = std::sync::Arc::new(fixture.stage().unwrap());
    NativePluginArtifactStaging::register_admitted(&retained).unwrap();
    let retained_root = retained.root.clone();
    let dependency_path = retained_root.join(&fixture.dependency.file_name);
    let dependency =
        unsafe { libloading::os::windows::Library::load_with_flags(&dependency_path, 0x800) }
            .unwrap();
    let previous_value =
        unsafe { dependency.get::<unsafe extern "C" fn() -> i32>(b"fixture_dependency\0") }
            .unwrap();
    assert_eq!(unsafe { previous_value() }, 42);

    let next = fixture
        .stage()
        .expect("a guarded admitted image may be reused");
    assert_eq!(next.retained_stagings.len(), 1);
    drop(retained);
    assert!(
        dependency_path.exists(),
        "next generation must retain its owner"
    );
    let main = unsafe {
        libloading::os::windows::Library::load_with_flags(next.library_path(), 0x100 | 0x800)
    }
    .unwrap();
    assert!(fixture.marker.exists());
    assert_eq!(unsafe { previous_value() }, 42);
    drop(main);
    drop(dependency);
    drop(next);
    assert!(!retained_root.exists());
}

#[test]
fn preloaded_dependency_path_replacement_is_not_a_trust_proof() {
    const CHILD: &str = "ZIRCON_NATIVE_ADMISSION_REPLACED_LOADED_DLL";
    if let Some(input) = std::env::var_os(CHILD) {
        let (main, digest, dependency, marker, spoof_path, child_started): (
            PathBuf,
            NativePluginArtifactDigest,
            NativePluginArtifactDependency,
            PathBuf,
            PathBuf,
            PathBuf,
        ) = serde_json::from_str(input.to_str().unwrap()).unwrap();
        std::fs::write(&child_started, b"entered").unwrap();
        let spoof = unsafe {
            libloading::os::windows::Library::load_with_flags(&spoof_path, 0x100 | 0x800)
        }
        .unwrap();
        let spoof_value =
            unsafe { spoof.get::<unsafe extern "C" fn() -> i32>(b"fixture_dependency\0") }.unwrap();
        assert_eq!(unsafe { spoof_value() }, 7);

        std::fs::rename(&spoof_path, spoof_path.with_extension("loaded")).unwrap();
        std::fs::copy(
            main.parent().unwrap().join(&dependency.file_name),
            &spoof_path,
        )
        .unwrap();
        let error = NativePluginArtifactStaging::prepare("fixture", &main, &digest, &[dependency])
            .expect_err("path replacement must not authenticate an already loaded image");
        assert!(error.to_string().contains("previously admitted"), "{error}");
        assert_eq!(unsafe { spoof_value() }, 7);
        assert!(!marker.exists());
        drop(spoof);
        return;
    }

    let fixture = DllFixture::build();
    let spoof_root = fixture.root.join("replaced-preload");
    std::fs::create_dir(&spoof_root).unwrap();
    let spoof_source = spoof_root.join("spoof.c");
    std::fs::write(
        &spoof_source,
        "__declspec(dllexport) int fixture_dependency(void) { return 7; }",
    )
    .unwrap();
    let spoof_path = spoof_root.join(&fixture.dependency.file_name);
    compile_dll(&spoof_source, &spoof_path, &[]);
    let child_started = fixture.root.join("preloaded-replacement-child-started");
    let input = serde_json::to_string(&(
        &fixture.main,
        &fixture.main_digest,
        &fixture.dependency,
        &fixture.marker,
        &spoof_path,
        &child_started,
    ))
    .unwrap();
    let result = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg(format!(
            "{}::preloaded_dependency_path_replacement_is_not_a_trust_proof",
            module_path!().split_once("::").unwrap().1
        ))
        .arg("--nocapture")
        .env(CHILD, input)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        child_started.exists(),
        "rename regression child must execute its exact test body"
    );
    assert!(!fixture.marker.exists());
}

#[test]
fn cwd_and_path_spoof_dll_cannot_override_authenticated_dependency() {
    const CHILD: &str = "ZIRCON_NATIVE_ADMISSION_SEARCH_FIXTURE";
    if let Some(input) = std::env::var_os(CHILD) {
        let (main, digest, dependency, marker): (
            PathBuf,
            NativePluginArtifactDigest,
            NativePluginArtifactDependency,
            PathBuf,
        ) = serde_json::from_str(input.to_str().unwrap()).unwrap();
        let staging =
            NativePluginArtifactStaging::prepare("fixture", &main, &digest, &[dependency]).unwrap();
        let library = unsafe {
            libloading::os::windows::Library::load_with_flags(staging.library_path(), 0x100 | 0x800)
        }
        .unwrap();
        let value =
            unsafe { library.get::<unsafe extern "C" fn() -> i32>(b"fixture_value\0") }.unwrap();
        assert_eq!(unsafe { value() }, 42);
        assert!(marker.exists());
        drop(library);
        return;
    }
    let fixture = DllFixture::build();
    let poison_root = fixture.root.join("poison-search");
    std::fs::create_dir(&poison_root).unwrap();
    let source = poison_root.join("poison.c");
    std::fs::write(
        &source,
        "__declspec(dllexport) int fixture_dependency(void) { return 7; }",
    )
    .unwrap();
    compile_dll(
        &source,
        &poison_root.join(&fixture.dependency.file_name),
        &[],
    );
    let mut search = vec![poison_root.clone()];
    if let Some(path) = std::env::var_os("PATH") {
        search.extend(std::env::split_paths(&path));
    }
    let input = serde_json::to_string(&(
        &fixture.main,
        &fixture.main_digest,
        &fixture.dependency,
        &fixture.marker,
    ))
    .unwrap();
    let result = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg(format!(
            "{}::cwd_and_path_spoof_dll_cannot_override_authenticated_dependency",
            module_path!().split_once("::").unwrap().1
        ))
        .arg("--nocapture")
        .env(CHILD, input)
        .env("PATH", std::env::join_paths(search).unwrap())
        .current_dir(&poison_root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        fixture.marker.exists(),
        "child must execute the positive admission test"
    );
}
