//! Storage utilities for the Mock ERC-3643 Token

use crate::TokenMetadata;
use soroban_sdk::{contracttype, Address, Env, Map};

const ONE_DAY_LEDGERS: u32 = 17280;
const LEDGER_THRESHOLD: u32 = ONE_DAY_LEDGERS * 90;
const LEDGER_BUMP: u32 = ONE_DAY_LEDGERS * 120;

/// Storage keys for the ERC-3643 token
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Token metadata (name, symbol, decimals)
    Metadata,
    /// Identity registry address
    IdentityRegistry,
    /// Compliance contract address
    Compliance,
    /// Whether token is paused
    Paused,
    /// Total supply
    TotalSupply,
    /// Balance for an address
    Balance(Address),
    /// Allowance (from, spender)
    Allowance(Address, Address),
    /// Whether an address is frozen
    AddressFrozen(Address),
    /// Amount of frozen tokens for an address
    FrozenTokens(Address),
    /// Map of agent addresses
    Agents,
}

/// Allowance info with expiration
#[derive(Clone)]
#[contracttype]
pub struct AllowanceInfo {
    pub amount: i128,
    pub expiration_ledger: u32,
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Admin storage
pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

// Metadata storage
pub fn get_metadata(e: &Env) -> TokenMetadata {
    e.storage().instance().get(&DataKey::Metadata).unwrap()
}

pub fn set_metadata(e: &Env, metadata: &TokenMetadata) {
    e.storage().instance().set(&DataKey::Metadata, metadata);
}

// Identity registry storage
pub fn get_identity_registry(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::IdentityRegistry)
        .unwrap()
}

pub fn set_identity_registry(e: &Env, registry: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::IdentityRegistry, registry);
}

// Compliance storage
pub fn get_compliance(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Compliance).unwrap()
}

pub fn set_compliance(e: &Env, compliance: &Address) {
    e.storage().instance().set(&DataKey::Compliance, compliance);
}

// Paused storage
pub fn is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn set_paused(e: &Env, paused: bool) {
    e.storage().instance().set(&DataKey::Paused, &paused);
}

// Total supply storage
pub fn get_total_supply(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn set_total_supply(e: &Env, supply: i128) {
    e.storage().instance().set(&DataKey::TotalSupply, &supply);
}

// Balance storage
pub fn get_balance(e: &Env, address: &Address) -> i128 {
    let key = DataKey::Balance(address.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(e: &Env, address: &Address, balance: i128) {
    let key = DataKey::Balance(address.clone());
    e.storage().persistent().set(&key, &balance);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Allowance storage
pub fn get_allowance(e: &Env, from: &Address, spender: &Address) -> AllowanceInfo {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    e.storage().persistent().get(&key).unwrap_or(AllowanceInfo {
        amount: 0,
        expiration_ledger: 0,
    })
}

pub fn set_allowance(
    e: &Env,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    let info = AllowanceInfo {
        amount,
        expiration_ledger,
    };
    e.storage().persistent().set(&key, &info);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Address frozen storage
pub fn is_address_frozen(e: &Env, address: &Address) -> bool {
    let key = DataKey::AddressFrozen(address.clone());
    e.storage().persistent().get(&key).unwrap_or(false)
}

pub fn set_address_frozen(e: &Env, address: &Address, frozen: bool) {
    let key = DataKey::AddressFrozen(address.clone());
    e.storage().persistent().set(&key, &frozen);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Frozen tokens storage
pub fn get_frozen_tokens(e: &Env, address: &Address) -> i128 {
    let key = DataKey::FrozenTokens(address.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_frozen_tokens(e: &Env, address: &Address, amount: i128) {
    let key = DataKey::FrozenTokens(address.clone());
    e.storage().persistent().set(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Agent storage
pub fn get_agents(e: &Env) -> Map<Address, bool> {
    e.storage()
        .instance()
        .get(&DataKey::Agents)
        .unwrap_or(soroban_sdk::map![e])
}

pub fn set_agents(e: &Env, agents: &Map<Address, bool>) {
    e.storage().instance().set(&DataKey::Agents, agents);
}
