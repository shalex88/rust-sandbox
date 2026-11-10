use std::sync::Arc;

use super::{Api, rest::RestApi};
use crate::core::Core;

pub enum ApiType {
    Rest,
}

pub fn create(api_type: ApiType, core: Core) -> RestApi {
    match api_type {
        ApiType::Rest => RestApi::new(Arc::new(Api::new(core))),
    }
}
