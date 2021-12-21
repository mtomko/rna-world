use actix_web::{HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use derive_more::{Display, From};
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

#[derive(Display, From, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum RWError {
    CsvError,
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
}

impl std::error::Error for RWError {}

impl ResponseError for RWError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            RWError::CsvError => HttpResponse::InternalServerError().finish(),
            RWError::PoolError(ref err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            RWError::PGError(ref err) => HttpResponse::InternalServerError().body(err.to_string()),
            RWError::PGMError(ref err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
}
