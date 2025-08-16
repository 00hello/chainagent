class SendEth {
  doc: "Send ETH from one address to another"
  
  input {
    from: string
    to: string  
    amount_eth: string
    simulate?: bool = true
    fork_block?: number
  }
  
  output {
    tx_hash: string
    success: bool
    gas_used?: number
  }
}

class GetErc20Balance {
  doc: "Get ERC-20 token balance for a holder address"
  
  input {
    token: string
    holder: string
  }
  
  output {
    amount: string
  }
}

class IsDeployed {
  doc: "Check if an address has deployed code"
  
  input {
    addr: string
  }
  
  output {
    deployed: bool
    bytecode_len: number
  }
}

class GetEthBalance {
  doc: "Get ETH balance of an address or ENS name"
  
  input {
    who: string
  }
  
  output {
    balance: string
  }
}
