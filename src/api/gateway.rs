// API Gateway implementation
// This module handles API requests to cryptocurrency exchanges

pub struct APIGateway {
    base_url: String,
    api_key: Option<String>,
}

impl APIGateway {
    pub fn new(base_url: String) -> Self {
        APIGateway {
            base_url,
            api_key: None,
        }
    }
    
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
}
