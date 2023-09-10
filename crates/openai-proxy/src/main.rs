use openai_api_rust::{
    chat::{ChatApi, ChatBody},
    completions::Completion,
    Auth, Choice, Message, OpenAI, Usage,
};
use openai_proxy::settings::Settings;
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    web::Data,
    EndpointExt, Result, Route, Server,
};
use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi, OpenApiService};

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionMessage {
    content: String,
    role: String,
}

/// Allow converting ChatCompletionMessage into Message
impl From<ChatCompletionMessage> for openai_api_rust::Message {
    fn from(msg: ChatCompletionMessage) -> Self {
        let role = match msg.role.as_str() {
            "user" => openai_api_rust::Role::User,
            "assistant" => openai_api_rust::Role::Assistant,
            "system" => openai_api_rust::Role::System,
            _ => panic!("Invalid role"),
        };
        openai_api_rust::Message {
            content: msg.content,
            role,
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionRequest {
    messages: Vec<ChatCompletionMessage>,
    model: String,
}

/// Allow converting ChatCompletionRequest into ChatBody
impl From<ChatCompletionRequest> for ChatBody {
    fn from(req: ChatCompletionRequest) -> Self {
        ChatBody {
            model: req.model,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: req.messages.into_iter().map(|m| m.into()).collect(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

/// Allow converting Usage into ChatCompletionUsage
impl From<Usage> for ChatCompletionUsage {
    fn from(usage: Usage) -> Self {
        ChatCompletionUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionChoiceMessage {
    content: String,
    role: String,
}

/// Allow converting Message into ChatCompletionChoiceMessage
impl From<Message> for ChatCompletionChoiceMessage {
    fn from(msg: Message) -> Self {
        ChatCompletionChoiceMessage {
            content: msg.content,
            role: match msg.role {
                openai_api_rust::Role::User => "user",
                openai_api_rust::Role::Assistant => "assistant",
                openai_api_rust::Role::System => "system",
            }
            .to_string(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionChoice {
    text: Option<String>,
    index: u32,
    logprobs: Option<String>,
    finish_reason: Option<String>,
    message: Option<ChatCompletionChoiceMessage>,
}

/// Allow converting Choice into ChatCompletionChoice
impl From<Choice> for ChatCompletionChoice {
    fn from(choice: Choice) -> Self {
        ChatCompletionChoice {
            text: choice.text,
            index: choice.index,
            logprobs: choice.logprobs,
            finish_reason: choice.finish_reason,
            message: choice.message.map(|m| m.into()),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletion {
    id: Option<String>,
    object: Option<String>,
    created: u64,
    model: Option<String>,
    choices: Vec<ChatCompletionChoice>,
    usage: ChatCompletionUsage,
}

/// Allow converting Completion into ChatCompletion
impl From<Completion> for ChatCompletion {
    fn from(completion: Completion) -> Self {
        ChatCompletion {
            id: completion.id,
            object: completion.object,
            created: completion.created,
            model: completion.model,
            choices: completion.choices.into_iter().map(|c| c.into()).collect(),
            usage: completion.usage.into(),
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
            .expect("bad response from openai");

        Ok(ChatCompletionResponse::Success(Json(completion.into())))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;
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
