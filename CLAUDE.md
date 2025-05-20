# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Midas is a web application built with Rust using the Axum web framework and Maud for HTML templating. It includes hot reloading for development and uses TailwindCSS for styling. The project appears to be a product/graphics card availability checker according to the README.

## Architecture

- **Web Server**: Built with Axum, serving HTML content and handling websocket connections
- **HTML Templating**: Uses Maud for type-safe HTML templates
- **CSS**: TailwindCSS for styling, processed during build
- **Hot Reloading**: JavaScript-based hot reloading for development through websockets
- **Logging**: Uses tracing and tracing-subscriber for structured logging
- **Deployment**: Docker image configuration available through Nix flakes

## Development Environment

This project uses Nix flakes for development environment management. Make sure you have Nix with flakes enabled.

## Commands

### Build and Run

```bash
# Enter development shell with all dependencies
nix develop

# Build the project
cargo build

# Run the project (development mode with hot reloading)
cargo run
```

### Development with Hot Reloading

```bash
# Watch for changes and rebuild automatically
cargo watch -x run
```

### TailwindCSS

TailwindCSS is configured in the build.rs script and runs during the build process. It processes `src/main.css` into `assets/output.css`.

### Code Quality

```bash
# Run clippy with strict settings
cargo clippy --all-targets -- --deny warnings

# Format code
cargo fmt
```

### Docker

```bash
# Build docker image
nix build .#docker
```

## Project Structure

- **src/main.rs**: Main application entry point and web server setup
- **src/hot_reload.js**: JavaScript for development hot reloading
- **src/main.css**: TailwindCSS configuration and imports
- **build.rs**: Build script to generate TailwindCSS output
- **assets/**: Static files served by the web server

## Notes

- The development server runs on port 3000
- Hot reloading is only enabled in debug mode
- The application uses HTMX for enhanced interactivity

## Maud Reference

Maud is a compile-time HTML templating engine for Rust used in this project.

### Basic Syntax

```rust
html! {
    h1 { "Title" }
    p { "Paragraph with " strong { "bold text" } "." }
    a href="https://example.com" { "Link" }
    div.container#main { "Class and ID shortcuts" }
}
```

### Variables and Expressions

```rust
let name = "User";
html! {
    p { "Hello, " (name) "!" }
    p { "2 + 2 = " (2 + 2) }
}
```

### Loops and Conditionals

```rust
let items = vec!["one", "two", "three"];
let show_section = true;

html! {
    ul {
        @for item in &items {
            li { (item) }
        }
    }
    
    @if show_section {
        section { "Visible content" }
    } @else {
        p { "Alternative content" }
    }
}
```

### Components and Layouts

```rust
fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" href="/static/css/main.css";
            }
            body {
                header { (navbar("home")) }
                main { (content) }
                footer { "© 2025 My Website" }
            }
        }
    }
}

async fn page_handler() -> Markup {
    layout("Page Title", html! {
        div#content {
            "Page content"
        }
    })
}
```

### HTMX Integration

```rust
html! {
    button 
        type="button"
        hx-post="/api/endpoint"
        hx-target="#target-element"
        hx-swap="innerHTML"
    {
        "Click me"
    }
}
```

## HTMX Reference

HTMX is a lightweight JavaScript library that allows you to access modern browser features directly from HTML using attributes without writing JavaScript.

### Core Concepts

HTMX extends HTML as hypertext by allowing:
- Any element to issue HTTP requests (not just anchors and forms)
- Any event to trigger requests (not just clicks or form submissions)
- Any HTTP verb to be used (GET, POST, PUT, DELETE, etc.)
- Any element to be the target for updates (not just the entire window)

The server typically responds with HTML fragments, not JSON, maintaining the hypertext programming model.

### Key Attributes

Here are the most commonly used HTMX attributes:

| Attribute | Description |
|-----------|-------------|
| `hx-get`, `hx-post`, `hx-put`, `hx-delete` | Specifies the URL to make an AJAX request to and the HTTP method |
| `hx-trigger` | The event that triggers the request (default: click for buttons, submit for forms) |
| `hx-target` | The element to target for the response (CSS selector) |
| `hx-swap` | How to swap the response content (innerHTML, outerHTML, beforeend, afterbegin) |
| `hx-push-url` | Push a new URL into the browser history stack (true, false) |
| `hx-select` | Select a subset of the response to be swapped in |
| `hx-indicator` | Element to show during the request (for loading indicators) |
| `hx-confirm` | Shows a confirmation dialog before making the request |
| `hx-boost` | Enhances regular anchors and forms with AJAX |

### Example Patterns

**Request With Loading Indicator:**
```rust
html! {
    button 
        hx-post="/api/process"
        hx-target="#results"
        hx-indicator="#spinner"
    {
        "Process Data"
    }
    
    div#spinner.htmx-indicator {
        // Spinner content
    }
}
```

**Polling for Updates:**
```rust
html! {
    div 
        hx-get="/api/updates"
        hx-trigger="every 2s"
        hx-swap="innerHTML"
    {
        // Content updated every 2 seconds
    }
}
```

**Form With Validation:**
```rust
html! {
    form 
        hx-post="/api/submit"
        hx-target="#form-result"
        hx-swap="outerHTML"
    {
        // Form fields
        input type="text" name="username" {}
        button { "Submit" }
    }
}
```

### Server-Side Integration with Axum

When processing HTMX requests in Axum handlers:

1. Check for the `HX-Request` header to identify HTMX requests
2. Return HTML fragments instead of full pages for HTMX requests
3. Use appropriate HTTP status codes for validation errors (422 for form validation)

```rust
async fn handle_htmx(
    headers: HeaderMap,
    form: Form<MyForm>,
) -> impl IntoResponse {
    let is_htmx = headers.contains_key("HX-Request");
    
    if is_htmx {
        // Return HTML fragment
        html! {
            div { "Success! Item created." }
        }
    } else {
        // Return full page
        layout("Success", html! {
            div { "Success! Item created." }
        })
    }
}
```

### Event Handling

HTMX triggers various events during its request lifecycle that can be captured with JavaScript:

- `htmx:load` - fired when new content is added to the DOM
- `htmx:beforeRequest` - fired before an AJAX request is made
- `htmx:afterRequest` - fired after an AJAX request completes
- `htmx:responseError` - fired when an error response is received

These events can be used to integrate with other JavaScript or perform additional actions.