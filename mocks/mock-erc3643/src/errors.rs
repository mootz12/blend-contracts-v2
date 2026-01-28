//! Error definitions for the Mock ERC-3643 Token

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ERC3643Error {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// Token is paused
    TokenPaused = 3,
    /// Wallet is frozen
    FrozenWallet = 4,
    /// Insufficient balance
    InsufficientBalance = 5,
    /// Receiver identity not verified
    InvalidIdentity = 6,
    /// Compliance check failed
    ComplianceFailure = 7,
    /// Insufficient allowance
    InsufficientAllowance = 8,
    /// Allowance has expired
    AllowanceExpired = 9,
    /// Array lengths do not match
    ArrayLengthMismatch = 10,
    /// Insufficient frozen tokens to unfreeze
    InsufficientFrozenTokens = 11,
    /// Cannot transfer to self
    SelfTransfer = 12,
}
