use axum::Router;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::extract::Form;
use axum_tws::WebSocket;
use axum_tws::WebSocketUpgrade;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::html;
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::signal;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = Router::new()
        .route("/", get(index))
        .route("/login", post(login_handler))
        .route("/dashboard", get(dashboard))
        .route("/clicked", post(clicked))
        .nest_service("/assets", ServeDir::new("assets"));

    if cfg!(debug_assertions) {
        app = app.route("/_reload", get(handle_upgrade));
    }

    // Get port from environment variable, or use 3000 as default
    let port = env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(3000);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server started at http://0.0.0.0:{}", port);
    
    // Set up graceful shutdown
    let server = axum::serve(listener, app);
    
    // Handle both SIGINT and SIGTERM
    server.with_graceful_shutdown(shutdown_signal()).await?;
    
    println!("Server shutdown complete");
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
        script src="https://unpkg.com/htmx.org@2.0.4" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" crossorigin="anonymous" {}
        link href="/assets/output.css" rel="stylesheet";
        @if cfg!(debug_assertions) {
            script {
                (PreEscaped(include_str!("hot_reload.js")))
            }
        }
    }
}

async fn clicked() -> Markup {
    html! {
        p {
            "wowowowo"
        }
    }
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn index() -> impl IntoResponse {
    html! {
        (header())
        body class="font-display flex items-center justify-center min-h-screen bg-gray-100" {
            div class="w-full max-w-md p-8 space-y-8 bg-white rounded-lg shadow-md" {
                div class="text-center" {
                    h1 class="text-3xl font-bold text-gray-900" { "Midas" }
                    p class="mt-2 text-gray-600" { "Please sign in to your account" }
                }
                
                form class="mt-8 space-y-6" action="/login" method="POST" {
                    div class="space-y-4" {
                        div {
                            label class="block text-sm font-medium text-gray-700" for="username" { "Username" }
                            input id="username" name="username" type="text" required
                                class="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
                        }
                        
                        div {
                            label class="block text-sm font-medium text-gray-700" for="password" { "Password" }
                            input id="password" name="password" type="password" required
                                class="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
                        }
                    }
                    
                    div {
                        button type="submit" 
                            class="w-full px-4 py-2 text-white bg-indigo-600 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}

async fn login_handler(Form(form): Form<LoginForm>) -> impl IntoResponse {
    // In a real app, you would validate the credentials here
    // For demo purposes, we'll just redirect to the dashboard
    if !form.username.is_empty() && !form.password.is_empty() {
        // Redirect to dashboard on successful login
        axum::response::Redirect::to("/dashboard").into_response()
    } else {
        // Return to login page if validation fails (in a real app, you'd add an error message)
        axum::response::Redirect::to("/").into_response()
    }
}

async fn dashboard() -> impl IntoResponse {
    html! {
        (header())
        body class="font-display" {
            div class="p-8" {
                h1 class="text-3xl font-bold mb-4" { "Dashboard" }
                p class="mb-6" { "Welcome to your account dashboard!" }
                
                div class="flex space-x-4" {
                    a href="/" class="px-4 py-2 bg-gray-200 rounded-md hover:bg-gray-300" { "Back to Login" }
                    button hx-post="/clicked" hx-swap="outerHTML" class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700" {
                        "Click Me"
                    }
                }
            }
        }
    }
}

/// Handle Ctrl+C (SIGINT) and SIGTERM signals for graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("Received Ctrl+C, initiating graceful shutdown");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
        println!("Received SIGTERM, initiating graceful shutdown");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either signal
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
