//! Middleware to retry failed HTTP requests built on [`reqwest_middleware`].
//!
//! Use [`ChainMiddleware`] to retry HTTP requests under specific conditions,
//! where custom logic is needed before the next retry attempt.
//!
//! ## Example
//!
//! ```
//! use reqwest::{header::{AUTHORIZATION, HeaderValue}, StatusCode};
//! use reqwest_chain::{Chainer, ChainMiddleware};
//! use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
//!
//! // Mimic some external function that returns a valid token.
//! fn fetch_token() -> String {
//!     "valid-token".to_string()
//! }
//!
//! struct FetchTokenMiddleware;
//!
//! #[async_trait::async_trait]
//! impl Chainer for FetchTokenMiddleware {
//!     // We don't need it here, but you can choose to keep track of state between
//!     // chained retries.
//!     type State = ();
//!
//!     async fn chain(
//!         &self,
//!         result: Result<reqwest::Response, Error>,
//!         _state: &mut Self::State,
//!         request: &mut reqwest::Request,
//!     ) -> Result<Option<reqwest::Response>, Error> {
//!         let response = result?;
//!         if response.status() != StatusCode::UNAUTHORIZED {
//!             return Ok(Some(response))
//!         };
//!         request.headers_mut().insert(
//!             AUTHORIZATION,
//!             HeaderValue::from_str(&format!("Bearer {}", fetch_token())).expect("invalid header value"),
//!         );
//!         Ok(None)
//!     }
//! }
//!
//! async fn run() {
//!     let client = ClientBuilder::new(reqwest::Client::new())
//!         .with(ChainMiddleware::new(FetchTokenMiddleware))
//!         .build();
//!
//!     client
//!         .get("https://example.org")
//!         // If this token is invalid, the request will be automatically retried
//!         // with an updated token.
//!         .header(AUTHORIZATION, "Bearer expired-token")
//!         .send()
//!         .await
//!         .unwrap();
//! }
//! ```
//!

mod chainable;
mod middleware;

pub use chainable::{ChainMiddleware, Chainer};
