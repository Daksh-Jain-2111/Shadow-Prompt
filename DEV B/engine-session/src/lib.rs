use zeroize::Zeroizing;

/// The SessionMap represents our STM (Short-Term Memory).
/// It MUST NOT be persisted to disk to maintain Zero-Trust.
pub struct SessionMap {
    pub session_id: String,
    // Zeroizing<String> wraps the string and overwrites it with 0s when dropped.
    pub sensitive_input: Zeroizing<String>,
    pub redacted_output: String,
}

impl SessionMap {
    pub fn new(id: &str, input: String) -> Self {
        Self {
            session_id: id.to_string(),
            sensitive_input: Zeroizing::new(input),
            redacted_output: String::new(),
        }
    }
}

// Zero-Trust Test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroize_on_drop() {
        let ptr: *const u8;
        let len: usize;

        {
            let secret = Zeroizing::new(String::from("CONFIDENTIAL_PII_123"));
            ptr = secret.as_ptr();
            len = secret.len();
            
            // Prove data exists in memory while in scope
            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            assert_eq!(slice, b"CONFIDENTIAL_PII_123");
        } // 'secret' goes out of scope and is dropped here.

        // After drop, verify the memory at that location is now 0.
        let slice_after = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert!(slice_after.iter().all(|&b| b == 0), "Memory was NOT zeroed!");
    }
}
