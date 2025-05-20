# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Midas is a web application built with Rust using the Axum web framework and Maud for HTML templating. It includes hot reloading for development and uses TailwindCSS for styling. The project appears to be a product/graphics card availability checker according to the README.

## Architecture

- **Web Server**: Built with Axum, serving HTML content and handling websocket connections
- **HTML Templating**: Uses Maud for type-safe HTML templates
- **CSS**: TailwindCSS for styling, processed during build
- **Hot Reloading**: JavaScript-based hot reloading for development through websockets
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