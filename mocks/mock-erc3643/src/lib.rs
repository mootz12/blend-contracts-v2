//! Mock ERC-3643 Security Token Contract
//!
//! This contract implements a simplified version of the ERC-3643 (T-REX) security token
//! standard for use in the Blend protocol PoC. It provides compliant transfer functionality
//! that checks identity verification and compliance rules before allowing transfers.
//!
//! Key Features (per ERC-3643):
//! - ERC-20 compatible (SEP-41 on Stellar)
//! - Identity verification via Identity Registry
//! - Compliance checks via Compliance contract
//! - Token pausing and freezing
//! - Forced transfers (agent-only)
//! - Address freezing
//! - Partial token freezing
//! - Token recovery
//!
//! Reference: https://eips.ethereum.org/EIPS/eip-3643

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token::TokenInterface, Address, Env, MuxedAddress,
    String, Vec,
};

mod compliance;
mod errors;
mod events;
mod storage;

mod dependencies;

pub use compliance::*;
pub use errors::ERC3643Error;
pub use events::*;
pub use storage::*;

/// Token metadata
#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[contract]
pub struct MockERC3643Token;

/// Main ERC-3643 token trait
pub trait ERC3643TokenTrait {
    // --- Getters ---

    /// Get the total supply of the token
    fn total_supply(e: Env) -> i128;

    /// Get the identity registry address
    fn identity_registry(e: Env) -> Address;

    /// Get the compliance contract address
    fn compliance(e: Env) -> Address;

    /// Check if the token is paused
    fn paused(e: Env) -> bool;

    /// Check if an address is frozen
    fn is_frozen(e: Env, user_address: Address) -> bool;

    /// Get the amount of frozen tokens for an address
    fn get_frozen_tokens(e: Env, user_address: Address) -> i128;

    // --- Setters (Owner/Agent only) ---

    /// Set the token name
    fn set_name(e: Env, name: String);

    /// Set the token symbol
    fn set_symbol(e: Env, symbol: String);

    /// Pause the token
    fn pause(e: Env);

    /// Unpause the token
    fn unpause(e: Env);

    /// Freeze or unfreeze an address
    fn set_address_frozen(e: Env, user_address: Address, freeze: bool);

    /// Freeze a partial amount of tokens for an address
    fn freeze_partial_tokens(e: Env, user_address: Address, amount: i128);

    /// Unfreeze a partial amount of tokens for an address
    fn unfreeze_partial_tokens(e: Env, user_address: Address, amount: i128);

    /// Set the identity registry
    fn set_identity_registry(e: Env, identity_registry: Address);

    /// Set the compliance contract
    fn set_compliance(e: Env, compliance: Address);

    // --- Transfer Actions (Agent only) ---

    /// Force a transfer (bypasses sender restrictions, receiver still needs verification)
    fn forced_transfer(e: Env, from: Address, to: Address, amount: i128) -> bool;

    /// Mint tokens to an address (receiver must be verified)
    fn mint(e: Env, to: Address, amount: i128);

    /// Recover tokens from a lost wallet to a new wallet
    fn recovery_address(
        e: Env,
        lost_wallet: Address,
        new_wallet: Address,
        investor_identity: Address,
    ) -> bool;

    // --- Batch Functions ---

    /// Batch transfer to multiple addresses
    fn batch_transfer(e: Env, from: Address, to_list: Vec<Address>, amounts: Vec<i128>);

    /// Batch forced transfer
    fn batch_forced_transfer(
        e: Env,
        from_list: Vec<Address>,
        to_list: Vec<Address>,
        amounts: Vec<i128>,
    );

    /// Batch mint
    fn batch_mint(e: Env, to_list: Vec<Address>, amounts: Vec<i128>);

    /// Batch burn
    fn batch_burn(e: Env, user_addresses: Vec<Address>, amounts: Vec<i128>);

    /// Batch freeze addresses
    fn batch_set_address_frozen(e: Env, user_addresses: Vec<Address>, freeze: Vec<bool>);

    // --- Agent Management ---

    /// Add an agent
    fn add_agent(e: Env, agent: Address);

    /// Remove an agent
    fn remove_agent(e: Env, agent: Address);

    /// Check if an address is an agent
    fn is_agent(e: Env, agent: Address) -> bool;

    // --- Compliance Check ---

    /// Check if a transfer can be made according to compliance rules only
    /// Per ERC-3643, this only checks the compliance contract rules
    fn can_transfer(e: Env, from: Address, to: Address, amount: i128) -> bool;

    /// Check if an address is verified in the identity registry
    /// Per ERC-3643, this checks if the address is registered and meets claim requirements
    fn is_verified(e: Env, user_address: Address) -> bool;
}

#[contractimpl]
impl MockERC3643Token {
    /// # Arguments
    /// * `admin` - The admin/owner address
    /// * `name` - Token name
    /// * `symbol` - Token symbol
    /// * `decimals` - Number of decimals
    /// * `identity_registry` - Address of the identity registry contract
    /// * `compliance` - Address of the compliance contract (optional,
    pub fn __constructor(
        e: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        identity_registry: Address,
        compliance: Address,
    ) {
        if storage::has_admin(&e) {
            panic_with_error!(&e, ERC3643Error::AlreadyInitialized);
        }

        storage::set_admin(&e, &admin);
        storage::set_metadata(
            &e,
            &TokenMetadata {
                name,
                symbol,
                decimals,
            },
        );
        storage::set_identity_registry(&e, &identity_registry);
        storage::set_compliance(&e, &compliance);
        storage::set_paused(&e, false);
        storage::set_total_supply(&e, 0);
        storage::set_agents(&e, &soroban_sdk::map![&e]);
    }
}

#[contractimpl]
impl TokenInterface for MockERC3643Token {
    fn name(e: Env) -> String {
        storage::extend_instance(&e);
        storage::get_metadata(&e).name
    }

    fn symbol(e: Env) -> String {
        storage::extend_instance(&e);
        storage::get_metadata(&e).symbol
    }

    fn decimals(e: Env) -> u32 {
        storage::extend_instance(&e);
        storage::get_metadata(&e).decimals
    }

    fn balance(e: Env, id: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_balance(&e, &id)
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_allowance(&e, &from, &spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        storage::extend_instance(&e);
        from.require_auth();

        storage::set_allowance(&e, &from, &spender, amount, expiration_ledger);

        events::emit_approval(&e, from, spender, amount, expiration_ledger);
    }

    fn transfer(e: Env, from: Address, to: MuxedAddress, amount: i128) {
        storage::extend_instance(&e);
        from.require_auth();

        do_transfer(&e, &from, &to.address(), amount, true);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        storage::extend_instance(&e);
        spender.require_auth();

        spend_allowance(&e, &from, &spender, amount);
        do_transfer(&e, &from, &to, amount, true);
    }

    fn burn(e: Env, from: Address, amount: i128) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        let balance = storage::get_balance(&e, &from);
        if balance < amount {
            panic_with_error!(&e, ERC3643Error::InsufficientBalance);
        }

        storage::set_balance(&e, &from, balance - amount);

        let total = storage::get_total_supply(&e);
        storage::set_total_supply(&e, total - amount);

        // Notify compliance
        compliance::notify_destroyed(&e, &from, amount);

        events::emit_burn(&e, from, amount);
    }

    fn burn_from(e: Env, _spender: Address, _from: Address, _amount: i128) {
        panic_with_error!(&e, ERC3643Error::Unauthorized);
    }
}

#[contractimpl]
impl ERC3643TokenTrait for MockERC3643Token {
    fn total_supply(e: Env) -> i128 {
        storage::extend_instance(&e);
        storage::get_total_supply(&e)
    }

    fn identity_registry(e: Env) -> Address {
        storage::extend_instance(&e);
        storage::get_identity_registry(&e)
    }

    fn compliance(e: Env) -> Address {
        storage::extend_instance(&e);
        storage::get_compliance(&e)
    }

    fn paused(e: Env) -> bool {
        storage::extend_instance(&e);
        storage::is_paused(&e)
    }

    fn is_frozen(e: Env, user_address: Address) -> bool {
        storage::extend_instance(&e);
        storage::is_address_frozen(&e, &user_address)
    }

    fn get_frozen_tokens(e: Env, user_address: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_frozen_tokens(&e, &user_address)
    }

    fn set_name(e: Env, name: String) {
        storage::extend_instance(&e);
        require_admin(&e);

        let mut metadata = storage::get_metadata(&e);
        metadata.name = name;
        storage::set_metadata(&e, &metadata);
    }

    fn set_symbol(e: Env, symbol: String) {
        storage::extend_instance(&e);
        require_admin(&e);

        let mut metadata = storage::get_metadata(&e);
        metadata.symbol = symbol;
        storage::set_metadata(&e, &metadata);
    }

    fn pause(e: Env) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        storage::set_paused(&e, true);
        events::emit_paused(&e);
    }

    fn unpause(e: Env) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        storage::set_paused(&e, false);
        events::emit_unpaused(&e);
    }

    fn set_address_frozen(e: Env, user_address: Address, freeze: bool) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        storage::set_address_frozen(&e, &user_address, freeze);
        events::emit_address_frozen(&e, user_address, freeze);
    }

    fn freeze_partial_tokens(e: Env, user_address: Address, amount: i128) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        let current_frozen = storage::get_frozen_tokens(&e, &user_address);
        let balance = storage::get_balance(&e, &user_address);

        if current_frozen + amount > balance {
            panic_with_error!(&e, ERC3643Error::InsufficientBalance);
        }

        storage::set_frozen_tokens(&e, &user_address, current_frozen + amount);
        events::emit_tokens_frozen(&e, user_address, amount);
    }

    fn unfreeze_partial_tokens(e: Env, user_address: Address, amount: i128) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        let current_frozen = storage::get_frozen_tokens(&e, &user_address);

        if amount > current_frozen {
            panic_with_error!(&e, ERC3643Error::InsufficientFrozenTokens);
        }

        storage::set_frozen_tokens(&e, &user_address, current_frozen - amount);
        events::emit_tokens_unfrozen(&e, user_address, amount);
    }

    fn set_identity_registry(e: Env, identity_registry: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        storage::set_identity_registry(&e, &identity_registry);
        events::emit_identity_registry_added(&e, identity_registry);
    }

    fn set_compliance(e: Env, compliance: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        storage::set_compliance(&e, &compliance);
        events::emit_compliance_added(&e, compliance);
    }

    fn forced_transfer(e: Env, from: Address, to: Address, amount: i128) -> bool {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        // Forced transfer bypasses sender restrictions but still checks receiver
        do_transfer(&e, &from, &to, amount, false);
        true
    }

    fn mint(e: Env, to: Address, amount: i128) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        // Check receiver is verified
        check_receiver_verified(&e, &to);

        let balance = storage::get_balance(&e, &to);
        storage::set_balance(&e, &to, balance + amount);

        let total = storage::get_total_supply(&e);
        storage::set_total_supply(&e, total + amount);

        // Notify compliance
        compliance::notify_created(&e, &to, amount);

        events::emit_mint(&e, to, amount);
    }

    fn recovery_address(
        e: Env,
        lost_wallet: Address,
        new_wallet: Address,
        _investor_identity: Address,
    ) -> bool {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        // Check new wallet is verified
        check_receiver_verified(&e, &new_wallet);

        // Transfer all tokens from lost wallet to new wallet
        let balance = storage::get_balance(&e, &lost_wallet);
        let frozen = storage::get_frozen_tokens(&e, &lost_wallet);

        storage::set_balance(&e, &lost_wallet, 0);
        storage::set_frozen_tokens(&e, &lost_wallet, 0);

        storage::set_balance(&e, &new_wallet, balance);
        storage::set_frozen_tokens(&e, &new_wallet, frozen);

        events::emit_recovery_success(&e, lost_wallet, new_wallet);
        true
    }

    fn batch_transfer(e: Env, from: Address, to_list: Vec<Address>, amounts: Vec<i128>) {
        storage::extend_instance(&e);
        from.require_auth();

        if to_list.len() != amounts.len() {
            panic_with_error!(&e, ERC3643Error::ArrayLengthMismatch);
        }

        for i in 0..to_list.len() {
            let to = to_list.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            do_transfer(&e, &from, &to, amount, true);
        }
    }

    fn batch_forced_transfer(
        e: Env,
        from_list: Vec<Address>,
        to_list: Vec<Address>,
        amounts: Vec<i128>,
    ) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if from_list.len() != to_list.len() || from_list.len() != amounts.len() {
            panic_with_error!(&e, ERC3643Error::ArrayLengthMismatch);
        }

        for i in 0..from_list.len() {
            let from = from_list.get(i).unwrap();
            let to = to_list.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            do_transfer(&e, &from, &to, amount, false);
        }
    }

    fn batch_mint(e: Env, to_list: Vec<Address>, amounts: Vec<i128>) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if to_list.len() != amounts.len() {
            panic_with_error!(&e, ERC3643Error::ArrayLengthMismatch);
        }

        for i in 0..to_list.len() {
            let to = to_list.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            check_receiver_verified(&e, &to);

            let balance = storage::get_balance(&e, &to);
            storage::set_balance(&e, &to, balance + amount);

            let total = storage::get_total_supply(&e);
            storage::set_total_supply(&e, total + amount);

            compliance::notify_created(&e, &to, amount);
            events::emit_mint(&e, to, amount);
        }
    }

    fn batch_burn(e: Env, user_addresses: Vec<Address>, amounts: Vec<i128>) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if user_addresses.len() != amounts.len() {
            panic_with_error!(&e, ERC3643Error::ArrayLengthMismatch);
        }

        for i in 0..user_addresses.len() {
            let user = user_addresses.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            let balance = storage::get_balance(&e, &user);
            if balance < amount {
                panic_with_error!(&e, ERC3643Error::InsufficientBalance);
            }

            storage::set_balance(&e, &user, balance - amount);

            let total = storage::get_total_supply(&e);
            storage::set_total_supply(&e, total - amount);

            compliance::notify_destroyed(&e, &user, amount);
            events::emit_burn(&e, user, amount);
        }
    }

    fn batch_set_address_frozen(e: Env, user_addresses: Vec<Address>, freeze: Vec<bool>) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if user_addresses.len() != freeze.len() {
            panic_with_error!(&e, ERC3643Error::ArrayLengthMismatch);
        }

        for i in 0..user_addresses.len() {
            let user = user_addresses.get(i).unwrap();
            let frozen = freeze.get(i).unwrap();
            storage::set_address_frozen(&e, &user, frozen);
            events::emit_address_frozen(&e, user, frozen);
        }
    }

    fn add_agent(e: Env, agent: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        let mut agents = storage::get_agents(&e);
        agents.set(agent.clone(), true);
        storage::set_agents(&e, &agents);

        events::emit_agent_added(&e, agent);
    }

    fn remove_agent(e: Env, agent: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        let mut agents = storage::get_agents(&e);
        agents.remove(agent.clone());
        storage::set_agents(&e, &agents);

        events::emit_agent_removed(&e, agent);
    }

    fn is_agent(e: Env, agent: Address) -> bool {
        storage::extend_instance(&e);
        let agents = storage::get_agents(&e);
        agents.get(agent).unwrap_or(false)
    }

    /// Check if a transfer can be made according to compliance rules only
    /// Per ERC-3643 spec, canTransfer ONLY checks compliance contract rules
    /// Use is_verified() to check identity registry verification
    fn can_transfer(e: Env, from: Address, to: Address, amount: i128) -> bool {
        storage::extend_instance(&e);

        // Per ERC-3643, canTransfer only checks compliance rules
        compliance::can_transfer(&e, &from, &to, amount)
    }

    /// Check if an address is verified in the identity registry
    /// Per ERC-3643 spec: "isVerified function is called from within the transfer
    /// functions to instruct the Identity Registry to check if the receiver is a valid investor"
    fn is_verified(e: Env, user_address: Address) -> bool {
        storage::extend_instance(&e);
        is_receiver_verified(&e, &user_address)
    }
}

// ==================== Internal Helper Functions ====================

use soroban_sdk::panic_with_error;

use crate::dependencies::identity_registry;

/// Require that the caller is the admin
fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}

/// Require that the caller is an agent or admin
/// Note: In this simplified mock, we just require admin auth.
/// A full implementation would check both admin and agent authorizations.
fn require_agent_or_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}

/// Check if receiver is verified via identity registry
fn is_receiver_verified(e: &Env, to: &Address) -> bool {
    let registry_addr = storage::get_identity_registry(e);
    let registry = identity_registry::Client::new(e, &registry_addr);
    registry.is_verified(to)
}

/// Check receiver verified and panic if not
fn check_receiver_verified(e: &Env, to: &Address) {
    if !is_receiver_verified(e, to) {
        panic_with_error!(e, ERC3643Error::InvalidIdentity);
    }
}

/// Perform a transfer with all compliance checks
fn do_transfer(e: &Env, from: &Address, to: &Address, amount: i128, check_sender: bool) {
    // Check paused
    if storage::is_paused(e) {
        panic_with_error!(e, ERC3643Error::TokenPaused);
    }

    // Check sender restrictions (if not forced transfer)
    if check_sender {
        if storage::is_address_frozen(e, from) {
            panic_with_error!(e, ERC3643Error::FrozenWallet);
        }

        // Check free balance
        let balance = storage::get_balance(e, from);
        let frozen_tokens = storage::get_frozen_tokens(e, from);
        if amount > balance - frozen_tokens {
            panic_with_error!(e, ERC3643Error::InsufficientBalance);
        }
    }

    // Check receiver restrictions
    if storage::is_address_frozen(e, to) {
        panic_with_error!(e, ERC3643Error::FrozenWallet);
    }

    // Check receiver is verified
    check_receiver_verified(e, to);

    // Check compliance (if checking sender - forced transfers bypass compliance)
    if check_sender && !compliance::can_transfer(e, from, to, amount) {
        panic_with_error!(e, ERC3643Error::ComplianceFailure);
    }

    // Perform transfer
    let from_balance = storage::get_balance(e, from);
    let to_balance = storage::get_balance(e, to);

    if from_balance < amount {
        panic_with_error!(e, ERC3643Error::InsufficientBalance);
    }

    storage::set_balance(e, from, from_balance - amount);
    storage::set_balance(e, to, to_balance + amount);

    // Notify compliance
    compliance::notify_transferred(e, from, to, amount);

    events::emit_transfer(e, from.clone(), to.clone(), amount);
}

/// Spend allowance
fn spend_allowance(e: &Env, from: &Address, spender: &Address, amount: i128) {
    let allowance = storage::get_allowance(e, from, spender);

    if allowance.expiration_ledger < e.ledger().sequence() {
        panic_with_error!(e, ERC3643Error::AllowanceExpired);
    }

    if allowance.amount < amount {
        panic_with_error!(e, ERC3643Error::InsufficientAllowance);
    }

    if allowance.amount > 0 {
        let new_amount = allowance.amount - amount;
        storage::set_allowance(e, from, spender, new_amount, allowance.expiration_ledger);
    }
}

#[cfg(test)]
mod test;
