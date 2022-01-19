pub mod assets_service;
pub mod blockchain_updates;
pub mod data_service;
pub mod levex;
pub mod matcher;
pub mod node;
pub mod rates_service;
pub mod state_service;

pub use assets_service::AssetsSvcApi;
pub use blockchain_updates::BlockchainUpdApi;
pub use data_service::DataSvcApi;
pub use levex::LevexApi;
pub use matcher::MatcherApi;
pub use node::NodeApi;
pub use rates_service::RatesSvcApi;
pub use state_service::StateSvcApi;

pub trait BaseApi: Sync + Clone {}
