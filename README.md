# Ferret

A small but eager chatbot with web search capabilities, powered by Ollama and Brave Search.

## Features

- **Conversational AI**: Uses local Ollama models for natural language understanding
- **Web Search**: Integrates Brave Search API for real-time web queries
- **Page Fetching**: Can retrieve and summarize content from web pages
- **Session Management**: Maintains conversation context with automatic cleanup
- **Streaming Responses**: Server-sent events for real-time chat responses
- **Web Interface**: Clean, simple chat UI

## Prerequisites

- **Rust** 1.70 or later
- **Ollama** installed and running locally ([download here](https://ollama.ai))
- **Brave Search API Key** ([get one here](https://brave.com/search/api/))

## Quick Start

1. **Clone the repository**:
   ```bash
   git clone https://github.com/lawless-m/Ferret.git
   cd Ferret
   ```

2. **Set up Ollama**:
   ```bash
   # Pull a model (e.g., qwen2.5:7b)
   ollama pull qwen2.5:7b

   # Ensure Ollama is running
   ollama serve
   ```

3. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env and set your BRAVE_API_KEY
   ```

4. **Build and run**:
   ```bash
   cargo build --release
   cargo run --release
   ```

5. **Open your browser**:
   Navigate to `http://localhost:3000`

## Configuration

Configure Ferret via environment variables or `.env` file:

| Variable | Default | Description |
|----------|---------|-------------|
| `OLLAMA_URL` | `http://localhost:11434` | Ollama API endpoint |
| `OLLAMA_MODEL` | `qwen2.5:7b` | Model to use for chat |
| `BRAVE_API_KEY` | *(required)* | Your Brave Search API key |
| `BIND_ADDRESS` | `0.0.0.0:3000` | Server bind address |
| `SESSION_TIMEOUT_MINS` | `60` | Session expiry time |
| `RUST_LOG` | `info,ferret=debug` | Logging level |

## API Endpoints

- `GET /` - Web chat interface
- `POST /chat` - Send a chat message (returns SSE stream)
- `POST /clear` - Clear conversation history
- `GET /health` - Health check endpoint

### Chat Request Format

```json
{
  "message": "Search for recent news about Rust"
}
```

The chatbot automatically uses tools when needed:
- `brave_search` - Search the web with Brave Search API
- `fetch_page` - Retrieve and extract text from a URL

## Architecture

```
src/
├── main.rs           # Application entry point and server setup
├── config.rs         # Configuration management
├── error.rs          # Error types
├── chat/             # Chat handling and streaming
│   ├── handler.rs    # Request processing
│   └── stream.rs     # SSE response streaming
├── ollama/           # Ollama client integration
│   ├── client.rs     # HTTP client for Ollama API
│   └── types.rs      # Request/response types
├── routes/           # HTTP route handlers
├── session/          # Session management
│   ├── manager.rs    # Session storage and cleanup
│   └── types.rs      # Session data structures
└── tools/            # Tool calling system
    ├── executor.rs   # Tool execution coordinator
    ├── parser.rs     # Parse tool calls from LLM output
    ├── search.rs     # Brave Search integration
    └── fetch.rs      # Web page fetching
```

## How It Works

1. User sends a message via the web interface
2. Ferret adds the message to the conversation history
3. The conversation is sent to Ollama with tool definitions
4. If Ollama requests tool use, Ferret executes the tool and continues
5. Responses are streamed back to the browser in real-time
6. Sessions persist conversation history for context

## Development

```bash
# Run in development mode with auto-reload
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
