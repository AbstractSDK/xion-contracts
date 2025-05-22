use abstract_interface::Abstract;
use cosmos_sdk_proto::{
    cosmos::feegrant::v1beta1::BasicAllowance, prost::Name, traits::MessageExt, Any,
};
use cw_orch::{anyhow, daemon::Daemon, prelude::*};
use treasury::{
    grant::{
        AllowanceData, AllowanceOnAccount, AuthorizationData, AuthorizationOnAccount,
        FeeConfigStorage, GrantConfigStorage,
    },
    msg::InstantiateMsg,
    Treasury, XION_TESTNET_2,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    let chain = Daemon::builder(XION_TESTNET_2).build()?;
    let treasury = Treasury::new("account_treasury_contract", chain.clone());
    let abstract_ = Abstract::load_from(chain.clone())?;

    treasury.upload_if_needed()?;
    treasury.instantiate(
        &InstantiateMsg {
            admin: Some(chain.sender_addr()),
            type_urls: vec!["/cosmwasm.wasm.v1.MsgExecuteContract".to_string()],
            grant_configs: vec![GrantConfigStorage {
                description: "First grant for execution on the account itself".to_string(),
                authorization: AuthorizationData::ExecuteOnAccount(AuthorizationOnAccount {
                    limit: Some(
                        Any {
                            type_url: cosmos_sdk_proto::cosmwasm::wasm::v1::MaxCallsLimit::type_url(),
                            value:
                                cosmos_sdk_proto::cosmwasm::wasm::v1::MaxCallsLimit { remaining: 1 }
                                .to_bytes()?,
                        }
                        .into()),
                    filter: Some(
                        Any {
                            type_url: cosmos_sdk_proto::cosmwasm::wasm::v1::AcceptedMessageKeysFilter::type_url(),
                            value:
                                cosmos_sdk_proto::cosmwasm::wasm::v1::AcceptedMessageKeysFilter {
                                    keys: vec!["install_modules".to_string()]
                                }
                                .to_bytes()?,
                        }
                        .into(),
                    ),
                }),
                optional: false,
            }],
            fee_config: FeeConfigStorage {
                description: "First grant for execution on the account itself".to_string(),
                allowance: Some(AllowanceData::AllowanceOnAccount(AllowanceOnAccount {
                    allowance: Some(
                        Any {
                            type_url: "/cosmos.feegrant.v1beta1.BasicAllowance".to_string(),
                            value: BasicAllowance {
                                spend_limit: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                    denom: "uxion".to_string(),
                                    amount: "1000000000".to_string(),
                                }],
                                expiration: None,
                            }
                            .to_bytes()?,
                        }
                        .into(),
                    ),
                })),
                expiration: None,
            },
            abstract_code_id: abstract_.registry.code_id()?,
        },
        None,
        &[],
    )?;

    Ok(())
}
