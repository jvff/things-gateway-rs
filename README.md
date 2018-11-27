# A Rust implementation for the Mozilla Things Gateway

Done as a part of the Rustfest 2018 Rome impl days. This repository is a fork of the [initial
prototype][initial]. It extends it with some [experimental Rust code][ipc] for the addon IPC
handling. The idea was to extend the webserver so that it could serve through a REST interface the
devices registered through IPC.

This is very early and experimental code part of a proof-of-concept, so be warned :)

[The original implemenation](https://github.com/mozilla-iot/gateway/) was done in Node, this PoC
should show that it's possible to use Rust instead.

# License

MIT

[initial]: https://github.com/celaus/things-gateway-rs
[ipc]: https://github.com/jvff/rust-things-gateway-adapter-ipc-test
