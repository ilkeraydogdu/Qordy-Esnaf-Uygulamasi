//! QORDY POS - Server Module
//! Re-exports from the modular `api` package for clean architecture and backwards compatibility.

#[allow(unused_imports)]
pub use crate::api::{create_router, start_server, AppState};
