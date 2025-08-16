## PRD: Agentic MCP Toolbox for EVM (BAML Client + Foundry Server)

### Introduction / Overview
Build a model-agnostic, type-safe agent interface that turns natural language into deterministic, safe on-chain actions. The system exposes Ethereum operations as MCP tools (server) and uses BAML on the client to strictly type inputs/outputs and route user intents. Goal: demonstrate end-to-end reliability on a local mainnet fork (Anvil) while establishing a reusable toolbox for future integrations.

### Goals
- Deliver a demoable CLI that completes the three “golden” tasks on an Anvil mainnet fork.
- Provide a reusable, type-safe MCP toolbox surface for future agent clients.
- Ensure deterministic simulation first for any state-changing transaction.
- Keep the client model-agnostic with a pluggable provider; demo with Claude.

### Target Users
- Internal demo now: founder/interview evaluation.
- Developer-facing later: builders integrating agents into wallets/DeFi.

### User Stories
- As an interviewer, I want to see “send 1 ETH”, “USDC balance”, and “is Uniswap Router deployed?” succeed on a fork so I can verify correctness quickly.
- As a developer, I want typed tools with clear schemas so I can integrate any LLM without changing chain logic.
- As a CLI user, I want to issue natural language commands and see a clear, typed call echoed with a concise result.

### Functional Requirements
1. CLI REPL with BAML mapping
   - Parse natural language into BAML functions with strict schemas.
   - Echo the typed function call and print a compact result plus raw JSON.
2. Identity and address book
   - Use Foundry’s default accounts; alias (0)=Alice, (1)=Bob.
3. Core commands (“golden three”)
   3.1 Send ETH
   - Input: `from`, `to`, `amount_eth`.
   - Behavior: validate inputs; optional simulate-first; then send; return tx hash and receipt summary.
   3.2 ERC‑20 balance (USDC)
   - Input: `holder`, `token` (seed USDC mainnet address).
   - Behavior: call `balanceOf(holder)`; return numeric balance.
   3.3 Deployment check (Uniswap V2 Router02)
   - Input: `address` (seed canonical Router02 address 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D).
   - Behavior: `eth_getCode`/equivalent; return `{ deployed: bool, bytecode_len: number }`.
4. ENS resolution
   - Accept ENS or hex; resolve before execution; enforce checksum format on outputs.
5. Guardrails
   - Enforce EIP‑55 checksum, correct chain id, and reasonable gas caps (e.g., default hard cap; configurable).
6. Simulation-first for state changes
   - All state‑changing tools accept `simulate: bool` and `forkBlock: Option<u64>`; fail safe if preconditions unmet.
7. Error handling
   - Fail fast with clear errors; add one retry for transient RPC failures.
8. Network/fork
   - Use Anvil fork of Ethereum mainnet via provided RPC; server uses `http://127.0.0.1:8545`.
9. Seeds
   - Include USDC mainnet address and Uniswap V2 Router02 canonical address.
10. Type‑safe toolbox
   - Define a `Toolbox` trait with explicit input/output structs. Expose at minimum:
     - `balance(who: String) -> wei`
     - `code(addr: String) -> { bytecode_len: u64 }`
     - `send(from: String, to: String, eth: String, simulate: bool, forkBlock: Option<u64>) -> TxResult`
     - `erc20_balanceOf(token: String, holder: String) -> u256`
11. Bonus scope (optional)
   - Uniswap V2 `swapExactETHForTokens` tool with `simulate` first.
   - External API integration for price/address discovery (e.g., DefiLlama/0x/Brave Search).

### Non‑Goals (Out of Scope)
- Web UI.
- Multi‑chain support.
- Hardware wallet signing.
- Large token registry.
- Complex client‑side planning/strategy beyond simple intent mapping.

### Design Considerations
- Type‑safe tool surface
  - Define a `Toolbox` trait with explicit request/response structs. This makes tools self‑documenting and robust to model variability.
- Deterministic simulation first
  - Every state‑changing call supports `simulate` and `forkBlock`. Prefer fail‑safe behavior.
- Zero‑trust prompt wiring
  - The client never constructs raw transactions. It only requests MCP tools to do so and displays typed results.
- Cache and discovery (forward‑looking)
  - Add an LRU cache for verified contracts/ABIs with a read‑only Etherscan fallback. Keep an interface to plug L2Beat‑style discovery later.
- Extensibility (client)
  - Leave hooks for BAML function schemas to expand; keep provider pluggable (Claude for demo).

### Technical Considerations
- Stack
  - Language: Rust (stable toolchain).
  - MCP server: Anthropic Rust SDK, MCP spec 2025‑06‑18.
  - Chain tooling: Foundry (Anvil fork, cast‑equivalents/JSON‑RPC).
  - Client: BAML for schema‑first intent mapping in a CLI.
- Data
  - Address seeds: USDC mainnet; Uniswap V2 Router02.
  - Account aliases: Foundry defaults as Alice/Bob.
- Guard defaults (suggested)
  - Gas cap for simple sends: e.g., ≤ 500,000; configurable.
  - Chain id check: must match forked mainnet id.
  - Address validation: enforce EIP‑55 checksum.
- Error handling
  - Clear, structured errors; one retry on idempotent/transient RPC failures; no silent retries on state changes.

### Success Metrics
- 100% success rate on the three golden tasks on a fresh Anvil fork.
- All state‑changing tools support `simulate=true` and reject unsafe execution.
- Inputs rejected with actionable errors when invalid (addresses, amounts, chain id).
- README includes exact demo commands and short script.
- Minimal tests cover tool I/O and one end‑to‑end ETH send.

### Acceptance Criteria
- Three golden tasks succeed on a fresh fork.
- State‑changing tools accept `simulate` and optional `forkBlock`.
- Inputs validated and checksummed.
- README with exact commands and demo script.
- Minimal tests for tool I/O and "send 1 ETH" flow.

### Open Questions
- RPC endpoint to standardize in docs (Alchemy, Reth, or other)?
- Default gas cap values per tool—finalize thresholds.
- ENS resolution behavior on failures—fallback or hard fail?
- External API choice for bonus (DefiLlama vs 0x vs Brave) and rate limit handling.

### Appendix: Example NL Commands
- send 1 ETH from Alice to Bob
- How much USDC does Alice have?
- Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
