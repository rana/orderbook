# orderbook
An example order book which merges two exchanges.

![Running the orderbook server](imgs/orderbook.gif)

## Challenge

Code a mini project that:
1. connects to two exchanges' websocket feeds at the same time,
2. pulls order books, using these streaming connections, for a given traded pair of currencies (configurable), from each exchange,
3. merges and sorts the order books to create a combined order book,
4. from the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server.


* 10 lowest asks, 10 highest bids
* Configure with `ETH/BTC`
* Input websocket stream from `Binance`
* Input websocket stream from `Bitstamp`
* Merge two orderbooks into one order book.
* Update output on every change from any input order book.
* Output as streaming gRPC server
* Use format similar to:
    ```json
    {
    “spread”: 2.72,
    “asks”: [
    { exchange: "binance", price: 8491.25, amount: 0.008 },
    { exchange: "coinbase", price: 8496.37, amount: 0.0303 },
    ...
    ],
    “bids”: [
    { exchange: "binance", price: 8488.53, amount: 0.002 },
    { exchange: "kraken", price: 8484.71, amount: 1.0959 },
    ...
    ]
    }
    ```
    > Not the actual format.

## Usage

Run the server.
```sh
cd /home/rana/prj/orderbook/srv
clear && cargo r -q -- -d
```

Run the client.
```sh
cd /home/rana/prj/orderbook/cli
clear && cargo r -q
```

## Development notes

[Binance Spot API Docs: Web sockets](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md)

[Bitstamp: API Docs: Web sockets](https://www.bitstamp.net/websocket/v2/)

* Evaluated Rust WebSocket crates:
  * [tungstenite](https://crates.io/crates/tungstenite)
    * "Lightweight stream-based WebSocket implementation"
  * [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite)
    * "Tokio binding for Tungstenite, the Lightweight stream-based WebSocket implementation"
    * [Documentation](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/)
    * Says `fastbwebsockets` is 30% faster than `tungstenite`.
  * [fastwebsockets](https://crates.io/crates/fastwebsockets)
    * "a minimal, fast WebSocket server implementation."
    * Last updated 5 days ago.
    * [GitHub](https://github.com/denoland/fastwebsockets)
    * [Benchmarks](https://github.com/denoland/fastwebsockets/tree/main/benches)
    * [Docs](https://docs.rs/fastwebsockets/latest/fastwebsockets/)
  * [websocket](https://crates.io/crates/websocket)
    * Depreciated
* Exchange API crates
  * [binance](https://crates.io/crates/binance)
    * Well designed.
    * Uses tungstenite.
  * [bitstamp_client_ws](https://crates.io/crates/bitstamp_client_ws)
    * [Docs](https://docs.rs/bitstamp_client_ws/0.2.0/bitstamp_client_ws/)
    * [GitHub](https://github.com/gmosx/bitstamp_sdk_rust/tree/main/bitstamp_client_ws)
    * Uses tungstenite.
    * Created 3 days ago
* Binaance API
  * [Partial Book Depth Streams](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#partial-book-depth-streams)
    * Update Speed: 1000ms or 100ms
  * [Individual Symbol Book Ticker Streams](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#individual-symbol-book-ticker-streams)
    * Update Speed: Real-time
  * [How to manage a local order book correctly](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#how-to-manage-a-local-order-book-correctly)
    * Helpful "how to" advice.

### Generate protobufs

[grpc-rs usage](https://github.com/tikv/grpc-rs#usage)

```sh
> cd /home/rana/prj/orderbook/src/protos
protoc --rust_out=. --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_rust_plugin` orderbook.proto
```

### Example order book messages

Binanace order book message. 10 at a time.
```
[Bids { price: 0.06273, qty: 1.6059 }, 
Bids { price: 0.06272, qty: 13.5593 }, 
Bids { price: 0.06266, qty: 0.008 }, 
Bids { price: 0.06255, qty: 0.6934 }, 
Bids { price: 0.06254, qty: 0.2701 }, 
Bids { price: 0.06253, qty: 0.1732 }, 
Bids { price: 0.06252, qty: 0.5891 }, 
Bids { price: 0.06251, qty: 0.0506 }, 
Bids { price: 0.0625, qty: 0.6553 }, 
Bids { price: 0.06249, qty: 0.0271 }], 
Ask: [Asks { price: 0.06281, qty: 3.9175 }, 
Asks { price: 0.06282, qty: 11.65 }, 
Asks { price: 0.06291, qty: 0.0077 }, 
Asks { price: 0.06297, qty: 0.0774 }, 
Asks { price: 0.06304, qty: 0.6803 }, 
Asks { price: 0.06305, qty: 0.0084 }, 
Asks { price: 0.06309, qty: 0.2978 }, 
Asks { price: 0.0631, qty: 0.0298 }, 
Asks { price: 0.06315, qty: 0.1656 }, 
Asks { price: 0.06316, qty: 0.4258 }]
```

Bitstamp order book message. 100 at a time.
```
bids: [("0.06265043", "0.48398810"), 
("0.06265042", "1.00161186"), 
("0.06265041", "0.82108740"), 
...

asks: [("0.06268070", "1.50000000"), 
("0.06268590", "0.82083095"), 
("0.06269300", "0.00324301"), 
...
```