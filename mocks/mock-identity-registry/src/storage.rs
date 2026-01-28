//! Storage utilities for the Mock Identity Registry

use crate::{DataKey, IdentityInfo};
use soroban_sdk::{Address, Env, Map, Vec};

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

pub fn init_agents(e: &Env) {
    let agents: Map<Address, bool> = soroban_sdk::map![e];
    e.storage().instance().set(&DataKey::Agents, &agents);
}

// Identity storage
pub fn has_identity(e: &Env, user: &Address) -> bool {
    let key = DataKey::Identity(user.clone());
    e.storage().persistent().has(&key)
}

pub fn get_identity(e: &Env, user: &Address) -> IdentityInfo {
    let key = DataKey::Identity(user.clone());
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    e.storage().persistent().get(&key).unwrap()
}

pub fn set_identity(e: &Env, user: &Address, info: &IdentityInfo) {
    let key = DataKey::Identity(user.clone());
    e.storage().persistent().set(&key, info);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn remove_identity(e: &Env, user: &Address) {
    let key = DataKey::Identity(user.clone());
    e.storage().persistent().remove(&key);
}

// Registered addresses storage
pub fn get_registered_addresses(e: &Env) -> Vec<Address> {
    e.storage()
        .instance()
        .get(&DataKey::RegisteredAddresses)
        .unwrap_or(soroban_sdk::vec![e])
}

pub fn set_registered_addresses(e: &Env, addresses: &Vec<Address>) {
    e.storage()
        .instance()
        .set(&DataKey::RegisteredAddresses, addresses);
}

// Allowed countries storage (using u32 for Soroban compatibility)
pub fn get_allowed_countries(e: &Env) -> Vec<u32> {
    e.storage()
        .instance()
        .get(&DataKey::AllowedCountries)
        .unwrap_or(soroban_sdk::vec![e])
}

pub fn set_allowed_countries(e: &Env, countries: &Vec<u32>) {
    e.storage()
        .instance()
        .set(&DataKey::AllowedCountries, countries);
}
