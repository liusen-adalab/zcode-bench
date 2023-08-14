use std::panic::Location;

use actix_web::{
    body::BoxBody, dev::Server, http::StatusCode, web, App, HttpResponse, HttpServer, ResponseError,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{file_system, settings::get_settings};

type ApiResult<T, E = ApiError> = std::result::Result<T, E>;
pub type ApiResponse<T> = ApiResult<web::Json<Response<T>>>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<T: Serialize> {
    pub status: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub err_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[macro_export]
macro_rules! api_ok {
    ($data:expr) => {
        Ok(::actix_web::web::Json(crate::restful_api::Response {
            status: 0,
            err_msg: None,
            data: Some($data),
        }))
    };
}

#[repr(u8)]
#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "[debug only] server internal err at {location}: {source}")]
    Internal {
        source: anyhow::Error,
        location: &'static Location<'static>,
    } = 1,

    #[display(fmt = "bad request: code = {_0}, msg = {_1}")]
    BizErr(u32, String),
}

impl From<anyhow::Error> for ApiError {
    #[track_caller]
    fn from(value: anyhow::Error) -> Self {
        let location = core::panic::Location::caller();
        Self::Internal {
            source: value,
            location: location,
        }
    }
}

pub trait ToAnyhow<T, E> {
    fn to_anyhow(self) -> anyhow::Result<T>;
}

impl<T, E> ToAnyhow<T, E> for ApiResult<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn to_anyhow(self) -> anyhow::Result<T> {
        Ok(self?)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ApiError::Internal { .. } => {
                error!(err = %self, "server internal err");
            }
            _ => {
                warn!(event = %self, "received bad request")
            }
        }
        let resp = Response::<()> {
            status: 1,
            err_msg: Some(self.to_string()),
            data: None,
        };
        let body = serde_json::to_string(&resp).unwrap();
        HttpResponse::build(self.status_code()).body(body)
    }
}

pub fn register_apis(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/api/fs/")
            .service(web::resource("load_dir").route(web::get().to(file_system::load_in_dir)))
            .service(
                web::resource("load_structure").route(web::get().to(file_system::load_fs_dir_tree)),
            ),
    );
}

#[derive(Deserialize)]
pub struct ServerConfig {
    bind: String,
}

pub async fn build_server() -> anyhow::Result<Server> {
    let config = &get_settings().http_server;

    let server = HttpServer::new(move || App::new().configure(register_apis))
        .bind(&config.bind)?
        .workers(2)
        .run();
    Ok(server)
}
