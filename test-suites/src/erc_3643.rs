use soroban_sdk::{Address, Env, String};

mod erc_3643_token {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/mock_erc3643.wasm");
}

mod erc_3643_compliance {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/mock_compliance.wasm");
}

mod erc_3643_identity_registry {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/mock_identity_registry.wasm"
    );
}

pub use erc_3643_compliance::Client as ComplianceClient;
pub use erc_3643_identity_registry::Client as IdentityRegistryClient;
pub use erc_3643_token::Client as Erc3643Client;

pub struct Erc3643Fixture<'a> {
    pub token: Erc3643Client<'a>,
    pub compliance: ComplianceClient<'a>,
    pub id_registry: IdentityRegistryClient<'a>,
}

impl Erc3643Fixture<'_> {
    /// Assumes env is already configured
    pub fn create<'a>(
        e: &Env,
        admin: &Address,
        symbol: String,
        decimals: u32,
    ) -> Erc3643Fixture<'a> {
        let compliance_address = e.register(erc_3643_compliance::WASM, (admin.clone(),));
        let compliance_client = ComplianceClient::new(e, &compliance_address);

        let id_registry_address = e.register(erc_3643_identity_registry::WASM, (admin.clone(),));
        let id_registry_client = IdentityRegistryClient::new(e, &id_registry_address);

        let token_address = e.register(
            erc_3643_token::WASM,
            (
                admin.clone(),
                symbol.clone(),
                symbol,
                decimals,
                id_registry_address.clone(),
                compliance_address.clone(),
            ),
        );
        let token_client = Erc3643Client::new(e, &token_address);

        Erc3643Fixture {
            token: token_client,
            compliance: compliance_client,
            id_registry: id_registry_client,
        }
    }
}
