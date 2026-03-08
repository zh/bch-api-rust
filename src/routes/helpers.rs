use crate::clients::ApiError;
use bitcoincash_addr::{Address, Network};
use serde_json::Value;

pub const MAX_ARRAY_SIZE: usize = 24;

#[derive(serde::Deserialize)]
pub struct VerboseQuery {
    pub verbose: Option<bool>,
}

/// Validate a hex hash (txid, blockhash) is exactly 64 hex characters.
pub fn validate_hash(hash: &str, label: &str) -> Result<(), ApiError> {
    if hash.is_empty() {
        return Err(ApiError::InvalidInput(format!("{label} is required")));
    }
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::InvalidInput(format!(
            "{label} must be a 64-character hex string"
        )));
    }
    Ok(())
}

/// Validate and normalize a BCH address. Returns canonical cashaddr form.
pub fn validate_address(address: &str) -> Result<String, ApiError> {
    if address.is_empty() {
        return Err(ApiError::InvalidInput("address is required".into()));
    }

    let addr = Address::decode(address).map_err(|_| {
        ApiError::InvalidInput(format!(
            "Invalid BCH address. Double check your address is valid: {address}"
        ))
    })?;

    if addr.network != Network::Main {
        return Err(ApiError::InvalidInput(
            "Invalid network. Only mainnet addresses are supported.".into(),
        ));
    }

    Ok(addr.encode().unwrap_or_else(|_| address.to_string()))
}

/// Extract a JSON array from a body, validating it exists and respects max size.
pub fn validate_array(val: Option<&Value>, field: &str) -> Result<Vec<Value>, ApiError> {
    let arr = val.and_then(|v| v.as_array()).ok_or_else(|| {
        ApiError::InvalidInput(format!(
            "{field} needs to be an array. Use GET for single {field}."
        ))
    })?;

    if arr.is_empty() {
        return Err(ApiError::InvalidInput(format!("{field} array is empty")));
    }
    if arr.len() > MAX_ARRAY_SIZE {
        return Err(ApiError::InvalidInput("Array too large.".into()));
    }

    Ok(arr.clone())
}

/// Extract an array of strings from a JSON body field.
pub fn extract_string_array(body: &Value, field: &str) -> Result<Vec<String>, ApiError> {
    let arr = validate_array(body.get(field), field)?;
    arr.iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| ApiError::InvalidInput(format!("{field} must contain strings")))
        })
        .collect()
}

/// Extract an array of hex hashes from a JSON body field.
pub fn extract_hash_array(body: &Value, field: &str) -> Result<Vec<String>, ApiError> {
    let strings = extract_string_array(body, field)?;
    for s in &strings {
        validate_hash(s, field)?;
    }
    Ok(strings)
}

/// Extract an array of BCH addresses from a JSON body field, validating each.
pub fn extract_address_array(body: &Value, field: &str) -> Result<Vec<String>, ApiError> {
    let strings = extract_string_array(body, field)?;
    strings.iter().map(|s| validate_address(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_hash_valid() {
        let hash = "000000000000000002e63058c9bda37ad72fc98e3154cce2de15e76a33f9e71e";
        assert!(validate_hash(hash, "txid").is_ok());
    }

    #[test]
    fn validate_hash_empty() {
        let err = validate_hash("", "txid").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("is required")));
    }

    #[test]
    fn validate_hash_wrong_length() {
        let err = validate_hash("abcdef", "blockhash").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("64-character")));
    }

    #[test]
    fn validate_hash_non_hex() {
        let bad = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let err = validate_hash(bad, "txid").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(_)));
    }

    #[test]
    fn validate_address_empty() {
        let err = validate_address("").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("is required")));
    }

    #[test]
    fn validate_address_invalid() {
        let err = validate_address("notanaddress").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("Invalid BCH address")));
    }

    #[test]
    fn validate_array_not_array() {
        let val = json!("string");
        let err = validate_array(Some(&val), "hashes").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("needs to be an array")));
    }

    #[test]
    fn validate_array_empty() {
        let val = json!([]);
        let err = validate_array(Some(&val), "txids").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("empty")));
    }

    #[test]
    fn validate_array_too_large() {
        let val: Value = (0..25).collect::<Vec<i32>>().into();
        let err = validate_array(Some(&val), "items").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(msg) if msg.contains("too large")));
    }

    #[test]
    fn validate_array_ok() {
        let val = json!(["a", "b"]);
        let result = validate_array(Some(&val), "items").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn extract_string_array_ok() {
        let body = json!({"hexes": ["aa", "bb"]});
        let result = extract_string_array(&body, "hexes").unwrap();
        assert_eq!(result, vec!["aa", "bb"]);
    }

    #[test]
    fn extract_hash_array_validates() {
        let body = json!({"txids": ["short"]});
        let err = extract_hash_array(&body, "txids").unwrap_err();
        assert!(matches!(err, ApiError::InvalidInput(_)));
    }
}
