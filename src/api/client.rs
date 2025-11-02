// API Client implementation
// This module provides HTTP client functionality for API calls

use reqwest::Client;

pub struct APIClient {
    client: Client,
}

impl APIClient {
    pub fn new() -> Self {
        APIClient {
            client: Client::new(),
        }
    }
}
