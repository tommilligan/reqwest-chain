use reqwest_middleware::Error;

/// Describes:
///
/// - which request outcomes should be retried
/// - how the request should be updated to retry
///
#[async_trait::async_trait]
pub trait Chainer {
    type State: Default;

    /// Inspect the result of the previous request, to decide whether to make
    /// another request.
    ///
    /// If another request is required, return `true`.
    fn should_chain(&self, result: &Result<reqwest::Response, Error>) -> bool;

    /// Update the previous request to form the next request in the chain.
    ///
    /// Information is available from:
    ///
    /// - self (global state, instantiated at middleware creation)
    /// - state (local state, instantiated for each request chain)
    /// - result (the result of the previous request)
    ///
    /// Global side effects can be managed via interior mutability of `self`.
    async fn chain(
        &self,
        result: Result<reqwest::Response, Error>,
        state: &mut Self::State,
        request: &mut reqwest::Request,
    ) -> Result<(), Error>;

    /// Safety valve to protect against infinite chaining.
    ///
    /// This value may be overriden by the user.
    fn max_chain_length(&self) -> u32 {
        /// We limit the number of retries to avoid stack-overflow issues due to the recursion.
        ///
        /// This can be increased by the user.
        const DEFAULT_MAXIMUM_CHAIN_LENGTH: u32 = 10;
        DEFAULT_MAXIMUM_CHAIN_LENGTH
    }
}

pub struct ChainMiddleware<T>(pub T);
