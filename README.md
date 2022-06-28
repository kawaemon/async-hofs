# async-hofs
Various extention traits for providing asynchronous higher-order functions.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![CI Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/async-hofs.svg?style=for-the-badge
[crates-url]: https://crates.io/crates/async-hofs

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge
[mit-url]: https://github.com/kawaemon/async-hofs/blob/master/LICENSE

[actions-badge]: https://img.shields.io/github/workflow/status/kawaemon/async-hofs/ci?style=for-the-badge
[actions-url]: https://github.com/kawaemon/async-hofs/actions?query=workflow%3ACI+branch%3Amaster

```rs
// This won't make any name conflicts since all imports inside prelude are anonymous.
use async_hofs::prelude::*;

assert_eq!(
    Some(1).async_map(|x| async move { x + 2 }).await,
    Some(3),
);

type Result = core::result::Result<i32, i32>;

assert_eq!(
    Result::Ok(1).async_and_then(|_| async move { Err(77) }).await,
    Result::Err(77)
);
```
