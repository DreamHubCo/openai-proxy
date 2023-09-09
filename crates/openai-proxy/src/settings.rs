use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    /// The port to bind the http server to. Defaults to 3000.
    #[serde(default = "Settings::default_port")]
    pub port: u16,
    /// The public url of the server. Defaults to http://localhost:3000.
    #[serde(default = "Settings::default_public_url")]
    pub public_url: String,
    /// The OpenAI API key. Required.
    pub openai_api_key: String,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::Environment::with_prefix("OPENAI_PROXY").separator("_"))
            .build()?
            .try_deserialize()?;
        Ok(settings)
    }

    /// Returns the address to bind the http server to.
    pub fn bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }

    fn default_port() -> u16 {
        3000
    }

    fn default_public_url() -> String {
        format!("http://localhost:{}", Self::default_port())
    }
}
