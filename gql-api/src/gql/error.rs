use crate::error::GrpcError;
use displaydoc::Display as DisplayDoc;
use juniper::{graphql_value, FieldError, GraphQLObject, ScalarValue};
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Debug, GraphQLObject)]
pub struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Field: {}, Message: {})", self.field, self.message)
    }
}

#[derive(Debug, DisplayDoc, Error)]
pub enum GqlError {
    /// Unknown event error: `{0}`
    UnknownEventStatus(String),
    /// Parse UUID error
    ParseUUID,
    /// Unexpected Internal error
    UnexpectedInternal,
    /// Validation error: `{0}`
    Validation(ValidationError),
    /// Database error: `{0}`
    Database(tokio_postgres::Error),
    /// Grpc error: `{0}`
    Grpc(GrpcError),
}

impl<S: ScalarValue> juniper::IntoFieldError<S> for GqlError {
    fn into_field_error(self) -> FieldError<S> {
        match self {
            GqlError::UnknownEventStatus(status) => FieldError::new(
                format!("Unknown event status ({status}) error"),
                graphql_value!({
                    "type": "PARSE"
                }),
            ),
            GqlError::ParseUUID => FieldError::new(
                "Parse UUID error",
                graphql_value!({
                    "type": "PARSE"
                }),
            ),
            GqlError::Validation(error) => FieldError::new(
                error.to_string(),
                graphql_value!({
                    "type": "VALIDATION"
                }),
            ),
            GqlError::Database(error) => {
                let msg = error.to_string();
                FieldError::new(
                    "Database error",
                    graphql_value!({
                        "type": "DATABASE",
                        "error": msg
                    }),
                )
            }
            GqlError::UnexpectedInternal => FieldError::new(
                "Unexpected Error",
                graphql_value!({
                    "type": "INTERNAL"
                }),
            ),
            GqlError::Grpc(error) => {
                let msg = error.to_string();
                FieldError::new(
                    "Grpc Error",
                    graphql_value!({
                        "type": "INTERNAL",
                        "error": msg
                    }),
                )
            }
        }
    }
}
