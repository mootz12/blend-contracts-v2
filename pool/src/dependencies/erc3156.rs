use soroban_sdk::{contractclient, Address, Env};

/// Interface for ERC-3156 compliant flash loan receivers.
/// See: https://github.com/xycloo/xycloans/tree/main/moderc3156
#[allow(dead_code)]
#[contractclient(name = "ERC3156FlashBorrower")]
pub trait Erc3156 {
    fn exec_op(env: Env, caller: Address, token: Address, amount: i128, fee: i128);
}
