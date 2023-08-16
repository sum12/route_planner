use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    // internal errors
    // database connection error
    // redis not rechable
    // ...
    NoValidMapDefined,
    LayoutUpdateFailureDetected,

    // errors reported to the client
    // TODO: make this a seperate enum
    // TODO: additional infomration could be added
    NodesMustBeUnique,
    EdgesMustBeUnique,
    InvalidNodeInEdge,
    EdgeMissingSourceOrSink,
    NodeNeedsMoreDriveways,
    // DisconnectedNodesFound
    NodesNotFound, // when start and goal for /query are not invalid
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("->> {:<12} - {self:?}", "ERROR_RESP");
        println!();

        let error_json = json!({ "error": { "details": self.as_ref() } });
        (self.status_code(), Json(error_json)).into_response()
    }
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NodesMustBeUnique
            | Self::EdgesMustBeUnique
            | Self::InvalidNodeInEdge
            | Self::EdgeMissingSourceOrSink
            | Self::NodeNeedsMoreDriveways => StatusCode::BAD_REQUEST,
            Self::NodesNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Self::NodesMustBeUnique => "nodes must be unique",
            Self::EdgesMustBeUnique => "Edges must be unique",
            Self::InvalidNodeInEdge => "invalid node in edge",
            Self::EdgeMissingSourceOrSink => "edge is missing a source or sink",
            Self::NodeNeedsMoreDriveways => "node does not have enough number of driveways",
            Self::NodesNotFound => "the provided nodes were not found",
            _ => "internal server error",
        }
    }
}
