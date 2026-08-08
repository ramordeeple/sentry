use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

use crate::{domain::scenario::ValidationError, errors::storage_error::StorageError};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("XML serialization error: {0}")]
    Serialization(#[from] quick_xml::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Validation(error) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()),
            Self::Storage(StorageError::NotFound) => (
                StatusCode::NOT_FOUND,
                "scenario is not registered".to_owned(),
            ),
            Self::Storage(StorageError::Database(_)) | Self::Serialization(_) => {
                eprintln!("{self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_owned(),
                )
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
