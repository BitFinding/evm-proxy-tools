use alloy_primitives::{Address, Bytes};
use crate::{ProxyType, ProxyDispatch, Result, errors::ProxyError};
use super::DetectionStrategy;

/// Detector for static bytecode analysis
#[derive(Default)]
pub struct StaticDetector;

impl DetectionStrategy for StaticDetector {
    fn detect(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        if code.is_empty() {
            return Ok(None);
        }

        // First try EIP-1167
        if let Some(result) = self.detect_minimal_proxy(code)? {
            return Ok(Some(result));
        }
        
        // Then try EIP-7511
        if let Some(result) = self.detect_eip7511(code)? {
            return Ok(Some(result));
        }

        // Finally try EIP-3448
        if let Some(result) = self.detect_eip3448(code)? {
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn name(&self) -> &'static str {
        "StaticDetector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn test_minimal_proxy_detection() {
        let detector = StaticDetector::default();
        
        // Test EIP-1167 long format
        let code = hex!("363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3");
        let result = detector.detect(&code.into()).unwrap();
        assert!(matches!(
            result,
            Some((ProxyType::EIP_1167, ProxyDispatch::Static(_)))
        ));

        // Test invalid code
        let invalid_code = hex!("1234");
        assert!(detector.detect(&invalid_code.into()).unwrap().is_none());
    }

    #[test]
    fn test_eip7511_detection() {
        let detector = StaticDetector::default();
        
        // Test EIP-7511 long format
        let code = hex!("365f5f375f5f365f73bebebebebebebebebebebebebebebebebebebebe5af43d5f5f3e5f3d91602a57fd5bf3");
        let result = detector.detect(&code.into()).unwrap();
        assert!(matches!(
            result,
            Some((ProxyType::EIP_7511, ProxyDispatch::Static(_)))
        ));
    }

    #[test]
    fn test_empty_code() {
        let detector = StaticDetector::default();
        assert!(detector.detect(&Bytes::new()).unwrap().is_none());
    }
}

impl StaticDetector {
    #[inline(always)]
    fn extract_minimal_contract<const ADDR_SIZE: usize>(
        code: &[u8],
        min_size: usize,
        first_part: &[u8],
        second_part: &[u8]
    ) -> Option<Address> {
        let second_start = first_part.len() + ADDR_SIZE;
        if code.len() >= min_size 
            && &code[0..first_part.len()] == first_part 
            && &code[second_start..second_start + second_part.len()] == second_part {
            
            let addr = &code[first_part.len()..second_start];
            if ADDR_SIZE == 16 {
                let mut addr_vec = vec![0; 20];
                addr_vec[4..].copy_from_slice(addr);
                Some(Address::from_slice(&addr_vec))
            } else {
                Some(Address::from_slice(addr))
            }
        } else {
            None
        }
    }

    fn detect_minimal_proxy(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        // EIP-1167 patterns
        const EIP_1167_FIRST: &[u8] = &hex_literal::hex!("363d3d373d3d3d363d73");
        const EIP_1167_SECOND: &[u8] = &hex_literal::hex!("5af43d82803e903d91602b57fd5bf3");
        const EIP_1167_SHORT_FIRST: &[u8] = &hex_literal::hex!("363d3d373d3d3d363d6f");
        
        // Try long format first
        if let Some(addr) = Self::extract_minimal_contract::<20>(
            code, 
            45, 
            EIP_1167_FIRST, 
            EIP_1167_SECOND
        ) {
            return Ok(Some((ProxyType::EIP_1167, ProxyDispatch::Static(addr))));
        }
        
        // Then try short format
        if let Some(addr) = Self::extract_minimal_contract::<16>(
            code,
            41,
            EIP_1167_SHORT_FIRST,
            EIP_1167_SECOND
        ) {
            return Ok(Some((ProxyType::EIP_1167, ProxyDispatch::Static(addr))));
        }
        
        Ok(None)
    }

    fn detect_eip7511(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        const EIP_7511_LONG: &[u8] = &hex_literal::hex!("365f5f375f5f365f73");
        const EIP_7511_SHORT: &[u8] = &hex_literal::hex!("365f5f375f5f365f6f");
        
        if let Some(addr) = self.extract_address(code, EIP_7511_LONG, 20)? {
            return Ok(Some((ProxyType::EIP_7511, ProxyDispatch::Static(addr))));
        }
        
        if let Some(addr) = self.extract_address(code, EIP_7511_SHORT, 16)? {
            return Ok(Some((ProxyType::EIP_7511, ProxyDispatch::Static(addr))));
        }
        
        Ok(None)
    }

    fn detect_eip3448(&self, code: &Bytes) -> Result<Option<(ProxyType, ProxyDispatch)>> {
        const EIP_3448_LONG: &[u8] = &hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d73");
        const EIP_3448_SHORT: &[u8] = &hex_literal::hex!("363d3d373d3d3d3d60368038038091363936013d6f");
        
        if let Some(addr) = self.extract_address(code, EIP_3448_LONG, 20)? {
            return Ok(Some((ProxyType::EIP_3448, ProxyDispatch::Static(addr))));
        }
        
        if let Some(addr) = self.extract_address(code, EIP_3448_SHORT, 16)? {
            return Ok(Some((ProxyType::EIP_3448, ProxyDispatch::Static(addr))));
        }
        
        Ok(None)
    }

    fn extract_address(&self, code: &[u8], pattern: &[u8], addr_size: usize) -> Result<Option<Address>> {
        if code.len() < pattern.len() + addr_size {
            return Ok(None);
        }

        if !code.starts_with(pattern) {
            return Ok(None);
        }

        let addr_start = pattern.len();
        let addr_end = addr_start + addr_size;
        
        if addr_end > code.len() {
            return Err(ProxyError::InvalidBytecode {
                address: Address::ZERO,
                reason: format!("Expected address of size {} but found {}", addr_size, code.len() - pattern.len())
            });
        }

        let addr_slice = &code[addr_start..addr_end];
        
        // Validate address bytes
        if addr_slice.iter().all(|&b| b == 0) {
            return Err(ProxyError::InvalidBytecode {
                address: Address::ZERO,
                reason: "Implementation address cannot be zero".into()
            });
        }

        let addr = if addr_size == 16 {
            let mut addr_vec = vec![0; 20];
            addr_vec[4..].copy_from_slice(addr_slice);
            Address::from_slice(&addr_vec)
        } else {
            Address::from_slice(addr_slice)
        };

        Ok(Some(addr))
    }
}
