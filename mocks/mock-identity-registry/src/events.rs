//! Events for the Mock Identity Registry

use soroban_sdk::{contractevent, Address, Env};

/// Emitted when an agent is added
#[contractevent(data_format = "single-value")]
pub struct AgentAdded {
    pub agent: Address,
}

/// Emitted when an agent is removed
#[contractevent(data_format = "single-value")]
pub struct AgentRemoved {
    pub agent: Address,
}

/// Emitted when an identity is registered
#[contractevent]
pub struct IdentityRegistered {
    pub investor_address: Address,
    pub identity: Address,
}

/// Emitted when an identity is removed
#[contractevent]
pub struct IdentityRemoved {
    pub investor_address: Address,
    pub identity: Address,
}

/// Emitted when an identity is updated
#[contractevent]
pub struct IdentityUpdated {
    pub old_identity: Address,
    pub new_identity: Address,
}

/// Emitted when a country is updated
#[contractevent]
pub struct CountryUpdated {
    pub investor_address: Address,
    pub country: u32,
}

pub fn emit_agent_added(e: &Env, agent: Address) {
    AgentAdded { agent }.publish(e);
}

pub fn emit_agent_removed(e: &Env, agent: Address) {
    AgentRemoved { agent }.publish(e);
}

pub fn emit_identity_registered(e: &Env, investor_address: Address, identity: Address) {
    IdentityRegistered {
        investor_address,
        identity,
    }
    .publish(e);
}

pub fn emit_identity_removed(e: &Env, investor_address: Address, identity: Address) {
    IdentityRemoved {
        investor_address,
        identity,
    }
    .publish(e);
}

pub fn emit_identity_updated(e: &Env, old_identity: Address, new_identity: Address) {
    IdentityUpdated {
        old_identity,
        new_identity,
    }
    .publish(e);
}

pub fn emit_country_updated(e: &Env, investor_address: Address, country: u32) {
    CountryUpdated {
        investor_address,
        country,
    }
    .publish(e);
}
