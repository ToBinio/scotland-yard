#![allow(dead_code)]

use std::sync::Arc;

use axum_test::TestServer;
use server::{app, services::data::service::DataService};

pub mod connection;
pub mod data;
pub mod ws;

pub fn test_server() -> TestServer {
    let app = app(Arc::new(DataService));
    TestServer::builder().http_transport().build(app).unwrap()
}
