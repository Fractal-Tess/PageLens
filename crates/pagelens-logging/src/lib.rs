use std::sync::OnceLock;

use tracing_subscriber::EnvFilter;

pub use tracing::{debug, error, info, instrument, trace, warn};

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init(service_name: &str) {
    LOGGING_INIT.get_or_init(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(format!(
                "info,{service_name}=info,pagelens_app=info,pagelens_api=info,pagelens_cli=info,pagelens_core=info,pagelens_db=info"
            ))
        });

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_names(true)
            .try_init();
    });
}
