use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use zircon_runtime::core::{
    CoreError, CoreResult, CoreWeak, ModuleContext, ModuleDescriptor, ModuleLifecycle,
    TaskGraphShutdownReport,
};
use zircon_runtime::plugin::{
    PluginPackageRole, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use crate::entry::product_shutdown::retained_owner::pending_owner_count;
use crate::entry::{
    ProductCloseError, ProductComposition, ProductCompositionFailure, ProductCompositionRequest,
};

pub(in crate::entry) fn close_composition(composition: ProductComposition) {
    let report = composition
        .close_until(deadline())
        .expect("healthy original product must close explicitly");
    assert_graph_closed(&report);
}

pub(in crate::entry) fn close_runtime(runtime: &zircon_runtime::core::CoreRuntime) {
    assert_graph_closed(
        &runtime
            .shutdown_until(deadline())
            .expect("direct bootstrap must close the original Core"),
    );
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

fn assert_graph_closed(report: &TaskGraphShutdownReport) {
    assert!(
        !report.has_in_flight_work(),
        "actual graph receipt: {report:?}"
    );
    assert!(!report.timer_started || report.timer_joined);
}

type CleanupGate = (mpsc::SyncSender<()>, mpsc::Receiver<()>);

struct Control {
    fail_cleanup: AtomicBool,
    finish_calls: AtomicUsize,
    cleanup_calls: AtomicUsize,
    pin_drops: AtomicUsize,
    gate: Mutex<Option<CleanupGate>>,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            fail_cleanup: AtomicBool::new(false),
            finish_calls: AtomicUsize::new(0),
            cleanup_calls: AtomicUsize::new(0),
            pin_drops: AtomicUsize::new(0),
            gate: Mutex::new(None),
        }
    }
}

struct CodePin(Arc<Control>);

impl Drop for CodePin {
    fn drop(&mut self) {
        self.0.pin_drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct RetryLifecycle {
    id: &'static str,
    control: Arc<Control>,
    _code_pin: Arc<CodePin>,
}

impl ModuleLifecycle for RetryLifecycle {
    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.control.finish_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.control.cleanup_calls.fetch_add(1, Ordering::SeqCst);
        let gate = self.control.gate.lock().unwrap().take();
        if let Some((entered, continue_cleanup)) = gate {
            entered.send(()).unwrap();
            continue_cleanup.recv().unwrap();
        }
        if self.control.fail_cleanup.load(Ordering::SeqCst) {
            Err(CoreError::ChannelSend(format!(
                "{} cleanup is blocked",
                self.id
            )))
        } else {
            Ok(())
        }
    }
}

struct Original {
    control: Arc<Control>,
    core: CoreWeak,
    lifecycle: Weak<RetryLifecycle>,
    code_pin: Weak<CodePin>,
    _config_file: ConfigFile,
}

struct ConfigFile(PathBuf);

impl ConfigFile {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-product-close-{}-{stamp}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("exclusive private Foundation config fixture");
        std::io::Write::write_all(&mut file, b"{}").unwrap();
        Self(path)
    }
}

impl Drop for ConfigFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0)
            .expect("remove only this exclusively-created private config file");
    }
}

fn acquire(id: &'static str) -> (ProductComposition, Original) {
    let config_file = ConfigFile::new();
    let control = Arc::new(Control::default());
    let pin = Arc::new(CodePin(control.clone()));
    let lifecycle = Arc::new(RetryLifecycle {
        id,
        control: control.clone(),
        _code_pin: pin.clone(),
    });
    let lifecycle_weak = Arc::downgrade(&lifecycle);
    let pin_weak = Arc::downgrade(&pin);
    let descriptor = RuntimePluginDescriptor::builder(
        id,
        id,
        zircon_runtime::builtin::RuntimePluginId::new(id),
        format!("zircon_plugin_{id}_runtime"),
    )
    .with_target_modes([
        zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
    ])
    .with_package_role(PluginPackageRole::Production)
    .with_module_descriptor(
        ModuleDescriptor::new(format!("{id}.runtime"), id).with_lifecycle(lifecycle.clone()),
    )
    .build();
    let report = RuntimePluginRegistrationReport::from_plugin(&descriptor);
    let composition = ProductCompositionRequest::new(super::explicit_headless_config(
        super::explicit_manifest(id, true),
    ))
    .with_config_file_path(config_file.0.clone())
    .with_runtime_plugin_registrations([report])
    .compose()
    .expect("real production module must activate through public product composition");
    assert_eq!(control.finish_calls.load(Ordering::SeqCst), 1);
    assert!(composition
        .module_selection_report()
        .module_keys()
        .contains(&format!("{id}.runtime").as_str()));
    let core = composition.core().downgrade();
    // These are genuine descriptor/code pins. None of the fixture's extra strong clones may
    // keep the original generation alive while the close/drop oracle runs.
    drop(descriptor);
    drop(lifecycle);
    drop(pin);
    (
        composition,
        Original {
            control,
            core,
            lifecycle: lifecycle_weak,
            code_pin: pin_weak,
            _config_file: config_file,
        },
    )
}

fn assert_retained(original: &Original) {
    assert!(original.core.upgrade().is_some());
    assert!(original.lifecycle.upgrade().is_some());
    assert!(original.code_pin.upgrade().is_some());
    assert_eq!(original.control.pin_drops.load(Ordering::SeqCst), 0);
}

fn assert_released(original: &Original) {
    assert!(original.core.upgrade().is_none());
    assert!(original.lifecycle.upgrade().is_none());
    assert!(original.code_pin.upgrade().is_none());
    assert_eq!(original.control.pin_drops.load(Ordering::SeqCst), 1);
}

fn assert_close_primary(failure: &ProductCompositionFailure, id: &str) {
    let primary = failure
        .primary()
        .downcast_ref::<ProductCloseError>()
        .expect("typed close primary");
    match primary {
        ProductCloseError::Core(error) => match error.module_error() {
            Some(CoreError::ChannelSend(message)) => {
                assert_eq!(message, &format!("{id} cleanup is blocked"))
            }
            other => panic!("unexpected module primary: {other:?}"),
        },
        other => panic!("unexpected primary: {other:?}"),
    }
    assert_graph_closed(primary.graph_report().expect("actual owned graph receipt"));
}

#[test]
fn two_preacquired_products_retain_distinct_owners_until_exact_retry() {
    assert_eq!(pending_owner_count(), 0);
    let (a, original_a) = acquire("retained_a");
    let (b, original_b) = acquire("retained_b");
    // Both public compositions were healthy and acquired before either close failed.
    original_a
        .control
        .fail_cleanup
        .store(true, Ordering::SeqCst);
    original_b
        .control
        .fail_cleanup
        .store(true, Ordering::SeqCst);
    let failed_a = a
        .close_until(deadline())
        .expect_err("actual A module cleanup failure");
    let failed_b = b
        .close_until(deadline())
        .expect_err("actual B module cleanup failure");
    assert_close_primary(&failed_a, "retained_a");
    assert_close_primary(&failed_b, "retained_b");
    assert!(!Arc::ptr_eq(
        failed_a.owner.as_ref().unwrap(),
        failed_b.owner.as_ref().unwrap()
    ));
    assert_eq!(pending_owner_count(), 2);
    assert_retained(&original_a);
    assert_retained(&original_b);
    assert!(
        ProductCompositionRequest::new(crate::EntryConfig::for_runtime_profile(
            zircon_runtime::core::framework::project::RuntimeProfileId::Minimal
        ))
        .compose()
        .expect_err("pending original packets block this host's new admission")
        .to_string()
        .contains("product admission pending")
    );

    original_a
        .control
        .fail_cleanup
        .store(false, Ordering::SeqCst);
    let a_report = failed_a
        .retry_cleanup_until(deadline())
        .unwrap()
        .expect("A's actual original graph report");
    assert_graph_closed(&a_report);
    assert_eq!(original_a.control.cleanup_calls.load(Ordering::SeqCst), 2);
    assert!(!failed_a.cleanup_pending());
    assert_eq!(pending_owner_count(), 1);
    assert_released(&original_a);
    assert_retained(&original_b);
    assert_close_primary(&failed_b, "retained_b");
    assert!(
        ProductCompositionRequest::new(crate::EntryConfig::for_runtime_profile(
            zircon_runtime::core::framework::project::RuntimeProfileId::Minimal
        ))
        .compose()
        .is_err()
    );

    // A stale exact capability can observe A's completed receipt but cannot remove B.
    assert_graph_closed(&failed_a.retry_cleanup_until(deadline()).unwrap().unwrap());
    assert_eq!(pending_owner_count(), 1);
    assert_eq!(original_a.control.cleanup_calls.load(Ordering::SeqCst), 2);
    assert_retained(&original_b);
    original_b
        .control
        .fail_cleanup
        .store(false, Ordering::SeqCst);
    let b_report = failed_b
        .retry_cleanup_until(deadline())
        .unwrap()
        .expect("B's actual original graph report");
    assert_graph_closed(&b_report);
    assert_released(&original_b);
    assert_eq!(original_b.control.cleanup_calls.load(Ordering::SeqCst), 2);
    assert_eq!(pending_owner_count(), 0);
    close_composition(
        ProductCompositionRequest::new(crate::EntryConfig::for_runtime_profile(
            zircon_runtime::core::framework::project::RuntimeProfileId::Minimal,
        ))
        .compose()
        .unwrap(),
    );
}

#[test]
fn core_only_retry_moves_threads_and_busy_diagnostics_do_not_wait_for_cleanup() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProductCompositionFailure>();
    assert_send_sync::<ProductCloseError>();
    let (composition, original) = acquire("busy_core");
    original.control.fail_cleanup.store(true, Ordering::SeqCst);
    let failure = composition.close_until(deadline()).unwrap_err();
    original.control.fail_cleanup.store(false, Ordering::SeqCst);
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (continue_tx, continue_rx) = mpsc::sync_channel(1);
    *original.control.gate.lock().unwrap() = Some((entered_tx, continue_rx));
    let exact_owner = failure.clone();
    let retry_deadline = deadline();
    let worker = std::thread::spawn(move || exact_owner.retry_cleanup_until(retry_deadline));
    entered_rx
        .recv_timeout(retry_deadline.saturating_duration_since(Instant::now()))
        .expect("actual cleanup callback must enter within the retry budget");
    // This is an actual lifecycle callback holding the original cleanup mutex. No private
    // synthetic receipt, elapsed-time assertion or sleep replaces the real production path.
    assert!(failure.cleanup_pending());
    assert!(matches!(
        failure.cleanup_observation(),
        Err(ProductCloseError::Busy)
    ));
    assert!(failure.to_string().contains("cleanup busy"));
    assert!(format!("{failure:?}").contains("cleanup_pending: true"));
    assert_eq!(pending_owner_count(), 1);
    assert_retained(&original);
    continue_tx.send(()).unwrap();
    let report = worker
        .join()
        .unwrap()
        .unwrap()
        .expect("actual Core-only graph receipt on the retrying host thread");
    assert_graph_closed(&report);
    assert_released(&original);
    assert_eq!(pending_owner_count(), 0);
}

#[test]
fn ordinary_host_primary_stays_typed_when_original_cleanup_has_a_secondary_error() {
    let (composition, original) = acquire("primary_core");
    original.control.fail_cleanup.store(true, Ordering::SeqCst);
    let failure = composition.fail_until(
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "ordinary host input is invalid",
        ),
        deadline(),
    );
    assert_eq!(
        failure
            .primary()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    let (secondary, report) = failure.cleanup_observation().unwrap();
    assert!(matches!(
        secondary,
        Some(ProductCloseError::Core(error))
            if matches!(error.module_error(), Some(CoreError::ChannelSend(_)))
    ));
    assert_graph_closed(&report.expect("actual owned graph receipt"));
    assert_retained(&original);
    original.control.fail_cleanup.store(false, Ordering::SeqCst);
    assert_graph_closed(&failure.retry_cleanup_until(deadline()).unwrap().unwrap());
    assert_eq!(
        failure
            .primary()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    assert_released(&original);
    assert_eq!(pending_owner_count(), 0);
}

#[test]
#[ignore = "requires the existing compatible admitted V8 DLL selected by ZIRCON_RUNTIME_LIBRARY"]
fn first_observation_keeps_actual_core_receipt_while_real_runtime_is_shared() {
    use crate::entry::product_shutdown::retained_owner::RetainedPacket;
    use crate::entry::runtime_library::{LoadedRuntime, RuntimeSession};

    assert_eq!(pending_owner_count(), 0);
    // Reuse the production ABI loader and create path; no fake session/API table or project is used.
    let runtime = Arc::new(
        RuntimeSession::create_with_profile_and_project(
            LoadedRuntime::load_default().expect("the selected real V8 DLL must be admitted"),
            b"minimal",
            None,
            None,
            None,
            None,
        )
        .expect("the actual ABI minimal/Core profile must create successfully"),
    );
    let (composition, original) = acquire("shared_runtime_receipt");
    let mut packet = composition.into_packet().with_runtime(Arc::clone(&runtime));
    let primary = packet
        .close_until(deadline())
        .expect_err("the real shared Runtime must remain owned after Core graph close");
    assert!(matches!(primary, ProductCloseError::RuntimeShared));
    assert_eq!(original.control.cleanup_calls.load(Ordering::SeqCst), 1);
    let failure = ProductCompositionFailure::retained(Arc::new(primary), packet);

    // This is the first observation, before any retry can repair receipt propagation.
    let (secondary, report) = failure.cleanup_observation().unwrap();
    assert!(secondary.is_none());
    let report = report.expect("retain must carry the graph receipt already returned by Core");
    assert_graph_closed(&report);
    assert!(failure.cleanup_pending());
    assert_eq!(pending_owner_count(), 1);
    assert_retained(&original);

    drop(runtime);
    let retry_report = failure
        .retry_cleanup_until(deadline())
        .expect("actual creator-thread ABI destroy must succeed")
        .expect("the original completed Core graph receipt must remain available");
    assert_graph_closed(&retry_report);
    assert_eq!(retry_report, report);
    assert_eq!(original.control.cleanup_calls.load(Ordering::SeqCst), 1);
    assert!(!failure.cleanup_pending());
    assert_released(&original);
    assert_eq!(pending_owner_count(), 0);
}
