pub mod identity_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/mock_identity_registry.wasm"
    );
}

pub mod compliance {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/mock_compliance.wasm");
}
