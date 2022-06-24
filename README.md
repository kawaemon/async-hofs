# async-hofs
Various extention traits for providing asynchronous higher-order functions

```rs
// This won't make any name conflicts since all imports inside prelude are anonymous.
use async_hofs::prelude::*;

assert_eq!(
    Some(1).async_map(|x| async move { x + 2 }).await,
    Some(3),
);
```
