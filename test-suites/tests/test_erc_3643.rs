#![cfg(test)]

use pool::{PoolClient, Request, RequestType, ReserveEmissionMetadata};
use sep_40_oracle::testutils::Asset;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _},
    vec as svec, Address, BytesN, Error, IntoVal, String, Symbol, Vec,
};
use test_suites::{
    erc_3643::Erc3643Fixture,
    oracle::create_mock_oracle,
    pool::default_reserve_metadata,
    test_fixture::{TestFixture, TokenIndex, SCALAR_7},
};

#[test]
fn test_erc_3643_happy_path() {
    // ========== SETUP ==========

    // deploy Blend and setup env
    let fixture = TestFixture::create(true);
    let rwa_admin = Address::generate(&fixture.env);
    let _bombadil = fixture.bombadil.clone();
    let _frodo = fixture.users[0].clone();

    // create two erc3643 tokens
    let trex_1 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX1"),
        9,
    );
    let trex_2 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX2"),
        9,
    );
    const SCALAR_9: i128 = 10i128.pow(9);

    // create pool
    let pool = setup_pool(&fixture, &rwa_admin, &trex_1, &trex_2);
    let usdc = fixture.tokens.get(1).unwrap();

    fixture.jump(60 * 60);

    // ========== TEST ==========

    // create users
    let samwise = Address::generate(&fixture.env);

    // setup compliance
    trex_1.id_registry.add_allowed_country(&123);
    trex_1
        .id_registry
        .register_identity(&samwise, &samwise, &123);
    trex_1
        .compliance
        .set_transfer_limit(&samwise, &(10_000 * SCALAR_9));

    trex_2.id_registry.add_allowed_country(&123);
    trex_2
        .id_registry
        .register_identity(&samwise, &samwise, &123);
    trex_2
        .compliance
        .set_transfer_limit(&samwise, &(10 * SCALAR_9));

    // setup samwise initial state
    let samwise_initial_balance_trex_1 = 1_000 * SCALAR_9;
    trex_1.token.mint(&samwise, &samwise_initial_balance_trex_1);

    // record initial pool state
    let pool_initial_balance_trex_1 = trex_1.token.balance(&pool.address);
    let pool_initial_balance_usdc = usdc.balance(&pool.address);

    // deposit collateral
    let deposit_amount_1 = samwise_initial_balance_trex_1 / 2;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: deposit_amount_1
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(
        samwise_initial_balance_trex_1 - deposit_amount_1,
        trex_1.token.balance(&samwise)
    );
    assert_eq!(
        pool_initial_balance_trex_1 + deposit_amount_1,
        trex_1.token.balance(&pool.address)
    );

    // borrow stables against collateral
    let borrow_amount_2 = 1 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::Borrow as u32,
            address: usdc.address.clone(),
            amount: borrow_amount_2
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(borrow_amount_2, usdc.balance(&samwise));
    assert_eq!(
        pool_initial_balance_usdc - borrow_amount_2,
        usdc.balance(&pool.address)
    );
}

#[test]
fn test_erc_3643_supply_over_transfer_limit() {
    // ========== SETUP ==========
    // Note: ERC-3643 transfer limits apply to the RECEIVER of a transfer and are CUMULATIVE.
    // When a user withdraws from a pool, they receive tokens, so the user's limit is checked.
    // We test that a user cannot withdraw more than their remaining cumulative transfer limit.

    // deploy Blend and setup env
    let fixture = TestFixture::create(true);
    let rwa_admin = Address::generate(&fixture.env);

    // create two erc3643 tokens
    let trex_1 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX1"),
        9,
    );
    let trex_2 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX2"),
        9,
    );
    const SCALAR_9: i128 = 10i128.pow(9);

    // create pool
    let pool = setup_pool(&fixture, &rwa_admin, &trex_1, &trex_2);

    fixture.jump(60 * 60);

    // ========== TEST ==========

    // create user
    let samwise = Address::generate(&fixture.env);

    // Note: Transfer limits are CUMULATIVE. When we mint tokens to samwise, that counts
    // toward their cumulative transfer limit. So we need to set the limit high enough
    // to allow minting + some withdrawals.
    let mint_amount = 1_000 * SCALAR_9;
    let withdrawal_budget = 200 * SCALAR_9;
    let total_transfer_limit = mint_amount + withdrawal_budget; // 1200 * SCALAR_9

    // setup compliance
    trex_1.id_registry.add_allowed_country(&123);
    trex_1
        .id_registry
        .register_identity(&samwise, &samwise, &123);
    trex_1
        .compliance
        .set_transfer_limit(&samwise, &total_transfer_limit);

    // mint user tokens (this consumes 1000 * SCALAR_9 of the transfer limit)
    trex_1.token.mint(&samwise, &mint_amount);

    // deposit all tokens to the pool (pool has i128::MAX limit, so this succeeds)
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: mint_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(0, trex_1.token.balance(&samwise));

    // first withdrawal of 100 tokens should succeed
    // (cumulative transferred: 1000 + 100 = 1100 <= 1200 limit)
    let first_withdraw_amount = 100 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::WithdrawCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: first_withdraw_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);
    assert_eq!(first_withdraw_amount, trex_1.token.balance(&samwise));

    // second withdrawal of 100 tokens should succeed
    // (cumulative transferred: 1000 + 100 + 100 = 1200 <= 1200 limit)
    let second_withdraw_amount = 100 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::WithdrawCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: second_withdraw_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);
    assert_eq!(
        first_withdraw_amount + second_withdraw_amount,
        trex_1.token.balance(&samwise)
    );

    // third withdrawal should fail - would exceed cumulative limit
    // (cumulative transferred: 1200 + 1 = 1201 > 1200 limit)
    let third_withdraw_amount = 1 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::WithdrawCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: third_withdraw_amount
        },
    ];
    let result = pool.try_submit(&samwise, &samwise, &samwise, &requests);
    assert!(result.is_err());

    // verify user balance unchanged after failed withdrawal
    assert_eq!(
        first_withdraw_amount + second_withdraw_amount,
        trex_1.token.balance(&samwise)
    );
}

#[test]
fn test_erc_3643_supply_blocked_after_country_removed() {
    // ========== SETUP ==========
    // Note: ERC-3643 identity verification applies to the RECEIVER of a transfer.
    // When a user withdraws from a pool, the user is the receiver.
    // We test that a user cannot withdraw if their country is removed from the allowed list.

    // deploy Blend and setup env
    let fixture = TestFixture::create(true);
    let rwa_admin = Address::generate(&fixture.env);

    // create two erc3643 tokens
    let trex_1 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX1"),
        9,
    );
    let trex_2 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX2"),
        9,
    );
    const SCALAR_9: i128 = 10i128.pow(9);

    // create pool
    let pool = setup_pool(&fixture, &rwa_admin, &trex_1, &trex_2);

    fixture.jump(60 * 60);

    // ========== TEST ==========

    // create user
    let samwise = Address::generate(&fixture.env);
    let country_code = 456u32;

    // setup compliance
    trex_1.id_registry.add_allowed_country(&country_code);
    trex_1
        .id_registry
        .register_identity(&samwise, &samwise, &country_code);
    trex_1
        .compliance
        .set_transfer_limit(&samwise, &(10_000 * SCALAR_9));

    // mint user tokens
    let samwise_initial_balance_trex_1 = 1_000 * SCALAR_9;
    trex_1.token.mint(&samwise, &samwise_initial_balance_trex_1);

    // deposit all tokens
    let deposit_amount = samwise_initial_balance_trex_1;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: deposit_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(0, trex_1.token.balance(&samwise));

    // first withdrawal should succeed
    let first_withdraw_amount = 100 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::WithdrawCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: first_withdraw_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(first_withdraw_amount, trex_1.token.balance(&samwise));

    // remove the country from allowed list
    trex_1.id_registry.remove_allowed_country(&country_code);

    // second withdrawal should fail - country no longer allowed, samwise is no longer verified receiver
    let second_withdraw_amount = 100 * SCALAR_9;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::WithdrawCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: second_withdraw_amount
        },
    ];
    let result = pool.try_submit(&samwise, &samwise, &samwise, &requests);
    assert!(result.is_err());

    // verify user balance unchanged after failed second withdrawal
    assert_eq!(first_withdraw_amount, trex_1.token.balance(&samwise));
}

#[test]
fn test_erc_3643_rwa_admin_unwind_revoked_user() {
    // ========== SETUP ==========

    // deploy Blend and setup env
    let fixture = TestFixture::create(true);
    let rwa_admin = Address::generate(&fixture.env);

    // create two erc3643 tokens
    let trex_1 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX1"),
        9,
    );
    let trex_2 = Erc3643Fixture::create(
        &fixture.env,
        &fixture.bombadil,
        String::from_str(&fixture.env, "TREX2"),
        9,
    );
    const SCALAR_9: i128 = 10i128.pow(9);

    // create pool
    let pool = setup_pool(&fixture, &rwa_admin, &trex_1, &trex_2);
    let usdc = fixture.tokens.get(1).unwrap();

    fixture.jump(60 * 60);

    // ========== TEST ==========

    // create users
    let samwise = Address::generate(&fixture.env);
    let rescue_account = Address::generate(&fixture.env);

    // setup compliance for samwise
    trex_1.id_registry.add_allowed_country(&789);
    trex_1
        .id_registry
        .register_identity(&samwise, &samwise, &789);
    trex_1
        .compliance
        .set_transfer_limit(&samwise, &(10_000 * SCALAR_9));

    // setup compliance for rescue account
    trex_1
        .id_registry
        .register_identity(&rescue_account, &rescue_account, &789);
    trex_1
        .compliance
        .set_transfer_limit(&rescue_account, &i128::MAX);

    // mint user tokens
    let samwise_initial_balance_trex_1 = 1_000 * SCALAR_9;
    trex_1.token.mint(&samwise, &samwise_initial_balance_trex_1);

    // samwise deposits collateral
    let deposit_amount = samwise_initial_balance_trex_1;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex_1.token.address.clone(),
            amount: deposit_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    // samwise borrows USDC against their collateral
    // trex_1 price is $2, c_factor is 0.8, so $1000 collateral * $2 * 0.8 = $1600 borrow capacity
    // borrow $800 worth of USDC (50% utilization)
    let borrow_amount = 800 * SCALAR_7;
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::Borrow as u32,
            address: usdc.address.clone(),
            amount: borrow_amount
        },
    ];
    pool.submit(&samwise, &samwise, &samwise, &requests);

    assert_eq!(borrow_amount, usdc.balance(&samwise));

    // get samwise's positions to understand their collateral
    let samwise_positions = pool.get_positions(&samwise);
    let samwise_collateral = samwise_positions.collateral.get(1).unwrap(); // trex1 is reserve index 1

    // now revoke samwise's identity - delete their registration
    trex_1.id_registry.delete_identity(&samwise);

    // ========== RWA ADMIN UNWIND PROCESS ==========

    // attempt to transfer ALL collateral - should fail because it would undercollateralize the loan
    // the user needs collateral to maintain their USDC liability
    let transfer_all_result = pool.try_authorized_transfer(
        &trex_1.token.address,
        &samwise,
        &rescue_account,
        &samwise_collateral,
        &true,
    );
    // should fail with InvalidHf error (#1205)
    assert_eq!(
        transfer_all_result.err(),
        Some(Ok(Error::from_contract_error(1205)))
    );

    // calculate safe amount to transfer - need to keep enough collateral for the loan
    // borrow_amount is $800, needs $1000 worth of collateral at 0.8 c_factor ($800 / 0.8 = $1000)
    // at $2 per token, need 500 * SCALAR_9 tokens as collateral
    // we have 1000 * SCALAR_9, so we can safely transfer about 500 * SCALAR_9 (minus some buffer)
    let safe_transfer_amount = 400 * SCALAR_9; // transfer 400 tokens, keep 600 for safety

    // convert to bTokens - for simplicity assuming 1:1 ratio at start
    // in reality would need to check the reserve's b_rate
    let safe_transfer_btokens = safe_transfer_amount;

    // RWA admin transfers partial collateral to rescue account
    // this call should NOT require samwise's signature
    pool.authorized_transfer(
        &trex_1.token.address,
        &samwise,
        &rescue_account,
        &safe_transfer_btokens,
        &true,
    );

    // validate that the user (samwise) did NOT have to sign the transaction
    // only the rwa_admin should have authorized
    let auths = fixture.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(
        auths[0],
        (
            rwa_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&fixture.env, "authorized_transfer"),
                    svec![
                        &fixture.env,
                        trex_1.token.address.to_val(),
                        samwise.to_val(),
                        rescue_account.to_val(),
                        safe_transfer_btokens.into_val(&fixture.env),
                        true.into_val(&fixture.env),
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // RWA admin repays the liability on behalf of samwise
    // Get the actual liability amount first (for reference, though we use a fixed amount)
    let samwise_positions_before_repay = pool.get_positions(&samwise);
    let _usdc_liability = samwise_positions_before_repay.liabilities.get(0).unwrap(); // USDC is reserve index 0

    // mint USDC to rwa_admin so they can repay (extra for interest and rounding)
    let repay_amount = borrow_amount + 100 * SCALAR_7;
    usdc.mint(&rwa_admin, &repay_amount);

    let repay_requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::Repay as u32,
            address: usdc.address.clone(),
            amount: repay_amount // use specific amount instead of i128::MAX
        },
    ];
    // submit on behalf of samwise using rwa_admin as spender
    pool.submit(&samwise, &rwa_admin, &rwa_admin, &repay_requests);

    // verify samwise's liability is cleared
    let samwise_positions_after_repay = pool.get_positions(&samwise);
    assert!(samwise_positions_after_repay.liabilities.is_empty());

    // now transfer remaining collateral - should succeed since no more liabilities
    let remaining_collateral = samwise_positions_after_repay.collateral.get(1).unwrap();
    pool.authorized_transfer(
        &trex_1.token.address,
        &samwise,
        &rescue_account,
        &remaining_collateral,
        &true,
    );

    // verify samwise's position is fully unwound
    let samwise_final_positions = pool.get_positions(&samwise);
    assert!(samwise_final_positions.collateral.is_empty());
    assert!(samwise_final_positions.liabilities.is_empty());

    // verify rescue account received all the collateral (as underlying tokens when withdrawn)
    // the rescue account now holds the bTokens in the pool
    let rescue_positions = pool.get_positions(&rescue_account);
    assert!(rescue_positions.collateral.get(1).unwrap() > 0);
}

/// Create a pool with USDC, and two ERC-3643 tokens
///
/// Registers frodo (user 0) and pool as country code 0 (the ether), and allows them both to transfer
/// i128::MAX amount of the trex tokens, and approves country code 0 for both tokens
fn setup_pool<'a>(
    fixture: &'a TestFixture,
    rwa_admin: &Address,
    trex1: &'a Erc3643Fixture,
    trex2: &'a Erc3643Fixture,
) -> PoolClient<'a> {
    let usdc = fixture.tokens.get(1).unwrap();
    let frodo = fixture.users[0].clone();

    // create oracle
    let (mock_oracle, mock_oracle_client) = create_mock_oracle(&fixture.env);
    mock_oracle_client.set_data(
        &fixture.bombadil,
        &Asset::Other(Symbol::new(&fixture.env, "USD")),
        &svec![
            &fixture.env,
            Asset::Stellar(usdc.address.clone()),
            Asset::Stellar(trex1.token.address.clone()),
            Asset::Stellar(trex2.token.address.clone()),
        ],
        &7,
        &300,
    );
    mock_oracle_client.set_price_stable(&svec![
        &fixture.env,
        1_0000000,   // usdc
        2_0000000,   // trex1
        500_0000000, // trex2
    ]);

    // create pool
    let pool_name = String::from_str(&fixture.env, "ERC3643");
    let pool_address = fixture.pool_factory.deploy(
        &fixture.bombadil,
        &pool_name,
        &BytesN::<32>::random(&fixture.env),
        &mock_oracle,
        &100_0000,
        &6,
        &1_0000000,
    );

    let pool_client = PoolClient::new(&fixture.env, &pool_address);

    // initialize pool to active state with reserves ($10k supplied, $5k borrowed each)

    // 1. init lp
    fixture.tokens[TokenIndex::BLND].mint(&frodo, &(110_000_000 * SCALAR_7));
    fixture.tokens[TokenIndex::USDC].mint(&frodo, &(2_600_000 * SCALAR_7));
    fixture.lp.join_pool(
        &(10_000_000 * SCALAR_7),
        &svec![&fixture.env, 110_000_000 * SCALAR_7, 2_600_000 * SCALAR_7,],
        &frodo,
    );

    // 2. create reserves
    let mut usdc_config = default_reserve_metadata();
    usdc_config.decimals = 7;
    usdc_config.c_factor = 0_900_0000;
    usdc_config.l_factor = 0_950_0000;
    usdc_config.util = 0_700_0000;
    pool_client.queue_set_reserve(&fixture.tokens[TokenIndex::USDC].address, &usdc_config);
    pool_client.set_reserve(&fixture.tokens[TokenIndex::USDC].address);

    let mut trex_1_config = default_reserve_metadata();
    trex_1_config.decimals = trex1.token.decimals();
    trex_1_config.c_factor = 0_800_0000;
    trex_1_config.l_factor = 0;
    trex_1_config.util = 0_500_0000;
    trex_1_config.rwa = true;
    trex_1_config.rwa_admin = Some(rwa_admin.clone());
    pool_client.queue_set_reserve(&trex1.token.address, &trex_1_config);
    pool_client.set_reserve(&trex1.token.address);

    let mut trex_2_config = default_reserve_metadata();
    trex_2_config.decimals = trex2.token.decimals();
    trex_2_config.c_factor = 0_850_0000;
    trex_2_config.l_factor = 0;
    trex_2_config.util = 0_500_0000;
    trex_2_config.rwa = true;
    trex_2_config.rwa_admin = Some(rwa_admin.clone());
    pool_client.queue_set_reserve(&trex2.token.address, &trex_2_config);
    pool_client.set_reserve(&trex2.token.address);

    // 4. setup emissions
    let reserve_emissions: soroban_sdk::Vec<ReserveEmissionMetadata> = svec![
        &fixture.env,
        ReserveEmissionMetadata {
            res_index: 1, // trex1
            res_type: 0,  // d_token
            share: 0_600_0000
        },
        ReserveEmissionMetadata {
            res_index: 2, // trex2
            res_type: 1,  // b_token
            share: 0_400_0000
        },
    ];
    pool_client.set_emissions_config(&reserve_emissions);

    // deposit into backstop, add to reward zone
    fixture
        .backstop
        .deposit(&frodo, &pool_client.address, &(50_000 * SCALAR_7));
    fixture.backstop.add_reward(&pool_client.address, &None);
    pool_client.set_status(&3);
    pool_client.update_status();

    // enable emissions
    fixture.emitter.distribute();
    fixture.backstop.distribute();
    pool_client.gulp_emissions();

    fixture.jump_with_sequence(60);

    // deposit funds into the pool to seed some liquidity

    // step 1 - setup compliance so pool and frodo can hold / transfer tokens
    trex1.id_registry.add_allowed_country(&0);
    trex1
        .id_registry
        .register_identity(&pool_client.address, &pool_client.address, &0);
    trex1.id_registry.register_identity(&frodo, &frodo, &0);
    trex1
        .compliance
        .set_transfer_limit(&pool_client.address, &i128::MAX);
    trex1.compliance.set_transfer_limit(&frodo, &i128::MAX);

    trex2.id_registry.add_allowed_country(&0);
    trex2
        .id_registry
        .register_identity(&pool_client.address, &pool_client.address, &0);
    trex2.id_registry.register_identity(&frodo, &frodo, &0);
    trex2
        .compliance
        .set_transfer_limit(&pool_client.address, &i128::MAX);
    trex2.compliance.set_transfer_limit(&frodo, &i128::MAX);

    // step 2 - mint tokens to frodo
    fixture.tokens[TokenIndex::USDC].mint(&frodo, &(10_000 * SCALAR_7));
    let trex1_decimals = trex1.token.decimals();
    let trex2_decimals = trex2.token.decimals();
    trex1
        .token
        .mint(&frodo, &(5_000 * 10i128.pow(trex1_decimals)));
    trex2.token.mint(&frodo, &(20 * 10i128.pow(trex2_decimals)));

    // step 3 - deposit and borrow tokens
    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::USDC].address.clone(),
            amount: 10_000 * SCALAR_7,
        },
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex1.token.address.clone(),
            amount: 5_000 * 10i128.pow(trex1_decimals),
        },
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: trex2.token.address.clone(),
            amount: 20 * 10i128.pow(trex2_decimals),
        },
    ];
    pool_client.submit(&frodo, &frodo, &frodo, &requests);

    let requests: Vec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::USDC].address.clone(),
            amount: 5_000 * SCALAR_7,
        },
    ];
    pool_client.submit(&frodo, &frodo, &frodo, &requests);

    pool_client
}
