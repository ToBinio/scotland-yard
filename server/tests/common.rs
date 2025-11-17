use axum_test::TestServer;
use server::app;

pub fn test_server() -> TestServer {
    let app = app();
    TestServer::builder().http_transport().build(app).unwrap()
}
