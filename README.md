# reqwest-chain

[![Crates.io](https://img.shields.io/crates/v/reqwest-chain)](https://crates.io/crates/reqwest-chain)
[![docs.rs](https://img.shields.io/badge/docs-available-brightgreen)](https://tommilligan.github.io/reqwest-chain/)
[![GitHub](https://img.shields.io/github/license/tommilligan/reqwest-chain)](https://github.com/tommilligan/reqwest-chain/blob/master/LICENSE)

Apply custom criteria to any `reqwest` response, deciding when and how to retry.

`reqwest-chain` builds on `reqwest-middleware`, to allow you to focus on your core logic without the boilerplate.

## Use

```rust
use reqwest::{header::{AUTHORIZATION, HeaderValue}, StatusCode};
use reqwest_chain::{Chainer, ChainMiddleware};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};

// Mimic some external function that returns a valid token.
fn fetch_token() -> String {
    "valid-token".to_string()
}

struct FetchTokenChainer;

#[async_trait::async_trait]
impl Chainer for FetchTokenChainer {
    type State = ();

    // Retry if the server tells us we are unauthorized (the token has expired)
    fn should_chain(&self, result: &Result<reqwest::Response, Error>) -> bool {
        result
            .as_ref()
            .map(|response| response.status() == StatusCode::UNAUTHORIZED)
            .unwrap_or_default()
    }

    async fn chain(
        &self,
        _result: Result<reqwest::Response, Error>,
        _state: &mut Self::State,
        request: &mut reqwest::Request,
    ) -> Result<(), Error> {
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", fetch_token())).expect("invalid header value"),
        );
        Ok(())
    }
}

async fn run_retries() {
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(ChainMiddleware(FetchTokenChainer))
        .build();

    client
        .get("https://example.org")
        .header(AUTHORIZATION, "Bearer expired-token")
        .send()
        .await
        .unwrap();
}
```
