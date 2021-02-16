# RetryIter
RetryIter is a small helper crate which adds retry support to **std::iter::Iterator.**
It does it by implementing the crate's **IntoRetryIter**
for all **std::iter::Iterator** types. In addition to retries support, the main
feature of this crate is to preserve the iterator items during 
**std::future::Future** cancellation in asynchronous processing.
It is explained with example in
[Crate's Main Feature](https://docs.rs/retryiter/0.4.0/retryiter/#crates-main-feature)
 section.
## Documentation

Documentation of this create available at [doc.rs](https://docs.rs/retryiter/0.4.0/retryiter/index.html)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
retryiter = "0.4"
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/VigneshChennai/retryiter/blob/main/LICENSE