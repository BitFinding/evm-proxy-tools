mod consts;
mod read;
mod detect;
mod types;
pub mod utils;
mod proxy_inspector;

pub use types::{ProxyType, ProxyDispatch};
pub use read::{get_proxy_implementation, ProxyImplementation, ProxyReadError};
pub use detect::get_proxy_type;
