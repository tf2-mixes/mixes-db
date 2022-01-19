pub mod query_error;
pub mod search_params;
use std::thread;
use std::time::Duration;

use json::JsonValue;
pub use query_error::*;

pub mod log;
pub use log::*;
use reqwest::blocking as reqwest;

use self::search_params::SearchParams;

const LOGS_TF_API_BASE: &str = "https://logs.tf/api/v1/log";

/// Sleep for a little time before making a request to logs.tf. The API is very
/// sensitive to quickly making queries to it and will respond with invalid
/// responses otherwise.
pub(self) fn log_delay() { thread::sleep(Duration::from_millis(500)) }

/// Checks for the `"success": true` field in the json value, which is always
/// set by logs.tf. If `"success": false` is set, it will parse the error and
/// return a `QueryError`.
fn check_json_success(json: &JsonValue) -> QueryResult<()>
{
    let success = json["success"].as_bool().unwrap();

    if success {
        Ok(())
    }
    else {
        let error = json["error"].as_str().unwrap();
        Err(QueryError::Unsuccessful(error.to_owned()))
    }
}

/// Query logs.tf for logs with the given parameters
///
/// # Returns
/// The metadata of all logs that fit the search parameters
pub fn search_logs(search_params: SearchParams) -> QueryResult<Vec<LogMetadata>>
{
    let request = reqwest::Client::builder().build()?.get(LOGS_TF_API_BASE);
    let request = search_params.add_params_to_request(request);

    log_delay();
    let response = request.send()?;
    let json = json::parse(&(response.text()?)).unwrap();
    check_json_success(&json)?;

    Ok(json["logs"]
        .members()
        .map(|meta| LogMetadata::from_json(&meta))
        .collect())
}
