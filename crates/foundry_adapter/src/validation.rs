use ethers_core::types::Address as EthAddress;
use ethers_core::utils::to_checksum;
use std::str::FromStr;

/// Normalize an address string to lowercase for consistent comparison
pub fn normalize(address: &str) -> String {
    address.to_lowercase()
}

/// Validate and normalize an address with EIP-55 checksum
#[allow(dead_code)]
pub fn validate_and_normalize_address(address: &str) -> Result<String, String> {
    // Parse the address
    let addr = EthAddress::from_str(address)
        .map_err(|_| format!("Invalid address format: {}", address))?;
    
    // Convert to checksum format
    let checksum = to_checksum(&addr, None);
    
    // Validate that the original address matches the checksum (case-insensitive)
    if address.to_lowercase() != checksum.to_lowercase() {
        return Err(format!(
            "Address checksum mismatch. Expected: {}, Got: {}",
            checksum, address
        ));
    }
    
    Ok(checksum)
}

/// Check if an address is a valid EIP-55 checksum address
#[allow(dead_code)]
pub fn is_valid_checksum_address(address: &str) -> bool {
    validate_and_normalize_address(address).is_ok()
}

/// Extract address from various formats (with or without 0x prefix)
#[allow(dead_code)]
pub fn parse_address(input: &str) -> Result<EthAddress, String> {
    let clean_input = if input.starts_with("0x") {
        input.to_string()
    } else {
        format!("0x{}", input)
    };
    
    EthAddress::from_str(&clean_input)
        .map_err(|_| format!("Invalid address: {}", input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("0x1234567890123456789012345678901234567890"), "0x1234567890123456789012345678901234567890");
        assert_eq!(normalize("0X1234567890123456789012345678901234567890"), "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_validate_checksum() {
        // Valid checksum address
        assert!(is_valid_checksum_address("0x5B38Da6a701c568545dCfcB03FcB875f56beddC4"));
        
        // Invalid format
        assert!(!is_valid_checksum_address("not-an-address"));
    }

    #[test]
    fn test_parse_address() {
        // With 0x prefix
        assert!(parse_address("0x1234567890123456789012345678901234567890").is_ok());
        
        // Without 0x prefix
        assert!(parse_address("1234567890123456789012345678901234567890").is_ok());
        
        // Invalid
        assert!(parse_address("invalid").is_err());
    }
}
