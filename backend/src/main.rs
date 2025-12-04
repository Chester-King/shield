mod handlers;
mod middleware;
mod models;
mod utils;
mod zcash;
mod solana;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Json, Router,
};
use handlers::{auth, balance, send, solana_wallet, transactions, user, wallet, AppState};
use middleware::{auth::AuthState, auth_middleware};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::cors::{CorsLayer, Any};
use utils::JwtManager;

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_access_token_expiry: i64 = env::var("JWT_ACCESS_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "900".to_string())
        .parse()
        .expect("JWT_ACCESS_TOKEN_EXPIRY must be a valid number");
    let jwt_refresh_token_expiry: i64 = env::var("JWT_REFRESH_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "604800".to_string())
        .parse()
        .expect("JWT_REFRESH_TOKEN_EXPIRY must be a valid number");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    // Create database connection pool
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // Create JWT manager
    let jwt_manager = Arc::new(JwtManager::new(
        jwt_secret,
        jwt_access_token_expiry,
        jwt_refresh_token_expiry,
    ));

    // Create application state
    let app_state = AppState {
        db: db.clone(),
        jwt_manager: jwt_manager.clone(),
    };

    let auth_state = AuthState {
        jwt_manager: jwt_manager.clone(),
    };

    // Create balance state
    let balance_state = balance::BalanceState {
        db: db.clone(),
    };

    // Create send state
    let send_state = send::SendState {
        db: db.clone(),
    };

    // Create transactions state
    let transactions_state = transactions::TransactionsState {
        db: db.clone(),
    };

    // Build public routes (no auth required)
    let public_routes = Router::new()
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/google", get(auth::google_auth_init))
        .route("/auth/google/callback", get(auth::google_auth_callback))
        .route("/wallet/create", post(wallet::create_wallet))
        .route("/wallet/address", post(wallet::get_address))
        .with_state(app_state.clone());

    // Build balance routes (separate state)
    let balance_routes = Router::new()
        .route("/wallet/balance", post(balance::get_balance))
        .with_state(balance_state);

    // Build send routes (separate state)
    let send_routes = Router::new()
        .route("/wallet/send", post(send::send_transaction))
        .route("/wallet/estimate-fee", post(send::estimate_fee))
        .with_state(send_state);

    // Build transactions routes (separate state)
    let transactions_routes = Router::new()
        .route("/wallet/transactions", post(transactions::get_transactions))
        .with_state(transactions_state);

    // Build Solana routes (protected, require auth)
    let solana_routes = Router::new()
        .route("/solana/balance", post(solana_wallet::get_balance))
        .route("/solana/bridge/quote", post(solana_wallet::get_bridge_quote))
        .route("/solana/bridge/execute", post(solana_wallet::execute_bridge))
        .route("/solana/bridge/status", post(solana_wallet::get_bridge_status))
        .layer(axum_middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(Extension(db.clone()));

    // Build protected routes (auth required)
    let protected_routes = Router::new()
        .route("/users/me", get(user::get_me))
        .layer(axum_middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(Extension(db.clone()));

    // Merge routes
    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(balance_routes)
        .merge(send_routes)
        .merge(transactions_routes)
        .merge(solana_routes);

    // Build main app
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Backend server running on http://{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
