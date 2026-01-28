//! Mock Predicate Library
//!
//! This library provides a mock implementation of transaction authorization
//! for use in the Blend protocol PoC. It simulates attestation-based authorization
//! where an authorized attester signs off on transactions.
//!
//! This is NOT a contract, just a library exposing mock authorization functions.

#![no_std]

use soroban_sdk::{Address, Bytes, Env, String, Val, Vec, contracttype};

/// Valid signature constant for testing - when used, authorization succeeds
pub const VALID_SIGNATURE: &[u8] = b"valid_signature";

/// Invalid signature constant for testing - when used, authorization fails
pub const INVALID_SIGNATURE: &[u8] = b"invalid_signature";

/// Attestation struct that bundles together an attestation's parameters for validation.
///
/// An attestation is a signed approval from an authorized attester.
/// The signature is created by signing the hash of the corresponding Statement.
/// The hash includes chain ID for domain separation to prevent cross-chain replay.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Attestation {
    /// The unique identifier for the attestation
    pub uuid: String,
    /// The timestamp by which the attestation must be executed
    pub expiration: u64,
    /// The address of the attester
    pub attester: Address,
    /// The signature from the attestation
    pub signature: Bytes,
}

/// Error type for predicate authorization failures
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum PredicateError {
    /// The attestation is not invalid
    Invalid,
}

/// Authorizes a transaction based on the provided attestation.
///
/// This mock implementation simply checks if the signature matches
/// the `VALID_SIGNATURE` constant. In a real implementation, this would
/// verify the cryptographic signature against the attester's public key.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `attestation` - The attestation containing the signature to verify
/// * `_args` - The transaction arguments (unused in mock)
/// * `_sender` - The sender address (unused in mock)
///
/// # Returns
/// * `Ok(())` - If the signature is valid
/// * `Err(PredicateError::InvalidSignature)` - If the signature is invalid
pub fn authorize_transaction(
    env: &Env,
    attestation: &Attestation,
    _args: &Vec<Val>,
    _sender: &Address,
) -> Result<(), PredicateError> {
    let valid_sig = Bytes::from_slice(env, VALID_SIGNATURE);

    if attestation.signature == valid_sig {
        Ok(())
    } else {
        Err(PredicateError::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_authorize_transaction_with_valid_signature() {
        let env = Env::default();

        let attestation = Attestation {
            uuid: String::from_str(&env, "test-uuid"),
            expiration: 1000,
            attester: Address::generate(&env),
            signature: Bytes::from_slice(&env, VALID_SIGNATURE),
        };

        let args = Vec::new(&env);
        let sender = Address::generate(&env);

        let result = authorize_transaction(&env, &attestation, &args, &sender);
        assert!(result.is_ok());
    }

    #[test]
    fn test_authorize_transaction_with_invalid_signature() {
        let env = Env::default();

        let attestation = Attestation {
            uuid: String::from_str(&env, "test-uuid"),
            expiration: 1000,
            attester: Address::generate(&env),
            signature: Bytes::from_slice(&env, INVALID_SIGNATURE),
        };

        let args = Vec::new(&env);
        let sender = Address::generate(&env);

        let result = authorize_transaction(&env, &attestation, &args, &sender);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PredicateError::Invalid);
    }

    #[test]
    fn test_authorize_transaction_with_random_signature() {
        let env = Env::default();

        let attestation = Attestation {
            uuid: String::from_str(&env, "test-uuid"),
            expiration: 1000,
            attester: Address::generate(&env),
            signature: Bytes::from_slice(&env, b"random_bytes"),
        };

        let args = Vec::new(&env);
        let sender = Address::generate(&env);

        let result = authorize_transaction(&env, &attestation, &args, &sender);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PredicateError::Invalid);
    }
}
