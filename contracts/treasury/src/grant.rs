pub mod allowance;

use abstract_std::{
    account::state::ACCOUNT_ID,
    objects::{
        module::ModuleInfo, module_factory::ModuleFactoryContract,
        module_reference::ModuleReference, registry::RegistryContract,
        salt::generate_instantiate_salt,
    },
};
use cosmos_sdk_proto::{prost::Name, traits::MessageExt};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Deps};

use crate::{error::ContractResult, state::ABSTRACT_CODE_ID};

#[cw_serde]
pub struct GrantConfig {
    description: String,
    pub authorization: Any,
    pub optional: bool,
}

#[cw_serde]

pub enum AuthorizationData {
    Any(Any),
    ExecuteOnAccount(AuthorizationOnAccount),
    ExecuteOnModule(AuthorizationOnModule),
}

#[cw_serde]

pub struct AuthorizationOnAccount {
    pub limit: Option<Any>,
    pub filter: Option<Any>,
}

#[cw_serde]

pub struct AuthorizationOnModule {
    pub module_id: ModuleInfo,
    pub limit: Option<Any>,
    pub filter: Option<Any>,
}

#[cw_serde]

pub struct GrantConfigStorage {
    pub description: String,
    pub authorization: AuthorizationData,
    pub optional: bool,
}

impl AuthorizationData {
    pub fn try_into_any(self, deps: Deps, address: String) -> ContractResult<Any> {
        Ok(match self {
            AuthorizationData::Any(any) => any,
            AuthorizationData::ExecuteOnAccount(auth) => {
                // Handle the case where authorization is on the account itself
                Any {
                        type_url:  cosmos_sdk_proto::cosmwasm::wasm::v1::ContractExecutionAuthorization::type_url(),
                        value:
                            cosmos_sdk_proto::cosmwasm::wasm::v1::ContractExecutionAuthorization {
                                grants: vec![cosmos_sdk_proto::cosmwasm::wasm::v1::ContractGrant {
                                    contract: address,
                                    limit: auth.limit.map(Into::into),
                                    filter:auth.filter.map(Into::into),
                                }],
                            }
                            .to_bytes()?
                            .into(),
                    }
            }
            AuthorizationData::ExecuteOnModule(auth) => {
                // Handle the case where authorization is on an abstract module
                let module_address = query_module_address(deps, auth.module_id, address)?;

                Any {
                        type_url: cosmos_sdk_proto::cosmwasm::wasm::v1::ContractExecutionAuthorization::type_url(),
                        value:
                            cosmos_sdk_proto::cosmwasm::wasm::v1::ContractExecutionAuthorization {
                                grants: vec![cosmos_sdk_proto::cosmwasm::wasm::v1::ContractGrant {
                                    contract: module_address,
                                    limit: auth.limit.map(Into::into),
                                    filter:auth.filter.map(Into::into),
                                }],
                            }
                            .to_bytes()?
                            .into(),
                    }
            }
        })
    }
}

impl GrantConfigStorage {
    pub fn try_into_grant_config(self, deps: Deps, address: String) -> ContractResult<GrantConfig> {
        Ok(GrantConfig {
            description: self.description,
            authorization: self.authorization.try_into_any(deps, address)?,
            optional: self.optional,
        })
    }
}

#[cw_serde]
pub struct FeeConfigStorage {
    pub description: String,
    pub allowance: Option<AllowanceData>,
    pub expiration: Option<u32>,
}

#[cw_serde]
pub struct FeeConfig {
    description: String,
    pub allowance: Option<Any>,
    pub expiration: Option<u32>,
}

#[cw_serde]
pub enum AllowanceData {
    Any(Any),
    AllowanceOnAccount(AllowanceOnAccount),
    AlowanceOnModule(AllowanceOnModule),
}
#[cw_serde]
pub struct AllowanceOnAccount {
    pub allowance: Option<Any>,
}
#[cw_serde]
pub struct AllowanceOnModule {
    pub module_id: ModuleInfo,
    pub allowance: Option<Any>,
}

impl FeeConfigStorage {
    pub fn try_into_fee_config(self, deps: Deps, address: String) -> ContractResult<FeeConfig> {
        Ok(FeeConfig {
            description: self.description,
            allowance: match self.allowance {
                None => None,
                Some(allowance) => Some(allowance.try_into_any(deps, address)?),
            },
            expiration: self.expiration,
        })
    }
}

impl AllowanceData {
    pub fn try_into_any(self, deps: Deps, address: String) -> ContractResult<Any> {
        Ok(match self {
            AllowanceData::Any(any) => any,
            AllowanceData::AllowanceOnAccount(allowance) => {
                // Handle the case where authorization is on the account itself
                // (TODO, for now we just pass the specified allowance, we have a bug with ContractsAllowance)
                allowance.allowance.unwrap()
            }
            AllowanceData::AlowanceOnModule(allowance) => {
                // Handle the case where authorization is on the account itself
                // (TODO, for now we just pass the specified allowance, we have a bug with ContractsAllowance)
                allowance.allowance.unwrap()
            }
        })
    }
}

#[cw_serde]
pub struct Any {
    pub type_url: String,
    pub value: Binary,
}

impl From<cosmos_sdk_proto::Any> for Any {
    fn from(value: cosmos_sdk_proto::Any) -> Self {
        Any {
            type_url: value.type_url,
            value: Binary::from(value.value),
        }
    }
}

impl From<Any> for cosmos_sdk_proto::Any {
    fn from(value: Any) -> Self {
        cosmos_sdk_proto::Any {
            type_url: value.type_url,
            value: value.value.to_vec(),
        }
    }
}

fn query_module_address(
    deps: Deps,
    module_id: ModuleInfo,
    account_address: String,
) -> ContractResult<String> {
    // We need to resolve the module address
    let abstract_code_id = ABSTRACT_CODE_ID.load(deps.storage)?;
    let registry = RegistryContract::new(deps, abstract_code_id)?;
    let module_type = &registry.query_modules_configs(vec![module_id], &deps.querier)?[0];

    let module_address = match module_type.module.reference {
        ModuleReference::Adapter(ref module_address)
        | ModuleReference::Native(ref module_address)
        | ModuleReference::Service(ref module_address) => module_address.to_string(),

        abstract_std::objects::module_reference::ModuleReference::App(code_id)
        | abstract_std::objects::module_reference::ModuleReference::Standalone(code_id) => {
            let module_factory = ModuleFactoryContract::new(deps, abstract_code_id)?;
            let account_id =
                ACCOUNT_ID.query(&deps.querier, deps.api.addr_validate(&account_address)?)?;
            let canonical_module_factory = deps
                .api
                .addr_canonicalize(module_factory.address.as_str())?;
            let salt: Binary = generate_instantiate_salt(&account_id);

            let checksum = deps.querier.query_wasm_code_info(code_id)?.checksum;
            let module_address = cosmwasm_std::instantiate2_address(
                checksum.as_slice(),
                &canonical_module_factory,
                &salt,
            )?;
            deps.api.addr_humanize(&module_address)?.to_string()
        }
        _ => panic!("Unsupported module type"),
    };

    Ok(module_address)
}
