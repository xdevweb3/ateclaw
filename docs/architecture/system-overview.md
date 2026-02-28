# BizClaw — System Architecture Overview

> **Version**: 0.2.0  
> **Last Updated**: 2026-02-28  
> **Status**: Production  
> **Language**: Rust (100%)

---

## 1. Vision

BizClaw is a **Rust-based AI Agent Infrastructure Platform** that enables:
- Multi-tenant AI agent deployment
- Multi-channel communication (Telegram, Zalo, WhatsApp, Email, Discord, Webhook, CLI)
- Local LLM inference (Brain Engine via llama.cpp FFI)
- 15 LLM providers (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Mistral, MiniMax, xAI, ByteDance, Ollama, llama.cpp, Brain, CLIProxy, vLLM, OpenRouter)
- Multi-Agent Orchestrator with agent-to-agent delegation, handoff, teams
- Provider failover chain for automatic fallback
- 13 built-in tools + MCP support (unlimited extensions)
- Lane-based task scheduler (main/cron/subagent/delegate)
- SKILL.md injection for autonomous Hands
- Dual-mode Knowledge RAG: FTS5/BM25 + PageIndex MCP (reasoning-based)
- Session context with auto-compaction
- 51 business agent templates (13 categories)

---

## 2. Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        BizClaw Platform                              │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │            bizclaw-platform (Multi-Tenant Manager)           │   │
│  │  • Per-tenant config (config.toml)                           │   │
│  │  • Tenant CRUD + systemd service management                  │   │
│  │  • JWT authentication + bcrypt for admin panel               │   │
│  │  • Audit log                                                 │   │
│  └─────────────┬────────────────────────────────────────────────┘   │
│                │ spawns per-tenant                                    │
│  ┌─────────────▼────────────────────────────────────────────────┐   │
│  │              bizclaw (Single Tenant Instance)                │   │
│  │                                                              │   │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │   │
│  │  │ Gateway │  │  Agent   │  │ Channels │  │   Memory    │  │   │
│  │  │  (Axum) │  │  Engine  │  │ (9 types)│  │ (SQLite)    │  │   │
│  │  │  38+API │  │ 13 tools │  │ Telegram │  │ FTS5 search │  │   │
│  │  │  routes │  │ MCP      │  │ Zalo P/O │  │ Auto-compact│  │   │
│  │  │  WS     │  │ RAG      │  │ WhatsApp │  │ Brain WS    │  │   │
│  │  └─────────┘  └──────────┘  │ Discord  │  └─────────────┘  │   │
│  │                             │ Email    │                    │   │
│  │  ┌──────────────────────┐   │ Webhook  │  ┌─────────────┐  │   │
│  │  │  Multi-Agent Orch.   │   │ Web CLI  │  │  Scheduler  │  │   │
│  │  │  • Named agents      │   └──────────┘  │  Cron/Once  │  │   │
│  │  │  • Delegation        │                  │  Interval   │  │   │
│  │  │  • Broadcast         │  ┌──────────┐   │  Retry+EBO  │  │   │
│  │  │  • Telegram Bot ↔    │  │ Knowledge│   └─────────────┘  │   │
│  │  │    Agent mapping     │  │ RAG+FTS5 │                    │   │
│  │  └──────────────────────┘  │ PageIndex │                   │   │
│  │                            └──────────┘                    │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │              Providers Layer (15 built-in)            │  │   │
│  │  │  OpenAI │ Anthropic │ Gemini │ DeepSeek │ Groq       │  │   │
│  │  │  Mistral │ MiniMax │ xAI (Grok) │ ByteDance ModelArk │  │   │
│  │  │  Ollama │ Brain Engine │ llama.cpp │ CLIProxy │ vLLM  │  │   │
│  │  └──────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                     Dashboard (SPA)                           │   │
│  │  18 pages │ i18n VI/EN │ Dark theme │ Path-based routing     │   │
│  │  Pairing code auth │ WebSocket real-time │ Responsive        │   │
│  │  LLM Traces │ Cost Tracking │ Activity Feed                  │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 3. Crate Architecture (16 crates)

| Crate | LOC | Purpose |
|-------|-----|---------|
| `bizclaw-core` | ~1,964 | Config, error types, traits (Channel, Provider, Identity) |
| `bizclaw-agent` | ~2,182 | Agent engine, Think-Act-Observe loop, MCP, Orchestrator |
| `bizclaw-providers` | ~1,054 | 15 LLM providers (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Mistral, MiniMax, xAI, ByteDance, Ollama, etc.) |
| `bizclaw-channels` | ~4,980 | 9 channel types (Telegram, Zalo P/OA, WhatsApp, Discord, Email, Webhook, CLI) |
| `bizclaw-tools` | ~4,949 | 13 built-in tools (plan_mode, execute_code, etc.) |
| `bizclaw-memory` | ~1,008 | SQLite FTS5, Brain workspace, auto-compaction |
| `bizclaw-brain` | ~3,273 | GGUF inference, mmap, SIMD (ARM NEON, x86 SSE2/AVX2) |
| `bizclaw-mcp` | ~556 | Model Context Protocol client (JSON-RPC 2.0 via stdio) |
| `bizclaw-gateway` | ~6,378 | Axum HTTP server, 38+ API routes, WebSocket, 18-page Dashboard |
| `bizclaw-scheduler` | ~2,504 | Cron/interval/once task scheduling, retry with exponential backoff |
| `bizclaw-knowledge` | ~408 | RAG knowledge store with FTS5 document chunking |
| `bizclaw-security` | ~646 | AES-256, command allowlist, input sanitization, path validation |
| `bizclaw-runtime` | ~93 | Runtime abstraction (native/Docker) |
| `bizclaw-platform` | ~3,700 | Multi-tenant manager, JWT auth, admin panel, audit log |
| `bizclaw-db` | ~1,945 | Gateway database layer (SQLite per-tenant) |
| `bizclaw-hands` | ~1,117 | Agent hands — external action execution (WIP) |

**Total**: ~36,757 LOC Rust + ~3,000 LOC HTML/JS/CSS

---

## 4. API Surface (38 Routes)

### Core
- `GET /` — Dashboard SPA
- `GET /health` — Health check
- `POST /api/v1/verify-pairing` — Pairing auth

### Config
- `GET /api/v1/info` — System info  
- `GET /api/v1/config` — Get config
- `POST /api/v1/config/update` — Update config
- `GET /api/v1/config/full` — Full config

### Providers  
- `GET /api/v1/providers` — List providers
- `GET /api/v1/ollama/models` — List Ollama models
- `GET /api/v1/brain/models` — Scan GGUF models

### Channels
- `GET /api/v1/channels` — List channels
- `POST /api/v1/channels/update` — Update channel

### WebSocket
- `GET /ws` — Real-time chat (with pairing code query param)

### Multi-Agent
- `GET /api/v1/agents` — List agents
- `POST /api/v1/agents` — Create agent
- `PUT /api/v1/agents/{name}` — Update agent
- `DELETE /api/v1/agents/{name}` — Delete agent
- `POST /api/v1/agents/{name}/chat` — Chat with agent
- `POST /api/v1/agents/broadcast` — Broadcast to all

### Telegram Bot ↔ Agent
- `POST /api/v1/agents/{name}/telegram` — Connect bot
- `DELETE /api/v1/agents/{name}/telegram` — Disconnect bot
- `GET /api/v1/agents/{name}/telegram` — Bot status

### Knowledge Base
- `POST /api/v1/knowledge/search` — Search RAG
- `GET /api/v1/knowledge/documents` — List docs
- `POST /api/v1/knowledge/documents` — Add doc
- `DELETE /api/v1/knowledge/documents/{id}` — Remove doc

### Brain Workspace
- `GET /api/v1/brain/files` — List brain files
- `GET /api/v1/brain/files/{filename}` — Read file
- `PUT /api/v1/brain/files/{filename}` — Write file
- `DELETE /api/v1/brain/files/{filename}` — Delete file
- `POST /api/v1/brain/personalize` — AI personalization

### Scheduler
- `GET /api/v1/scheduler/tasks` — List tasks
- `POST /api/v1/scheduler/tasks` — Add task
- `DELETE /api/v1/scheduler/tasks/{id}` — Remove task
- `GET /api/v1/scheduler/notifications` — Notification history

### Health
- `GET /api/v1/health` — System health check

### Webhooks
- `GET /api/v1/webhook/whatsapp` — Meta verification
- `POST /api/v1/webhook/whatsapp` — WhatsApp messages
- `POST /api/v1/zalo/qr` — Zalo QR login

---

## 5. Dashboard (18 Pages)

| Page | Path | Purpose |
|------|------|---------|
| Dashboard | `/` | System stats, health overview |
| WebChat | `/chat` | Agent chat with WebSocket streaming, agent selector |
| Settings | `/settings` | Provider, model, identity config |
| Providers | `/providers` | Card-based provider config (15 providers) |
| Channels | `/channels` | Channel management (9 types) |
| Tools | `/tools` | 13 built-in tools display |
| Brain Engine | `/brain` | Local LLM, brain workspace, health check |
| MCP Servers | `/mcp` | Model Context Protocol servers |
| Multi-Agent | `/agents` | Create/edit/delete agents, Telegram bot |
| Orchestration | `/orchestration` | Multi-agent delegation, broadcast |
| Groups | `/groups` | Agent group chat |
| Gallery | `/gallery` | 51 agent templates (13 categories) |
| Knowledge | `/knowledge` | RAG document management |
| Scheduler | `/scheduler` | Task scheduling management |
| LLM Traces | `/traces` | LLM request/response tracing |
| Cost Tracking | `/costs` | API cost tracking per agent/provider |
| Activity Feed | `/activity` | Real-time system activity log |
| Config File | `/configfile` | Raw config.toml editor |

---

## 6. Security Architecture

| Layer | Protection |
|-------|-----------|
| **Auth** | Pairing code (session-based) |
| **API** | X-Pairing-Code header middleware |
| **WebSocket** | Query param `?code=` validation |
| **CORS** | Configurable via `BIZCLAW_CORS_ORIGINS` |
| **Input** | `bizclaw-security` crate (path traversal, sanitization) |
| **Secrets** | API keys in config.toml, not in URLs |
| **Encryption** | AES-256 for sensitive data, HMAC-SHA256 for integrity |
| **Auth Hashing** | bcrypt for password storage |
| **Rate Limiting** | Per-tenant rate limiting |
| **Error Sanitization** | Zero information disclosure in error responses |
| **Multi-Tenant** | JWT for platform admin, isolated tenant configs |
| **Security Score** | 88/100 (audited 2026-02-25) |

---

## 7. Deployment

| Target | Method |
|--------|--------|
| **VPS** | Direct binary + systemd (`bizclaw-platform.service`) |
| **Docker** | `docker-compose.yml` with multi-stage build |
| **One-Click** | `curl -sSL https://bizclaw.vn/install.sh \| bash` |
| **Production** | bizclaw.vn (116.118.2.98), Nginx reverse proxy, subdomain routing |

### Binary Sizes
- `bizclaw`: ~12 MB (release, LTO + strip)
- `bizclaw-platform`: ~7.7 MB (release, LTO + strip)

---

## 8. Current Tenants

| Tenant | Subdomain | Port |
|--------|-----------|------|
| platform | apps.bizclaw.vn | 3001 |
| demo | demo.bizclaw.vn | 10001 |
| sales | sales.bizclaw.vn | 10002 |
| dev | internal | 10004 |

---

## 9. Tech Stack Summary

| Category | Technology |
|----------|-----------|
| Language | Rust 2024 edition |
| Web Framework | Axum 0.8 |
| Database | SQLite (FTS5, per-tenant) |
| LLM Inference | llama.cpp (C FFI), GGUF, SIMD |
| Frontend | Vanilla HTML/JS/CSS (SPA, 18 pages) |
| RAG | FTS5/BM25 + PageIndex MCP (reasoning-based) |
| Deployment | systemd + Nginx + deploy.sh |
| Container | Docker multi-stage |
| CI/CD | GitHub Actions (future) |
| Monitoring | `tracing` crate + systemd journal |
| Cross-compile | cargo-zigbuild (Mac → Linux x86_64) |
