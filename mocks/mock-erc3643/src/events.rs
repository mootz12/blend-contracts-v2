//! Events for the Mock ERC-3643 Token

use soroban_sdk::{contractevent, Address, Env};

/// Emitted when tokens are transferred
#[contractevent]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

/// Emitted when an allowance is approved
#[contractevent]
pub struct Approval {
    pub from: Address,
    pub spender: Address,
    pub amount: i128,
    pub expiration: u32,
}

/// Emitted when tokens are minted
#[contractevent]
pub struct Mint {
    pub to: Address,
    pub amount: i128,
}

/// Emitted when tokens are burned
#[contractevent]
pub struct Burn {
    pub from: Address,
    pub amount: i128,
}

/// Emitted when the token is paused
#[contractevent]
pub struct Paused {
    pub paused: bool,
}

/// Emitted when the token is unpaused
#[contractevent]
pub struct Unpaused {
    pub unpaused: bool,
}

/// Emitted when an address is frozen or unfrozen
#[contractevent]
pub struct AddressFrozen {
    pub user: Address,
    pub frozen: bool,
}

/// Emitted when tokens are partially frozen for an address
#[contractevent]
pub struct TokensFrozen {
    pub user: Address,
    pub amount: i128,
}

/// Emitted when tokens are partially unfrozen for an address
#[contractevent]
pub struct TokensUnfrozen {
    pub user: Address,
    pub amount: i128,
}

/// Emitted when the identity registry is set
#[contractevent]
pub struct IdentityRegistryAdded {
    pub registry: Address,
}

/// Emitted when the compliance contract is set
#[contractevent]
pub struct ComplianceAdded {
    pub compliance: Address,
}

/// Emitted when tokens are recovered to a new wallet
#[contractevent]
pub struct RecoverySuccess {
    pub lost_wallet: Address,
    pub new_wallet: Address,
}

/// Emitted when an agent is added
#[contractevent]
pub struct AgentAdded {
    pub agent: Address,
}

/// Emitted when an agent is removed
#[contractevent]
pub struct AgentRemoved {
    pub agent: Address,
}

/// Emit transfer event
pub fn emit_transfer(e: &Env, from: Address, to: Address, amount: i128) {
    Transfer { from, to, amount }.publish(e);
}

/// Emit approval event
pub fn emit_approval(e: &Env, from: Address, spender: Address, amount: i128, expiration: u32) {
    Approval {
        from,
        spender,
        amount,
        expiration,
    }
    .publish(e);
}

/// Emit mint event
pub fn emit_mint(e: &Env, to: Address, amount: i128) {
    Mint { to, amount }.publish(e);
}

/// Emit burn event
pub fn emit_burn(e: &Env, from: Address, amount: i128) {
    Burn { from, amount }.publish(e);
}

/// Emit paused event
pub fn emit_paused(e: &Env) {
    Paused { paused: true }.publish(e);
}

/// Emit unpaused event
pub fn emit_unpaused(e: &Env) {
    Unpaused { unpaused: true }.publish(e);
}

/// Emit address frozen event
pub fn emit_address_frozen(e: &Env, user: Address, frozen: bool) {
    AddressFrozen { user, frozen }.publish(e);
}

/// Emit tokens frozen event
pub fn emit_tokens_frozen(e: &Env, user: Address, amount: i128) {
    TokensFrozen { user, amount }.publish(e);
}

/// Emit tokens unfrozen event
pub fn emit_tokens_unfrozen(e: &Env, user: Address, amount: i128) {
    TokensUnfrozen { user, amount }.publish(e);
}

/// Emit identity registry added event
pub fn emit_identity_registry_added(e: &Env, registry: Address) {
    IdentityRegistryAdded { registry }.publish(e);
}

/// Emit compliance added event
pub fn emit_compliance_added(e: &Env, compliance: Address) {
    ComplianceAdded { compliance }.publish(e);
}

/// Emit recovery success event
pub fn emit_recovery_success(e: &Env, lost_wallet: Address, new_wallet: Address) {
    RecoverySuccess {
        lost_wallet,
        new_wallet,
    }
    .publish(e);
}

/// Emit agent added event
pub fn emit_agent_added(e: &Env, agent: Address) {
    AgentAdded { agent }.publish(e);
}

/// Emit agent removed event
pub fn emit_agent_removed(e: &Env, agent: Address) {
    AgentRemoved { agent }.publish(e);
}
