use abstract_interface::Abstract;
use abstract_std::objects::module::ModuleInfo;
use cosmos_sdk_proto::{
    cosmos::feegrant::v1beta1::BasicAllowance, prost::Name, traits::MessageExt, Any,
};
use cw_orch::{anyhow, daemon::Daemon, prelude::*};
use treasury::{
    grant::{
        AllowanceData, AllowanceOnModule, AuthorizationData, AuthorizationOnModule,
        FeeConfigStorage, GrantConfigStorage,
    },
    msg::InstantiateMsg,
    Treasury, XION_TESTNET_2,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    let chain = Daemon::builder(XION_TESTNET_2).build()?;
    let treasury = Treasury::new("treasury_for_module", chain.clone());
    let abstract_ = Abstract::load_from(chain)?;

    let target_module_id = ModuleInfo::from_id_latest("abstract:dex").unwrap();

    treasury.upload()?;
    treasury.instantiate(
        &InstantiateMsg {
            admin: None,
            type_urls: vec!["/cosmwasm.wasm.v1.MsgExecuteContract".to_string()],
            grant_configs: vec![GrantConfigStorage {
                description: "First grant for execution on the account itself".to_string(),
                authorization: AuthorizationData::ExecuteOnModule(AuthorizationOnModule {
                    limit: Some(
                        Any {
                            type_url: cosmos_sdk_proto::cosmwasm::wasm::v1::MaxCallsLimit::type_url(
                            ),
                            value: cosmos_sdk_proto::cosmwasm::wasm::v1::MaxCallsLimit {
                                remaining: 1,
                            }
                            .to_bytes()?,
                        }
                        .into(),
                    ),
                    module_id: target_module_id.clone(),
                    filter: None,
                }),
                optional: false,
            }],
            fee_config: FeeConfigStorage {
                description: "First grant for execution on the account itself".to_string(),
                allowance: Some(AllowanceData::AlowanceOnModule(AllowanceOnModule {
                    allowance: Some(
                        Any {
                            type_url: cosmos_sdk_proto::cosmos::base::v1beta1::Coin::type_url(),
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
                    module_id: target_module_id,
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
