//! Mock Compliance Contract
//!
//! This contract implements a simplified version of the ERC-3643 Compliance interface
//! for use in the Blend protocol PoC. It provides compliance checking functionality
//! that ERC-3643 tokens use to validate transfers against compliance rules.
//!
//! Features implemented:
//! - Token transfer limits per user
//! - Transfer tracking (via `transferred` callback)
//! - Admin-controlled limit configuration
//!
//! Reference: https://eips.ethereum.org/EIPS/eip-3643

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, panic_with_error, Address, Env};

mod errors;
mod events;
mod storage;

pub use errors::ComplianceError;
pub use events::*;
pub use storage::*;

/// Storage keys for the Compliance contract
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// The token contract this compliance is bound to
    BoundToken,
    /// Transfer limit for a user (max tokens they can receive)
    TransferLimit(Address),
    /// Current transferred amount for a user (tokens received)
    TransferredAmount(Address),
}

#[contract]
pub struct MockCompliance;

/// Trait defining the Compliance interface based on ERC-3643
pub trait ComplianceTrait {
    // ==================== Token Binding (per ERC-3643) ====================

    /// Bind this compliance contract to a token
    ///
    /// # Arguments
    /// * `token` - The token contract address to bind to
    fn bind_token(e: Env, token: Address);

    /// Unbind this compliance contract from a token
    ///
    /// # Arguments
    /// * `token` - The token contract address to unbind
    fn unbind_token(e: Env, token: Address);

    /// Check if a token is bound to this compliance contract
    ///
    /// # Arguments
    /// * `token` - The token contract address to check
    fn is_token_bound(e: Env, token: Address) -> bool;

    /// Get the bound token address
    fn get_token_bound(e: Env) -> Address;

    // ==================== Compliance Check Functions ====================

    /// Check if a transfer can be made according to compliance rules
    /// This checks if the receiver has not exceeded their transfer limit
    ///
    /// # Arguments
    /// * `from` - The sender address
    /// * `to` - The receiver address
    /// * `amount` - The amount to transfer
    fn can_transfer(e: Env, from: Address, to: Address, amount: i128) -> bool;

    /// Called by the token contract when a transfer occurs
    /// Updates the transferred amount for the receiver
    ///
    /// # Arguments
    /// * `from` - The sender address
    /// * `to` - The receiver address
    /// * `amount` - The amount transferred
    fn transferred(e: Env, from: Address, to: Address, amount: i128);

    /// Called by the token contract when tokens are created (minted)
    ///
    /// # Arguments
    /// * `to` - The receiver address
    /// * `amount` - The amount minted
    fn created(e: Env, to: Address, amount: i128);

    /// Called by the token contract when tokens are destroyed (burned)
    ///
    /// # Arguments
    /// * `from` - The address tokens were burned from
    /// * `amount` - The amount burned
    fn destroyed(e: Env, from: Address, amount: i128);

    // ==================== Admin Functions ====================

    /// Set the transfer limit for a user
    /// Setting limit to 0 means unlimited
    ///
    /// # Arguments
    /// * `user` - The user address
    /// * `limit` - The maximum amount the user can receive (0 = unlimited)
    fn set_transfer_limit(e: Env, user: Address, limit: i128);

    /// Get the transfer limit for a user
    ///
    /// # Arguments
    /// * `user` - The user address
    fn get_transfer_limit(e: Env, user: Address) -> i128;

    /// Get the current transferred amount for a user
    ///
    /// # Arguments
    /// * `user` - The user address
    fn get_transferred_amount(e: Env, user: Address) -> i128;

    /// Reset the transferred amount for a user (admin only)
    ///
    /// # Arguments
    /// * `user` - The user address
    fn reset_transferred_amount(e: Env, user: Address);

    /// Get the admin address
    fn get_admin(e: Env) -> Address;
}

#[contractimpl]
impl MockCompliance {
    /// Constructor for the compliance contract
    ///
    /// # Arguments
    /// * `admin` - The admin address who can manage compliance rules
    pub fn __constructor(e: Env, admin: Address) {
        if storage::has_admin(&e) {
            panic_with_error!(&e, ComplianceError::AlreadyInitialized);
        }
        storage::set_admin(&e, &admin);
    }
}

#[contractimpl]
impl ComplianceTrait for MockCompliance {
    fn bind_token(e: Env, token: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_bound_token(&e, &token);
        events::emit_token_bound(&e, token);
    }

    fn unbind_token(e: Env, token: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        // Verify this is the bound token
        if let Some(bound) = storage::get_bound_token(&e) {
            if bound == token {
                storage::remove_bound_token(&e);
                events::emit_token_unbound(&e, token);
            }
        }
    }

    fn is_token_bound(e: Env, token: Address) -> bool {
        storage::extend_instance(&e);
        if let Some(bound) = storage::get_bound_token(&e) {
            return bound == token;
        }
        false
    }

    fn get_token_bound(e: Env) -> Address {
        storage::extend_instance(&e);
        storage::get_bound_token(&e).unwrap_or_else(|| {
            panic_with_error!(&e, ComplianceError::NoTokenBound);
        })
    }

    fn can_transfer(e: Env, _from: Address, to: Address, amount: i128) -> bool {
        storage::extend_instance(&e);

        // Get the transfer limit for the receiver
        let limit = storage::get_transfer_limit(&e, &to);

        // Check if receiver has reached their limit
        let current_transferred = storage::get_transferred_amount(&e, &to);
        let new_total = current_transferred + amount;

        // Return true if the new total would not exceed the limit
        new_total <= limit
    }

    fn transferred(e: Env, _from: Address, to: Address, amount: i128) {
        storage::extend_instance(&e);

        // Update the transferred amount for the receiver
        let current = storage::get_transferred_amount(&e, &to);
        storage::set_transferred_amount(&e, &to, current + amount);

        events::emit_transferred(&e, to, amount);
    }

    fn created(e: Env, to: Address, amount: i128) {
        storage::extend_instance(&e);

        // Minting also counts towards transfer limits
        let current = storage::get_transferred_amount(&e, &to);
        storage::set_transferred_amount(&e, &to, current + amount);

        events::emit_created(&e, to, amount);
    }

    fn destroyed(e: Env, from: Address, amount: i128) {
        storage::extend_instance(&e);

        // Burning reduces the transferred amount
        let current = storage::get_transferred_amount(&e, &from);
        let new_amount = if current > amount {
            current - amount
        } else {
            0
        };
        storage::set_transferred_amount(&e, &from, new_amount);

        events::emit_destroyed(&e, from, amount);
    }

    fn set_transfer_limit(e: Env, user: Address, limit: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_transfer_limit(&e, &user, limit);
        events::emit_transfer_limit_set(&e, user, limit);
    }

    fn get_transfer_limit(e: Env, user: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_transfer_limit(&e, &user)
    }

    fn get_transferred_amount(e: Env, user: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_transferred_amount(&e, &user)
    }

    fn reset_transferred_amount(e: Env, user: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_transferred_amount(&e, &user, 0);
        events::emit_transferred_reset(&e, user);
    }

    fn get_admin(e: Env) -> Address {
        storage::extend_instance(&e);
        storage::get_admin(&e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockCompliance, (&admin,));
        let client = MockComplianceClient::new(&e, &contract_id);

        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_transfer_limits() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockCompliance, (&admin,));
        let client = MockComplianceClient::new(&e, &contract_id);

        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        // Set limit for user2
        client.set_transfer_limit(&user2, &500);
        assert_eq!(client.get_transfer_limit(&user2), 500);

        // Can transfer up to limit
        assert!(client.can_transfer(&user1, &user2, &500));
        assert!(client.can_transfer(&user1, &user2, &400));

        // Cannot exceed limit
        assert!(!client.can_transfer(&user1, &user2, &501));
    }

    #[test]
    fn test_transferred_tracking() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockCompliance, (&admin,));
        let client = MockComplianceClient::new(&e, &contract_id);

        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        // Set limit for user2
        client.set_transfer_limit(&user2, &1000);

        // Record transfers
        client.transferred(&user1, &user2, &300);
        assert_eq!(client.get_transferred_amount(&user2), 300);

        // Can still transfer more
        assert!(client.can_transfer(&user1, &user2, &700));
        assert!(!client.can_transfer(&user1, &user2, &701));

        // Record another transfer
        client.transferred(&user1, &user2, &500);
        assert_eq!(client.get_transferred_amount(&user2), 800);

        // Limit check should account for previous transfers
        assert!(client.can_transfer(&user1, &user2, &200));
        assert!(!client.can_transfer(&user1, &user2, &201));
    }

    #[test]
    fn test_reset_transferred() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockCompliance, (&admin,));
        let client = MockComplianceClient::new(&e, &contract_id);

        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        // Set limit and transfer
        client.set_transfer_limit(&user2, &500);
        client.transferred(&user1, &user2, &500);

        // At limit
        assert!(!client.can_transfer(&user1, &user2, &1));

        // Reset
        client.reset_transferred_amount(&user2);
        assert_eq!(client.get_transferred_amount(&user2), 0);

        // Can transfer again
        assert!(client.can_transfer(&user1, &user2, &500));
    }

    #[test]
    fn test_token_binding() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockCompliance, (&admin,));
        let client = MockComplianceClient::new(&e, &contract_id);

        let token = Address::generate(&e);

        // Bind token
        client.bind_token(&token);
        assert!(client.is_token_bound(&token));
        assert_eq!(client.get_token_bound(), token);

        // Unbind token
        client.unbind_token(&token);
        assert!(!client.is_token_bound(&token));
    }
}
