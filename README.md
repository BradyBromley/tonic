![](https://github.com/hyperium/tonic/raw/master/.github/assets/tonic-banner.svg?sanitize=true)

A rust implementation of [gRPC], a high performance, open source, general
RPC framework that puts mobile and HTTP/2 first.

[`tonic`] is a gRPC over HTTP/2 implementation focused on high performance, interoperability, and flexibility. This library was created to have first class support of async/await and to act as a core building block for production systems written in Rust.

[![Crates.io](https://img.shields.io/crates/v/tonic)](https://crates.io/crates/tonic)
[![Documentation](https://docs.rs/tonic/badge.svg)](https://docs.rs/tonic)
[![Crates.io](https://img.shields.io/crates/l/tonic)](LICENSE)


[Examples] | [Website] | [Docs] | [Chat]

## Overview

[`tonic`] is composed of three main components: the generic gRPC implementation, the high performance HTTP/2
implementation and the codegen powered by [`prost`]. The generic implementation can support any HTTP/2
implementation and any encoding via a set of generic traits. The HTTP/2 implementation is based on [`hyper`],
a fast HTTP/1.1 and HTTP/2 client and server built on top of the robust [`tokio`] stack. The codegen
contains the tools to build clients and servers from [`protobuf`] definitions.

## Features

- Bi-directional streaming
- High performance async io
- Interoperability
- TLS backed via either [`openssl`] or [`rustls`]
- Load balancing
- Custom metadata
- Authentication

## Getting Started

Examples can be found in [`tonic-examples`] and for more complex scenarios [`tonic-interop`]
may be a good resource as it shows examples of many of the gRPC features.

### Rust Version

`tonic` currently works on rust `1.39-beta` and above as it requires support for the `async_await`
feature. To install the beta simply follow the commands below:

```bash
$ rustup install beta
$ rustup component add rustfmt --toolchain beta
$ cargo +beta build
```

### Examples

<details>
  <summary>Helloworld</summary>

#### `Cargo.toml`

```toml
tonic = "*"
bytes = "0.4"
prost = "0.5"
prost-derive = "0.5"
```

#### Protobuf

```protobuf
package helloworld;

// The greeting service definition.
service Greeter {
  // Sends a greeting
  rpc SayHello (HelloRequest) returns (HelloReply) {}
}

// The request message containing the user's name.
message HelloRequest {
  string name = 1;
}

// The response message containing the greetings
message HelloReply {
  string message = 1;
}
```

#### `build.rs`

```rust
fn main() {
    tonic_build::compile_protos("proto/helloworld/helloworld.proto").unwrap();
}
```

#### Client

```rust
pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::{client::GreeterClient, HelloRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051")?;

    let request = tonic::Request::new(HelloRequest {
        name: "hello".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
```

#### Server

```rust
use tonic::{transport::Server, Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::{
    server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

#[derive(Default)]
pub struct MyGreeter {
    data: String,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let string = &self.data;

        println!("My data: {:?}", string);

        let reply = hello_world::HelloReply {
            message: "Zomg, it works!".into(),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    Server::builder()
        .serve(addr, GreeterServer::new(greeter))
        .await?;

    Ok(())
}
```

</details>

## Getting Help

First, see if the answer to your question can be found in the API documentation.
If the answer is not there, there is an active community in
the [Tonic Discord channel][chat]. We would be happy to try to answer your
question. If that doesn't work, try opening an [issue] with the question.

[chat]: https://discord.gg/6yGkFeN
[issue]: https://github.com/hyperium/tonic/issues/new

## Project Layout

- [`tonic`](https://github.com/hyperium/tonic/tree/master/tonic): Generic gRPC and HTTP/2 client/server
implementation.
- [`tonic-build`](https://github.com/hyperium/tonic/tree/master/tonic-build): [`prost`] based service codegen.
- [`tonic-examples`](https://github.com/hyperium/tonic/tree/master/tonic-examples): Example gRPC implementations showing off
tls, load balancing and bi-directional streaming.
- [`tonic-interop`](https://github.com/hyperium/tonic/tree/master/tonic-interop): Interop tests implementation.

## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][guide] to help you get involved in the Tonic
project.

[guide]: CONTRIBUTING.md

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tonic by you, shall be licensed as MIT, without any additional
terms or conditions.


[gRPC]: https://grpc.io
[`tonic`]: https://github.com/hyperium/tonic
[`tokio`]: https://github.com/tokio-rs/tokio
[`hyper`]: https://github.com/hyperium/hyper
[`prost`]: https://github.com/danburkert/prost
[`protobuf`]: https://developers.google.com/protocol-buffers
[`rustls`]: https://github.com/ctz/rustls
[`openssl`]: https://www.openssl.org/
[`tonic-examples`]: https://github.com/hyperium/tonic/tree/master/tonic-examples
[`tonic-interop`]: https://github.com/hyperium/tonic/tree/master/tonic-interop
[Examples]: https://github.com/hyperium/tonic/tree/master/tonic-examples
[Website]: https://github.com/hyperium/tonic
[Docs]: https://docs.rs/tonic
[Chat]: https://discord.gg/6yGkFeN
