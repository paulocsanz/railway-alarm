use color_eyre::Result;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let _ = dotenv::from_filename(".env.local")?;

    #[cfg(not(debug_assertions))]
    let _ = dotenv::dotenv();

    #[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        #[cfg(not(debug_assertions))]
        let val = "alarms=info";

        #[cfg(debug_assertions)]
        let val = "alarms=debug";

        std::env::set_var("RUST_LOG", val);
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "alarms=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    alarms::run().await?;

    Ok(())
}
