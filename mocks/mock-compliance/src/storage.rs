//! Storage utilities for the Mock Compliance Contract

use crate::DataKey;
use soroban_sdk::{Address, Env};

const ONE_DAY_LEDGERS: u32 = 17280;
const LEDGER_THRESHOLD: u32 = ONE_DAY_LEDGERS * 90;
const LEDGER_BUMP: u32 = ONE_DAY_LEDGERS * 120;

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

// Bound token storage
pub fn get_bound_token(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::BoundToken)
}

pub fn set_bound_token(e: &Env, token: &Address) {
    e.storage().instance().set(&DataKey::BoundToken, token);
}

pub fn remove_bound_token(e: &Env) {
    e.storage().instance().remove(&DataKey::BoundToken);
}

// Transfer limit storage
pub fn get_transfer_limit(e: &Env, user: &Address) -> i128 {
    let key = DataKey::TransferLimit(user.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_transfer_limit(e: &Env, user: &Address, limit: i128) {
    let key = DataKey::TransferLimit(user.clone());
    e.storage().persistent().set(&key, &limit);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

// Transferred amount storage
pub fn get_transferred_amount(e: &Env, user: &Address) -> i128 {
    let key = DataKey::TransferredAmount(user.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_transferred_amount(e: &Env, user: &Address, amount: i128) {
    let key = DataKey::TransferredAmount(user.clone());
    e.storage().persistent().set(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
