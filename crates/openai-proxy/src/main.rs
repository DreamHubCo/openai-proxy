use async_openai::{config::OpenAIConfig, Client};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use openai_proxy::{
    conversion::{ChatCompletion, ChatCompletionRequest},
    settings::Settings,
};
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    web::Data,
    EndpointExt, Request, Result, Route, Server,
};
use poem_openapi::{
    auth::Bearer, payload::Json, ApiResponse, OpenApi, OpenApiService, SecurityScheme,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    /// The user's ID as parsed from their JWT.
    /// This will be sent to OpenAI as the user ID.
    sub: String,
}

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    key_name = "Authorization",
    key_in = "header",
    checker = "JWTAuth::check"
)]
struct JWTAuth(User);

impl JWTAuth {
    pub async fn check(req: &Request, bearer: Bearer) -> Option<User> {
        let hs256_secret = req.data::<AppData>().unwrap().settings.hs256_secret.clone();
        let token_message = decode::<User>(
            &bearer.token,
            &DecodingKey::from_secret(hs256_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );

        match token_message {
            Ok(token) => Some(token.claims),
            Err(e) => {
                eprintln!("Failed to decode JWT: {}", e);
                None
            }
        }
    }
}

#[derive(ApiResponse)]
enum ChatCompletionResponse {
    #[oai(status = 200)]
    Success(Json<ChatCompletion>),
}

#[derive(Clone, Debug)]
struct AppData {
    openai: Client<OpenAIConfig>,
    settings: Settings,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/chat/completions", method = "post")]
    async fn chat_completion(
        &self,
        req: Json<ChatCompletionRequest>,
        data: Data<&AppData>,
        auth: JWTAuth,
    ) -> Result<ChatCompletionResponse> {
        req.0.validate(&data.0.settings)?;

        let mut chat_req: async_openai::types::CreateChatCompletionRequest = req.0.into();
        chat_req.user = Some(auth.0.sub);
        let completion = data
            .0
            .openai
            .chat()
            .create(chat_req)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create chat completion: {}", e))?;

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

    let mut openai_config = OpenAIConfig::new().with_api_key(settings.openai_api_key.clone());
    if let Some(org_id) = settings.openai_org_id.clone() {
        openai_config = openai_config.with_org_id(org_id);
    }
    let openai = Client::with_config(openai_config);
    let data = AppData {
        openai,
        settings: settings.clone(),
    };

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
