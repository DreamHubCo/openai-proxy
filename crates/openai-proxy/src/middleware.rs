use poem::{http::StatusCode, Endpoint, Request, Response, Result};

use crate::{data::AppData, user::User};

/// Rate limits requests.
/// If this middleware is used, the `AppData` must have a `Limiter` set.
/// If the rate limit is exceeded, a 429 response is returned.
/// This middleware is only used if rate limit settings are set.
/// Limits are set per endpoint and based on the user making the request.
/// A user is required to be in the request data.
pub async fn rate_limit<E: Endpoint>(next: E, req: Request) -> Result<Response> {
    let mut limiter = &req.data::<AppData>().unwrap().limiter.unwrap();
    let user = req.data::<User>().unwrap();
    let key = format!("{}:{}", user.sub, req.uri().path());

    match limiter.check(&key).await {
        Ok(_) => next.call(req).await,
        Err(e) => Ok(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(e.to_string())?),
    }
}
