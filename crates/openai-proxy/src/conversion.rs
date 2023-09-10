use poem_openapi::Object;

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
pub struct ChatCompletionRequest {
    messages: Vec<ChatCompletionMessage>,
    model: String,
}

/// Allow converting ChatCompletionRequest into ChatBody
impl From<ChatCompletionRequest> for openai_api_rust::chat::ChatBody {
    fn from(req: ChatCompletionRequest) -> Self {
        openai_api_rust::chat::ChatBody {
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
impl From<openai_api_rust::Usage> for ChatCompletionUsage {
    fn from(usage: openai_api_rust::Usage) -> Self {
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
impl From<openai_api_rust::Message> for ChatCompletionChoiceMessage {
    fn from(msg: openai_api_rust::Message) -> Self {
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
impl From<openai_api_rust::Choice> for ChatCompletionChoice {
    fn from(choice: openai_api_rust::Choice) -> Self {
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
pub struct ChatCompletion {
    id: Option<String>,
    object: Option<String>,
    created: u64,
    model: Option<String>,
    choices: Vec<ChatCompletionChoice>,
    usage: ChatCompletionUsage,
}

/// Allow converting Completion into ChatCompletion
impl From<openai_api_rust::completions::Completion> for ChatCompletion {
    fn from(completion: openai_api_rust::completions::Completion) -> Self {
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
