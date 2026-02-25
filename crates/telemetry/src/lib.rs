use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialise the global tracing subscriber.
///
/// Configuration is driven entirely by environment variables so that the same
/// binary can produce different output in development vs. production without
/// recompilation.
///
/// | Variable          | Values                          | Default   |
/// |-------------------|---------------------------------|-----------|
/// | `RUST_LOG`        | standard `tracing` directives   | `"info"`  |
/// | `RUST_LOG_FORMAT` | `"json"` or anything else       | compact   |
///
/// Call this **once**, as the very first statement of `main()`, before any
/// other crate initialisation.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match std::env::var("RUST_LOG_FORMAT").as_deref() {
        Ok("json") => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().compact())
                .init();
        }
    }
}
