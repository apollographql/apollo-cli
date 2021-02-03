use crate::RoverClientError;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

const JSON_CONTENT_TYPE: &str = "application/json";
const CLIENT_NAME: &str = "rover-client";

/// Function for building a [HeaderMap] for making http requests. Use for
/// Generic requests to any graphql endpoint.
///
/// Takes a single argument, list of header key/value pairs
pub fn build(header_map: &HashMap<String, String>) -> Result<HeaderMap, RoverClientError> {
    let mut headers = HeaderMap::new();

    // this should be consistent for any graphql requests
    let content_type = HeaderValue::from_str(JSON_CONTENT_TYPE)?;
    headers.append("Content-Type", content_type);

    for (key, value) in header_map {
        let header_key = HeaderName::from_bytes(key.as_bytes())?;
        let header_value = HeaderValue::from_str(&value)?;
        headers.append(header_key, header_value);
    }

    Ok(headers)
}

/// Function for building a [HeaderMap] for making http requests. Use for making
/// requests to Apollo Studio. We're leaving this separate from `build` since we
/// need to be able to mark the api_key as sensitive (at the bottom)
///
/// Takes an `api_key` and a `client_version`, and returns a [HeaderMap].
pub fn build_studio_headers(
    api_key: &str,
    client_version: &str,
) -> Result<HeaderMap, RoverClientError> {
    let mut headers = HeaderMap::new();

    let content_type = HeaderValue::from_str(JSON_CONTENT_TYPE)?;
    headers.insert("Content-Type", content_type);

    // The headers "apollographql-client-name" and "apollographql-client-version"
    // are used for client identification in Apollo Studio.

    // This provides metrics in Studio that help keep track of what parts of the schema
    // Rover uses, which ensures future changes to the API do not break Rover users.
    // more info here:
    // https://www.apollographql.com/docs/studio/client-awareness/#using-apollo-server-and-apollo-client

    let client_name = HeaderValue::from_str(CLIENT_NAME)?;
    headers.insert("apollographql-client-name", client_name);
    log::trace!("client version: {}", &client_version);
    let client_version = HeaderValue::from_str(&client_version)?;
    headers.insert("apollographql-client-version", client_version);

    let mut api_key = HeaderValue::from_str(api_key)?;
    api_key.set_sensitive(true);
    headers.insert("x-api-key", api_key);

    Ok(headers)
}
