use actix_web::{web, App, HttpServer, middleware};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handlers;
mod models;
mod middleware as app_middleware;
mod services;
mod di;

#[derive(OpenApi)]
#[openapi(
    info(title = "OpenDuckRust API", version = "0.1.0"),
    paths(),
    components(schemas())
)]
pub struct ApiDoc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    tracing::info!("Starting openduckrust API server");

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            // TODO: Add tenant middleware, routes
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
