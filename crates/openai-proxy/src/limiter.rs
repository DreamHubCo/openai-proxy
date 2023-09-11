/// Rate limiter using redis
#[derive(Debug, Clone)]
pub struct Limiter {
    limit: usize,
    timeframe_seconds: usize,
    redis: redis::aio::Connection,
}

impl Limiter {
    /// Creates a new rate limiter.
    pub async fn new(settings: &crate::settings::RateLimitSettings) -> anyhow::Result<Self> {
        let redis = redis::Client::open(settings.redis_url.as_str())?
            .get_async_connection()
            .await?;
        Ok(Self {
            limit: settings.limit,
            timeframe_seconds: settings.timeframe_seconds,
            redis,
        })
    }

    /// Checks if the rate limit is exceeded.
    /// Returns the number of requests remaining in the timeframe.
    pub async fn check(&mut self, key: &str) -> anyhow::Result<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let timeframe_start = now - self.timeframe_seconds as u64;
        let timeframe_end = now + 1;
        let timeframe = format!("{}:{}:{}", key, timeframe_start, timeframe_end);

        let mut pipe = redis::pipe();
        pipe.atomic()
            .incr(key, 1)
            .expire(key, self.timeframe_seconds)
            .incr(&timeframe, 1)
            .expire(&timeframe, self.timeframe_seconds);
        let (count, timeframe_count): (usize, usize) = pipe.query_async(&mut self.redis).await?;

        let remaining = self.limit.saturating_sub(timeframe_count);
        if remaining == 0 {
            return Err(anyhow::anyhow!("rate limit exceeded"));
        }
        Ok(remaining)
    }
}
