use reqwest_chain::{ChainMiddleware, Chainer};
use reqwest_middleware::reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest_middleware::reqwest::{Client, Request, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, Error};
use std::sync::Arc;
use tokio::sync::Mutex;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Default)]
struct RegenerateTokenChainer {
    auth_token: Arc<Mutex<u64>>,
}

#[async_trait::async_trait]
impl Chainer for RegenerateTokenChainer {
    type State = ();

    async fn chain(
        &self,
        result: Result<Response, Error>,
        _state: &mut Self::State,
        request: &mut Request,
    ) -> Result<Option<Response>, Error> {
        let response = result?;
        if response.status() != StatusCode::UNAUTHORIZED {
            return Ok(Some(response));
        };

        let mut auth_token = self.auth_token.lock().await;
        *auth_token += 1;

        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("{auth_token}")).expect("invalid header value"),
        );
        Ok(None)
    }
}

#[tokio::test]
async fn regenerate_token_works() {
    let server = MockServer::start().await;

    // For the first token, return unauthorized
    Mock::given(method("GET"))
        .and(path("/ping"))
        .and(header(AUTHORIZATION, "0"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    // For the second token, succeed
    Mock::given(method("GET"))
        .and(path("/ping"))
        .and(header(AUTHORIZATION, "1"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let reqwest_client = Client::builder().build().unwrap();
    let client = ClientBuilder::new(reqwest_client)
        .with(ChainMiddleware::new(RegenerateTokenChainer::default()))
        .build();

    let response = client
        .get(format!("{}/ping", server.uri()))
        .header(AUTHORIZATION, "0")
        .timeout(std::time::Duration::from_millis(100))
        .send()
        .await
        .expect("call failed");

    assert_eq!(response.status(), 200);
}
