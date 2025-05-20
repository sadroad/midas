use axum::Router;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::extract::Form;
use axum::extract::State;
use axum_tws::WebSocket;
use axum_tws::WebSocketUpgrade;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::html;
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::signal;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = create_app_state();
    
    let mut app = Router::new()
        .route("/", get(index))
        .route("/login", post(login_handler))
        .route("/dashboard", get(dashboard))
        .route("/add-product", post(add_product))
        .route("/products", get(view_products))
        .route("/clicked", post(clicked))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

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

#[derive(Debug, Clone, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

// Role enum to track user permissions
#[derive(Debug, Clone, PartialEq)]
enum UserRole {
    Regular,
    Admin,
}

// User structure to store user information
#[derive(Debug, Clone)]
struct User {
    username: String,
    role: UserRole,
}

// Check if a username has admin privileges
fn is_admin(username: &str) -> bool {
    // For simplicity, only "admin" username has admin privileges
    username.to_lowercase() == "admin"
}

#[derive(Debug, Clone, Deserialize)]
struct ProductForm {
    url: String,
    name: String,
    retailer: String,
    target_price: Option<String>,
}

#[derive(Debug, Clone)]
struct Product {
    url: String,
    name: String,
    retailer: String,
    target_price: Option<f64>,
    added_by: String,
    created_at: std::time::SystemTime,
}

// List of supported retailers
fn supported_retailers() -> Vec<&'static str> {
    vec!["Best Buy", "Amazon"] // Supported retailers
}

type AppState = Arc<Mutex<Vec<Product>>>;

fn create_app_state() -> AppState {
    Arc::new(Mutex::new(Vec::new()))
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
        // Check if the user has admin privileges
        let is_admin_user = is_admin(&form.username);
        
        // Redirect to dashboard on successful login with username and role as query params
        // In a real app, you would use proper session management (JWT, cookies, etc.)
        let redirect_url = format!("/dashboard?user={}&role={}", 
            form.username, 
            if is_admin_user { "admin" } else { "regular" }
        );
        axum::response::Redirect::to(&redirect_url).into_response()
    } else {
        // Return to login page if validation fails (in a real app, you'd add an error message)
        axum::response::Redirect::to("/").into_response()
    }
}

async fn dashboard(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Get username and role from query params
    let username = params.get("user").cloned().unwrap_or_else(|| "Anonymous".to_string());
    let role = params.get("role").cloned().unwrap_or_else(|| "regular".to_string());
    let is_admin_user = role == "admin";
    
    // Check for error or success messages
    let error_message = params.get("error").map(|e| match e.as_str() {
        "invalid_retailer" => "Invalid retailer. Please select a supported retailer from the dropdown.",
        "invalid_url" => "The URL doesn't match the selected retailer. Please enter a valid product URL.",
        _ => "An error occurred. Please try again."
    });
    
    let success_message = params.get("success").map(|_| "Product successfully added for tracking!");
    
    html! {
        (header())
        body class="font-display" {
            div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" {
                div class="mb-10" {
                    div class="flex justify-between items-center mb-6" {
                        div class="flex items-center" {
                            h1 class="text-3xl font-bold text-gray-900" { "Dashboard" }
                            @if is_admin_user {
                                span class="ml-3 inline-flex items-center rounded-full bg-purple-100 px-2.5 py-0.5 text-xs font-medium text-purple-800" {
                                    "Admin"
                                }
                            }
                        }
                        a href="/" class="text-indigo-600 hover:text-indigo-800" { "Sign Out" }
                    }
                    p class="text-gray-600" { 
                        @if is_admin_user {
                            "Admin dashboard - you can view and manage all user products" 
                        } @else {
                            "Welcome to your Midas Product Tracker dashboard!"
                        }
                    }
                    
                    // Show error message if present
                    @if let Some(message) = error_message {
                        div class="mt-4 p-4 border border-red-300 bg-red-50 text-red-800 rounded-md" {
                            div class="flex" {
                                svg class="h-5 w-5 text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20" {
                                    path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" {}
                                }
                                p { (message) }
                            }
                        }
                    }
                    
                    // Show success message if present
                    @if let Some(message) = success_message {
                        div class="mt-4 p-4 border border-green-300 bg-green-50 text-green-800 rounded-md" {
                            div class="flex" {
                                svg class="h-5 w-5 text-green-400 mr-2" fill="currentColor" viewBox="0 0 20 20" {
                                    path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" {}
                                }
                                p { (message) }
                            }
                        }
                    }
                }
                
                // Product Tracker Section
                div class="bg-white shadow rounded-lg p-6 mb-8" {
                    h2 class="text-2xl font-bold mb-4 text-gray-800" { "Add Product to Track" }
                    p class="mb-6 text-gray-600" { "Submit products you'd like to track for availability and price changes." }
                    
                    div class="mb-6 bg-blue-50 rounded-lg p-4 border border-blue-200" {
                        div class="flex items-center" {
                            svg class="h-5 w-5 text-blue-400 mr-2" fill="currentColor" viewBox="0 0 20 20" {
                                path d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 10-2 0v2H7a1 1 0 100 2h2v2a1 1 0 102 0v-2h2a1 1 0 100-2h-2V7z" clip-rule="evenodd" fill-rule="evenodd" {}
                            }
                            span class="text-blue-800 font-medium" { "Currently Supported Retailers:" }
                        }
                        div class="mt-2 flex flex-wrap gap-2" {
                            @for retailer in supported_retailers() {
                                @let (bg_color, text_color) = match retailer {
                                    "Amazon" => ("bg-orange-100", "text-orange-800"),
                                    "Best Buy" => ("bg-blue-100", "text-blue-800"),
                                    _ => ("bg-gray-100", "text-gray-800"),
                                };
                                span class=(format!("inline-flex items-center rounded-full {} {} px-3 py-1 text-sm font-medium", bg_color, text_color)) {
                                    (retailer)
                                }
                            }
                        }
                    }
                    
                    form class="space-y-4" action=(format!("/add-product?user={}&role={}", username, role)) method="POST" {
                        div {
                            label class="block text-sm font-medium text-gray-700" for="url" { "Product URL" }
                            input id="url" name="url" type="url" required placeholder="https://www.amazon.com/dp/B08FC6MR62 or https://www.bestbuy.com/site/..."
                                class="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
                        }
                        
                        div {
                            label class="block text-sm font-medium text-gray-700" for="name" { "Product Name" }
                            input id="name" name="name" type="text" required placeholder="e.g. PlayStation 5 Digital Edition"
                                class="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
                        }
                        
                        div {
                            label class="block text-sm font-medium text-gray-700" for="retailer" { "Retailer" }
                            select id="retailer" name="retailer" required
                                class="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500" {
                                @for retailer in supported_retailers() {
                                    option value=(retailer) { (retailer) }
                                }
                            }
                        }
                        
                        div {
                            label class="block text-sm font-medium text-gray-700" for="target_price" { "Target Price (Optional)" }
                            div class="mt-1 relative rounded-md shadow-sm" {
                                div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none" {
                                    span class="text-gray-500 sm:text-sm" { "$" }
                                }
                                input id="target_price" name="target_price" type="text" placeholder="399.99"
                                    class="w-full pl-7 pr-12 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500";
                            }
                        }
                        
                        div {
                            button type="submit" 
                                class="w-full px-4 py-2 text-white bg-indigo-600 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" {
                                "Add Product"
                            }
                        }
                    }
                }
                
                // View Products Section
                div class="bg-white shadow rounded-lg p-6" {
                    div class="flex justify-between items-center mb-4" {
                        h2 class="text-2xl font-bold text-gray-800" { "Your Tracked Products" }
                        a href="/products" class="text-indigo-600 hover:text-indigo-800" { "View All Products" }
                    }
                    
                    @let all_products = state.lock().unwrap();
                    
                    // Filter products based on user role - admins see all, regular users see only their own
                    @let visible_products: Vec<_> = if is_admin_user {
                        all_products.iter().collect()
                    } else {
                        all_products.iter().filter(|p| p.added_by == username).collect()
                    };
                    
                    @if visible_products.is_empty() {
                        div class="text-center py-8 text-gray-500" {
                            p { "You haven't added any products to track yet." }
                        }
                    } @else {
                        // Admin badge if applicable
                        @if is_admin_user {
                            div class="mb-4 flex justify-between items-center" {
                                span class="inline-flex items-center rounded-full bg-purple-100 px-2.5 py-0.5 text-xs font-medium text-purple-800" {
                                    svg class="h-3 w-3 mr-1" fill="currentColor" viewBox="0 0 20 20" {
                                        path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" {}
                                    }
                                    "Admin View"
                                }
                                span class="text-xs text-gray-500" { "Showing all user products" }
                            }
                        }
                        
                        // Display the 3 most recent products
                        div class="space-y-4" {
                            @for product in visible_products.iter().rev().take(3) {
                                div class="border rounded-lg p-4 hover:bg-gray-50" {
                                    div class="flex justify-between" {
                                        h3 class="font-semibold text-lg text-gray-800" { (product.name) }
                                        
                                        @if is_admin_user && product.added_by != username {
                                            span class="text-xs bg-gray-100 text-gray-700 px-2 py-1 rounded" {
                                                "Added by: " (product.added_by)
                                            }
                                        }
                                    }
                                    
                                    div class="text-sm text-gray-600 mt-1 overflow-hidden text-ellipsis" {
                                        a href=(product.url) target="_blank" class="text-indigo-600 hover:underline" { "View on " (product.retailer) }
                                    }
                                    @if let Some(price) = product.target_price {
                                        p class="mt-2 text-sm text-gray-700" { "Target Price: $" (format!("{:.2}", price)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn add_product(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Form(form): Form<ProductForm>,
) -> impl IntoResponse {
    // Get username and role from query params
    let username = params.get("user").cloned().unwrap_or_else(|| "Anonymous".to_string());
    let role = params.get("role").cloned().unwrap_or_else(|| "regular".to_string());
    
    // Validate that the URL is from a supported retailer
    let is_valid_retailer = supported_retailers().contains(&form.retailer.as_str());
    
    // Validate that URLs actually come from the corresponding domains
    let is_valid_url = match form.retailer.as_str() {
        "Best Buy" => form.url.to_lowercase().contains("bestbuy.com"),
        "Amazon" => {
            let url = form.url.to_lowercase();
            url.contains("amazon.com") || url.contains("amzn.to") || url.contains("a.co")
        },
        _ => false,
    };
    
    // If validation fails, redirect back to dashboard with error
    if !is_valid_retailer || !is_valid_url {
        // Construct appropriate error message
        let error_msg = if !is_valid_retailer {
            "invalid_retailer"
        } else {
            "invalid_url"
        };
        
        let redirect_url = format!("/dashboard?user={}&role={}&error={}", username, role, error_msg);
        return axum::response::Redirect::to(&redirect_url).into_response();
    }
    
    // Convert target price from string to float if provided
    let target_price = form.target_price
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<f64>().ok());
    
    // Create new product
    let product = Product {
        url: form.url,
        name: form.name,
        retailer: form.retailer,
        target_price,
        added_by: username.clone(),
        created_at: std::time::SystemTime::now(),
    };
    
    // Add to state
    state.lock().unwrap().push(product);
    
    // Redirect back to dashboard
    let redirect_url = format!("/dashboard?user={}&role={}&success=true", username, role);
    axum::response::Redirect::to(&redirect_url).into_response()
}

async fn view_products(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let username = params.get("user").cloned().unwrap_or_else(|| "Anonymous".to_string());
    let role = params.get("role").cloned().unwrap_or_else(|| "regular".to_string());
    let is_admin_user = role == "admin";
    
    html! {
        (header())
        body class="font-display" {
            div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" {
                div class="flex justify-between items-center mb-6" {
                    h1 class="text-3xl font-bold text-gray-900" { 
                        @if is_admin_user {
                            "All User Products"
                        } @else {
                            "Your Tracked Products"
                        }
                    }
                    a href=(format!("/dashboard?user={}&role={}", username, role)) class="text-indigo-600 hover:text-indigo-800" { "Back to Dashboard" }
                }
                
                @if is_admin_user {
                    div class="mb-6 bg-purple-50 p-4 rounded-lg border border-purple-200 flex items-center" {
                        svg class="h-5 w-5 text-purple-600 mr-2" fill="currentColor" viewBox="0 0 20 20" {
                            path fill-rule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" {}
                        }
                        span class="font-medium text-purple-800" { "Admin View: " }
                        span class="ml-1 text-purple-700" { "You can see all user products" }
                    }
                }
                
                div class="bg-white shadow rounded-lg p-6" {
                    @let all_products = state.lock().unwrap();
                    
                    // Filter products based on user role - admins see all, regular users see only their own
                    @let visible_products: Vec<_> = if is_admin_user {
                        all_products.iter().collect()
                    } else {
                        all_products.iter().filter(|p| p.added_by == username).collect()
                    };
                    
                    @if visible_products.is_empty() {
                        div class="text-center py-10 text-gray-500" {
                            p class="text-lg" { "No products found" }
                            p class="mt-2" { "Add your first product on the dashboard" }
                        }
                    } @else {
                        // Admin tools if admin user
                        @if is_admin_user {
                            div class="mb-6 flex justify-between items-center" {
                                div class="text-sm text-gray-500" {
                                    span class="font-medium" { "Total products: " } (visible_products.len())
                                }
                                
                                // In a real app, you'd have filtering options here
                                div class="flex space-x-2 text-sm" {
                                    span class="text-gray-600" { "Filter by:" }
                                    a href="#" class="text-indigo-600 hover:text-indigo-800" { "All" }
                                    a href="#" class="text-gray-600 hover:text-indigo-600" { "Amazon" }
                                    a href="#" class="text-gray-600 hover:text-indigo-600" { "Best Buy" }
                                }
                            }
                        }
                    
                        div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3" {
                            @for product in visible_products.iter().rev() {
                                @let (border_color, bg_hover) = match product.retailer.as_str() {
                                    "Amazon" => ("border-orange-200", "hover:bg-orange-50"),
                                    "Best Buy" => ("border-blue-200", "hover:bg-blue-50"),
                                    _ => ("border-gray-200", "hover:bg-gray-50"),
                                };
                                
                                div class=(format!("border rounded-lg p-6 shadow-sm hover:shadow-md transition-shadow {} {}", border_color, bg_hover)) {
                                    div class="flex justify-between items-start" {
                                        h3 class="font-semibold text-lg text-gray-800" { (product.name) }
                                        
                                        @let (badge_color, badge_text) = match product.retailer.as_str() {
                                            "Amazon" => ("bg-orange-100 text-orange-800", "Amazon"),
                                            "Best Buy" => ("bg-blue-100 text-blue-800", "Best Buy"),
                                            _ => ("bg-gray-100 text-gray-800", product.retailer.as_str()),
                                        };
                                        
                                        span class=(format!("text-xs rounded-full px-2 py-1 {}", badge_color)) {
                                            (badge_text)
                                        }
                                    }
                                    
                                    div class="text-sm text-gray-600 mt-2 truncate" {
                                        a href=(product.url) target="_blank" class="text-indigo-600 hover:underline" { "View product" }
                                    }
                                    
                                    @if let Some(price) = product.target_price {
                                        p class="mt-3 text-sm text-gray-700" { "Target Price: $" (format!("{:.2}", price)) }
                                    }
                                    
                                    @if is_admin_user || product.added_by == username {
                                        div class="mt-4 pt-3 border-t border-gray-100 flex justify-between items-center" {
                                            p class="text-xs text-gray-500" { 
                                                "Added by: " 
                                                span class=(if product.added_by == username { "font-medium text-indigo-600" } else { "text-gray-600" }) {
                                                    (product.added_by)
                                                }
                                            }
                                            
                                            @if is_admin_user {
                                                // Admin actions (in a real app, these would be functional)
                                                div class="flex space-x-1" {
                                                    button type="button" class="text-xs text-gray-600 hover:text-indigo-600" {
                                                        "Edit"
                                                    }
                                                    button type="button" class="text-xs text-gray-600 hover:text-red-600" {
                                                        "Delete"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
