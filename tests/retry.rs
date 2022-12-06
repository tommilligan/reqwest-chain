use reqwest::Client;
use reqwest_middleware::{ClientBuilder, Error};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use reqwest_chain::{ChainAction, ChainMiddleware, Chainer};

struct RetryOnServerError {
    pub retries: u32,
}

#[async_trait::async_trait]
impl Chainer for RetryOnServerError {
    type State = u32;

    async fn chain(
        &self,
        result: Result<reqwest::Response, Error>,
        state: &mut Self::State,
        _request: &mut reqwest::Request,
    ) -> Result<ChainAction, Error> {
        *state += 1;
        let response = result?;
        if response.status().is_server_error() && *state < self.retries {
            Ok(ChainAction::Retry)
        } else {
            Ok(ChainAction::Response(response))
        }
    }
}

#[tokio::test]
async fn retry_works() {
    let server = MockServer::start().await;

    // For the first token, return unauthorized
    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(500))
        .expect(5)
        .mount(&server)
        .await;

    let reqwest_client = Client::builder().build().unwrap();
    let client = ClientBuilder::new(reqwest_client)
        .with(ChainMiddleware(RetryOnServerError { retries: 5 }))
        .build();

    let response = client
        .get(&format!("{}/ping", server.uri()))
        .send()
        .await
        .expect("call failed");

    assert_eq!(response.status(), 500);
}
