# Foameo Labs EVM Toolbox Demo

This demo showcases the agentic MCP toolbox for EVM operations with natural language interface.

## Prerequisites

1. **Anvil Fork Running**:
   ```bash
   anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY --fork-block-number latest
   ```

2. **Environment Setup**:
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   export RPC_URL=http://127.0.0.1:8545
   ```

3. **Build Workspace**:
   ```bash
   cargo build
   ```

## Demo Script

### Golden Task 1: Get ETH Balance

**Command**:
```bash
cargo run -p baml_client -- -q "What's vitalik.eth's balance?"
```

**Expected Output**:
```
Processing query: What's vitalik.eth's balance?
MCP server: http://localhost:3000
Selected function: balance
Function validated: Get ETH balance of an address or ENS name
Function: balance
Response: {
  "balance": "1234567890123456789"
}
```

**What Happens**:
1. CLI parses natural language query
2. BAML agent selects `GetEthBalance` function
3. ENS name "vitalik.eth" is resolved to address
4. MCP server queries blockchain for balance
5. Returns balance in wei as JSON

---

### Golden Task 2: Check Code Deployment

**Command**:
```bash
cargo run -p baml_client -- -q "Check if 0x0000000000000000000000000000000000000000 has deployed code"
```

**Expected Output**:
```
Processing query: Check if 0x0000000000000000000000000000000000000000 has deployed code
MCP server: http://localhost:3000
Selected function: code
Function validated: Check if address has deployed code
Function: code
Response: {
  "deployed": false,
  "bytecode_len": 0
}
```

**What Happens**:
1. CLI parses query and selects `IsDeployed` function
2. Address is validated and normalized
3. MCP server queries blockchain for code at address
4. Returns deployment status and bytecode length

---

### Golden Task 3: Send ETH (Simulation)

**Command**:
```bash
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob"
```

**Expected Output**:
```
Processing query: Send 0.1 ETH from Alice to Bob
MCP server: http://localhost:3000
Selected function: send
Function validated: Send ETH from one address to another
Function: send
Response: {
  "tx_hash": "",
  "success": true
}
```

**What Happens**:
1. CLI parses query and selects `SendEth` function
2. Alice/Bob aliases resolve to Anvil account addresses
3. MCP server simulates transaction first (simulate=true)
4. Gas estimation and validation performed
5. Returns simulation result (no actual transaction sent)

---

### Bonus Task: ERC-20 Balance Check

**Command**:
```bash
cargo run -p baml_client -- -q "Get USDC balance for 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
```

**Expected Output**:
```
Processing query: Get USDC balance for 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
MCP server: http://localhost:3000
Selected function: erc20_balance_of
Function validated: Get ERC-20 token balance for holder
Function: erc20_balance_of
Response: {
  "amount": "0"
}
```

**What Happens**:
1. CLI parses query and selects `GetErc20Balance` function
2. USDC token address is hardcoded (0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)
3. MCP server calls ERC-20 `balanceOf` function
4. Returns token balance in smallest units

---

## Error Handling Examples

### Invalid Address
```bash
cargo run -p baml_client -- -q "Check if invalid-address has deployed code"
```
**Expected**: Error with "invalid address" message

### Chain ID Mismatch
```bash
# Set wrong chain ID in adapter
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob"
```
**Expected**: Error with "unexpected chain id" message

### Gas Cap Exceeded
```bash
# Set very low gas cap
cargo run -p baml_client -- -q "Send 0.1 ETH from Alice to Bob"
```
**Expected**: Error with "estimated gas exceeds cap" message

---

## Architecture Highlights

### Type Safety
- All inputs/outputs are strongly typed
- BAML schemas enforce function contracts
- Domain models prevent invalid states

### Simulation First
- All state-changing operations simulate before execution
- Gas estimation and validation performed upfront
- Clear separation between simulation and execution

### Zero-Trust Design
- LLM never constructs raw transactions
- All blockchain interactions go through MCP tools
- Structured error handling with actionable messages

### Extensibility
- Pluggable LLM providers (Anthropic, Mock)
- Trait-based interfaces for contract discovery
- Feature-flagged bonus functionality

---

## Acceptance Criteria Met

✅ **Type-safe MCP tools** with structured errors  
✅ **BAML-driven CLI** with natural language parsing  
✅ **Deterministic simulation** for all state changes  
✅ **ENS resolution** and address validation  
✅ **Comprehensive test coverage** including E2E tests  
✅ **Error handling** with clear, actionable messages  
✅ **Documentation** with architecture and usage examples  

---

## Next Steps

1. **Enable Bonus Features**: Set `BONUS=1` environment variable
2. **Add More Tools**: Implement Uniswap swap, external API lookup
3. **RAG Integration**: Add documentation retrieval capabilities
4. **Production Deployment**: Configure for production networks
5. **Performance Optimization**: Add caching and connection pooling
