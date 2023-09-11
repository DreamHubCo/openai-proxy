use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// The port to bind the http server to. Defaults to 4000.
    #[serde(default = "Settings::default_port")]
    pub port: u16,
    /// The public url of the server. Defaults to http://localhost:4000.
    #[serde(default = "Settings::default_public_url")]
    pub public_url: String,
    /// The host to allow for CORS. Defaults to localhost:3000.
    /// Set to "*" to allow all hosts.
    #[serde(default = "Settings::default_cors_host")]
    pub cors_host: String,

    /// The OpenAI API key. Required.
    pub openai_api_key: String,
    /// The models to allow for chat completions. Defaults to all models with an empty list.
    #[serde(default)]
    pub allowed_models: Vec<String>,
    /// The OpenAI organization ID. Optional.
    #[serde(default)]
    pub openai_org_id: Option<String>,

    /// The secret to use for HS256 JWT validation. Required.
    pub hs256_secret: String,

    /// The rate limit settings. Optional.
    /// When not set, rate limiting is disabled.
    pub rate_limit: Option<RateLimitSettings>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitSettings {
    /// The number of requests allowed in the timeframe.
    pub limit: usize,
    /// How often the sliding window resets, in seconds.
    pub timeframe_seconds: usize,
    /// The redis URL to use for rate limiting.
    pub redis_url: String,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .build()?
            .try_deserialize()?;
        Ok(settings)
    }

    /// Returns the address to bind the http server to.
    pub fn bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }

    fn default_port() -> u16 {
        4000
    }

    fn default_public_url() -> String {
        format!("http://localhost:{}", Self::default_port())
    }

    fn default_cors_host() -> String {
        "http://localhost:3000".to_string()
    }
}
