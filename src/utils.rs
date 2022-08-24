// Copyright 2020-2022 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

use anyhow::Result;
use sp_core::{crypto::Pair, crypto::Ss58Codec, sr25519};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    AccountId32,
};
use subxt::{tx::PairSigner, Config, OnlineClient};

/// Create client
pub async fn create_manta_client<T: Config>(url: &str) -> Result<OnlineClient<T>> {
    let client = OnlineClient::from_url(url).await?;

    Ok(client)
}

/// Deserialize account id.
pub fn to_account_id(address: &str) -> Result<AccountId32> {
    let account_id = AccountId32::from_string(address)?;

    Ok(account_id)
}

/// Create signer
pub fn create_signer_from_string<T, P>(seed: &str) -> Result<PairSigner<T, P>>
where
    T: Config,
    T::Signature: From<P::Signature>,
    <T::Signature as Verify>::Signer: From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    P: Pair,
    P::Signature: From<sr25519::Signature>,
    P::Public: From<sr25519::Public>,
{
    let signer_pair = P::from_string(seed.as_ref(), None).unwrap();
    let signer = PairSigner::<T, P>::new(signer_pair);

    Ok(signer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MantaConfig;
    use sp_core::{crypto::Pair, crypto::Ss58Codec, sr25519, H256};
    use std::str::FromStr;

    #[test]
    fn create_signer_from_seed_should_work() {
        let seed = "//Alice";
        let signer = create_signer_from_string::<MantaConfig, sr25519::Pair>(seed).unwrap();
        assert_eq!(
            signer.account_id().to_string(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
    }

    #[tokio::test]
    async fn transfer_balances() {
        let url = "ws://127.0.0.1:9800";
        let api = create_manta_client::<MantaConfig>(url)
            .await
            .expect("Failed to create client.");

        let seed = "//Alice";
        let signer = create_signer_from_string::<MantaConfig, sr25519::Pair>(seed).unwrap();

        let ferdie = to_account_id("dmuauuCbvcAvoo2AsSWJWaEXLxTKzFj1Y36h8EHA2o2w177C5").unwrap();

        let tx = crate::dolphin_runtime::tx()
            .balances()
            .transfer(ferdie.into(), 123_456_789_012_345_000_000);

        let tx_hash = api
            .tx()
            .sign_and_submit_default(&tx, &signer)
            .await
            .unwrap();
        println!("Balance transfer extrinsic submitted: {}", tx_hash);
    }

    #[tokio::test]
    async fn batch_calls() {
        let url = "ws://127.0.0.1:9800";
        let api = create_manta_client::<MantaConfig>(url)
            .await
            .expect("Failed to create client.");

        let seed = "//Alice";
        let signer = create_signer_from_string::<MantaConfig, sr25519::Pair>(seed).unwrap();

        let ferdie = to_account_id("dmuauuCbvcAvoo2AsSWJWaEXLxTKzFj1Y36h8EHA2o2w177C5").unwrap();

        let call = crate::dolphin_runtime::runtime_types::pallet_balances::pallet::Call::transfer {
            dest: ferdie.into(),
            value: 123_456_789_012_345_000_000,
        };
        let call = crate::dolphin_runtime::runtime_types::dolphin_runtime::Call::Balances(call);
        let batched_extrinsic = crate::dolphin_runtime::tx().utility().batch(vec![call]);
        let tx_hash = api
            .tx()
            .sign_and_submit_default(&batched_extrinsic, &signer)
            .await
            .unwrap();
        println!("Balance transfer extrinsic submitted: {}", tx_hash);
    }

    #[tokio::test]
    async fn get_proposals() {
        // let url = "wss://ws.rococo.dolphin.engineering:443";
        // let url = "wss://public-rpc.pinknode.io/kusama:9944";
        let url = "wss://ws.calamari.systems:443";
        let api = create_manta_client::<MantaConfig>(url)
            .await
            .expect("Failed to create client.");

        let proposals = crate::calamari_runtime::storage().council().proposals();
        let proposals = api.storage().fetch(&proposals, None).await.unwrap();
        dbg!(proposals);

        let proposal_hash =
            H256::from_str("0x77a01b6ab263280e43f50c795610b10a2ee61456b01682413f2a5b4df5143592")
                .unwrap();
        let proposal = crate::calamari_runtime::storage()
            .council()
            .proposal_of(&proposal_hash);

        let proposal = api.storage().fetch(&proposal, None).await.unwrap();
        dbg!(proposal);
    }
}
