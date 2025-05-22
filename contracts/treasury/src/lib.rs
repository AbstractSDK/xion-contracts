extern crate core;

pub mod contract;
mod error;
mod execute;
pub mod msg;
pub mod state;

pub mod grant;
mod query;

pub const CONTRACT_NAME: &str = "treasury";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_arch = "wasm32"))]
pub use crate::interface::Treasury;
#[cfg(not(target_arch = "wasm32"))]
pub mod interface;

#[cfg(not(target_arch = "wasm32"))]
pub const XION_TESTNET_2: cw_orch::prelude::ChainInfo = cw_orch::prelude::ChainInfo {
    kind: cw_orch::environment::ChainKind::Testnet,
    chain_id: "xion-testnet-2",
    gas_denom: "uxion",
    gas_price: 0.001,
    grpc_urls: &["http://xion-testnet-grpc.polkachu.com:22390"],
    network_info: cw_orch::daemon::networks::xion::XION_NETWORK,
    lcd_url: None,
    fcd_url: None,
};
