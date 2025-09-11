//! Telegram Bot API Types Module
//!
//! This module contains all the auto-generated types from the Telegram Bot API
//! specification.

// Include the generated types
include!(concat!(env!("OUT_DIR"), "/tgapi_types.rs"));

// All types are automatically available when this module is used
// No need for re-exports since they're all public

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response<T = bool> {
    pub ok:          bool,
    pub result:      Option<T>,
    pub error_code:  Option<u32>,
    pub description: Option<String>,
}
