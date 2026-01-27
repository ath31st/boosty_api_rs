use reqwest::{Response, StatusCode};

use crate::{
    api_client::ApiClient,
    error::{ApiError, ResultApi},
};

impl ApiClient {
    /// Handle the response from a request, checking the status code and returning the response if successful.
    ///
    /// # Arguments
    /// * `path` - The path of the request.
    /// * `response` - The response from the request.
    ///
    /// # Returns
    /// * `ResultApi<Response>` - The response from the request if successful, otherwise an error.
    pub(crate) async fn handle_response(
        &self,
        path: &str,
        response: Response,
    ) -> ResultApi<Response> {
        let status = response.status();
        self.check_status(status, path)?;

        Ok(response)
    }

    fn check_status(&self, status: StatusCode, endpoint: &str) -> ResultApi<()> {
        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }

        if !status.is_success() {
            return Err(ApiError::HttpStatus {
                status,
                endpoint: endpoint.to_string(),
            });
        }

        Ok(())
    }

    /// Parse the JSON response from a request.
    ///
    /// # Arguments
    /// * `response` - The response from the request.
    ///
    /// # Returns
    /// * `ResultApi<T>` - The parsed JSON response if successful, otherwise an error.
    pub(crate) async fn parse_json<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> ResultApi<T> {
        let body = response.text().await?;

        let mut deserializer = serde_json::Deserializer::from_str(&body);
        serde_path_to_error::deserialize::<_, T>(&mut deserializer).map_err(|err| {
            ApiError::JsonParseDetailed {
                error: format!("path: {}, error: {}", err.path(), err.inner()),
            }
        })
    }
}
