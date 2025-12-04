use crate::{middleware::AppError, utils::JwtManager};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing or invalid authorization header".to_string()))?;

    let claims = state.jwt_manager.verify_token(token)?;

    // Check if it's an access token
    if claims.token_type != crate::utils::TokenType::Access {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    // Add user_id to request extensions
    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}
