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

4. **Run golden tasks**:
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
```

### Tool Guardrails

- **Chain ID validation**: Ensures operations on correct network
- **Gas cap enforcement**: Prevents excessive gas usage
- **Simulation-first**: All sends simulate before execution
- **ENS resolution**: Automatic resolution with fallback
- **EIP-55 checksum**: Address validation and normalization

### Tool registration and chat fallback

- **Native tool registration**: The client registers available tools with the LLM using JSON schemas (name, description, input types). This lets the model choose a tool when appropriate, or abstain when the input is general conversation.
- **No‑tool/small‑talk fallback**: If input doesn’t map to any tool (e.g., "hello"), the client responds without performing any state‑changing action. In mock mode, we route to a harmless read‑only check to keep the demo deterministic.

Examples:
```bash
# General small talk (no tool selection required)
cargo run -p baml_client -- -q "hello"

# Tool selection when relevant
cargo run -p baml_client -- -q "What's vitalik.eth's balance?"
```

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

