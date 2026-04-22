# Ragent — Rust AI Agent Platform

> **R**ust + **Agent** — 一键式 AI 员工对话平台

[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Architecture

```
┌──────────────┐     ┌──────────────┐
│  Dioxus UI   │────▶│  axum HTTP   │
│  (Frontend)  │◀────│  + WebSocket │
└──────────────┘     └──────┬───────┘
                            │
                    ┌───────┴────────┐
                    │   app-core     │
                    │ (Config/DB/Auth)│
                    └───────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
      ┌───────┴──────┐ ┌───┴────┐ ┌─────┴─────┐
      │  PostgreSQL  │ │ Redis  │ │  Ollama   │
      │  (pgvector)  │ │ (Cache)│ │  (LLM)    │
      └──────────────┘ └────────┘ └───────────┘
                            │
                    ┌───────┴────────┐
                    │   app-agent    │
                    │ (Tools: Calc/  │
                    │  Search/Sandbox│
                    └────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Dioxus 0.7 + Tailwind CSS |
| Backend | axum 0.8 + tokio |
| Database | PostgreSQL + sqlx (migrations) |
| Cache | Redis |
| Auth | JWT + Argon2 |
| AI | Ollama (local LLM) |
| Agent | Custom tool registry |
| Realtime | WebSocket (axum ws) |

## Getting Started

### Prerequisites

- Rust 1.85+
- PostgreSQL 15+
- Redis 7+
- Ollama (optional, for AI features)

### 1. Clone & Setup

```bash
git clone https://github.com/TeamMeng/Ragent.git
cd Ragent
cp .env.example .env
# Edit .env with your database/Redis/JWT settings
```

### 2. Database

```bash
createdb ragent
```

Migrations run automatically on first startup.

### 3. Run Server

```bash
cargo run -p app-api
```

Server starts at `http://127.0.0.1:8000`

### 4. (Optional) Start Ollama

```bash
ollama pull llama3
ollama serve
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/register` | Register new user |
| POST | `/api/auth/login` | Login, get JWT tokens |
| POST | `/api/auth/refresh` | Refresh access token |
| POST | `/api/sessions` | Create chat session |
| GET | `/api/sessions` | List sessions |
| POST | `/api/sessions/:id/messages` | Send message |
| GET | `/api/sessions/:id/messages` | List messages |
| GET | `/ws` | WebSocket connection |

## Project Structure

```
Ragent/
├── crates/
│   ├── app-core/      # Shared config, DB, auth, models
│   ├── app-proto/     # Shared types (WebSocket events)
│   ├── app-agent/     # AI agent + tools (calc/search/sandbox)
│   └── app-api/       # axum HTTP/WebSocket server
├── config/            # Configuration files
├── migrations/        # SQL migrations
├── Cargo.toml         # Workspace root
└── README.md
```

## Roadmap

- [x] **Phase 1** — MVP single-chat with 3 tools (v0.1.0)
- [ ] **Phase 2** — Multi-agent closed-loop group chat (v0.2.0)
- [ ] **Phase 3** — RAG knowledge base with pgvector (v0.3.0)
- [ ] **Phase 4** — Production hardening (v0.4.0)
- [ ] **Phase 5** — UX polish & GA release (v1.0.0)

## License

MIT
