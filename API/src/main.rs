use std::error::Error;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod app;
mod game;
mod lobby;
use app::{AppError, AppState, LobbyIdx, LobbyRef, SendJson, User};
use lobby::{ClientMsg, ServerMsg};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // routes specific to a lobby
    let lobby_router = Router::new()
        .route("/", get(lobby_state))
        .route("/join", post(join_lobby))
        .route("/chat", get(chat_before))
        .route("/ws", get(ws_handler));

    // api routes
    let api_router = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(current_user))
        .route("/create", post(create_lobby))
        .nest("/lobby/{lobby_code}", lobby_router)
        .with_state(AppState::new());

    // serve built files
    let router = Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("../dist"));

    Ok(router.into())
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

async fn create_lobby(
    State(state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, AppError> {
    let (id, lobby) = state.create_lobby().await;
    lobby.write().await.join(&user)?;
    Ok(Json(json!({ "lobbyCode": id })))
}

async fn lobby_state(LobbyIdx(lobby, idx): LobbyIdx) -> impl IntoResponse {
    let lobby_read = lobby.read().await;
    Json(lobby_read.serialize_idx(lobby_read.serialize(), idx))
}

async fn join_lobby(lobby: LobbyRef, user: User) -> Result<impl IntoResponse, AppError> {
    lobby.write().await.join(&user)
}

#[derive(Deserialize)]
struct ChatBeforeQuery {
    before: Option<u64>,
    limit: Option<usize>,
}
async fn chat_before(
    lobby: LobbyRef,
    Query(ChatBeforeQuery { before, limit }): Query<ChatBeforeQuery>,
) -> impl IntoResponse {
    let lobby = lobby.read().await;
    Json(lobby.chat_before(before, limit.unwrap_or(50).min(250)))
}

async fn ws_handler(ws: WebSocketUpgrade, lobby_idx: LobbyIdx) -> Response {
    ws.on_upgrade(move |socket| async {
        let _ = handle_socket(socket, lobby_idx).await;
    })
}

async fn handle_socket(
    socket: WebSocket,
    LobbyIdx(lobby, idx): LobbyIdx,
) -> Result<(), Box<dyn Error>> {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = lobby.read().await.subscribe();

    // check if user is spectator
    if idx.is_none() {
        while let Ok(msg) = rx.recv().await {
            sender.send_json(msg).await?;
        }
    }
    let idx = idx.unwrap();

    loop {
        tokio::select! {
            // handle client
            next = receiver.next() => {
                match next {
                    Some(Ok(Message::Text(text))) => match serde_json::from_str(text.as_str()) {
                        Ok(ClientMsg::Chat(msg)) => {
                            lobby.write().await.send_msg(idx, msg);
                        }
                        Ok(ClientMsg::Move(hand)) => {
                            if let Err(e) = lobby.write().await.play(idx, hand) {
                                sender.send_json(e).await?;
                            }
                        }
                        Err(e) => {
                            sender.send_json(AppError(e.to_string())).await?;
                        }
                    },
                    _ => break,
                }
            }
            // handle server
            Ok(msg) = rx.recv() => {
                sender
                    .send_json(match msg {
                        ServerMsg::Chat(_) => msg,
                        ServerMsg::State(state) => {
                            ServerMsg::State(lobby.read().await.serialize_idx(state, Some(idx)))
                        }
                    })
                    .await?;
            }
            else => break,
        }
    }
    Ok(())
}
