//! `ChainMiddleware` implements retrying requests on given conditions.

use crate::chainable::{ChainMiddleware, Chainer, ChainAction};
use anyhow::anyhow;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next, Result};
use task_local_extensions::Extensions;

#[async_trait::async_trait]
impl<T, S> Middleware for ChainMiddleware<T>
where
    T: Chainer<State = S> + Send + Sync + 'static,
    S: Send + Sync + Default + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        // TODO: Ideally we should create a new instance of the `Extensions` map to pass
        // downstream. This will guard against previous retries poluting `Extensions`.
        // That is, we only return what's populated in the typemap for the last retry attempt
        // and copy those into the the `global` Extensions map.
        execute_with_chain(&self.0, req, next, extensions).await
    }
}

/// This function will try to execute the request, chaining any additional
/// requests if required.
async fn execute_with_chain<'a, T, S>(
    chain_middleware: &'a T,
    mut request: Request,
    next: Next<'a>,
    ext: &'a mut Extensions,
) -> Result<Response>
where
    T: Chainer<State = S> + Sync,
    S: Default,
{
    let mut request_state = S::default();
    let mut n_past_retries = 0;
    let max_chain_length = chain_middleware.max_chain_length();
    loop {
        if n_past_retries >= max_chain_length {
            return Err(Error::Middleware(anyhow!(
                "Maximum chain length {max_chain_length} exceeded"
            )));
        };

        // Cloning the request object before-the-fact is not ideal..
        // However, if the body of the request is not static, e.g of type `Bytes`,
        // the Clone operation should be of constant complexity and not O(N)
        // since the byte abstraction is a shared pointer over a buffer.
        let duplicate_request = request.try_clone().ok_or_else(|| {
            Error::Middleware(anyhow!(
                "Request object is not clonable. Are you passing a streaming body?".to_string()
            ))
        })?;
        let result = next.clone().run(duplicate_request, ext).await;

        let action = chain_middleware.chain(result, &mut request_state, &mut request).await?;
        match action {
            ChainAction::Retry => {
                n_past_retries += 1;
            }
            ChainAction::Response(response) => {
                return Ok(response)
            }
        };
    }
}
