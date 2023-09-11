use std::ops::Deref;

use jsonwebtoken::{decode, DecodingKey, Validation};
use poem::{http::StatusCode, Endpoint, FromRequest, Request, RequestBody, Response, Result};
use serde::{Deserialize, Serialize};

use crate::data::AppData;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// The user's ID as parsed from their JWT.
    /// This will be sent to OpenAI as the user ID.
    pub sub: String,
}

impl User {
    /// Sets the user in the request data.
    pub async fn middleware<E: Endpoint>(next: E, mut req: Request) -> Result<Response> {
        let user = User::from_request(&req, &mut RequestBody::empty()).await?;
        req.set_data(user);
        next.call(req).await
    }
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for User {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        // Check if user is in the request data.
        if let Some(&user) = req.data::<User>() {
            return Ok(user);
        }

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                poem::Error::from_string("missing Authorization header", StatusCode::UNAUTHORIZED)
            })?;
        let token = token.strip_prefix("Bearer ").ok_or_else(|| {
            poem::Error::from_string("invalid Authorization header", StatusCode::UNAUTHORIZED)
        })?;

        let settings = req.data::<AppData>().unwrap().settings.clone();
        let result = decode::<User>(
            token,
            &DecodingKey::from_secret(settings.hs256_secret.as_ref()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|e| {
            poem::Error::from_string(
                format!("failed to decode JWT: {}", e),
                StatusCode::UNAUTHORIZED,
            )
        })?;
        Ok(result.claims)
    }
}
