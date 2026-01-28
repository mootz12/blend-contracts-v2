//! Events for the Mock Compliance Contract

use soroban_sdk::{contractevent, Address, Env};

/// Emitted when a token is bound to this compliance contract
#[contractevent]
pub struct TokenBound {
    pub token: Address,
}

/// Emitted when a token is unbound from this compliance contract
#[contractevent]
pub struct TokenUnbound {
    pub token: Address,
}

/// Emitted when a transfer limit is set for a user
#[contractevent]
pub struct TransferLimitSet {
    pub user: Address,
    pub limit: i128,
}

/// Emitted when tokens are transferred (callback from token)
#[contractevent]
pub struct Transferred {
    pub to: Address,
    pub amount: i128,
}

/// Emitted when tokens are created (minted)
#[contractevent]
pub struct Created {
    pub to: Address,
    pub amount: i128,
}

/// Emitted when tokens are destroyed (burned)
#[contractevent]
pub struct Destroyed {
    pub from: Address,
    pub amount: i128,
}

/// Emitted when transferred amount is reset
#[contractevent]
pub struct TransferredReset {
    pub user: Address,
}

pub fn emit_token_bound(e: &Env, token: Address) {
    TokenBound { token }.publish(e);
}

pub fn emit_token_unbound(e: &Env, token: Address) {
    TokenUnbound { token }.publish(e);
}

pub fn emit_transfer_limit_set(e: &Env, user: Address, limit: i128) {
    TransferLimitSet { user, limit }.publish(e);
}

pub fn emit_transferred(e: &Env, to: Address, amount: i128) {
    Transferred { to, amount }.publish(e);
}

pub fn emit_created(e: &Env, to: Address, amount: i128) {
    Created { to, amount }.publish(e);
}

pub fn emit_destroyed(e: &Env, from: Address, amount: i128) {
    Destroyed { from, amount }.publish(e);
}

pub fn emit_transferred_reset(e: &Env, user: Address) {
    TransferredReset { user }.publish(e);
}
