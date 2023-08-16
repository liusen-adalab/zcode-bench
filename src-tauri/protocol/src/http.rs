use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub status: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub err_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> Response<T> {
    pub fn to_result(self) -> anyhow::Result<Option<T>> {
        if self.status == 0 {
            return Ok(self.data);
        } else {
            bail!("{:?}", self.err_msg)
        }
    }
}
