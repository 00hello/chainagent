Skip to content
Chat history
You said:
Interview with Foameo Labs

Introductions and Background





I introduced myself, mentioning offices in Toronto and Sudbury



Discussed early involvement in Canadian crypto scene, including running Ottawa Bitcoin meetups



Described overlap with Ethereum’s founding team in Toronto



Shared experience building one of the first Bitcoin ATMs



Talked about the atmosphere of early crypto, legal uncertainty, and motivation by technology rather than money

AI and Blockchain Experience





I described early AI work: generating romance novels with GANs before LLMs existed



Explained transition from Bitcoin ATMs to AI, founding Deterministic House for generative fiction



Merged blockchain and AI in 2018, building distributed video stream analysis in Rust



Highlighted real-time, high-throughput requirements pushing me from Python to Rust



Detailed distributed inference pipelines for semantic event detection and digital twins

Technical Deep Dive: Agentic OS and Infrastructure





Cecilia (the founder) explained vision for agentic OS: an operating system-like runtime for AI agents on-chain



Goal: expose blockchain node capabilities as system calls for AI, abstracting away RPC and SDK complexity



Discussed need for standardized, plug-and-play infrastructure for LLMs to interact with blockchains



Compared approach to frameworks like Foundry, LangChain, and referenced inspiration from LATTA.AI



Emphasized model-agnostic, type-safe, and modular architecture in Rust

Contract Discovery and Code Generation





Cecilia described current TypeScript system (L2Beat) for contract discovery using breadth-first search over view functions



System can operate with or without manual configuration files (JSON), supporting both generic and custom logic



Plan to replicate and improve this system in Rust, enabling dynamic fetching, parsing, and compilation of unknown contracts



Discussed use of local verified-contract cache, async background agents for refreshing, and fallback to Etherscan/GitHub on cache miss



Outlined the importance of simulation (e.g., with Foundry) to prevent hallucinations and ensure safe, correct codegen

Product Vision and Go-to-Market





Product aims to be the “LangChain of crypto AI,” providing both dev tooling and reference apps for autonomous on-chain execution



Monetization strategies: API access, B2B2C integrations with wallets, order flow business model (taking a fee per transaction)



Long-term goal: land major integrations (Coinbase, Kraken, Stripe), eventually aiming for acquisition



Emphasized that dev tooling is a marketing strategy, but real value is in credible, safe, production-grade transaction crafting

Technical Assessment and Process





Cecilia described the technical assessment as a stripped-down version of their system, to be implemented in Rust



No strict deadline; process is “vibe-based” and flexible, as Cecilia is a solo founder



Assessment will evaluate code quality, architectural decisions, and ability to optimize agent thinking/context management



Provided guidance on preferred stack and dependencies in documentation

Miscellaneous Details





Discussed frameworks/tools: BAML, DSPY, YOLO, CLIP, and prompt engineering approaches



Talked about hackathon culture in AI and crypto, similarities in incentivizing builders



Cecilia values candidates with DevRel potential and ability to communicate with developers



Clarified my meetup organizing was in Ottawa, not Toronto

Closing





Agreed on next steps: I will review the assessment and submit when ready



Reiterated mutual interest and alignment in technical vision and experience



Ended on a positive note, with open communication for follow-ups





https://www.notion.so/Technical-Assessment-Rust-Engineer-21436be0954d80318ddecff5b3b4d522

https://github.com/BoundaryML/baml

https://foundry.sh/

https://aave.com/docs

https://github.com/l2beat/l2beat/tree/main
https://www.notion.so/l2b-Implementation-21036be0954d8096b398cab3b0db2837

https://latta.ai/#features




Technical Assessment - Rust Engineer


# Technical Assessment - Rust Engineer

<aside>
💡

**The us of AI is strongly encouraged!** You should leverage AI to accelerate your learning curve. Put the requirements in context to generate as much as code as possible, as long as it make sense. Think of Anthropic using Claude Code to develop Claude Code. We’re looking for 10x AI user for fast iterations as an AI company.

If I’m not seeing trace of vibe coding in your submission, that’s and red flag 🚩

</aside>

## Overview

Build an AI agent system that can interact with Ethereum blockchain using natural language commands. The system consists of two main components:

1. **AI Agent Client** - Built with RIG framework, provides a CLI REPL for user, and incorporate a client with Caulde API key that process user’s natural language.
    
    https://docs.rig.rs/docs/concepts/agent
    
    https://docs.anthropic.com/en/api/overview
    
2. **MCP Server** - Uses Anthropic Rust SDK to expose Foundry functionality as tools.
    
    https://modelcontextprotocol.io/specification/2025-06-18
    
    https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples/servers
    
3. **Transaction Generation** - Foundry is an Smart Contract dev tool that also serves as an light client to most EVM chains. You can send RPC call to blockchain endpoint through cast, which is an component of Foundry.
    
    

bash
    $ cast balance vitalik.eth --ether --rpc-url https://reth-ethereum.ithaca.xyz/rpc


    
    For this project, you can expose case method directly as MCP tool call like this:
    
    https://github.com/foundry-rs/foundry/blob/c78faa217c4ee7a60894c4f740f5c5a967ffb97b/crates/cast/src/lib.rs#L267
    
    

rust
     #[tool(tool_box)]
    impl MyMcp {
        #[tool(description = "Get the balance of an account in wei")]
        async fn balance (
            &self,
            #[tool(param)]
            #[schemars(description = "The address or ENS name to check balance for")]
            who: String,
        ) -> Result<CallToolResult, McpError> {
            let address = NameOrAddress::from(who)
                .resolve(&self.provider)
                .await
                .unwrap();
            let balance = self.provider.get_balance(address).await.unwrap();
    
            Ok(CallToolResult::success(vec![Content::text(
                balance.to_string(),
            )]))
        }
    }


    

## Technical Requirements

### Stack Requirements

- **Client Framework**: [RIG (Rust AI agent framework)](https://github.com/0xPlaygrounds/rig)
- **Server Framework**: [Anthropic Rust SDK for MCP](https://github.com/modelcontextprotocol/rust-sdk)
- **Blockchain Tool**: [Foundry](https://github.com/foundry-rs)
- **Language**: Rust
- **Interface**: CLI REPL

### System Architecture

┌─────────────────┐    MCP Protocol    ┌──────────────────┐
             │   RIG Agent     │◄──────────────────►│   MCP Server     │
             │   (Client)      │                    │                  │
             ├─────────────────┤                    ├──────────────────┤
User   ◄───► │ • CLI REPL      │                    │ • Foundry - Cast │
Claude ◄───► │ • LLM API Key   │                    │ • Tx Generation  │
             │ • User Input    │                    │ • State Fork     │
             │ • Response      │                    │ • Anthropic SDK  │
             └─────────────────┘                    └──────────────────┘
                      │                                       │
                      │                                       │
                      └───────────────┐           ┌───────────┘
                                      │           │
                                 ┌────▼───────────▼──────┐
                                 │   Forked Ethereum     │
                                 │     Test Network      │
                                 │   (via Foundry)       │
                                 └───────────────────────┘



## Part 1: Environment Setup

### Prerequisites

- Install Foundry
- Set up Anthropic API key
- Install RIG framework dependencies

### Foundry Test Network

bash
 $ anvil --fork-url  https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs



Your MCP server should access 127.0.0.1:8545 with these test accounts:

Available Accounts
==================
(0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000.000000000000000000 ETH)
(1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000.000000000000000000 ETH)
(2) 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC (10000.000000000000000000 ETH)
(3) 0x90F79bf6EB2c4f870365E785982E1f101E93b906 (10000.000000000000000000 ETH)
(4) 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (10000.000000000000000000 ETH)
(5) 0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc (10000.000000000000000000 ETH)
(6) 0x976EA74026E726554dB657fA54763abd0C3a0aa9 (10000.000000000000000000 ETH)
(7) 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955 (10000.000000000000000000 ETH)
(8) 0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f (10000.000000000000000000 ETH)
(9) 0xa0Ee7A142d267C1f36714E4a8F75612F20a79720 (10000.000000000000000000 ETH)

Private Keys
==================
(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
(1) 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
...



## Part 2: Core Requirements

### 2.1 Basic Functionality (Required)

Your system must handle these basic commands or answer these questions:

bash
# Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
# Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?



**Expected Behavior:**

- Parse natural language input
- Identify sender (default to account 0)
- Validate recipient address
- Generate transaction using Foundry
- Execute transaction on forked network
- Return transaction hash and confirmation

## Part 3: Bonus

### 3.1 Server-side External API integration

Integrate external API into your MCP server, for example

- https://brave.com/search/api/
- https://defillama.com/pro-api/docs
- https://0x.org/docs/api

bash
> Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account.



**Expected Behavior if you integrate standard web search:**

- Parse swap intent
- Integrate with a search API to find current Uniswap contract addresses
- Figure out the correct function to use. Hint: you can prompt the AI to call swapExactETHForTokens(uint256,address[],address,uint256)
- The LLM can either search on the internet, 0x, or DefiLama to figure out the price, or you can prompt it to use parameters you come up with.
- Formulate the call data and execute through the MCP server
- Return swap details and transaction hash
- **Example**  https://www.warp.dev/ **with Claude 4 🤔**
    
    ![Screenshot 2025-07-21 at 9.00.11 PM.png](attachment:28104033-cefd-498b-87af-71b2ee51bfce:Screenshot_2025-07-21_at_9.00.11_PM.png)
    
    ![Screenshot 2025-07-21 at 9.01.17 PM.png](attachment:44012e69-f3c7-4103-a754-7b86916de056:Screenshot_2025-07-21_at_9.01.17_PM.png)
    
    Wrap can already accomplish the this operation by searching the internet to figure out the steps, and use cast to call the node RPC. Wrap agent knows cast and anvil from it’s pre-training. This assessment requires you to integrate foundry stack a MCP servers, so instead of calling cast cli, your agent calls the MCP tools.
    
    ### Observations
    
    Wrap lacks blockchain-specific APIs to perform generalized on-chain action. As a result, this is not production-grade UX for non-developers. We’re trying to fill the gap.
    
    ![Screenshot 2025-07-21 at 10.38.59 PM.png](attachment:4ce241ee-7c0a-44f8-8511-c4db5ef76aba:Screenshot_2025-07-21_at_10.38.59_PM.png)
    
    ![Screenshot 2025-07-21 at 10.39.24 PM.png](attachment:e7085880-9cd4-4598-96fe-747a778f639a:Screenshot_2025-07-21_at_10.39.24_PM.png)
    

### **3.2 Client-side RAG for Docs and Source Code**

Implement a Retrieval-Augmented Generation (RAG) system on the client side that stores and queries Uniswap documentation and contract source code.

**Requirements:**

- **Document Storage**: Ingest and store Uniswap V2/V3 documentation, guides, and contract source code
- **Vector Embeddings**: Use a local embedding model
- **Context Integration**: Provide relevant docs to the LLM for better responses

bash
> How do I calculate slippage for Uniswap V3?
# Agent searches RAG system, finds relevant docs, provides detailed answer

> What's the difference between exactInput and exactOutput?
# Agent retrieves function documentation and explains differences

> Show me the SwapRouter contract interface
# Agent finds and displays relevant contract code sections

ChatGPT said:

Awesome debrief — this is a great fit for your brand. Here’s a tight execution plan to crush the assessment and turn it into a portfolio‑quality artifact.
What to Build (at a glance)

    CLI “Agent” client in Rust using RIG for tool‑calling and prompt orchestration.
    Rig Documentation
    Rig

    MCP server in Rust that exposes a small, safe toolbox over Foundry features like balance, send, call, and simple Uniswap helpers. Use the official MCP spec 2025‑06‑18 and Rust SDK examples as the baseline.
    Model Context Protocol+1
    GitHub+1

    Local forked chain via Anvil for deterministic demos and the 10 funded test accounts.
    getfoundry.sh+1

    Golden tasks to prove E2E:

        “send 1 ETH from Alice to Bob”

        “How much USDC does Alice have?”

        “Is Uniswap V2 Router deployed at 0x…2488D?” (address is canonical).
        docs.uniswap.org+1
        Ethereum (ETH) Blockchain Explorer

    Bonus: a small RAG sidecar for Uniswap docs and Router interfaces, plus one external DeFi API tool.

Suggested repo layout

foameo-assessment/
  README.md
  Cargo.toml
  crates/
    agent_cli/           # RIG-based REPL client
    mcp_server/          # MCP tools over Foundry
    foundry_adapter/     # thin wrapper around cast/anvil JSON-RPC
    domain/              # shared types, prompts, parsers
    rag_client/          # optional: local-docs RAG sidecar

Milestones and how to demo them
M1 — Environment up

    Install Foundry and RIG. Start a forked mainnet:

    anvil --fork-url https://reth-ethereum.ithaca.xyz/rpc

    Anvil exposes funded test accounts and 8545 RPC. Your README should show the exact “Alice/Bob” addresses that Foundry itself documents for forks.
    getfoundry.sh+1

M2 — MCP server with two tools

Implement balance(who: String) and send(from: String, to: String, eth: String) tools. Follow the MCP 2025‑06‑18 schema for tool definitions and the Rust SDK’s tool macro style.
Model Context Protocol
GitHub

    Under the hood, call Ethereum JSON‑RPC directly or shell out to the Foundry library routines equivalent to cast balance and cast send. Start by mirroring CLI semantics from the Foundry docs so your tool messages match what devs already know.
    getfoundry.sh+1

Acceptance checks

    > send 1 ETH from Alice to Bob returns tx hash and receipt.

    > How much ETH does Bob have? shows increased balance.

M3 — Address resolution and validation

    Support ENS or raw hex for who. Resolve via provider before sending.

    Basic guardrails: checksum enforcement, chain id check, gas limit caps.

M4 — “Core three” prompts in the client

Build a RIG REPL that:

    Parses NL into a small intent enum.

    Calls MCP tools.

    Prints structured JSON responses and a friendly summary.
    RIG’s “agents + tools” model is the right abstraction for this. Keep the agent model‑agnostic so you can drop in Claude easily.
    Rig Documentation

Acceptance checks

    > Is Uniswap V2 Router (0x7a25…2488D) deployed?
    Do an eth_getCode or simple cast code and return “deployed: true”. Canonical V2 Router02 address is 0x7a250d56…2488D.
    docs.uniswap.org+1
    Ethereum (ETH) Blockchain Explorer

M5 — Token balances and USDC symbol lookup

    Add an ERC‑20 balanceOf helper that takes (address, token) where token can be a checksum address or a known symbol from a tiny local map (seed USDC mainnet address).

    Use Anvil fork so the USDC state is realistic. Show balanceOf(Alice, USDC).

M6 — Bonus: Uniswap swap intent

    Add an MCP tool uniswap_v2_swap_exact_eth_for_tokens(amount_eth, token_out, min_out_bps, deadline_s) that builds calldata for swapExactETHForTokens(uint256,address[],address,uint256), routes via WETH -> token, and sends from Alice.

    Optionally enrich with quick price discovery through a public DeFi API tool or a web search tool to find Router addresses and ABI when missing. Keep it deterministic by allowing a --dry-run simulation first.
    getfoundry.sh

M7 — Bonus: Client‑side RAG for Uniswap docs

    Ingest Uniswap V2 docs and the Router02 interface text into a local store.

    Use a small local embedding model and a simple vector index to answer:

        “How do I calculate slippage for Uniswap V3?”

        “exactInput vs exactOutput?”

        “Show me the SwapRouter interface”
        The agent pulls top chunks into context before answering. Use the official docs text where possible.
        docs.uniswap.org

Key design choices Cecilia will care about

    Type‑safe tool surface
    Define a Toolbox trait with explicit structs for inputs and outputs. This makes the MCP tools self‑documenting and stable against model hallucination. Spec and SDK examples show the patterns.
    Model Context Protocol
    GitHub

    Deterministic simulation first
    Every state‑changing call has a simulate: bool and forkBlock: Option<u64>. Fail safe if preconditions are not met. Foundry materials explain forking and reproducible runs.
    getfoundry.sh

    Zero‑trust prompt wiring
    In the RIG client, the agent never constructs raw tx on its own. It only asks the MCP tools to do so. This matches the “OS syscalls for agents” mental model and aligns with what you discussed about an agentic runtime. (RIG agent abstractions fit this well.)
    Rig Documentation

    Cache and discovery
    Add an LRU cache for verified contracts and ABIs. Fall back to Etherscan read‑only when missing. Keep an interface to plug in L2Beat‑style discovery later. Cite L2 addresses when you mention Router02 to show diligence.
    Ethereum (ETH) Blockchain Explorer

    Extensibility
    Leave hooks for BAML prompt schemas on the client side, even if you do not ship them in v1. The repo is active and focused on schema‑first prompt engineering, which aligns with their “vibe” on fast iterations.
    GitHub+1

Concrete commands to put in your README
Start the fork

anvil --fork-url https://reth-ethereum.ithaca.xyz/rpc
# or any mainnet RPC; Anvil will show 10 funded accounts

getfoundry.sh
Query balance through the MCP tool

cargo run -p mcp_server  # starts MCP server on localhost
# in another shell
cargo run -p agent_cli
> balance vitalik.eth
> balance 0xf39f...92266

(Behavior equivalent to cast balance.)
getfoundry.sh
Send 1 ETH from Alice to Bob

> send 1 ETH from 0xf39f...92266 to 0x7099...9C8
# returns tx hash and receipt summary

Check Uniswap Router deployed

> code 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D
# deployed: true if bytecode len > 0

(That address is the canonical Uniswap V2 Router02 on mainnet.)
docs.uniswap.org+1
What to hand in

    Short demo video: run the three golden tasks in the REPL, then one bonus call.

    README with:

        1‑page architecture diagram and component bullets

        Quickstart and exact commands

        List of tool guards and safety checks

        Tradeoffs and future work

    Bench notes: how you used AI to scaffold code, where it helped, and what you fixed.

Talk track for follow‑ups

    Tie back to your tangible result story: you built an AI agent that reduced onboarding from 60 days to 1 day by reading complex docs and auto‑filling forms. Translate that into “credible, safe transaction crafting” on-chain.

    Stress that your design keeps model‑agnostic orchestration in the client, type‑safe syscalls in the server, and deterministic simulation before state changes.

    Name‑drop the right references when asked:

        RIG agent concepts and tool orchestration.
        Rig Documentation

        MCP 2025‑06‑18 spec, server examples, and the Rust SDK.
        Model Context Protocol
        GitHub+1

        Foundry anvil fork flow and cast parity.
        getfoundry.sh+1

        Canonical Uniswap addresses.
        docs.uniswap.org+1

        Optional: mention L2Beat discovery style, BAML schema prompts, LATTA inspiration if it comes up.

If you want, I can scaffold the Cargo workspace, add stubs for the MCP tools, and write the README you will screen‑share in your demo.
You said:
Instead of RIG, She says she's switcing to BAML. https://github.com/BoundaryML/baml So make that change
ChatGPT said:

Got it — let’s swap the client side from RIG to BAML while keeping the MCP + Foundry backbone intact.
Updated architecture (BAML-first)

             ┌─────────────────┐    MCP Protocol    ┌──────────────────┐
             │  BAML Client    │◄──────────────────►│   MCP Server     │
             │  (CLI REPL)     │                    │  (Rust)          │
             ├─────────────────┤                    ├──────────────────┤
User   ◄───► │ • .baml funcs   │                    │ • Foundry (cast) │
Claude ◄───► │ • type schemas  │                    │ • Tx generation  │
             │ • tool calls    │                    │ • Anvil fork     │
             └─────────────────┘                    └──────────────────┘
                      │                                       │
                      └─────────────── RPC/HTTP ──────────────┘
                              Forked Ethereum (Anvil)

    Client: BAML functions define your agent “API” with strict input/output schemas; you wire those into a tiny Rust (or TS) CLI that invokes MCP tools. BAML’s whole pitch is turning prompt engineering into schema engineering with strong types — perfect for “agentic OS syscalls.”
    GitHub
    BAML

    Server: Keep the MCP Rust server exposing a minimal, safe toolbox (balance, send, code, erc20_balanceOf, optional uniswap_v2_swap_exact_eth_for_tokens). Target the 2025‑06‑18 MCP spec and the Rust SDK patterns for tools.
    Model Context Protocol+1
    Docs.rs
    Crates

    Chain: Local mainnet fork via Anvil; parity with cast subcommands for easy verification.
    getfoundry.sh+1

Repo layout (revised)

foameo-assessment/
  README.md
  Cargo.toml
  crates/
    baml_client/         # CLI + .baml files + adapters to call MCP tools
    mcp_server/          # Rust MCP tools over Foundry
    foundry_adapter/     # thin wrapper: cast/json-rpc helpers
    domain/              # shared types (addresses, amounts, token map)
    rag_client/          # (bonus) Uniswap docs/code RAG sidecar
baml/
  agent.baml             # top-level prompts + function schemas
  tools.baml             # tool-call schemas that mirror MCP tools

What changes with BAML (practical steps)

    Define strongly-typed funcs in .baml

        function SendEth(from: Address, to: Address, amount_eth: Decimal) -> TxResult

        function GetErc20Balance(holder: Address, token: Token) -> Balance

        function IsDeployed(address: Address) -> Bool
        Keep the natural-language instruction concise; rely on types to prevent goofy outputs. BAML examples + VS Code extension help you validate quickly.
        GitHub

    Map BAML funcs → MCP tools

        In the CLI, the resolver takes BAML’s validated arguments and calls the MCP server’s balance, send, code, or erc20_balanceOf.

        Use JSON‑RPC or cast-equivalents under the hood (e.g., cast balance, cast call).
        getfoundry.sh
        登链社区

    Golden tasks (same UX, safer typing)

        send 1 ETH from Alice to Bob → SendEth → MCP send → anvil fork → tx hash back.

        How much USDC does Alice have? → GetErc20Balance (USDC address seeded) → cast call balanceOf → result.
        docs.hedera.com

        Is Uniswap V2 Router (0x7a25…2488D) deployed? → IsDeployed → eth_getCode/cast code. Canonical Router02 address confirmed in Uniswap docs.
        docs.uniswap.org+1

    Fork & accounts (unchanged)

        Document anvil --fork-url <mainnet RPC> and the 10 funded accounts in README; Foundry docs show mainnet forking workflow.
        getfoundry.sh

Toolbox (MCP server) you’ll expose

    balance(who: String) -> wei (mirror cast balance)
    getfoundry.sh

    code(addr: String) -> bytecode_len (simple deploy check)

    send(from: String, to: String, eth: String) -> TxResult

    erc20_balanceOf(token: String, holder: String) -> u256 (ABI balanceOf(address))
    登链社区

    Bonus: uniswap_v2_swap_exact_eth_for_tokens(amount_eth, token_out, min_out_bps, deadline_s) that prepares calldata for swapExactETHForTokens(uint256,address[],address,uint256) and submits; router address = 0x7a250d56…F2488D on mainnet. Ship a --simulate first.
    docs.uniswap.org

CLI flow (BAML-driven)

    Parse NL → BAML function (BAML handles structuring + validation).

    Echo the typed call (great DX signal for Cecilia/DevRel) then invoke MCP.

    Pretty print a compact result and the raw JSON.

Guardrails & “agentic OS” goodness

    Type safety at the edge: BAML schemas prevent ambiguous amounts/addresses before any on-chain action.
    GitHub
    BAML

    Deterministic first: every state‑changing tool supports simulate=true against an Anvil fork and optional forkBlock.
    getfoundry.sh

    Canonical refs: Hard‑code a tiny token map (USDC) and the Uniswap V2 Router02 address; fetch others later.
    docs.uniswap.org

    MCP spec compliance: use the 2025‑06‑18 tool metadata/schema shape so any Claude‑style client understands the tools cleanly.
    Model Context Protocol+1

README snippets to include

Start the fork

anvil --fork-url https://eth.merkle.io

(Any mainnet RPC works; Foundry quickstart shows alternatives.)
getfoundry.sh
GitHub

Run server & client

cargo run -p mcp_server
cargo run -p baml_client
> send 1 ETH from 0xf39f...92266 to 0x7099...9C8
> erc20_balance 0xa0b8... (USDC) 0xf39f...92266
> is_deployed 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D

(Uniswap V2 Router02 canonical address above.)
docs.uniswap.org
Bonus: Client-side RAG (unchanged)

Keep the plan: ingest Uniswap docs + Router interfaces, embed locally, surface top chunks into the BAML function context to answer: slippage (V3), exactInput vs exactOutput, SwapRouter interface. Start with the official v2/v3 docs for ground truth.
docs.uniswap.org

If you want, I’ll scaffold:

    a baml/agent.baml with the three golden functions + schemas,

    the CLI runner that compiles BAML → calls MCP,

    the mcp_server with the five tools above,

    and a crisp README with exact demo commands.

You said:
What is the point of this thing? what problem does it solve?
ChatGPT said:

The “thing” — the Foameo Labs agentic OS — is meant to solve a real and growing gap between AI agents and blockchain infrastructure.

The core problem:
Right now, if you want an AI agent to interact with a blockchain (send a transaction, query a contract, run a swap), you have to glue together a bunch of disparate pieces:

    Blockchain SDKs or RPC calls with custom code

    Contract ABI lookups and parsing

    Language model orchestration logic

    Tooling for safety checks, simulation, and repeatable execution

That’s slow to build, hard to maintain, and easy to get wrong — especially if you’re not both a blockchain developer and an AI engineer.

What this project does:

    Creates an “operating system” for AI agents on-chain — exposing blockchain actions as safe, well-defined system calls so an AI can transact without knowing low-level RPC/ABI details.

    Standardizes how LLMs talk to blockchains — giving them plug-and-play tools to do things like query balances, send transactions, or interact with DeFi protocols, all in a type-safe and simulation-first way.

    Makes it model-agnostic and modular — so you can use Claude, OpenAI, local LLMs, etc., without rewriting blockchain interaction code.

    Bridges natural language commands → on-chain execution in a single REPL/workflow.

    Positions itself like a LangChain for crypto, but with production-grade transaction crafting and safety built in.

Who it’s for:

    Developers who want to integrate AI agents into wallets, DeFi apps, prediction markets, or other on-chain systems.

    Companies who want AI-driven automation in blockchain environments without reinventing the wheel every time.

If you’d like, I can also write you a short, two-sentence “problem + solution” statement that you can use in interviews or the README for your assessment.
You said:
How does it solve the problem of being model agnostic
ChatGPT said:

Great question. Short version: it makes the model replaceable by pushing all blockchain “doing” into a typed tool layer and letting the LLM do only “deciding.”
How it stays model-agnostic

    Clear contract over MCP

        All on-chain actions live behind MCP tools with JSON-schema’d inputs and outputs.

        Any LLM that can call tools (Claude, OpenAI, local Llama) just picks a tool and fills fields. The server executes, not the model.

    Types first with BAML

        You define functions like SendEth(from, to, amount_eth) -> TxResult in .baml.

        BAML validates and normalizes arguments before anything touches the chain, so prompts are portable across providers.

    Thin “LLM adapter” layer

        The client has a tiny interface like ChatProvider { chat(), call_tools() }.

        Swap Claude for OpenAI or a local model by changing only this adapter and API key wiring. No tool or chain code changes.

    Deterministic execution on the server

        Foundry/JSON-RPC runs every call (simulate first, then send).

        The model never crafts raw transactions. It only proposes a structured tool call. Execution is identical regardless of model.

    Strict schemas and guards

        Addresses, amounts, deadlines, slippage are strongly typed and range-checked.

        Safety checks (allow lists, gas caps, forked-block simulation) live in the server, not in prompts.

    Portable context strategy

        Client-side RAG feeds the same small, relevant snippets (docs, ABIs) to any model.

        Embeddings are pluggable too: local model or vendor API behind a simple trait.

    Black-box tests for compliance

        A test suite hits the MCP server with canonical JSON tool calls for tasks like “send 1 ETH” or “check Router deployed.”

        If a new model can produce the same JSON, it will pass the suite without code changes.

One-liner you can say in interviews

“We keep the LLM as a planner that emits typed tool calls, and we keep all blockchain side effects in a deterministic server. Because the contract is the schema, not the prompt, we can swap models without touching the on-chain logic.”

