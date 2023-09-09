use openai_proxy::settings::Settings;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};

struct Api;

#[OpenApi]
impl Api {
    /// Hello world
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<&'static str> {
        PlainText("Hello World")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;

    let api_service =
        OpenApiService::new(Api, "OpenAI Proxy", "1.0").server(settings.public_url.clone());
    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/", api_service).nest("/docs", ui);

    Server::new(TcpListener::bind(settings.bind_addr()))
        .run(app)
        .await?;
    Ok(())
}
