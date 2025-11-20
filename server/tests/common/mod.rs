use axum_test::TestServer;
use server::app;

pub mod connection;
pub mod ws;

pub fn test_server() -> TestServer {
    let app = app();
    TestServer::builder().http_transport().build(app).unwrap()
}
