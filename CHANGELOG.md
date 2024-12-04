# Unreleased

# v1.0.0

This release updates outdated dependencies.

As these dependencies are visible in the public api of `reqwest-chain`, this is a breaking change.

## Changed

- Upgraded to `reqwest-middleware 0.4` ([#6](https://github.com/tommilligan/reqwest-chain/pull/6))
  - Removed `reqwest` as a direct dependency, it is now imported from `reqwest-middleware`'s re-export

# v0.2.0

This release updates outdated dependencies.

As these dependencies are visible in the public api of `reqwest-chain`, this is a breaking change.

## Changed

- Upgraded to `reqwest 0.12` and `reqwest-middleware 0.3` ([#2](https://github.com/tommilligan/reqwest-chain/pull/2)). Thanks to [@kirinse](https://github.com/kirinse) for the contribution!
  - Peer dependency `task_local_extensions` changed to `http`

# v0.1.0

Initial implementation.
