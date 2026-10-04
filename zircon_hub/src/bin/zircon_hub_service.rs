use std::time::Duration;
use zircon_hub::service;

const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

fn main() {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            eprintln!("Hub service: runtime_unavailable");
            std::process::exit(1);
        }
    };
    let result = runtime.block_on(service::run());
    if let Err(error) = &result {
        eprintln!("Hub service: {}", error.code());
    }
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
    if result.is_err() {
        std::process::exit(1);
    }
}
