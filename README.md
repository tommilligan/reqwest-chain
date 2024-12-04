# reqwest-chain

[![Crates.io](https://img.shields.io/crates/v/reqwest-chain)](https://crates.io/crates/reqwest-chain)
[![docs.rs](https://img.shields.io/badge/docs-available-brightgreen)](https://tommilligan.github.io/reqwest-chain/)
[![GitHub](https://img.shields.io/github/license/tommilligan/reqwest-chain)](https://github.com/tommilligan/reqwest-chain/blob/master/LICENSE)

Apply custom criteria to any `reqwest` response, deciding when and how to retry.

`reqwest-chain` builds on `reqwest-middleware`, to allow you to focus on your core logic without the boilerplate.

## Use case

This crate is a more general framework than [`reqwest-retry`](https://github.com/TrueLayer/reqwest-middleware). It allows inspection of:

- The result of the previous request
- The state of the retry chain
- The global state of your middleware

Based on this information, it allows updating any aspect of the next request.

If all you need is a simple retry, you should use `reqwest-retry`.

## Getting started

See the `tests` directory for several examples.

You should implement `Chainer` for your middleware struct. This uses the `chain`
method to make a decision after each request respones:

- If another request is required, update the previous request to form the
  next request in the chain, and return `Ok(None)`.
- If the response is ready, return it inside `Ok(Some(response))`.
- If an error occurs and you cannot continue, return `Err(error)`.

Below is the initial use case; refresh some authorization credential on request failure.

```rust
use reqwest_chain::{Chainer, ChainMiddleware};
use reqwest_middleware::reqwest::{Client, Request, Response, StatusCode};
use reqwest_middleware::reqwest::header::{AUTHORIZATION, HeaderValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};

// Mimic some external function that returns a valid token.
fn fetch_token() -> String {
    "valid-token".to_string()
}

struct FetchTokenMiddleware;

#[async_trait::async_trait]
impl Chainer for FetchTokenMiddleware {
    // We don't need it here, but you can choose to keep track of state between
    // chained retries.
    type State = ();

    async fn chain(
        &self,
        result: Result<Response, Error>,
        _state: &mut Self::State,
        request: &mut Request,
    ) -> Result<Option<Response>, Error> {
        let response = result?;
        if response.status() != StatusCode::UNAUTHORIZED {
            return Ok(Some(response))
        };
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", fetch_token())).expect("invalid header value"),
        );
        Ok(None)
    }
}

async fn run() {
    let client = ClientBuilder::new(Client::new())
        .with(ChainMiddleware::new(FetchTokenMiddleware))
        .build();

    client
        .get("https://example.org")
        // If this token is invalid, the request will be automatically retried
        // with an updated token.
        .header(AUTHORIZATION, "Bearer expired-token")
        .send()
        .await
        .unwrap();
}
```

## Thanks

Many thanks to the prior work in this area, namely:

- [`reqwest-middleware`](https://github.com/TrueLayer/reqwest-middleware) for the underlying framework
- [`reqwest-retry`](https://github.com/TrueLayer/reqwest-middleware) for inspiration
