use std::{error::Error, fmt::Display, sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        FromRequestParts, Path,
    },
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use futures_util::{stream::SplitSink, SinkExt};
use moka::future::Cache;
use rand::{distr::Alphabetic, Rng};
use serde::{ser::SerializeStruct, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::lobby::Lobby;
pub type LobbyRef = Arc<RwLock<Lobby>>;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<Cache<Uuid, String>>,
    lobbies: Arc<Cache<String, LobbyRef>>,
}
impl AppState {
    const EXPIRATION: Duration = Duration::from_secs(2 * 24 * 60 * 60);

    pub fn new() -> Self {
        Self {
            users: Arc::new(Cache::builder().time_to_idle(Self::EXPIRATION).build()),
            lobbies: Arc::new(Cache::builder().time_to_idle(Self::EXPIRATION).build()),
        }
    }

    async fn generate_lobby_id(&self) -> String {
        loop {
            let id = rand::rng()
                .sample_iter(&Alphabetic)
                .map(char::from)
                .take(4)
                .collect::<String>()
                .to_uppercase();
            if self.lobbies.get(&id).await.is_none() {
                return id;
            }
        }
    }

    pub async fn create_lobby(&self) -> (String, LobbyRef) {
        let id = self.generate_lobby_id().await;
        let lobby = Arc::new(RwLock::new(Lobby::new()));
        self.lobbies.insert(id.clone(), Arc::clone(&lobby)).await;
        (id, lobby)
    }
}

// wraps error string
#[derive(Debug)]
pub struct AppError(pub String);
impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        Self(err.to_string())
    }
}
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("msg", &self.0)?;
        state.end()
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}
impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for AppError {}

pub struct User {
    pub id: Uuid,
    pub username: String,
}
impl FromRequestParts<AppState> for User {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let id = jar
            .get("session")
            .and_then(|c| Uuid::parse_str(c.value()).ok())
            .ok_or(
                (
                    StatusCode::UNAUTHORIZED,
                    AppError::from("missing session cookie"),
                )
                    .into_response(),
            )?;

        let username = state.users.get(&id).await.ok_or(
            (
                StatusCode::FORBIDDEN,
                jar.remove(Cookie::from("session")),
                AppError::from("invalid session cookie"),
            )
                .into_response(),
        )?;

        Ok(Self { id, username })
    }
}

impl FromRequestParts<AppState> for LobbyRef {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(lobby_code) = Path::<String>::from_request_parts(parts, &state)
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    AppError::from("missing lobby code in path"),
                )
                    .into_response()
            })?;

        let lobby = state.lobbies.get(&lobby_code).await.ok_or(
            (
                StatusCode::NOT_FOUND,
                AppError(format!("lobby {} not found", lobby_code)),
            )
                .into_response(),
        )?;

        Ok(lobby)
    }
}

pub struct LobbyIdx(pub LobbyRef, pub Option<usize>);
impl FromRequestParts<AppState> for LobbyIdx {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let lobby = LobbyRef::from_request_parts(parts, &state).await?;
        let user = User::from_request_parts(parts, &state).await?;
        let idx = lobby.read().await.user_idx(&user);

        Ok(Self(lobby, idx))
    }
}

pub trait SendApp {
    async fn send_json(&mut self, msg: impl Serialize) -> Result<(), Box<dyn Error>>;
    async fn send_result(&mut self, e: Result<(), impl Error>) -> Result<(), Box<dyn Error>>;
}
impl SendApp for SplitSink<WebSocket, Message> {
    async fn send_json(&mut self, msg: impl Serialize) -> Result<(), Box<dyn Error>> {
        let text = serde_json::to_string(&msg)?;
        self.send(Message::Text(text.into())).await?;
        Ok(())
    }
    async fn send_result(&mut self, error: Result<(), impl Error>) -> Result<(), Box<dyn Error>> {
        if let Err(e) = error {
            self.send_json(json!({ "Error": e.to_string() })).await
        } else {
            Ok(())
        }
    }
}
