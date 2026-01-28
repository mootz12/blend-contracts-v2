//! Mock Identity Registry Contract
//!
//! This contract implements a simplified version of the ERC-3643 Identity Registry
//! for use in the Blend protocol PoC. It provides identity verification functionality
//! that ERC-3643 tokens use to check if addresses are eligible to hold/receive tokens.
//!
//! The Identity Registry establishes the link between:
//! - A wallet address
//! - An Identity contract (simplified as an Address in this mock)
//! - A country code (ISO-3166 compliant)
//!
//! Reference: https://eips.ethereum.org/EIPS/eip-3643

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, panic_with_error, Address, Env, Vec};

mod errors;
mod events;
mod storage;

pub use errors::IdentityRegistryError;
pub use events::*;
pub use storage::*;

/// Identity information stored for each investor
#[derive(Clone)]
#[contracttype]
pub struct IdentityInfo {
    /// The identity contract address (simplified from IIdentity)
    pub identity: Address,
    /// Country code (ISO-3166) - using u32 for Soroban compatibility
    pub country: u32,
}

/// Storage keys for the Identity Registry
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Map of agent addresses
    Agents,
    /// Identity info for a user address
    Identity(Address),
    /// List of all registered addresses (for iteration)
    RegisteredAddresses,
    /// Claim topics required for verification (simplified)
    RequiredClaimTopics,
    /// Allowed countries for the token
    AllowedCountries,
}

#[contract]
pub struct MockIdentityRegistry;

/// Trait defining the Identity Registry interface based on ERC-3643
pub trait IdentityRegistryTrait {
    // ==================== Agent Management (IAgentRole) ====================

    /// Add an agent who can modify the registry
    ///
    /// # Arguments
    /// * `agent` - The address to add as an agent
    fn add_agent(e: Env, agent: Address);

    /// Remove an agent
    ///
    /// # Arguments
    /// * `agent` - The address to remove as an agent
    fn remove_agent(e: Env, agent: Address);

    /// Check if an address is an agent
    ///
    /// # Arguments
    /// * `agent` - The address to check
    fn is_agent(e: Env, agent: Address) -> bool;

    // ==================== Identity Registry Functions ====================

    /// Register an identity for a user address
    ///
    /// # Arguments
    /// * `user_address` - The wallet address of the investor
    /// * `identity` - The identity contract address
    /// * `country` - The country code (ISO-3166)
    fn register_identity(e: Env, user_address: Address, identity: Address, country: u32);

    /// Delete an identity from the registry
    ///
    /// # Arguments
    /// * `user_address` - The wallet address to remove
    fn delete_identity(e: Env, user_address: Address);

    /// Update the country for a registered identity
    ///
    /// # Arguments
    /// * `user_address` - The wallet address
    /// * `country` - The new country code
    fn update_country(e: Env, user_address: Address, country: u32);

    /// Update the identity contract for a registered address
    ///
    /// # Arguments
    /// * `user_address` - The wallet address
    /// * `identity` - The new identity contract address
    fn update_identity(e: Env, user_address: Address, identity: Address);

    /// Batch register multiple identities
    ///
    /// # Arguments
    /// * `user_addresses` - List of wallet addresses
    /// * `identities` - List of identity contract addresses
    /// * `countries` - List of country codes
    fn batch_register_identity(
        e: Env,
        user_addresses: Vec<Address>,
        identities: Vec<Address>,
        countries: Vec<u32>,
    );

    // ==================== Registry Consultation ====================

    /// Check if an address is registered in the registry
    ///
    /// # Arguments
    /// * `user_address` - The address to check
    fn contains(e: Env, user_address: Address) -> bool;

    /// Check if an address is verified (registered and meets all requirements)
    /// This is the main function called by ERC-3643 tokens during transfers
    ///
    /// # Arguments
    /// * `user_address` - The address to verify
    fn is_verified(e: Env, user_address: Address) -> bool;

    /// Get the identity contract for an address
    ///
    /// # Arguments
    /// * `user_address` - The wallet address
    fn identity(e: Env, user_address: Address) -> Address;

    /// Get the country code for an address
    ///
    /// # Arguments
    /// * `user_address` - The wallet address
    fn investor_country(e: Env, user_address: Address) -> u32;

    // ==================== Configuration ====================

    /// Set allowed countries for token holders
    ///
    /// # Arguments
    /// * `countries` - List of allowed country codes
    fn set_allowed_countries(e: Env, countries: Vec<u32>);

    /// Get allowed countries
    fn get_allowed_countries(e: Env) -> Vec<u32>;

    /// Add a single allowed country
    ///
    /// # Arguments
    /// * `country` - Country code to allow
    fn add_allowed_country(e: Env, country: u32);

    /// Remove a country from allowed list
    ///
    /// # Arguments
    /// * `country` - Country code to remove
    fn remove_allowed_country(e: Env, country: u32);

    // ==================== Mock-specific Functions ====================

    /// Mock Only: Directly set verification status for testing
    ///
    /// # Arguments
    /// * `user_address` - The address to set status for
    /// * `verified` - Whether the address should be verified
    fn mock_set_verified(e: Env, user_address: Address, identity: Address, country: u32);

    /// Mock Only: Get the admin address
    fn get_admin(e: Env) -> Address;
}

#[contractimpl]
impl MockIdentityRegistry {
    /// Constructor for the identity registry
    ///
    /// # Arguments
    /// * `admin` - The admin address who can manage the registry
    pub fn __constructor(e: Env, admin: Address) {
        if storage::has_admin(&e) {
            panic_with_error!(&e, IdentityRegistryError::AlreadyInitialized);
        }
        storage::set_admin(&e, &admin);
        storage::init_agents(&e);
        storage::set_registered_addresses(&e, &soroban_sdk::vec![&e]);
        storage::set_allowed_countries(&e, &soroban_sdk::vec![&e]);
    }
}

#[contractimpl]
impl IdentityRegistryTrait for MockIdentityRegistry {
    fn add_agent(e: Env, agent: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let mut agents = storage::get_agents(&e);
        agents.set(agent.clone(), true);
        storage::set_agents(&e, &agents);

        events::emit_agent_added(&e, agent);
    }

    fn remove_agent(e: Env, agent: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

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

    fn register_identity(e: Env, user_address: Address, identity: Address, country: u32) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityAlreadyRegistered);
        }

        let info = IdentityInfo {
            identity: identity.clone(),
            country,
        };
        storage::set_identity(&e, &user_address, &info);

        // Add to registered addresses list
        let mut addresses = storage::get_registered_addresses(&e);
        addresses.push_back(user_address.clone());
        storage::set_registered_addresses(&e, &addresses);

        events::emit_identity_registered(&e, user_address, identity);
    }

    fn delete_identity(e: Env, user_address: Address) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if !storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityNotFound);
        }

        let info = storage::get_identity(&e, &user_address);
        storage::remove_identity(&e, &user_address);

        events::emit_identity_removed(&e, user_address, info.identity);
    }

    fn update_country(e: Env, user_address: Address, country: u32) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if !storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityNotFound);
        }

        let mut info = storage::get_identity(&e, &user_address);
        info.country = country;
        storage::set_identity(&e, &user_address, &info);

        events::emit_country_updated(&e, user_address, country);
    }

    fn update_identity(e: Env, user_address: Address, identity: Address) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if !storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityNotFound);
        }

        let mut info = storage::get_identity(&e, &user_address);
        let old_identity = info.identity.clone();
        info.identity = identity.clone();
        storage::set_identity(&e, &user_address, &info);

        events::emit_identity_updated(&e, old_identity, identity);
    }

    fn batch_register_identity(
        e: Env,
        user_addresses: Vec<Address>,
        identities: Vec<Address>,
        countries: Vec<u32>,
    ) {
        storage::extend_instance(&e);
        require_agent_or_admin(&e);

        if user_addresses.len() != identities.len() || user_addresses.len() != countries.len() {
            panic_with_error!(&e, IdentityRegistryError::ArrayLengthMismatch);
        }

        for i in 0..user_addresses.len() {
            let user_address = user_addresses.get(i).unwrap();
            let identity = identities.get(i).unwrap();
            let country = countries.get(i).unwrap();

            if !storage::has_identity(&e, &user_address) {
                let info = IdentityInfo {
                    identity: identity.clone(),
                    country,
                };
                storage::set_identity(&e, &user_address, &info);

                let mut addresses = storage::get_registered_addresses(&e);
                addresses.push_back(user_address.clone());
                storage::set_registered_addresses(&e, &addresses);

                events::emit_identity_registered(&e, user_address, identity);
            }
        }
    }

    fn contains(e: Env, user_address: Address) -> bool {
        storage::extend_instance(&e);
        storage::has_identity(&e, &user_address)
    }

    fn is_verified(e: Env, user_address: Address) -> bool {
        storage::extend_instance(&e);

        // Check if identity exists
        if !storage::has_identity(&e, &user_address) {
            return false;
        }

        let info = storage::get_identity(&e, &user_address);
        let allowed_countries = storage::get_allowed_countries(&e);

        // If no countries are configured, allow all
        if allowed_countries.is_empty() {
            return true;
        }

        // Check if the investor's country is in the allowed list
        for i in 0..allowed_countries.len() {
            if allowed_countries.get(i).unwrap() == info.country {
                return true;
            }
        }

        false
    }

    fn identity(e: Env, user_address: Address) -> Address {
        storage::extend_instance(&e);
        if !storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityNotFound);
        }
        storage::get_identity(&e, &user_address).identity
    }

    fn investor_country(e: Env, user_address: Address) -> u32 {
        storage::extend_instance(&e);
        if !storage::has_identity(&e, &user_address) {
            panic_with_error!(&e, IdentityRegistryError::IdentityNotFound);
        }
        storage::get_identity(&e, &user_address).country
    }

    fn set_allowed_countries(e: Env, countries: Vec<u32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_allowed_countries(&e, &countries);
    }

    fn get_allowed_countries(e: Env) -> Vec<u32> {
        storage::extend_instance(&e);
        storage::get_allowed_countries(&e)
    }

    fn add_allowed_country(e: Env, country: u32) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let mut countries = storage::get_allowed_countries(&e);
        // Check if already exists
        for i in 0..countries.len() {
            if countries.get(i).unwrap() == country {
                return; // Already exists
            }
        }
        countries.push_back(country);
        storage::set_allowed_countries(&e, &countries);
    }

    fn remove_allowed_country(e: Env, country: u32) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let countries = storage::get_allowed_countries(&e);
        let mut new_countries: Vec<u32> = soroban_sdk::vec![&e];
        for i in 0..countries.len() {
            let c = countries.get(i).unwrap();
            if c != country {
                new_countries.push_back(c);
            }
        }
        storage::set_allowed_countries(&e, &new_countries);
    }

    fn mock_set_verified(e: Env, user_address: Address, identity: Address, country: u32) {
        storage::extend_instance(&e);
        let info = IdentityInfo { identity, country };
        storage::set_identity(&e, &user_address, &info);

        // Ensure this address is in the registered list
        if !storage::has_identity(&e, &user_address) {
            let mut addresses = storage::get_registered_addresses(&e);
            addresses.push_back(user_address);
            storage::set_registered_addresses(&e, &addresses);
        }
    }

    fn get_admin(e: Env) -> Address {
        storage::extend_instance(&e);
        storage::get_admin(&e)
    }
}

/// Helper function to check if the caller is an agent or admin
/// Note: In this simplified mock, we just require admin auth.
/// A full implementation would check both admin and agent authorizations.
fn require_agent_or_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
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
        let contract_id = e.register(MockIdentityRegistry, (&admin,));
        let client = MockIdentityRegistryClient::new(&e, &contract_id);

        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_register_and_verify() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockIdentityRegistry, (&admin,));
        let client = MockIdentityRegistryClient::new(&e, &contract_id);

        let user = Address::generate(&e);
        let identity = Address::generate(&e);

        // Register identity with US country code (840)
        client.register_identity(&user, &identity, &840);

        // Should be verified (no country restrictions)
        assert!(client.is_verified(&user));
        assert!(client.contains(&user));
        assert_eq!(client.identity(&user), identity);
        assert_eq!(client.investor_country(&user), 840);
    }

    #[test]
    fn test_country_restrictions() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let contract_id = e.register(MockIdentityRegistry, (&admin,));
        let client = MockIdentityRegistryClient::new(&e, &contract_id);

        let us_user = Address::generate(&e);
        let uk_user = Address::generate(&e);
        let identity = Address::generate(&e);

        // Register identities
        client.register_identity(&us_user, &identity, &840); // US
        client.register_identity(&uk_user, &identity, &826); // UK

        // Both verified without restrictions
        assert!(client.is_verified(&us_user));
        assert!(client.is_verified(&uk_user));

        // Add only US to allowed countries
        client.add_allowed_country(&840);

        // Now only US user is verified
        assert!(client.is_verified(&us_user));
        assert!(!client.is_verified(&uk_user));
    }
}
