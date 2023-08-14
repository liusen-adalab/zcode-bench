use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RegisterClientReq {
    SwitchProtocol(ClientType),
}

#[derive(Serialize, Deserialize)]
pub enum ClientType {
    Upload,
}

#[derive(Serialize, Deserialize)]
pub enum RegisterResult {
    Ok,
    Err(Option<String>),
}

crate::impl_codec!(RegisterClientReq, RegisterResult);
