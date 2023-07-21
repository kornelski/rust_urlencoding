# urlencoding

[![Latest Version](https://img.shields.io/crates/v/urlencoding.svg)](https://lib.rs/crates/urlencoding)

A tiny Rust library for performing encoding and decoding of URL paths and arguments. It percent-encodes everything except alphanumerics and `-`, `_`, `.`, `~`.

When decoding `+` is not treated as a space. Error recovery from incomplete percent-escapes follows the [WHATWG URL standard](https://url.spec.whatwg.org/).

## Usage

To encode a string, use the following:

```rust
use urlencoding::encode;

let encoded = encode("This string will be encoded to be URI-safe.");
println!("{}", encoded);
// This%20string%20will%20be%20encoded%20to%20be%20URI-safe.
```

Decoding to a string supports UTF-8:

```rust
use urlencoding::decode;

let decoded = decode("%F0%9F%91%BE%20Exterminate%21")?;
println!("{}", decoded);
// 👾 Exterminate!
```

To decode allowing arbitrary bytes and invalid UTF-8:

```rust
use urlencoding::decode_binary;

let binary = decode_binary(b"%F1%F2%F3%C0%C1%C2");
let decoded = String::from_utf8_lossy(&binary);
```

This library returns [`Cow`](https://doc.rust-lang.org/stable/std/borrow/enum.Cow.html) to avoid allocating when decoding/encoding is not needed. Call `.into_owned()` on the `Cow` to get a `Vec` or `String`.

[Encoding](https://docs.rs/urlencoding/latest/urlencoding/struct.Encoded.html) can also avoid unnecesary allocations:

```rust
use urlencoding::Encoded;

format!("https://lib.rs?q={}", Encoded("argument encoded without a temporay string"));
// https://lib.rs?q=argument%20encoded%20without%20a%20temporay%20string
```

## License

This project is licensed under the MIT license. For more information see the `LICENSE` file.
