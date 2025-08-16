## Relevant Files

- `Cargo.toml` - Workspace definition for multi-crate project (Rust 2024 edition).
- `rust-toolchain.toml` - Pin toolchain (rustc >= 1.86.0).
- `crates/domain/src/lib.rs` - Shared types: `Toolbox` trait, request/response structs, address/token types.
- `crates/foundry_adapter/src/lib.rs` - JSON-RPC helpers, ENS resolution, gas/chain checks, simulation utilities.
- `crates/mcp_server/src/main.rs` - MCP server entry; tool registration and provider wiring.
- `crates/mcp_server/src/toolbox.rs` - Tool implementations (`balance`, `code`, `send`, `erc20_balanceOf`).
- `crates/mcp_server/src/dto.rs` - DTOs for MCP request/response payloads; explicit conversions to/from domain.
- `crates/mcp_server/src/facade.rs` - Facade layer orchestrating adapter/domain; centralizes business logic.
- `crates/mcp_server/src/uniswap_v2.rs` - Optional: Uniswap V2 `swapExactETHForTokens` tool implementation (behind feature flag).
- `crates/mcp_server/src/external_api.rs` - Optional: external API clients (DefiLlama/0x/Brave) for token address discovery.
- `crates/baml_client/src/main.rs` - CLI REPL; NL → BAML function mapping → MCP calls; pretty printing.
- `baml/agent.baml` - Top-level prompts and function selection schemas.
- `baml/tools.baml` - Function schemas mirroring MCP tools (typed I/O).
- `crates/rag_client/src/main.rs` - Optional: tiny RAG sidecar to ingest/search Uniswap docs and Router interface.
- `README.md` - Quickstart, commands, demo script, acceptance checklist.
- `tests/e2e_send_eth.rs` - E2E: start server, call client to send 1 ETH, assert balances/receipt.
- `tests/uniswap_swap_e2e.rs` - Optional: E2E for Uniswap V2 swap (simulate first; then send).
- `crates/mcp_server/tests/toolbox_integration.rs` - Integration tests for tool endpoints.
- `crates/mcp_server/tests/external_api_lookup.rs` - Optional: token address lookup tests.
- `crates/baml_client/tests/cli_parsing.rs` - NL → typed call mapping snapshot tests.

### Notes

- If this project uses Rust, it uses **Rust 2024 Edition** with at least rustc 1.86.0. 
- **No `mod.rs` files needed** - Rust 2024 supports directory-based modules without `mod.rs`.
- Unit tests can be placed in the same file as the code they are testing, inside a `#[cfg(test)] mod tests { ... }` block.
- Use `cargo test` to run all tests. To run a specific test, you can use `cargo test <test_name>`.
- **Always include documentation updates** - Any task that adds or changes user-facing functionality (CLI commands, APIs, interfaces) must include a sub-task to update relevant documentation (README.md, help text, etc.).

#### Design conventions (encapsulation, DTOs, facades)

- All domain types must keep fields private; expose behavior via getters/setters and constructors/builders. Consumers never access fields directly.
- Use DTOs at boundaries (MCP I/O, serialization). Convert DTOs ↔ domain models explicitly.
- Apply a facade/helper module to host business logic instead of scattering it across types.
- If a field is widely used, replace direct access with accessor/utility functions.
- If a field may be removed/renamed, depend on trait/interface abstractions, not concrete fields.
- If a field will evolve, never make it `pub`; add getters now to preserve stability later.

| Problem | Strategy |
| --- | --- |
| Field is used everywhere | Replace with accessor or utility function |
| Want to remove or rename field | Use trait/interface abstraction |
| Field is serialized/deserialized | Introduce a DTO and conversion logic |
| Business logic is tied to field | Move logic into a facade/helper module |
| Field will likely evolve | Never make it `pub`, always use getters |

## Tasks

- [ ] 1.0 Scaffold workspace and environment
  - [x] 1.1 Initialize Cargo workspace (Rust 2024) with crates: `domain`, `foundry_adapter`, `mcp_server`, `baml_client`.
  - [x] 1.2 Add `rust-toolchain.toml` (pin >= 1.86.0) and baseline CI (fmt + clippy) config.
  - [x] 1.3 Add dependencies: MCP Rust SDK, Ethereum JSON-RPC client (e.g., ethers/alloy), serde, anyhow, thiserror, clap, tokio.
  - [x] 1.4 Create `baml/agent.baml` and `baml/tools.baml` placeholders; wire BAML validation workflow into repo (docs in README).
  - [x] 1.5 Create `.env.example` with `ANTHROPIC_API_KEY` and `RPC_URL`; add `.env` to `.gitignore`.
  - [x] 1.6 Draft `README.md` skeleton: architecture, quickstart, anvil fork command, golden tasks.

- [ ] 2.0 Implement MCP server toolbox (typed APIs, simulation-first)
  - [ ] 2.1 In `domain`, define `Toolbox` trait and request/response structs for: `balance`, `code`, `send`, `erc20_balanceOf`; include `simulate: bool`, `forkBlock: Option<u64>` on state-changing calls.
  - [x] 2.1a Ensure all domain structs keep fields private; expose getters/setters and constructors/builders (no `pub` fields).
  - [ ] 2.1b Add builders or smart constructors where multi-field invariants apply.
  - [ ] 2.2 Implement `foundry_adapter`: provider factory (RPC URL), ENS resolution helper, checksum validator, chain-id check, gas cap enforcement, retry-once policy for transient RPC.
  - [ ] 2.3 Build MCP server: register tools per MCP 2025-06-18; map JSON-schema to domain structs.
  - [x] 2.3a Introduce DTOs in `mcp_server::dto` for all tool inputs/outputs; implement `From`/`TryFrom` conversions to/from domain types.
  - [x] 2.3b Add a `facade` module that coordinates adapter and domain layers, centralizing business logic and guardrails.
  - [ ] 2.4 Implement `balance(who)` using address/ENS resolution; return wei as string/u256.
  - [ ] 2.5 Implement `code(addr)` returning `{ deployed: bool, bytecode_len }`.
  - [ ] 2.6 Implement `erc20_balanceOf(token, holder)` via minimal ERC-20 ABI and `eth_call`.
  - [ ] 2.7 Implement `send(from, to, eth, simulate, forkBlock)`: simulate path first; enforce gas cap and chain id; on success, send and return `TxResult` (hash, gas used, status).
  - [x] 2.8 Add structured errors (thiserror) with clear, actionable messages; log context.
  - [x] 2.9 Unit tests for each tool implementation (mock provider where practical).
  - [ ] 2.10 Tests to enforce encapsulation and conversions: domain structs expose no `pub` fields; DTO ↔ domain conversions validated.

- [ ] 3.0 Implement BAML-driven CLI client (NL → typed calls)
  - [ ] 3.1 Define BAML functions in `baml/tools.baml`: `SendEth`, `GetErc20Balance`, `IsDeployed`, mapping 1:1 to MCP tools.
  - [ ] 3.2 Define selection logic in `baml/agent.baml` for routing NL prompts to the above functions with strict typing.
  - [ ] 3.3 Implement CLI (`crates/baml_client`): read NL input, call Anthropic (pluggable) to choose BAML function + args; validate via BAML; invoke MCP; echo typed call and pretty-print JSON.
  - [ ] 3.4 Implement provider abstraction: `ChatProvider` trait so Claude/OpenAI/local can be swapped without changing call flow.
  - [ ] 3.5 Snapshot tests for NL → function mapping on the three golden prompts (allow deterministic fixtures).

- [ ] 4.0 Chain integration: Anvil fork, ENS resolution, guardrails & seeds
  - [ ] 4.1 Document starting Anvil fork in README and set default RPC `http://127.0.0.1:8545`.
  - [ ] 4.2 Seed constants: USDC mainnet address; Uniswap V2 Router02 (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D); Foundry accounts aliasing (Alice=0, Bob=1).
  - [ ] 4.3 Implement ENS resolution in adapter; fallback/error strategy defined and tested.
  - [ ] 4.4 Implement EIP-55 checksum enforcement; normalize inputs; reject invalid addresses with clear guidance.
  - [ ] 4.5 Implement chain id check tied to fork; add configurable gas caps; document defaults.
  - [ ] 4.6 Add LRU cache placeholders for ABIs/verified contracts; stub Etherscan read-only fallback; keep interface to add L2Beat-style discovery later.
  - [ ] 4.7 Integration tests: `code` returns deployed=true for Router02 on fork; `erc20_balanceOf` returns numeric value for Alice/USDC.

- [ ] 5.0 Validation & delivery: tests, README, demo script and acceptance checks
  - [ ] 5.1 E2E test: start MCP server, run CLI, execute “send 1 ETH from Alice to Bob”; assert Bob’s balance increases and receipt status is success.
  - [ ] 5.2 Tests: `balance`, `erc20_balanceOf`, `code` happy paths + invalid-input cases.
  - [ ] 5.3 Finalize README: architecture diagram bullets, exact commands, tool guardrails, acceptance criteria.
  - [ ] 5.4 Provide demo script transcript: three golden tasks and one bonus (if implemented).
  - [ ] 5.5 Tag open questions and known trade-offs; create follow-up issues for bonus scope (swap tool, external API).

- [ ] 6.0 Optional bonus block (feature-flagged)
  - [ ] 6.1 Uniswap V2 swap tool (`uniswap_v2_swap_exact_eth_for_tokens`)
    - [ ] 6.1.1 In `domain`, add request/response structs (amount_eth, token_out, min_out_bps, deadline_s; plus `simulate` default true).
    - [ ] 6.1.2 In `mcp_server/uniswap_v2.rs`, build calldata for `swapExactETHForTokens(uint256,address[],address,uint256)`; route via WETH→token; seed Router02 address; honor deadline.
    - [ ] 6.1.3 Enforce simulate-first; allow `--dry-run` in CLI; return structured result (tx hash, amounts, path, gas used).
    - [ ] 6.1.4 E2E test in `tests/uniswap_swap_e2e.rs` using Anvil fork; verify nonzero output and tx success.
  - [ ] 6.2 External API lookup tool (token address discovery)
    - [ ] 6.2.1 Implement client in `mcp_server/external_api.rs` for one provider (DefiLlama or 0x or Brave); schema: `{ symbol | name | chain } -> { address }`.
    - [ ] 6.2.2 Add basic caching (in-memory LRU) and rate limit/backoff handling; clear errors on misses.
    - [ ] 6.2.3 Integration tests in `crates/mcp_server/tests/external_api_lookup.rs` (mock HTTP where possible).
    - [ ] 6.2.4 Expose as an MCP tool; optional use by swap/erc20 helpers.
  - [ ] 6.3 Tiny local RAG sidecar (Uniswap docs and Router interface)
    - [ ] 6.3.1 Create `crates/rag_client` with ingest pipeline (docs and ABI/interface text) and local embeddings.
    - [ ] 6.3.2 Provide a simple `query` command returning top-k snippets; optional: surface snippets to the client before answering.
    - [ ] 6.3.3 Add README section describing how to run RAG build/query; keep it off the main path unless `--enable-rag` is set.
  - [ ] 6.4 Feature flagging and docs
    - [ ] 6.4.1 Add env/CLI flag `--enable-bonus` (or `BONUS=1`) to enable swap, API lookup, and RAG tools at runtime.
    - [ ] 6.4.2 Document flags in README and demo script; keep default build/runtime without bonus features.


