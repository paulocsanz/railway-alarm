use backend::{router, serve};

use axum::http::{HeaderValue, HeaderName, Method};
use tower_http::cors::CorsLayer;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    #[cfg(debug_assertions)]
    let _ = dotenv::from_filename(".env.local");

    #[cfg(not(debug_assertions))]
    let _ = dotenv::dotenv();

    #[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        #[cfg(not(debug_assertions))]
        let val = "backend=info";

        #[cfg(debug_assertions)]
        let val = "backend=debug";

        std::env::set_var("RUST_LOG", val);
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let origin = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_owned())
        .parse::<HeaderValue>()?;
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "4000".to_owned())
        .parse::<u16>()?;

    let app = router().layer(
        CorsLayer::new()
            .allow_credentials(false)
            .allow_headers(vec![
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
            ])
            .allow_methods([Method::POST])
            .allow_origin(origin)
    );
    serve(app, port).await?;

    Ok(())
}
