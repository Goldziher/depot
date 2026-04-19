use bytes::Bytes;

/// Compute blake3 hash of data, returning hex-encoded string.
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Verify data against an expected blake3 hex digest.
pub fn verify_blake3(data: &Bytes, expected: &str) -> bool {
    blake3_hex(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let data = Bytes::from_static(b"hello depot");
        let hash = blake3_hex(&data);
        assert!(verify_blake3(&data, &hash));
        assert!(!verify_blake3(
            &data,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
    }
}
