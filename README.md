## Chat example on top of hyper

It's a proof of concept, it means only bare minimum of features
are implemented. Built with `hyper`, `tokio` and `futures`.
It also contains a small subset of [RFC6455](https://tools.ietf.org/html/rfc6455).
It was a very nice time building this prototype.

### Instructions

To serve statics
```
cargo install https
http
```

To start server
```
cargo run
```

Then navigate to `http://localhost:8000`
