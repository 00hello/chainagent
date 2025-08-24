## Chain Agent

### Architecture

This workspace implements an agentic MCP toolbox for EVM operations with:
- **Type-safe tool surface**: BAML-defined functions with strict input/output schemas
- **Deterministic simulation**: All state-changing operations simulate first
- **Zero-trust prompt wiring**: LLM never constructs raw transactions
- **Cache and discovery**: LRU cache for contracts/ABIs with fallback to Etherscan
- **Extensibility**: Pluggable LLM providers and feature-flagged bonus tools

### Quickstart

1. **Start a mainnet fork (Anvil)**:
```bash
# Fork mainnet at latest block
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Or fork at specific block for reproducible testing
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY --fork-block-number 19000000
```

2. **Build workspace**:
```bash
cargo build
```

3. **Set environment variables**:
```bash
cp .env.example .env
# Edit .env with your API keys and RPC URL
export RPC_URL=http://127.0.0.1:8545  # Default Anvil URL
```

4. **Start the MCP server**:
```bash
# Standard server (recommended)
RPC_URL=http://127.0.0.1:8545 cargo run -p mcp_server

# Optional: enable bonus feature(s)
RPC_URL=http://127.0.0.1:8545 cargo run -p mcp_server --features bonus_uniswap_v2
```

5. **Run golden tasks**:
```bash
# Get ETH balance
cargo run -p baml_client -- -q "What's vitalik.eth's balance?"

# Check if address has code  
cargo run -p baml_client -- -q "Check if 0x0000000000000000000000000000000000000000 has deployed code"

# Send ETH (simulate first)
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob"
```

### Environment Variables

Create `.env` file:
```
RPC_URL=http://127.0.0.1:8545
ANTHROPIC_API_KEY=sk-ant-...
# Optional if using OpenAI models
OPENAI_API_KEY=sk-openai-...
```

### Tool Guardrails

- **Chain ID validation**: Ensures operations on correct network
- **Gas cap enforcement**: Prevents excessive gas usage
- **Simulation-first**: All sends simulate before execution
- **ENS resolution**: Automatic resolution with fallback
- **EIP-55 checksum**: Address validation and normalization

### Tool registration and chat fallback

- **Native tool registration**: The client registers available tools with the LLM using JSON schemas (name, description, input types). This lets the model choose a tool when appropriate.
- **Plain chat fallback**: If the model does not return a tool call JSON, the client treats the output as Chat and performs no tool execution. This covers greetings and general conversation (e.g., "hello", "How are you?").

Examples:
```bash
# General small talk (no tool execution)
cargo run -p baml_client -- -q "hello"
cargo run -p baml_client -- -q "How are you?"

# Tool selection when relevant
cargo run -p baml_client -- -q "What's vitalik.eth's balance?"
```

### Chain‑neutral tools and dynamic registry

- Internally the client exposes a chain‑neutral tool surface to the model:
  - `GetNativeBalance`, `SendNative`, `GetFungibleBalance`, `GetCode`
- A dynamic ToolRegistry assembles the tool list for the LLM at runtime (not hard‑coded), enabling future multi‑chain extension without changing parsing logic.

### Clarifying questions and partial intent

When the model selects a tool but omits required fields, the client responds with a brief clarifying message instead of failing, and embeds the incomplete tool call as a resumable block:

```
I need 'from', 'to', and 'amount_eth' to send. Please provide missing fields.
[[PARTIAL_INTENT]]
{"function": {"type": "SendNative", ...}}
[[/PARTIAL_INTENT]]
```

Provide the missing values in the next turn (ideally with the same session id) and the flow will resume.

### Session memory (one‑shot)

Session memory is stored in the MCP server’s RAM so you can keep the CLI one‑shot while retaining context across invocations.

```bash
# Start server (separate terminal)
RPC_URL=http://127.0.0.1:8545 cargo run -p mcp_server

# Start a session and chat
cargo run -p baml_client -- -q "hello" --session demo1

# Follow‑up with the same session id
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob" --session demo1

# Inspect stored turns (optional)
curl -sS "http://localhost:3000/session/get?session_id=demo1" | jq .
```

Notes:
- Volatile store (in‑memory): state is lost on server restart.
- Defaults: TTL ~ 1 hour; per‑session cap ~ 50 turns; total sessions ~ 1000.
- Endpoints for debugging/integration: `/session/get`, `/session/append`, `/session/partial_intent/get`, `/session/partial_intent/set`.

### Model selection (Anthropic/OpenAI)

- Default model: `claude-sonnet-4-20250514`. No flag required.
- Use `--model` to select a different model. If it starts with `claude`, Anthropic is used; otherwise OpenAI is used.

Examples:
```bash
# Default (Anthropic Claude) – no flag needed
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -p baml_client -- -q "hello"

# Explicit Anthropic model
cargo run -p baml_client -- -q "What's vitalik.eth's balance?" --model claude-sonnet-4-20250514

# OpenAI model
export OPENAI_API_KEY=sk-openai-...
cargo run -p baml_client -- -q "hello" --model gpt-4o-mini
```

Notes:
- Anthropic path uses native tools; OpenAI path uses function tools; both normalize back to the same tool JSON for the parser.

### Acceptance Criteria

- [x] Type-safe MCP tools with structured errors
- [x] BAML-driven CLI with natural language parsing
- [x] Deterministic simulation for all state changes
- [x] ENS resolution and address validation
- [x] Comprehensive test coverage including E2E tests
- [x] Error handling with clear, actionable messages
- [x] Documentation with architecture and usage examples

### Demo

Run the demo script to see the toolbox in action:
```bash
# Follow the demo script in demo.md
cargo run -p baml_client -- -q "What's vitalik.eth's balance?"
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob"
```

Bonus: A token address lookup tool and a tiny RAG sidecar are available for integration and testing.

### Bonus Features

Enable bonus features at runtime:
```bash
# either via CLI flag (client)
cargo run -p baml_client -- --enable-bonus -q "..."

# or via environment variable (client/server)
BONUS=1 cargo run -p baml_client -- -q "..."
```

Uniswap V2 swap scaffolding (feature-gated):
```bash
# Enable swap scaffolding on the server at compile-time
cargo run -p mcp_server --features bonus_uniswap_v2

# Run E2E swap test with the feature enabled
cargo test --features bonus_uniswap_v2 -- tests/uniswap_swap_e2e.rs
```

Included bonus tools:
- External API token lookup (server-side; address discovery by symbol/chain)
- Uniswap V2 swap scaffolding (simulate-first; feature-gated)
- Tiny local RAG sidecar (ingest + top-k query)

See `demo.md` for comprehensive demo script and `tasks/tasks-prd-agentic-mcp-toolbox-for-evm.md` for detailed implementation plan.

