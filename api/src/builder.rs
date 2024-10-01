use crate::{
    common_types::{KeyPair, Rpc, SubstrateApi},
    TitanhApi,
};
use sp_core::Pair;
use subxt::{backend::rpc::RpcClient, tx::PairSigner};

pub struct TitanhApiBuilder {
    /// The rpc url of the substrate node
    rpc_url: String,
    /// The eventual seed phrase of the user
    seed_phrase: Option<String>,
}

impl TitanhApiBuilder {
    pub fn rpc(url: &str) -> Self {
        TitanhApiBuilder {
            rpc_url: url.to_string(),
            seed_phrase: None,
        }
    }

    pub fn seed(self, seed: &str) -> Self {
        Self {
            seed_phrase: Some(seed.to_string()),
            ..self
        }
    }

    pub async fn build(self) -> TitanhApi {
        // Derive the key pair from the seed phrase (mnemonic)
        let signer = if let Some(seed_phrase) = self.seed_phrase {
            let pair = KeyPair::from_string(&seed_phrase, None).expect("Invalid key pair");
            let signer = PairSigner::new(pair);
            Some(signer)
        } else {
            None
        };
        // SECURITY NOTE: We are using an insecure connection assuming that the node is communicating with a trusted local network node: NO NOT RUN THIS CODE IN PRODUCTION
        let rpc_client = RpcClient::from_insecure_url(self.rpc_url)
            .await
            .expect("Invalid rpc url");
        let rpc = Rpc::new(rpc_client.clone());

        // We can use the same client to drive our full Subxt interface
        let api = SubstrateApi::from_rpc_client(rpc_client.clone())
            .await
            .expect("Expected a valid Substrate API");

        TitanhApi::new(api, rpc, signer)
    }
}
