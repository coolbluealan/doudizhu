use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        FromRef, FromRequestParts, Path, Query, State, WebSocketUpgrade,
    },
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use moka::future::Cache;
use rand::{distr::Alphabetic, Rng};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod game;
mod lobby;
use lobby::Lobby;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let lobby_router = Router::new()
        .route("/chat", get(chat_before))
        .route("/ws", get(ws_handler));

    let api_router = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(current_user))
        .route("/create", post(create_lobby))
        .nest("/lobby/{lobby_code}", lobby_router)
        .with_state(AppState::new());

    let router = Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("../dist"));

    Ok(router.into())
}

struct AppError(String);
impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        Self(err.to_string())
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "msg": self.0
            })),
        )
            .into_response()
    }
}

#[derive(Clone)]
struct AppState {
    users: Arc<Cache<Uuid, String>>,
    lobbies: Arc<Cache<String, Arc<RwLock<Lobby>>>>,
}
impl AppState {
    const EXPIRATION: Duration = Duration::from_secs(2 * 24 * 60 * 60);

    fn new() -> Self {
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

    async fn create_lobby(&self) -> String {
        let id = self.generate_lobby_id().await;
        let lobby = Arc::new(RwLock::new(Lobby::new()));
        self.lobbies.insert(id.clone(), lobby).await;
        id
    }
}

struct User {
    id: Uuid,
    username: String,
}
impl<S> FromRequestParts<S> for User
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let id = CookieJar::from_headers(&parts.headers)
            .get("session")
            .and_then(|c| Uuid::parse_str(c.value()).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let username = state.users.get(&id).await.ok_or(StatusCode::FORBIDDEN)?;
        Ok(User { id, username })
    }
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
}
async fn login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // validate username
    if form.username.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username cannot be empty"));
    }

    let id = Uuid::new_v4();
    state.users.insert(id, form.username).await;

    // set session cookie
    let jar = cookies.add(
        Cookie::build(("session", id.to_string()))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict),
    );
    Ok(jar)
}

async fn logout(State(state): State<AppState>, user: User, jar: CookieJar) -> impl IntoResponse {
    state.users.invalidate(&user.id).await;
    jar.remove(Cookie::from("session"))
}

async fn current_user(user: User) -> impl IntoResponse {
    Json(json!({ "username": user.username }))
}

struct LobbyExtractor(Arc<RwLock<Lobby>>);
impl<S> FromRequestParts<S> for LobbyExtractor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let Path(lobby_code) = Path::<String>::from_request_parts(parts, &state)
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing lobby code in path".to_string(),
                )
            })?;

        let lobby = state.lobbies.get(&lobby_code).await.ok_or((
            StatusCode::NOT_FOUND,
            format!("Lobby {} not found", lobby_code),
        ))?;

        Ok(LobbyExtractor(lobby))
    }
}

async fn create_lobby(State(state): State<AppState>, user: User) -> impl IntoResponse {
    let id = state.create_lobby().await;
    Json(json!({ "lobby_code": id }))
}

#[derive(Deserialize)]
struct ChatBeforeQuery {
    before: u64,
    limit: Option<usize>,
}
async fn chat_before(
    LobbyExtractor(lobby): LobbyExtractor,
    Query(query): Query<ChatBeforeQuery>,
) -> impl IntoResponse {
    let lobby = lobby.read().await;
    Json(lobby.chat_before(query.before, query.limit.unwrap_or(50).min(250)))
}

async fn ws_handler(ws: WebSocketUpgrade, user: User) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(message)) = socket.recv().await {
        if let Message::Text(text) = message {
            println!("Received: {}", text);
            if socket.send(Message::Text("Yay".into())).await.is_err() {
                break;
            }
        }
    }
}
