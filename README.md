# RetryIter
RetryIter is a small helper crate which adds retry support to **std::iter::Iterator.**
It does it by implementing the crate's **IntoRetryIter**
for all **std::iter::Iterator** types. In addition to retries support, the main
feature of this crate is to preserve the iterator items during 
**std::future::Future** cancellation in asynchronous processing.
It is explained with example in
[Crate's Main Feature](https://docs.rs/retryiter/#crates-main-feature)
 section.
## Documentation

Documentation of this crate available at [doc.rs](https://docs.rs/retryiter/)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
retryiter = "0.4"
```

## Example

```rust
use retryiter::{IntoRetryIter};

#[derive(Debug, Clone, PartialEq)]
struct ValueError;

let a = vec![1, 2, 3];

// Initializing retryiter with retry count 1.
// Also defined the error that can occur in while processing the item.
let mut iter = a.into_iter().retries::<ValueError>(1);

for item in &mut iter {
    if item == 3 {
        // Always failing for value 3.
        item.failed(ValueError);
    } else if item < 3 && item.attempt() == 1 {
        // Only fail on first attempt. The item with value 1 or 2 will
        // succeed on second attempt.
        item.failed(ValueError);
    } else {
        // Marking success for all the other case.
        item.succeeded();
    }
}
assert_eq!(vec![(3, ValueError)], iter.failed_items())
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/VigneshChennai/retryiter/blob/main/LICENSE