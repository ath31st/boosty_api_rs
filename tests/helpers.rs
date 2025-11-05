use mockito::{Server, ServerGuard};

pub fn api_path(path: &str) -> String {
    format!("/v1/{path}")
}

pub async fn setup() -> (ServerGuard, String) {
    let server = Server::new_async().await;
    let base = server.url();
    (server, base)
}
