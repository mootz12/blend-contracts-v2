use soroban_sdk::{Address, Env};

mod emitter {
    soroban_sdk::contractimport!(file = "../emitter/emitter_v1.0.0.wasm");
}

pub fn create_emitter<'a>(e: &Env) -> (Address, emitter::Client<'a>) {
    let contract_id = e.register(emitter::WASM, ());
    (contract_id.clone(), emitter::Client::new(e, &contract_id))
}

pub use emitter::Client as EmitterClient;