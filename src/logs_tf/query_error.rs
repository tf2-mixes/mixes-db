use std::error::Error;
use std::fmt;

use json::JsonError;
use reqwest::Error as HttpError;

/// Any error that may occur when querying data from logs.tf
#[derive(Debug)]
pub enum QueryError
{
    /// An error that can occur when the connection to logs.tf is unstable or
    /// the service is down.
    HttpResponse(HttpError),
    /// If for whatever reason an invalid Json file is returned by logs.tf or it
    /// is corrupted.
    JsonParseError(JsonError),
    /// The Json object returned always contains `"success": true` or
    /// `"success": false` to let the other party know if the query succeeded.
    /// If it is false, this error is returned.
    Unsuccessful(String),
}

pub type QueryResult<T> = Result<T, QueryError>;

impl From<HttpError> for QueryError
{
    fn from(e: HttpError) -> Self { Self::HttpResponse(e) }
}
impl From<JsonError> for QueryError
{
    fn from(e: JsonError) -> Self { Self::JsonParseError(e) }
}

impl fmt::Display for QueryError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match &self {
            &Self::HttpResponse(http_e) => {
                write!(f, "An error occured contacting logs.tf: {}", http_e)
            },
            &Self::JsonParseError(json_e) => {
                write!(f, "logs.tf did not return valid json: {}", json_e)
            },
            &Self::Unsuccessful(e) => {
                write!(
                    f,
                    "logs.tf could not successfully complete the query: {}",
                    e
                )
            },
        }
    }
}

impl Error for QueryError {}
