use config::Config;

lazy_static::lazy_static! {
    #[derive(Debug)]
    pub static ref CONFIG: Config = Config::builder()
    .add_source(config::Environment::with_prefix("OPENAI_PROXY").separator("_"))
    .build()
    .unwrap();
}

/// Get a configuration value from the static configuration object
pub fn get<'a, T: serde::Deserialize<'a>>(key: &str) -> T {
    // You shouldn't probably do it like that and actually handle that error that might happen
    // here, but for the sake of simplicity, we do it like this here
    CONFIG.get::<T>(key).unwrap()
}
