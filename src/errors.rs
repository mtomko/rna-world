use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};

#[derive(Display, From, Debug)]
pub enum RWError {
    CsvError,
}
impl std::error::Error for RWError {}

impl ResponseError for RWError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            RWError::CsvError => HttpResponse::InternalServerError().finish(),
        }
    }
}
