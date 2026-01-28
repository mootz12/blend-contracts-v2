//! Compliance module for Mock ERC-3643 Token
//!
//! This module calls the external mock-compliance contract to check and track
//! transfer compliance rules.

use crate::{dependencies::compliance, storage};
use soroban_sdk::{Address, Env};

/// Check if a transfer can be made according to compliance rules
///
/// Calls the external compliance contract to check if the transfer is allowed.
pub fn can_transfer(e: &Env, from: &Address, to: &Address, amount: i128) -> bool {
    let compliance_addr = storage::get_compliance(e);
    let compliance = compliance::Client::new(e, &compliance_addr);
    compliance.can_transfer(from, to, &amount)
}

/// Notify compliance contract that tokens were transferred
pub fn notify_transferred(e: &Env, from: &Address, to: &Address, amount: i128) {
    let compliance_addr = storage::get_compliance(e);
    let compliance = compliance::Client::new(e, &compliance_addr);
    compliance.transferred(from, to, &amount)
}

/// Notify compliance contract that tokens were created (minted)
pub fn notify_created(e: &Env, to: &Address, amount: i128) {
    let compliance_addr = storage::get_compliance(e);
    let compliance = compliance::Client::new(e, &compliance_addr);
    compliance.created(to, &amount)
}

/// Notify compliance contract that tokens were destroyed (burned)
pub fn notify_destroyed(e: &Env, from: &Address, amount: i128) {
    let compliance_addr = storage::get_compliance(e);
    let compliance = compliance::Client::new(e, &compliance_addr);
    compliance.destroyed(from, &amount)
}
