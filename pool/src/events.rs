use soroban_sdk::{contractevent, Address, Env, Vec};

use crate::{AuctionData, ReserveConfig};

/// Emitted when a new admin is set for a pool
///
/// - topics - `["set_admin", admin: Address]`
/// - data - `new_admin: Address`
#[contractevent(data_format = "single-value")]
pub struct SetAdmin {
    #[topic]
    pub admin: Address,
    pub new_admin: Address,
}

/// Emitted when pool parameters are updated
///
/// - topics - `["update_pool", admin: Address]`
/// - data - `[backstop_take_rate: u32, max_positions: u32, min_collateral: i128]`
#[contractevent(data_format = "vec")]
pub struct UpdatePool {
    #[topic]
    pub admin: Address,
    pub backstop_take_rate: u32,
    pub max_positions: u32,
    pub min_collateral: i128,
}

/// Emitted when a new reserve configuration change is queued
///
/// - topics - `["queue_set_reserve", admin: Address]`
/// - data - `[asset: Address, metadata: ReserveConfig]`
#[contractevent(data_format = "vec")]
pub struct QueueSetReserve {
    #[topic]
    pub admin: Address,
    pub asset: Address,
    pub metadata: ReserveConfig,
}

/// Emitted when a queued reserve configuration change is cancelled
///
/// - topics - `["cancel_set_reserve", admin: Address]`
/// - data - `asset: Address`
#[contractevent(data_format = "single-value")]
pub struct CancelSetReserve {
    #[topic]
    pub admin: Address,
    pub asset: Address,
}

/// Emitted when a reserve configuration change is set
///
/// - topics - `["set_reserve"]`
/// - data - `[asset: Address, index: u32]`
#[contractevent(data_format = "vec")]
pub struct SetReserve {
    pub asset: Address,
    pub index: u32,
}

/// Emitted when pool status is updated (non-admin)
///
/// - topics - `["set_status"]`
/// - data - `new_status: u32`
#[contractevent(topics = ["set_status"], data_format = "single-value")]
pub struct SetStatus {
    pub new_status: u32,
}

/// Emitted when pool status is updated by admin
///
/// - topics - `["set_status", admin: Address]`
/// - data - `pool_status: u32`
#[contractevent(topics = ["set_status"], data_format = "single-value")]
pub struct SetStatusAdmin {
    #[topic]
    pub admin: Address,
    pub pool_status: u32,
}

/// Emitted when reserve emissions are updated
///
/// - topics - `["reserve_emission_update"]`
/// - data - `[res_token_id: u32, eps: u64, expiration: u64]`
#[contractevent(data_format = "vec")]
pub struct ReserveEmissionUpdate {
    pub res_token_id: u32,
    pub eps: u64,
    pub expiration: u64,
}

/// Emitted when emissions are gulped
///
/// - topics - `["gulp_emissions"]`
/// - data - `emissions: i128`
#[contractevent(data_format = "single-value")]
pub struct GulpEmissions {
    pub emissions: i128,
}

/// Emitted when emissions are claimed
///
/// - topics - `["claim", from: Address]`
/// - data - `[reserve_token_ids: Vec<u32>, amount_claimed: i128]`
#[contractevent(data_format = "vec")]
pub struct Claim {
    #[topic]
    pub from: Address,
    pub reserve_token_ids: Vec<u32>,
    pub amount_claimed: i128,
}

/// Emitted when bad debt is recorded
///
/// - topics - `["bad_debt", user: Address, asset: Address]`
/// - data - `d_tokens: i128`
#[contractevent(data_format = "single-value")]
pub struct BadDebt {
    #[topic]
    pub user: Address,
    #[topic]
    pub asset: Address,
    pub d_tokens: i128,
}

/// Emitted when bad debt is defaulted
///
/// - topics - `["defaulted_debt", asset: Address]`
/// - data - `d_tokens_burnt: i128`
#[contractevent(data_format = "single-value")]
pub struct DefaultedDebt {
    #[topic]
    pub asset: Address,
    pub d_tokens_burnt: i128,
}

/// Emitted when tokens are supplied
///
/// - topics - `["supply", asset: Address, from: Address]`
/// - data - `[tokens_in: i128, b_tokens_minted: i128]`
#[contractevent(data_format = "vec")]
pub struct Supply {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_in: i128,
    pub b_tokens_minted: i128,
}

/// Emitted when tokens are withdrawn
///
/// - topics - `["withdraw", asset: Address, from: Address]`
/// - data - `[tokens_out: i128, b_tokens_burnt: i128]`
#[contractevent(data_format = "vec")]
pub struct Withdraw {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_out: i128,
    pub b_tokens_burnt: i128,
}

/// Emitted when collateral is supplied
///
/// - topics - `["supply_collateral", asset: Address, from: Address]`
/// - data - `[tokens_in: i128, b_tokens_minted: i128]`
#[contractevent(data_format = "vec")]
pub struct SupplyCollateral {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_in: i128,
    pub b_tokens_minted: i128,
}

/// Emitted when collateral is withdrawn
///
/// - topics - `["withdraw_collateral", asset: Address, from: Address]`
/// - data - `[tokens_out: i128, b_tokens_burnt: i128]`
#[contractevent(data_format = "vec")]
pub struct WithdrawCollateral {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_out: i128,
    pub b_tokens_burnt: i128,
}

/// Emitted when tokens are borrowed
///
/// - topics - `["borrow", asset: Address, from: Address]`
/// - data - `[tokens_out: i128, d_tokens_minted: i128]`
#[contractevent(data_format = "vec")]
pub struct Borrow {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_out: i128,
    pub d_tokens_minted: i128,
}

/// Emitted when a loan is repaid
///
/// - topics - `["repay", asset: Address, from: Address]`
/// - data - `[tokens_in: i128, d_tokens_burnt: i128]`
#[contractevent(data_format = "vec")]
pub struct Repay {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    pub tokens_in: i128,
    pub d_tokens_burnt: i128,
}

/// Emitted when an authorized transfer occurs
///
/// - topics - `["authorized_transfer", asset: Address, from: Address, to: Address, admin: Address]`
/// - data - `[tokens: i128, b_tokens: i128, collateral: bool]`
#[contractevent(data_format = "vec")]
pub struct AuthorizedTransfer {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    #[topic]
    pub admin: Address,
    pub tokens: i128,
    pub b_tokens: i128,
    pub collateral: bool,
}

/// Emitted during a flash loan
///
/// - topics - `["flash_loan", asset: Address, from: Address, contract: Address]`
/// - data - `[tokens_out: i128, d_tokens_minted: i128]`
#[contractevent(topics = ["flash_loan"], data_format = "vec")]
pub struct FlashLoanEvent {
    #[topic]
    pub asset: Address,
    #[topic]
    pub from: Address,
    #[topic]
    pub contract: Address,
    pub tokens_out: i128,
    pub d_tokens_minted: i128,
}

/// Emitted when a reserve gulps excess tokens
///
/// - topics - `["gulp", asset: Address]`
/// - data - `token_delta: i128`
#[contractevent(data_format = "single-value")]
pub struct Gulp {
    #[topic]
    pub asset: Address,
    pub token_delta: i128,
}

/// Emitted when a new auction is created
///
/// - topics - `["new_auction", auction_type: u32, user: Address]`
/// - data - `[percent: u32, auction_data: AuctionData]`
#[contractevent(data_format = "vec")]
pub struct NewAuction {
    #[topic]
    pub auction_type: u32,
    #[topic]
    pub user: Address,
    pub percent: u32,
    pub auction_data: AuctionData,
}

/// Emitted when an auction is filled
///
/// - topics - `["fill_auction", auction_type: u32, user: Address]`
/// - data - `[filler: Address, fill_percent: i128, filled_auction_data: AuctionData]`
#[contractevent(data_format = "vec")]
pub struct FillAuction {
    #[topic]
    pub auction_type: u32,
    #[topic]
    pub user: Address,
    pub filler: Address,
    pub fill_percent: i128,
    pub filled_auction_data: AuctionData,
}

/// Emitted when an auction is deleted
///
/// - topics - `["delete_auction", auction_type: u32, user: Address]`
/// - data - `()`
#[contractevent(data_format = "single-value")]
pub struct DeleteAuction {
    #[topic]
    pub auction_type: u32,
    #[topic]
    pub user: Address,
    pub empty: (),
}

pub struct PoolEvents {}

impl PoolEvents {
    pub fn set_admin(e: &Env, admin: Address, new_admin: Address) {
        SetAdmin { admin, new_admin }.publish(e);
    }

    pub fn update_pool(
        e: &Env,
        admin: Address,
        backstop_take_rate: u32,
        max_positions: u32,
        min_collateral: i128,
    ) {
        UpdatePool {
            admin,
            backstop_take_rate,
            max_positions,
            min_collateral,
        }
        .publish(e);
    }

    pub fn queue_set_reserve(e: &Env, admin: Address, asset: Address, metadata: ReserveConfig) {
        QueueSetReserve {
            admin,
            asset,
            metadata,
        }
        .publish(e);
    }

    pub fn cancel_set_reserve(e: &Env, admin: Address, asset: Address) {
        CancelSetReserve { admin, asset }.publish(e);
    }

    pub fn set_reserve(e: &Env, asset: Address, index: u32) {
        SetReserve { asset, index }.publish(e);
    }

    pub fn set_status(e: &Env, new_status: u32) {
        SetStatus { new_status }.publish(e);
    }

    pub fn set_status_admin(e: &Env, admin: Address, pool_status: u32) {
        SetStatusAdmin { admin, pool_status }.publish(e);
    }

    pub fn reserve_emission_update(e: &Env, res_token_id: u32, eps: u64, expiration: u64) {
        ReserveEmissionUpdate {
            res_token_id,
            eps,
            expiration,
        }
        .publish(e);
    }

    pub fn gulp_emissions(e: &Env, emissions: i128) {
        GulpEmissions { emissions }.publish(e);
    }

    pub fn claim(e: &Env, from: Address, reserve_token_ids: Vec<u32>, amount_claimed: i128) {
        Claim {
            from,
            reserve_token_ids,
            amount_claimed,
        }
        .publish(e);
    }

    pub fn bad_debt(e: &Env, user: Address, asset: Address, d_tokens: i128) {
        BadDebt {
            user,
            asset,
            d_tokens,
        }
        .publish(e);
    }

    pub fn defaulted_debt(e: &Env, asset: Address, d_tokens_burnt: i128) {
        DefaultedDebt {
            asset,
            d_tokens_burnt,
        }
        .publish(e);
    }

    pub fn supply(e: &Env, asset: Address, from: Address, tokens_in: i128, b_tokens_minted: i128) {
        Supply {
            asset,
            from,
            tokens_in,
            b_tokens_minted,
        }
        .publish(e);
    }

    pub fn withdraw(
        e: &Env,
        asset: Address,
        from: Address,
        tokens_out: i128,
        b_tokens_burnt: i128,
    ) {
        Withdraw {
            asset,
            from,
            tokens_out,
            b_tokens_burnt,
        }
        .publish(e);
    }

    pub fn supply_collateral(
        e: &Env,
        asset: Address,
        from: Address,
        tokens_in: i128,
        b_tokens_minted: i128,
    ) {
        SupplyCollateral {
            asset,
            from,
            tokens_in,
            b_tokens_minted,
        }
        .publish(e);
    }

    pub fn withdraw_collateral(
        e: &Env,
        asset: Address,
        from: Address,
        tokens_out: i128,
        b_tokens_burnt: i128,
    ) {
        WithdrawCollateral {
            asset,
            from,
            tokens_out,
            b_tokens_burnt,
        }
        .publish(e);
    }

    pub fn borrow(e: &Env, asset: Address, from: Address, tokens_out: i128, d_tokens_minted: i128) {
        Borrow {
            asset,
            from,
            tokens_out,
            d_tokens_minted,
        }
        .publish(e);
    }

    pub fn repay(e: &Env, asset: Address, from: Address, tokens_in: i128, d_tokens_burnt: i128) {
        Repay {
            asset,
            from,
            tokens_in,
            d_tokens_burnt,
        }
        .publish(e);
    }

    pub fn authorized_transfer(
        e: &Env,
        asset: Address,
        from: Address,
        to: Address,
        admin: Address,
        tokens: i128,
        b_tokens: i128,
        collateral: bool,
    ) {
        AuthorizedTransfer {
            asset,
            from,
            to,
            admin,
            tokens,
            b_tokens,
            collateral,
        }
        .publish(e);
    }

    pub fn flash_loan(
        e: &Env,
        asset: Address,
        from: Address,
        contract: Address,
        tokens_out: i128,
        d_tokens_minted: i128,
    ) {
        FlashLoanEvent {
            asset,
            from,
            contract,
            tokens_out: tokens_out,
            d_tokens_minted,
        }
        .publish(e);
    }

    pub fn gulp(e: &Env, asset: Address, token_delta: i128) {
        Gulp { asset, token_delta }.publish(e);
    }

    pub fn new_auction(
        e: &Env,
        auction_type: u32,
        user: Address,
        percent: u32,
        auction_data: AuctionData,
    ) {
        NewAuction {
            auction_type,
            user,
            percent,
            auction_data,
        }
        .publish(e);
    }

    pub fn fill_auction(
        e: &Env,
        auction_type: u32,
        user: Address,
        filler: Address,
        fill_percent: i128,
        filled_auction_data: AuctionData,
    ) {
        FillAuction {
            auction_type,
            user,
            filler,
            fill_percent,
            filled_auction_data,
        }
        .publish(e);
    }

    pub fn delete_auction(e: &Env, auction_type: u32, user: Address) {
        DeleteAuction {
            auction_type,
            user,
            empty: (),
        }
        .publish(e);
    }
}
