use poem_openapi::Object;

use crate::settings::Settings;

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionMessage {
    content: String,
    role: String,
}

/// Allow converting ChatCompletionMessage into Message
impl From<ChatCompletionMessage> for async_openai::types::ChatCompletionRequestMessage {
    fn from(msg: ChatCompletionMessage) -> Self {
        async_openai::types::ChatCompletionRequestMessageArgs::default()
            .content(msg.content)
            .role(match msg.role.as_str() {
                "user" => async_openai::types::Role::User,
                "assistant" => async_openai::types::Role::Assistant,
                "system" => async_openai::types::Role::System,
                _ => panic!("Invalid role {}", msg.role),
            })
            .build()
            .unwrap()
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct ChatCompletionRequest {
    messages: Vec<ChatCompletionMessage>,
    model: String,
}

impl ChatCompletionRequest {
    pub fn validate(&self, settings: &Settings) -> anyhow::Result<()> {
        if !settings.allowed_models.is_empty() && !settings.allowed_models.contains(&self.model) {
            return Err(anyhow::anyhow!("Model {} is not allowed", self.model));
        }
        Ok(())
    }
}

/// Allow converting ChatCompletionRequest into ChatBody
impl From<ChatCompletionRequest> for async_openai::types::CreateChatCompletionRequest {
    fn from(req: ChatCompletionRequest) -> Self {
        async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(req.model)
            .messages(
                req.messages
                    .into_iter()
                    .map(|m| m.into())
                    .collect::<Vec<async_openai::types::ChatCompletionRequestMessage>>(),
            )
            .build()
            .unwrap()
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Allow converting Usage into ChatCompletionUsage
impl From<async_openai::types::Usage> for ChatCompletionUsage {
    fn from(usage: async_openai::types::Usage) -> Self {
        ChatCompletionUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionChoiceMessage {
    content: Option<String>,
    role: String,
}

/// Allow converting Message into ChatCompletionChoiceMessage
impl From<async_openai::types::ChatCompletionResponseMessage> for ChatCompletionChoiceMessage {
    fn from(msg: async_openai::types::ChatCompletionResponseMessage) -> Self {
        ChatCompletionChoiceMessage {
            content: msg.content,
            role: match msg.role {
                async_openai::types::Role::User => "user",
                async_openai::types::Role::Assistant => "assistant",
                async_openai::types::Role::System => "system",
                async_openai::types::Role::Function => "function",
            }
            .to_string(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
struct ChatCompletionChoice {
    index: u32,
    finish_reason: Option<String>,
    message: ChatCompletionChoiceMessage,
}

/// Allow converting Choice into ChatCompletionChoice
impl From<async_openai::types::ChatChoice> for ChatCompletionChoice {
    fn from(choice: async_openai::types::ChatChoice) -> Self {
        ChatCompletionChoice {
            index: choice.index,
            finish_reason: choice.finish_reason,
            message: choice.message.into(),
        }
    }
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct ChatCompletion {
    id: String,
    object: String,
    created: u32,
    model: String,
    usage: Option<ChatCompletionUsage>,
    choices: Vec<ChatCompletionChoice>,
}

/// Allow converting Completion into ChatCompletion
impl From<async_openai::types::CreateChatCompletionResponse> for ChatCompletion {
    fn from(completion: async_openai::types::CreateChatCompletionResponse) -> Self {
        ChatCompletion {
            id: completion.id,
            object: completion.object,
            created: completion.created,
            model: completion.model,
            choices: completion
                .choices
                .into_iter()
                .map(|c| c.into())
                .collect::<Vec<ChatCompletionChoice>>(),
            usage: completion.usage.map(|u| u.into()),
        }
    }
}
