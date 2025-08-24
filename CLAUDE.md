# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running
- Build entire workspace: `cargo build`
- Start MCP server: `RPC_URL=http://127.0.0.1:8545 cargo run -p mcp_server`
- Start MCP server with bonus features: `RPC_URL=http://127.0.0.1:8545 cargo run -p mcp_server --features bonus_uniswap_v2`
- Run BAML client: `cargo run -p baml_client -- -q "query here"`
- Run with session memory: `cargo run -p baml_client -- -q "query here" --session session_id`
- Enable BAML validation: `ENABLE_BAML=1 cargo run -p baml_client -- -q "query"`

### Testing
- Run all tests: `cargo test`
- Run specific test: `cargo test test_name`
- Run integration tests: `cargo test --test integration`
- Run E2E with Uniswap features: `cargo test --features bonus_uniswap_v2 -- tests/uniswap_swap_e2e.rs`

### Development Setup
1. Start Anvil fork: `anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY`
2. Create `.env` with:
   ```
   RPC_URL=http://127.0.0.1:8545
   ANTHROPIC_API_KEY=sk-ant-...
   OPENAI_API_KEY=sk-openai-... # optional
   ENABLE_BAML=1 # optional
   ```

### Rust Toolchain
- Uses Rust 2024 Edition (channel 1.86.0)
- No `mod.rs` files needed due to Rust 2024 directory-based modules

## Architecture Overview

### Core Components
This is an agentic MCP (Model Context Protocol) toolbox for EVM blockchain operations with a multi-crate architecture:

**Domain Layer (`crates/domain`)**
- Core types: `Address`, `AddressOrEns`, request/response structs
- `Toolbox` trait defining blockchain operations interface
- `BlockchainProvider` trait for chain-neutral operations
- Builder patterns for complex requests (e.g., `SendRequest::builder()`)

**Foundry Adapter (`crates/foundry_adapter`)**
- JSON-RPC client for Ethereum interaction
- ENS resolution with fallback
- Gas estimation and chain ID validation
- Transaction simulation before execution
- Contract ABI caching with Etherscan fallback

**MCP Server (`crates/mcp_server`)**
- HTTP server exposing blockchain tools as REST endpoints
- Session memory store (in-memory with TTL)
- Tool implementations: balance, code, send, ERC-20 balance
- Optional Uniswap V2 swap functionality (feature-gated)
- External API integration for token address lookup

**BAML Client (`crates/baml_client`)**
- CLI interface with natural language processing
- Model provider abstraction (Anthropic/OpenAI)
- Session management for conversational memory
- Tool parsing and validation
- Mock provider for testing

### Key Design Patterns

**Type Safety & Encapsulation**
- All domain types use private fields with public getters
- Builder patterns for complex request construction
- DTOs for serialization boundaries with explicit conversions

**Error Handling**
- Comprehensive error types with context
- Simulation-first approach for state changes
- Graceful fallbacks (e.g., ENS to address resolution)

**Prototyping-Friendly Patterns** (per .cursor/rules)
- Liberal use of `unwrap()` and `todo!()` during prototyping
- Simple owned types (String, Vec) over complex lifetimes
- Flat module hierarchy in early development phases

### Tool Surface
The system exposes chain-neutral tools to LLMs:
- `GetNativeBalance` - Get ETH balance for address/ENS
- `GetCode` - Check if address has deployed code  
- `GetFungibleBalance` - Get ERC-20 token balance
- `SendNative` - Send ETH with simulation support
- Bonus tools: Uniswap V2 swaps, token lookup, RAG queries

### BAML Integration
- Natural language queries parsed to typed function calls
- Schema-first validation (optional via `ENABLE_BAML=1`)
- Function definitions in `baml/agent.baml` and `baml/tools.baml`
- Multiple model provider support with consistent tool interface

### Session Management
- In-memory session store on MCP server
- Conversational context across CLI invocations
- Partial intent resumption for incomplete queries
- TTL-based cleanup (default 1 hour, 50 turns/session, 1000 sessions)

### Feature Flags
- `bonus_uniswap_v2`: Enable Uniswap V2 swap functionality
- Runtime bonus features via `--enable-bonus` or `BONUS=1`
- BAML validation via `--enable-baml` or `ENABLE_BAML=1`

## Development Guidelines

### Testing Strategy
- Unit tests in same files as code (`#[cfg(test)]` blocks)
- Integration tests in `tests/` directory
- E2E tests that start server and make real client calls
- Mock providers for deterministic testing

### Code Organization
- Keep main logic in `main.rs` during prototyping
- Use inline modules before extracting to files
- Domain types in `domain` crate, adapters separate
- DTOs at boundaries with explicit conversions

### Environment Requirements
- Anvil fork for local development
- API keys for Anthropic/OpenAI models
- Optional external API keys for bonus features