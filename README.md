# VibeTorrent

VibeTorrent is a modern, responsive, and high-performance web interface for **rTorrent**, built entirely with **Rust**. It leverages the power of WebAssembly (WASM) for the frontend and a robust asynchronous backend to provide a seamless torrent management experience.

## 🚀 Tech Stack

This project is built as a Rust workspace containing a backend API, a WASM frontend, and a shared library.

### 🦀 Core
*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2021)
*   **Architecture:** Monorepo / Workspace

### 🖥️ Frontend (WebAssembly)
*   **Framework:** [Leptos](https://leptos.dev/) (v0.6) - A reactive web framework for building performant web apps.
*   **Rendering:** Client-Side Rendering (CSR) via WASM.
*   **Styling:**
    *   [Tailwind CSS](https://tailwindcss.com/) (v4) - Utility-first CSS framework.
    *   [DaisyUI](https://daisyui.com/) (v5) - Component library for Tailwind CSS.
    *   **Themes:** Supports multiple themes (Light, Dark, Dim, Nord, Cupcake, Dracula, Abyss, etc.).
*   **Build Tool:** [Trunk](https://trunkrs.dev/) - WASM web application bundler for Rust.
*   **Networking:** `gloo-net` for HTTP requests and Server-Sent Events (SSE).
*   **PWA:** Progressive Web App support with Manifest and Service Workers.

### ⚙️ Backend (API)
*   **Framework:** [Axum](https://github.com/tokio-rs/axum) (v0.8) - Ergonomic and modular web framework.
*   **Runtime:** [Tokio](https://tokio.rs/) - Asynchronous runtime for Rust.
*   **Protocols:**
    *   **SCGI:** Custom asynchronous SCGI implementation for communicating with rTorrent.
    *   **XML-RPC:** `quick-xml` based parsing/serialization for rTorrent commands.
*   **Real-time:** Server-Sent Events (SSE) for live torrent updates.
*   **Documentation:** [Utoipa](https://github.com/juhaku/utoipa) (Swagger UI) for API documentation.
*   **Middleware:** `tower-http` for CORS, Compression, and Static File Serving.
*   **Logging:** `tracing` for structured logging.

### 📦 Shared
*   **Common Crate:** A shared Rust library containing data models (`Torrent`, `GlobalStats`) and event types used by both backend and frontend to ensure type safety across the entire stack.

## 🛠️ Prerequisites

*   **Rust:** Latest stable version (`rustup update`)
*   **WebAssembly Target:** `rustup target add wasm32-unknown-unknown`
*   **Trunk:** `cargo install trunk`
*   **Node.js & NPM:** (Required for Tailwind CSS CLI)

## 🏃‍♂️ Getting Started

### 1. Backend

Create a `.env` file in the `backend` directory (see `.env.example`) and configure your rTorrent SCGI socket path.

```bash
cd backend
cargo run
```

The backend server will start (default: `http://localhost:3000`).

### 2. Frontend

Open a new terminal for the frontend.

```bash
cd frontend
npm install # Install Tailwind dependencies
trunk serve
```

The application will be available at `http://localhost:8080`.

## 🎨 Features

*   **Real-time Updates:** Live torrent progress, speeds, and status via SSE.
*   **Responsive Design:** Fully mobile-friendly UI with bottom navigation and touch gestures.
*   **Theme Support:** Multiple built-in themes with automatic/manual switching.
*   **Management:** Add, pause, resume, delete torrents.
*   **Filtering & Sorting:** Advanced filtering (All, Downloading, Paused, etc.) and sorting options.
*   **Speed Limits:** Global upload/download speed limit controls.

## 📄 License

[ISC](LICENSE)
