use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorBody {
                code,
                message,
                details: None,
            },
        }
    }

    pub fn bad_request_with_details(
        code: &'static str,
        message: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorBody {
                code,
                message,
                details: Some(details.into()),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let json = Json(serde_json::json!({ "error": self.body }));
        (self.status, json).into_response()
    }
}
