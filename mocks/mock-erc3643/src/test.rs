//! Tests for the Mock ERC-3643 Token

use crate::dependencies::{compliance, identity_registry};
use crate::MockERC3643Token;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_test() -> (Env, Address, Address, Address, Address, Address, Address) {
    let e = Env::default();
    e.mock_all_auths();

    // Deploy identity registry with constructor args
    let registry_admin = Address::generate(&e);
    let registry_id = e.register(identity_registry::WASM, (&registry_admin,));
    let registry_client = identity_registry::Client::new(&e, &registry_id);

    // Deploy compliance contract with constructor args
    let compliance_admin = Address::generate(&e);
    let compliance_id = e.register(compliance::WASM, (&compliance_admin,));
    let compliance_client = compliance::Client::new(&e, &compliance_id);

    // Deploy token
    let admin = Address::generate(&e);
    let name = String::from_str(&e, "TREX Token");
    let symbol = String::from_str(&e, "TREX");
    let token_id = e.register(
        MockERC3643Token,
        (&admin, &name, &symbol, &7_u32, &registry_id, &compliance_id),
    );

    // Bind token to compliance contract
    compliance_client.bind_token(&token_id);

    // Create test users
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // Register identities for users
    let identity1 = Address::generate(&e);
    let identity2 = Address::generate(&e);
    registry_client.mock_set_verified(&user1, &identity1, &840); // US
    registry_client.mock_set_verified(&user2, &identity2, &826); // UK

    // Give users the ability to transfer 1000 tokens via compliance
    compliance_client.set_transfer_limit(&user1, &1000_0000000);
    compliance_client.set_transfer_limit(&user2, &1000_0000000);

    (e, token_id, registry_id, compliance_id, admin, user1, user2)
}

#[test]
fn test_initialize() {
    let (e, token_id, _registry_id, _compliance_id, _admin, _user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    assert_eq!(client.name(), String::from_str(&e, "TREX Token"));
    assert_eq!(client.symbol(), String::from_str(&e, "TREX"));
    assert_eq!(client.decimals(), 7);
    assert_eq!(client.total_supply(), 0);
    assert!(!client.paused());
}

#[test]
fn test_mint_and_transfer() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens to user1
    client.mint(&user1, &1000);
    assert_eq!(client.balance(&user1), 1000);
    assert_eq!(client.total_supply(), 1000);

    // Transfer from user1 to user2
    client.transfer(&user1, &user2, &300);
    assert_eq!(client.balance(&user1), 700);
    assert_eq!(client.balance(&user2), 300);
}

#[test]
fn test_freeze_address() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Freeze user1
    client.set_address_frozen(&user1, &true);
    assert!(client.is_frozen(&user1));

    // Unfreeze
    client.set_address_frozen(&user1, &false);
    assert!(!client.is_frozen(&user1));
}

#[test]
fn test_freeze_partial_tokens() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Freeze partial tokens
    client.freeze_partial_tokens(&user1, &400);
    assert_eq!(client.get_frozen_tokens(&user1), 400);

    // Unfreeze some
    client.unfreeze_partial_tokens(&user1, &200);
    assert_eq!(client.get_frozen_tokens(&user1), 200);
}

#[test]
fn test_pause() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Pause token
    client.pause();
    assert!(client.paused());

    // Unpause
    client.unpause();
    assert!(!client.paused());
}

#[test]
fn test_forced_transfer() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Freeze user1
    client.set_address_frozen(&user1, &true);

    // Normal transfer would fail, but forced transfer should work
    client.forced_transfer(&user1, &user2, &500);
    assert_eq!(client.balance(&user1), 500);
    assert_eq!(client.balance(&user2), 500);
}

#[test]
fn test_burn() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);
    assert_eq!(client.total_supply(), 1000);

    // Burn tokens
    client.burn(&user1, &300);
    assert_eq!(client.balance(&user1), 700);
    assert_eq!(client.total_supply(), 700);
}

#[test]
fn test_recovery() {
    let (e, token_id, registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);
    let registry_client = identity_registry::Client::new(&e, &registry_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Freeze some tokens
    client.freeze_partial_tokens(&user1, &200);

    // Create new wallet for user1
    let new_wallet = Address::generate(&e);
    let new_identity = Address::generate(&e);
    registry_client.mock_set_verified(&new_wallet, &new_identity, &840);

    // Recover tokens to new wallet
    client.recovery_address(&user1, &new_wallet, &new_identity);

    assert_eq!(client.balance(&user1), 0);
    assert_eq!(client.get_frozen_tokens(&user1), 0);
    assert_eq!(client.balance(&new_wallet), 1000);
    assert_eq!(client.get_frozen_tokens(&new_wallet), 200);
}

#[test]
fn test_allowance_and_transfer_from() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Mint tokens
    client.mint(&user1, &1000);

    // Create spender
    let spender = Address::generate(&e);

    // Approve spender
    let expiration = e.ledger().sequence() + 1000;
    client.approve(&user1, &spender, &500, &expiration);
    assert_eq!(client.allowance(&user1, &spender), 500);

    // Transfer from
    client.transfer_from(&spender, &user1, &user2, &200);
    assert_eq!(client.balance(&user1), 800);
    assert_eq!(client.balance(&user2), 200);
    assert_eq!(client.allowance(&user1, &spender), 300);
}

#[test]
fn test_agent_management() {
    let (e, token_id, _registry_id, _compliance_id, _admin, _user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    let agent = Address::generate(&e);

    // Add agent
    client.add_agent(&agent);
    assert!(client.is_agent(&agent));

    // Remove agent
    client.remove_agent(&agent);
    assert!(!client.is_agent(&agent));
}

#[test]
fn test_batch_mint() {
    let (e, token_id, registry_id, _compliance_id, _admin, user1, user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // Create more verified users
    let registry_client = identity_registry::Client::new(&e, &registry_id);
    let user3 = Address::generate(&e);
    let identity3 = Address::generate(&e);
    registry_client.mock_set_verified(&user3, &identity3, &276); // Germany

    // Batch mint
    let to_list = soroban_sdk::vec![&e, user1.clone(), user2.clone(), user3.clone()];
    let amounts = soroban_sdk::vec![&e, 100_i128, 200_i128, 300_i128];
    client.batch_mint(&to_list, &amounts);

    assert_eq!(client.balance(&user1), 100);
    assert_eq!(client.balance(&user2), 200);
    assert_eq!(client.balance(&user3), 300);
    assert_eq!(client.total_supply(), 600);
}

#[test]
fn test_is_verified() {
    let (e, token_id, _registry_id, _compliance_id, _admin, user1, _user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);

    // user1 is verified (set up in setup_test)
    assert!(client.is_verified(&user1));

    // unverified user should return false
    let unverified_user = Address::generate(&e);
    assert!(!client.is_verified(&unverified_user));
}

#[test]
fn test_can_transfer_compliance() {
    let (e, token_id, _registry_id, compliance_id, _admin, user1, user2) = setup_test();
    let client = crate::MockERC3643TokenClient::new(&e, &token_id);
    let compliance_client = compliance::Client::new(&e, &compliance_id);

    // Mint tokens to user1
    client.mint(&user1, &1000);

    // Without limits, transfer should be allowed
    assert!(client.can_transfer(&user1, &user2, &500));

    // Set transfer limit for user2 (the receiver)
    // The compliance contract checks limits on the receiver
    compliance_client.set_transfer_limit(&user2, &200);

    // Transfer within limit should be allowed
    assert!(client.can_transfer(&user1, &user2, &200));

    // Transfer exceeding limit should be rejected
    assert!(!client.can_transfer(&user1, &user2, &300));
}
