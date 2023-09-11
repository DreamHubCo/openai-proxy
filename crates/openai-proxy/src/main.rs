use async_openai::{config::OpenAIConfig, Client};
use futures::StreamExt;
use openai_proxy::{data::AppData, limiter::Limiter, middleware, settings::Settings, user::User};
use poem::{
    handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{CatchPanic, Cors, Tracing},
    post,
    web::{
        sse::{Event, SSE},
        Data,
    },
    EndpointExt, FromRequest, IntoResponse, Response, Result, Route, Server,
};

#[handler]
async fn chat_completion(
    req: poem::web::Json<async_openai::types::CreateChatCompletionRequest>,
    data: Data<&AppData>,
    user: User,
) -> Result<Response> {
    if !data.settings.allowed_models.is_empty()
        && !data.settings.allowed_models.contains(&req.0.model)
    {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Model not allowed".to_string()));
    }

    let mut chat_req = req.0.clone();
    chat_req.user = Some(user.sub);

    if chat_req.stream.unwrap_or(false) {
        let openai_client = data.0.openai.clone();
        let stream = SSE::new(async_stream::stream! {
            let mut stream = openai_client.chat().create_stream(req.0.into()).await.unwrap();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(completion) => yield Event::message(serde_json::to_string(&completion).unwrap()),
                    Err(e) => {
                        eprintln!("Failed to get chat completion: {}", e);
                        break;
                    }
                }
            }
        });
        return Ok(stream.into_response());
    }

    let completion = data
        .0
        .openai
        .chat()
        .create(chat_req)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create chat completion: {}", e))?;

    Ok(poem::web::Json(completion).into_response())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let mut openai_config = OpenAIConfig::new().with_api_key(settings.openai_api_key.clone());
    if let Some(org_id) = settings.openai_org_id.clone() {
        openai_config = openai_config.with_org_id(org_id);
    }
    let openai = Client::with_config(openai_config);
    let limiter = match settings.rate_limit {
        Some(settings) => Some(Limiter::new(&settings).await?),
        None => None,
    };
    let data = AppData {
        openai,
        settings: settings.clone(),
        limiter,
    };

    let cors = Cors::new().allow_origin(settings.cors_host.clone());
    let mut app = Route::new()
        .at("/chat/completions", post(chat_completion))
        .with(cors)
        .with(Tracing::default())
        .with(CatchPanic::new())
        .with(User::middleware)
        .data(data);
    if settings.rate_limit.is_some() {
        app = app.with(middleware::rate_limit);
    }

    Server::new(TcpListener::bind(settings.bind_addr()))
        .run(app)
        .await?;
    Ok(())
}
