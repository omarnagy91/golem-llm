use reqwest::Client;

struct OllamaApi {
    model: String,
    base_url: String,
    client: Client,
}

impl OllamaApi {
    pub fn new(model: String, base_url: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to initialize HTTP client");
        Self {
            model,
            base_url,
            client,
        }
    }
}
