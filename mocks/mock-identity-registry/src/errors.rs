//! Error definitions for the Mock Identity Registry

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IdentityRegistryError {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// Identity not found in registry
    IdentityNotFound = 3,
    /// Identity already registered
    IdentityAlreadyRegistered = 4,
    /// Array lengths do not match for batch operations
    ArrayLengthMismatch = 5,
    /// Country not allowed
    CountryNotAllowed = 6,
}
