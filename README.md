## Foameo Labs Assessment Workspace

Quickstart:

1) Start a mainnet fork (Anvil)
```
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
```

2) Build workspace
```
cargo build
```

3) Run server/client placeholders
```
cargo run -p mcp_server
cargo run -p baml_client
```

Environment variables (create `.env`):
```
RPC_URL=http://127.0.0.1:8545
ANTHROPIC_API_KEY=sk-ant-...
```

See `tasks/tasks-prd-agentic-mcp-toolbox-for-evm.md` for implementation plan.

