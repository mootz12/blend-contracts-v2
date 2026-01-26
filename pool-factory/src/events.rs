use soroban_sdk::{contractevent, Address, Env};

/// Emitted when a pool is deployed by the factory
///
/// - topics - `["deploy"]`
/// - data - `Address`
#[contractevent(data_format = "single-value")]
pub struct Deploy {
    pub pool_address: Address,
}

pub struct PoolFactoryEvents {}

impl PoolFactoryEvents {
    pub fn deploy(e: &Env, pool_address: Address) {
        Deploy { pool_address }.publish(e);
    }
}
