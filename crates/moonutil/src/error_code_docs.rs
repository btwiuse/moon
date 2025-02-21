use once_cell::sync::Lazy;
use std::collections::HashMap;

// include the generated error code docs
include!(concat!(env!("OUT_DIR"), "/error_code_docs.rs"));

pub fn get_error_code_doc(error_code: &str) -> Option<&'static str> {
    ERROR_DOCS.get(error_code).copied()
}
