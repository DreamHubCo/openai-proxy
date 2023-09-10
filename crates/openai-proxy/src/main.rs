use async_openai::{config::OpenAIConfig, Client};
use futures::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use openai_proxy::settings::Settings;
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
    EndpointExt, FromRequest, IntoResponse, Request, RequestBody, Response, Result, Route, Server,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    /// The user's ID as parsed from their JWT.
    /// This will be sent to OpenAI as the user ID.
    sub: String,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for User {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                poem::Error::from_string("missing Authorization header", StatusCode::UNAUTHORIZED)
            })?;

        // req.data is None???
        let settings = req.data::<Data<AppData>>().unwrap().settings.clone();
        let result = decode::<User>(
            token,
            &DecodingKey::from_secret(settings.hs256_secret.as_ref()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|e| {
            poem::Error::from_string(
                format!("failed to decode JWT: {}", e),
                StatusCode::UNAUTHORIZED,
            )
        })?;
        Ok(result.claims)
    }
}

#[derive(Clone, Debug)]
struct AppData {
    openai: Client<OpenAIConfig>,
    settings: Settings,
}

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
    let data = AppData {
        openai,
        settings: settings.clone(),
    };

    let cors = Cors::new().allow_origin(settings.cors_host.clone());
    let app = Route::new()
        .at("/chat/completions", post(chat_completion))
        .with(cors)
        .with(Tracing::default())
        .with(CatchPanic::new())
        .data(data);

    Server::new(TcpListener::bind(settings.bind_addr()))
        .run(app)
        .await?;
    Ok(())
}
