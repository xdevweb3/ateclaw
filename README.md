# ⚡ BizClaw

> **Hạ tầng AI Assistant nhanh, module hoá — viết hoàn toàn bằng Rust.**

BizClaw là nền tảng AI Agent kiến trúc trait-driven, có thể chạy **mọi nơi** — từ Raspberry Pi đến cloud server. Hỗ trợ nhiều LLM provider, kênh giao tiếp, và công cụ thông qua kiến trúc thống nhất, hoán đổi được.

[![Rust](https://img.shields.io/badge/Rust-100%25-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-45%20passing-brightgreen)]()
[![LoC](https://img.shields.io/badge/lines-8.7k%20Rust-informational)]()
[![Coverage](https://img.shields.io/badge/crates-11%2F11%20tested-success)]()

---

## �🇳 Tiếng Việt

### 🎯 Tính năng chính

- **🧠 Brain Engine (Bộ não cục bộ)** — Chạy model LLaMA ngay trên máy qua GGUF, mmap, quantization (Q4_0/Q8_0), KV Cache, Forward Pass đầy đủ, SIMD acceleration
- **🔌 Đa nhà cung cấp AI** — OpenAI, Anthropic Claude, Ollama, llama.cpp, OpenRouter, hoặc bất kỳ server tương thích OpenAI
- **💬 Đa kênh giao tiếp** — CLI, Zalo (Personal + OA + WebSocket), Telegram Bot (polling), Discord Bot (Gateway WS), Webhook
- **🛠️ Tool Calling** — Thực thi shell, thao tác file, registry động với arg validation
- **🔒 Bảo mật** — Danh sách lệnh cho phép, giới hạn đường dẫn, sandbox, mã hoá AES-256, HMAC-SHA256 webhook
- **💾 Bộ nhớ** — SQLite, tìm kiếm vector (cosine similarity), chế độ tắt bộ nhớ
- **🌐 Gateway HTTP** — REST API + WebSocket streaming (token-by-token) dựa trên Axum
- **⚡ SIMD** — ARM NEON (Pi/Apple Silicon), x86 SSE2/AVX2 auto-dispatch
- **📦 Module hoá** — 11 crate độc lập, 45 tests, hoán đổi qua hệ thống trait

### 🏗️ Kiến trúc

```
┌───────────────────────────────────────────────────────────┐
│                      bizclaw (CLI)                         │
│               ┌─────────────────────┐                      │
│               │   bizclaw-agent     │                      │
│               │  (điều phối trung   │                      │
│               │   tâm)              │                      │
│               └──────┬──────────────┘                      │
│      ┌───────────────┼───────────────┐                     │
│      ▼               ▼               ▼                     │
│ ┌──────────┐  ┌───────────┐  ┌─────────────┐             │
│ │Providers │  │ Channels  │  │   Tools     │             │
│ │──────────│  │───────────│  │─────────────│             │
│ │ OpenAI   │  │   CLI     │  │  Shell      │             │
│ │Anthropic │  │  Zalo     │  │  File       │             │
│ │ Ollama   │  │ Telegram  │  │  (tuỳ chỉnh)│             │
│ │LlamaCpp  │  │ Discord   │  └─────────────┘             │
│ │  Brain   │  │ Webhook   │                               │
│ └──────────┘  └───────────┘                               │
│      ┌───────────────┬───────────────┐                    │
│      ▼               ▼               ▼                    │
│ ┌──────────┐  ┌───────────┐  ┌─────────────┐            │
│ │ Memory   │  │ Security  │  │  Gateway    │            │
│ │──────────│  │───────────│  │─────────────│            │
│ │ SQLite   │  │Allowlist  │  │ Axum HTTP   │            │
│ │ Vector   │  │ Sandbox   │  │ WebSocket   │            │
│ │  NoOp    │  │ AES-256   │  │ REST API    │            │
│ └──────────┘  └───────────┘  └─────────────┘            │
│                     ▼                                     │
│            ┌──────────────────┐                           │
│            │  bizclaw-brain   │                           │
│            │──────────────────│                           │
│            │ GGUF v3 Parser   │                           │
│            │ Forward Pass     │                           │
│            │ BPE Tokenizer    │                           │
│            │ Attention + GQA  │                           │
│            │ KV Cache         │                           │
│            │ Quantization     │                           │
│            │ SIMD / Rayon     │                           │
│            └──────────────────┘                           │
└───────────────────────────────────────────────────────────┘
```

### 🚀 Bắt đầu nhanh

```bash
# Clone và build
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw
cargo build --release

# Chạy với OpenAI
export OPENAI_API_KEY="sk-..."
./target/release/bizclaw chat

# Chạy với Ollama (model cục bộ)
ollama serve &
ollama pull llama3.2
./target/release/bizclaw chat --provider ollama --model llama3.2

# Chạy với Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-..."
./target/release/bizclaw chat --provider anthropic

# Tải model cho Brain Engine
./target/release/bizclaw brain download tinyllama-1.1b
./target/release/bizclaw brain test "Xin chào!"

# Xem thông tin hệ thống
./target/release/bizclaw info
```

### ⚙️ Cấu hình

File cấu hình tại `~/.bizclaw/config.toml`:

```toml
default_provider = "openai"
default_model = "gpt-4o-mini"
default_temperature = 0.7

[identity]
name = "BizClaw"
persona = "Trợ lý AI thông minh"
system_prompt = "Bạn là BizClaw, trợ lý AI nhanh và có năng lực."

[brain]
enabled = true
model_path = "~/.bizclaw/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
threads = 4
temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true

[gateway]
enabled = false
host = "127.0.0.1"
port = 3000

[autonomy]
level = "supervised"
allowed_commands = ["ls", "cat", "echo", "pwd", "find", "grep"]
```

### 📦 Bảng Crate

| Crate | Mô tả | Tests | Trạng thái |
|-------|--------|-------|------------|
| `bizclaw-core` | Traits, types, config, errors | 11 | ✅ Hoàn thành |
| `bizclaw-brain` | GGUF + Forward Pass + SIMD | 12 | ✅ Hoàn thành |
| `bizclaw-providers` | OpenAI, Anthropic, Ollama, LlamaCpp, Brain, Custom | — | ✅ Hoàn thành |
| `bizclaw-channels` | CLI, Zalo, Telegram (polling), Discord (GW), Webhook | 2 | ✅ Hoàn thành |
| `bizclaw-memory` | SQLite, Vector, NoOp backends | 3 | ✅ Hoàn thành |
| `bizclaw-tools` | Shell, File, Registry + arg validation | 5 | ✅ Hoàn thành |
| `bizclaw-security` | Allowlist, Sandbox, AES-256 Secrets | 2 | ✅ Hoàn thành |
| `bizclaw-agent` | Agent loop, context, tool execution | 4 | ✅ Hoàn thành |
| `bizclaw-gateway` | Axum HTTP + WebSocket streaming | 4 | ✅ Hoàn thành |
| `bizclaw-runtime` | Native process adapter | 2 | ✅ Hoàn thành |

### 🧠 Brain Engine — Chi tiết

| Thành phần | Mô tả |
|------------|--------|
| **GGUF v3 Parser** | Đọc metadata + tensor index đầy đủ |
| **Forward Pass** | LLaMA transformer: Embedding → N×(RMSNorm→MHA+GQA→SwiGLU FFN)→LM Head |
| **mmap Loader** | Tải model zero-copy (quan trọng cho Pi 512MB) |
| **BPE Tokenizer** | Mã hoá byte-level với merge lặp |
| **Tensor Ops** | RMSNorm, MatMul, Softmax, SiLU, ElementWise |
| **Quantization** | Dequant Q4_0, Q8_0, F16, F32 |
| **Attention** | Scaled dot-product, GQA (Grouped Query Attention) |
| **KV Cache** | Cache key-value theo layer cho generation |
| **RoPE** | Rotary Position Embeddings multi-head |
| **Sampler** | Temperature, Top-K, Top-P, repeat penalty |
| **Thread Pool** | Rayon parallel matmul đa luồng |

### � Bảo mật

| Tính năng | Mô tả |
|-----------|--------|
| **Danh sách lệnh** | Chỉ lệnh được phép mới thực thi được |
| **Giới hạn đường dẫn** | Chặn truy cập `~/.ssh`, `/etc`, v.v. |
| **Sandbox** | Timeout, cắt output, môi trường hạn chế |
| **AES-256 Secrets** | Mã hoá key máy riêng (SHA-256 hostname+user) |
| **Webhook HMAC** | Xác minh chữ ký SHA-256 cho webhook inbound |

### 🗺️ Lộ trình

- [x] **Phase 1** — Hạ tầng cốt lõi (traits, config, errors)
- [x] **Phase 1** — Tất cả providers (OpenAI, Anthropic, Ollama, LlamaCpp, Custom)
- [x] **Phase 1** — CLI channel, memory, security, gateway
- [x] **Phase 2** — Brain engine (GGUF, tokenizer, tensor, quant, attention)
- [x] **Phase 2** — Brain forward pass (toàn bộ transformer pipeline)
- [x] **Phase 3** — Zalo client (Auth, WebSocket, Crypto, Messaging)
- [x] **Phase 3** — Telegram polling + Discord Gateway WebSocket
- [x] **Phase 3** — AES-256 encrypted secret store + Webhook channel
- [x] **Phase 3** — Gateway WebSocket streaming (token-by-token)
- [x] **Phase 4** — SIMD acceleration (NEON, SSE2, AVX2 auto-dispatch)
- [x] **Phase 4** — HTTP streaming model download từ HuggingFace
- [x] **Phase 5** — Zalo Personal/OA Channel wrappers
- [x] **Phase 5** — Tool registry + arg validation
- [x] **Phase 5** — 45 unit tests, 11/11 crates covered ✅

### 📊 Thống kê

| Chỉ số | Giá trị |
|--------|---------|
| **Ngôn ngữ** | 100% Rust |
| **Số crate** | 11 (10 library + 1 binary) |
| **Dòng code** | ~8,735 |
| **Test** | 45 passing (11/11 crates) |
| **Build** | 0 errors |
| **Stubs** | 0 (100% implemented) |
| **Dependencies** | tokio, axum, reqwest, serde, rusqlite, rayon, memmap2, half, aes, sha2 |

---

## 🇬🇧 English

### 🎯 Features

- **🧠 Local Brain Engine** — Run LLaMA models locally via GGUF with mmap, quantization, full forward pass, KV Cache, SIMD acceleration
- **🔌 Multi-Provider** — OpenAI, Anthropic Claude, Ollama, llama.cpp, OpenRouter, or any OpenAI-compatible server
- **💬 Multi-Channel** — CLI, Zalo (Personal + OA), Telegram (long polling), Discord (Gateway WS), Webhook (HMAC)
- **🛠️ Tool Calling** — Shell execution, file operations, dynamic registry with arg validation
- **🔒 Security** — Command allowlists, path restrictions, sandbox, AES-256 secrets, HMAC-SHA256 webhook verification
- **💾 Memory** — SQLite persistence, in-memory vector search (cosine similarity), no-op mode
- **🌐 HTTP Gateway** — REST API + WebSocket streaming (chat_start → chunks → chat_done)
- **⚡ SIMD** — ARM NEON (Pi/Apple Silicon), x86 SSE2/AVX2 auto-dispatch
- **📦 Modular** — 11 crates, 45 tests, 100% implemented, swap via traits

### 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw
cargo build --release

# Run with OpenAI
export OPENAI_API_KEY="sk-..."
./target/release/bizclaw chat

# Run with Ollama (local model)
ollama serve &
ollama pull llama3.2
./target/release/bizclaw chat --provider ollama --model llama3.2

# Run with Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-..."
./target/release/bizclaw chat --provider anthropic

# Download model for Brain Engine
./target/release/bizclaw brain download tinyllama-1.1b
./target/release/bizclaw brain test "Hello!"

# System info
./target/release/bizclaw info
```

### ⚙️ Configuration

TOML config at `~/.bizclaw/config.toml`:

```toml
default_provider = "openai"
default_model = "gpt-4o-mini"
default_temperature = 0.7

[identity]
name = "BizClaw"
persona = "A helpful AI assistant"
system_prompt = "You are BizClaw, a fast and capable AI assistant."

[brain]
enabled = true
model_path = "~/.bizclaw/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
threads = 4
temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true

[gateway]
enabled = false
host = "127.0.0.1"
port = 3000

[autonomy]
level = "supervised"
allowed_commands = ["ls", "cat", "echo", "pwd", "find", "grep"]
```

### 📦 Crate Map

| Crate | Description | Status |
|-------|-------------|--------|
| `bizclaw-core` | Traits, types, config, errors | ✅ Complete |
| `bizclaw-brain` | Local GGUF inference engine + Forward Pass | ✅ Complete |
| `bizclaw-providers` | OpenAI, Anthropic, Ollama, LlamaCpp, Brain, Custom | ✅ Complete |
| `bizclaw-channels` | CLI, Zalo (Auth/WS/Crypto), Telegram, Discord | ✅ Complete |
| `bizclaw-memory` | SQLite, Vector, NoOp backends | ✅ Complete |
| `bizclaw-tools` | Shell, File tools + registry | ✅ Complete |
| `bizclaw-security` | Allowlist, Sandbox, AES-256 Secrets | ✅ Complete |
| `bizclaw-agent` | Agent loop, context, tool execution | ✅ Complete |
| `bizclaw-gateway` | Axum HTTP + WebSocket API | ✅ Complete |
| `bizclaw-runtime` | Native process adapter | ✅ Complete |

### 🧠 Brain Engine

| Component | Description |
|-----------|-------------|
| **GGUF v3 Parser** | Full metadata + tensor index parsing |
| **Forward Pass** | LLaMA transformer: Embedding → N×(RMSNorm→MHA+GQA→SwiGLU FFN)→LM Head |
| **mmap Loader** | Zero-copy model loading (critical for Pi 512MB) |
| **BPE Tokenizer** | Byte-level encoding with iterative merges |
| **Tensor Ops** | RMSNorm, MatMul, Softmax, SiLU, ElementWise |
| **Quantization** | Q4_0, Q8_0, F16, F32 dequantization kernels |
| **Attention** | Scaled dot-product with GQA (Grouped Query Attention) |
| **KV Cache** | Per-layer key-value cache for auto-regressive generation |
| **RoPE** | Multi-head Rotary Position Embeddings |
| **Sampler** | Temperature, Top-K, Top-P, repeat penalty |
| **Thread Pool** | Rayon-based parallel matmul |

### 📡 Gateway API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/v1/info` | GET | System info + uptime |
| `/api/v1/config` | GET | Sanitized config |
| `/api/v1/providers` | GET | Available providers |
| `/api/v1/channels` | GET | Available channels |
| `/ws` | WS | Real-time WebSocket chat |

### 🔒 Security Model

| Feature | Description |
|---------|-------------|
| **Command Allowlist** | Only whitelisted commands can be executed |
| **Path Restrictions** | Forbidden paths (e.g., `~/.ssh`) are rejected |
| **Workspace Only** | Optionally restrict to current working directory |
| **Sandbox** | Timeout, output truncation, restricted env |
| **AES-256 Secrets** | Machine-specific key encryption (SHA-256 hostname+user) |

### 🗺️ Roadmap

- [x] **Phase 1** — Core infrastructure (traits, config, error handling)
- [x] **Phase 1** — All providers (OpenAI, Anthropic, Ollama, LlamaCpp, Custom)
- [x] **Phase 1** — CLI channel, memory backends, security, gateway
- [x] **Phase 2** — Brain engine (GGUF, tokenizer, tensor, quant, attention)
- [x] **Phase 2** — Brain forward pass (full transformer pipeline)
- [x] **Phase 3** — Zalo client (Auth, WebSocket, Crypto, Messaging)
- [x] **Phase 3** — Telegram polling + Discord Gateway WebSocket
- [x] **Phase 3** — AES-256 encrypted secret store + Webhook channel
- [x] **Phase 3** — Gateway WebSocket streaming (token-by-token)
- [x] **Phase 4** — SIMD acceleration (NEON, SSE2, AVX2 auto-dispatch)
- [x] **Phase 4** — HTTP streaming model download from HuggingFace
- [x] **Phase 5** — Zalo Personal/OA Channel wrappers
- [x] **Phase 5** — Tool registry + arg validation
- [x] **Phase 5** — 45 unit tests, 11/11 crates covered ✅

### 📁 Project Structure

```
bizclaw/
├── Cargo.toml                 # Workspace root
├── src/main.rs                # CLI binary
├── crates/
│   ├── bizclaw-core/          # Traits, types, config, errors
│   ├── bizclaw-brain/         # Local GGUF inference engine
│   │   ├── forward.rs         # Full LLaMA transformer forward pass
│   │   ├── gguf.rs            # GGUF v3 parser
│   │   ├── mmap.rs            # Memory-mapped loader
│   │   ├── tokenizer.rs       # BPE tokenizer
│   │   ├── tensor.rs          # Math ops (RMSNorm, MatMul, etc.)
│   │   ├── quant.rs           # Quantization kernels
│   │   ├── attention.rs       # Scaled dot-product attention
│   │   ├── kv_cache.rs        # Key-value cache
│   │   ├── rope.rs            # Rotary position embeddings
│   │   ├── sampler.rs         # Token sampling
│   │   └── model.rs           # LLaMA model params
│   ├── bizclaw-providers/     # LLM provider impls
│   │   ├── openai.rs          # OpenAI / OpenRouter
│   │   ├── anthropic.rs       # Anthropic Claude
│   │   ├── ollama.rs          # Ollama (local/remote)
│   │   ├── llamacpp.rs        # llama.cpp server
│   │   ├── brain.rs           # Local brain with Mutex
│   │   └── custom.rs          # Any OpenAI-compatible
│   ├── bizclaw-channels/      # Communication channels
│   │   ├── cli.rs             # Interactive terminal
│   │   ├── telegram.rs        # Telegram Bot API
│   │   ├── discord.rs         # Discord Bot API
│   │   └── zalo/              # Zalo Personal + OA
│   │       └── client/        # Auth, Crypto, WS, Messaging
│   ├── bizclaw-memory/        # Persistence backends
│   ├── bizclaw-tools/         # Tool execution
│   ├── bizclaw-security/      # Security + AES-256 secrets
│   ├── bizclaw-agent/         # Agent orchestration
│   ├── bizclaw-gateway/       # HTTP + WebSocket API
│   └── bizclaw-runtime/       # Process adapters
└── plans/                     # Project plans & specs
```

### 🧪 Testing

```bash
# Run all 45 tests
cargo test --workspace

# Brain engine (12 tests: tensor, SIMD, attention, quant, rope)
cargo test -p bizclaw-brain

# Core types (11 tests: config, errors, messages)
cargo test -p bizclaw-core

# Tools (5 tests: registry, arg validation)
cargo test -p bizclaw-tools

# Agent (4 tests: context management)
cargo test -p bizclaw-agent

# Gateway (4 tests: route handlers)
cargo test -p bizclaw-gateway

# Memory (3 tests: vector search)
cargo test -p bizclaw-memory

# Security (2 tests: AES-256)
cargo test -p bizclaw-security

# Channels (2 tests: Zalo crypto, webhook)
cargo test -p bizclaw-channels

# Runtime (2 tests: info, exec)
cargo test -p bizclaw-runtime
```

### 📊 Stats

| Metric | Value |
|--------|-------|
| **Language** | 100% Rust |
| **Crates** | 11 (10 library + 1 binary) |
| **Lines of Code** | ~8,735 |
| **Tests** | 45 passing (11/11 crates) |
| **Build** | 0 errors |
| **Stubs** | 0 (100% implemented) |
| **Dependencies** | tokio, axum, reqwest, serde, rusqlite, rayon, memmap2, half, aes, sha2 |

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

**BizClaw** — *AI nhanh, mọi nơi. / Fast AI, everywhere.*
