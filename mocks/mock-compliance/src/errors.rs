//! Error definitions for the Mock Compliance Contract

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ComplianceError {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// No token is bound to this compliance contract
    NoTokenBound = 3,
    /// Transfer limit exceeded
    TransferLimitExceeded = 4,
}
