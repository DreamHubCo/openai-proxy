use async_openai::{config::OpenAIConfig, Client};

use crate::{limiter::Limiter, settings::Settings};

#[derive(Clone, Debug)]
pub struct AppData {
    pub openai: Client<OpenAIConfig>,
    pub settings: Settings,
    pub limiter: Option<Limiter>,
}
