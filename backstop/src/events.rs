use soroban_sdk::{contractevent, Address, Env};

/// Emitted when tokens are deposited into a backstop
///
/// - topics - `["deposit", pool_address: Address, from: Address]`
/// - data - `[tokens_in: i128, backstop_shares_minted: i128]`
#[contractevent(data_format = "vec")]
pub struct Deposit {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub from: Address,
    pub tokens_in: i128,
    pub backstop_shares_minted: i128,
}

/// Emitted when a withdrawal is queued
///
/// - topics - `["queue_withdrawal", pool_address: Address, from: Address]`
/// - data - `[amount: i128, expiration: u64]`
#[contractevent(data_format = "vec")]
pub struct QueueWithdrawal {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub from: Address,
    pub amount: i128,
    pub expiration: u64,
}

/// Emitted when a withdrawal is dequeued
///
/// - topics - `["dequeue_withdrawal", pool_address: Address, from: Address]`
/// - data - `amount: i128`
#[contractevent(data_format = "single-value")]
pub struct DequeueWithdrawal {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub from: Address,
    pub amount: i128,
}

/// Emitted when tokens are withdrawn from the backstop
///
/// - topics - `["withdraw", pool_address: Address, from: Address]`
/// - data - `[amount: i128, tokens_out: i128]`
#[contractevent(data_format = "vec")]
pub struct Withdraw {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub from: Address,
    pub amount: i128,
    pub tokens_out: i128,
}

/// Emitted when new emissions are distributed
///
/// - topics - `["distribute"]`
/// - data - `new_tokens_emitted: i128`
#[contractevent(data_format = "single-value")]
pub struct Distribute {
    pub new_tokens_emitted: i128,
}

/// Emitted when new emissions are gulped
///
/// - topics - `["gulp_emissions", pool_address: Address]`
/// - data - `[new_backstop_emissions: i128, new_pool_emissions: i128]`
#[contractevent(data_format = "vec")]
pub struct GulpEmissions {
    #[topic]
    pub pool_address: Address,
    pub new_backstop_emissions: i128,
    pub new_pool_emissions: i128,
}

/// Emitted when the reward zone is updated
///
/// - topics - `["rw_zone_add"]`
/// - data - `[to_add: Address, to_remove: Option<Address>]`
#[contractevent(data_format = "vec")]
pub struct RwZoneAdd {
    pub to_add: Address,
    pub to_remove: Option<Address>,
}

/// Emitted when a pool is removed from the reward zone
///
/// - topics - `["rw_zone_remove"]`
/// - data - `to_remove: Address`
#[contractevent(data_format = "single-value")]
pub struct RwZoneRemove {
    pub to_remove: Address,
}

/// Emitted when emissions are claimed
///
/// - topics - `["claim", from: Address]`
/// - data - `amount: i128`
#[contractevent(data_format = "single-value")]
pub struct Claim {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

/// Emitted when tokens are drawn from the backstop
///
/// - topics - `["draw", pool_address: Address]`
/// - data - `[to: Address, amount: i128]`
#[contractevent(data_format = "vec")]
pub struct Draw {
    #[topic]
    pub pool_address: Address,
    pub to: Address,
    pub amount: i128,
}

/// Emitted when tokens are donated to the backstop
///
/// - topics - `["donate", pool_address: Address, from: Address]`
/// - data - `amount: i128`
#[contractevent(data_format = "single-value")]
pub struct Donate {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub from: Address,
    pub amount: i128,
}

pub struct BackstopEvents {}

impl BackstopEvents {
    pub fn deposit(
        e: &Env,
        pool_address: Address,
        from: Address,
        tokens_in: i128,
        backstop_shares_minted: i128,
    ) {
        Deposit {
            pool_address,
            from,
            tokens_in,
            backstop_shares_minted,
        }
        .publish(e);
    }

    pub fn queue_withdrawal(
        e: &Env,
        pool_address: Address,
        from: Address,
        amount: i128,
        expiration: u64,
    ) {
        QueueWithdrawal {
            pool_address,
            from,
            amount,
            expiration,
        }
        .publish(e);
    }

    pub fn dequeue_withdrawal(e: &Env, pool_address: Address, from: Address, amount: i128) {
        DequeueWithdrawal {
            pool_address,
            from,
            amount,
        }
        .publish(e);
    }

    pub fn withdraw(e: &Env, pool_address: Address, from: Address, amount: i128, tokens_out: i128) {
        Withdraw {
            pool_address,
            from,
            amount,
            tokens_out,
        }
        .publish(e);
    }

    pub fn distribute(e: &Env, new_tokens_emitted: i128) {
        Distribute { new_tokens_emitted }.publish(e);
    }

    pub fn gulp_emissions(
        e: &Env,
        pool_address: Address,
        new_backstop_emissions: i128,
        new_pool_emissions: i128,
    ) {
        GulpEmissions {
            pool_address,
            new_backstop_emissions,
            new_pool_emissions,
        }
        .publish(e);
    }

    pub fn rw_zone_add(e: &Env, to_add: Address, to_remove: Option<Address>) {
        RwZoneAdd { to_add, to_remove }.publish(e);
    }

    pub fn rw_zone_remove(e: &Env, to_remove: Address) {
        RwZoneRemove { to_remove }.publish(e);
    }

    pub fn claim(e: &Env, from: Address, amount: i128) {
        Claim { from, amount }.publish(e);
    }

    pub fn draw(e: &Env, pool_address: Address, to: Address, amount: i128) {
        Draw {
            pool_address,
            to: to,
            amount,
        }
        .publish(e);
    }

    pub fn donate(e: &Env, pool_address: Address, from: Address, amount: i128) {
        Donate {
            pool_address,
            from,
            amount,
        }
        .publish(e);
    }
}
