use openai_api_rust::{
    chat::{ChatApi, ChatBody},
    Auth, OpenAI,
};
use openai_proxy::{
    conversion::{ChatCompletion, ChatCompletionRequest},
    settings::Settings,
};
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    web::Data,
    EndpointExt, Result, Route, Server,
};
use poem_openapi::{payload::Json, ApiResponse, OpenApi, OpenApiService};

#[derive(ApiResponse)]
enum ChatCompletionResponse {
    #[oai(status = 200)]
    Success(Json<ChatCompletion>),
}

#[derive(Clone, Debug)]
struct AppData {
    openai: OpenAI,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/chat/completions", method = "post")]
    async fn chat_completion(
        &self,
        req: Json<ChatCompletionRequest>,
        data: Data<&AppData>,
    ) -> Result<ChatCompletionResponse> {
        let body: ChatBody = req.0.into();
        let completion = data
            .0
            .openai
            .chat_completion_create(&body)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(ChatCompletionResponse::Success(Json(completion.into())))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let auth = Auth::new(&settings.openai_api_key);
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let data = AppData { openai };

    let api_service =
        OpenApiService::new(Api, "OpenAI Proxy", "1.0").server(settings.public_url.clone());
    let cors = Cors::new().allow_origin(settings.cors_host.clone());
    let app = Route::new()
        .nest("/", api_service)
        .with(cors)
        .with(Tracing::default())
        .data(data);

    Server::new(TcpListener::bind(settings.bind_addr()))
        .run(app)
        .await?;
    Ok(())
}
