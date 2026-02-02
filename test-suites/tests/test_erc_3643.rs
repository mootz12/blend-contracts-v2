#![cfg(test)]

use pool::{PoolClient, Request, RequestType, ReserveEmissionMetadata};
use sep_40_oracle::testutils::Asset;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    vec as svec, Address, BytesN, String, Symbol, Vec,
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
    let bombadil = fixture.bombadil.clone();
    let frodo = fixture.users[0].clone();

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
