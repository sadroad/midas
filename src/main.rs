use axum::Router;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum_tws::WebSocket;
use axum_tws::WebSocketUpgrade;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::html;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = Router::new()
        .route("/", get(index))
        .nest_service("/assets", ServeDir::new("assets"));

    if cfg!(debug_assertions) {
        app = app.route("/_reload", get(handle_upgrade));
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade({
        move |socket| async {
            if let Err(e) = handle_ws(socket).await {
                println!("Websocket Error: {:?}", e);
            }
        }
    })
}

async fn handle_ws(mut socket: WebSocket) -> anyhow::Result<()> {
    while let Some(Ok(msg)) = socket.recv().await {
        if msg.is_text() {
            socket.send(msg).await?;
        }
    }
    Ok(())
}

fn header() -> Markup {
    html! {
        (DOCTYPE)
        title { "midas" }
        meta charset="utf-8";
        link href="/assets/output.css" rel="stylesheet";
        @if cfg!(debug_assertions) {
            script {
                (PreEscaped(include_str!("hot_reload.js")))
            }
        }
    }
}

async fn index() -> impl IntoResponse {
    html! {
        (header())
        body class="font-display" {
            h1 { "Hello, World!" }
            p class="text-3xl text-green-600" { "Waht a syntax" }
        }
    }
}
