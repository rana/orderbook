use crate::*;
use bitstamp_client_ws::client::Client;
use bytes::Buf;
use futures::StreamExt;
use http::Uri;
use serde_json::Value;
use std::cmp;
use tokio::sync::mpsc::Sender;
use tokio_websockets::ClientBuilder;

/// The Binance exchange.
#[derive(Debug, Clone)]
pub struct Binance {
    /// An instrument to stream.
    pub instr: String,
}

impl Binance {
    /// Returns a new instance of a Binance exchange.
    pub fn new(instr: String) -> Self {
        Binance { instr }
    }

    /// Streams a Binance order book for a single instrument.
    pub async fn stream_orderbook(&self, tx_agg: Sender<Orderbook>) {
        // Get the application-wide debug setting.
        let dbg = DBG.get().unwrap();
        dbg.then(|| println!("Binance:stream_orderbook: Start"));

        let uri: Uri = format!(
            "wss://stream.binance.us:9443/ws/{}@depth10@100ms",
            self.instr
        )
        .parse()
        .unwrap();
        let mut client = ClientBuilder::from_uri(uri).connect().await.unwrap();
        while let Some(res) = client.next().await {
            // println!("Binance:stream_orderbook: res: {:?}", &res);
            match res {
                Err(e) => println!("Binance:stream_orderbook: error: {:?}", e),
                Ok(msg) => {
                    if msg.is_text() {
                        dbg.then(|| {
                            println!("--- Binance OrderBook Msg ---");
                            // println!("{:?}", &msg);
                        });

                        // Transform Binance order book for inter-thread communication.
                        let val: Value = serde_json::from_reader(msg.into_data().reader()).unwrap();
                        // println!("Binance:stream_orderbook: val: {:?}", &val);

                        // Parse bids.
                        let jbids = val.get("bids").unwrap().as_array().unwrap();
                        let bid_cnt = cmp::min(jbids.len(), BID_ASK_LEN);
                        let mut bids = Vec::with_capacity(bid_cnt);
                        for jbid in jbids.iter().take(bid_cnt) {
                            let jbid_arr = jbid.as_array().unwrap();
                            bids.push(Bid {
                                xch_id: BINANCE_XCH_ID,
                                prc: jbid_arr[0].as_str().unwrap().parse().unwrap(),
                                qty: jbid_arr[1].as_str().unwrap().parse().unwrap(),
                            })
                        }
                        // Parse asks.
                        let jasks = val.get("asks").unwrap().as_array().unwrap();
                        let ask_cnt = cmp::min(jasks.len(), BID_ASK_LEN);
                        let mut asks = Vec::with_capacity(ask_cnt);
                        for jask in jasks.iter().take(ask_cnt) {
                            let jask_arr = jask.as_array().unwrap();
                            asks.push(Ask {
                                xch_id: BINANCE_XCH_ID,
                                prc: jask_arr[0].as_str().unwrap().parse().unwrap(),
                                qty: jask_arr[1].as_str().unwrap().parse().unwrap(),
                            })
                        }

                        // Send Binance order book to aggregator thread.
                        tx_agg
                            .send(Orderbook {
                                spd: 0.0,
                                bids,
                                asks,
                            })
                            .await
                            .map_err(|e| println!("Binance:stream_orderbook: error: {:?}", e))
                            .unwrap();
                    }
                }
            }
        }

        dbg.then(|| println!("Binance:stream_orderbook: End"));
    }
}

/// The Bitstamp exchange.
#[derive(Debug, Clone)]
pub struct Bitstamp {
    /// An instrument to stream.
    pub instr: String,
}

impl Bitstamp {
    /// Returns a new instance of a Bitstamp exchange.
    pub fn new(instr: String) -> Self {
        Bitstamp { instr }
    }

    /// Streams a Bitstamp order book for a single instrument.
    pub async fn stream_orderbook(&self, tx_agg: Sender<Orderbook>) {
        // Get the application-wide debug setting.
        let dbg = DBG.get().unwrap();

        dbg.then(|| println!("Bitstamp:stream_orderbook: Start"));

        dbg.then(|| println!("Bitstamp:stream_orderbook: Connecting..."));
        let mut client = Client::connect_public().await.expect("cannot connect");

        client.subscribe_live_orderbook(&self.instr).await;

        let mut book_events = client.book_events.unwrap();

        while let Some(msg) = book_events.next().await {
            dbg.then(|| {
                println!("--- Bitstamp OrderBook Msg ---");
                // println!("{:?}", &msg);
            });

            // Transform Bitstamp order book for inter-thread communication.
            let bid_cnt = cmp::min(msg.data.bids.len(), BID_ASK_LEN);
            let mut bids = Vec::with_capacity(bid_cnt);
            for bid in msg.data.bids.iter().take(bid_cnt) {
                bids.push(Bid {
                    xch_id: BITSTAMP_XCH_ID,
                    prc: bid.0.parse().unwrap(),
                    qty: bid.1.parse().unwrap(),
                })
            }
            let ask_cnt = cmp::min(msg.data.asks.len(), BID_ASK_LEN);
            let mut asks = Vec::with_capacity(ask_cnt);
            for ask in msg.data.asks.iter().take(ask_cnt) {
                asks.push(Ask {
                    xch_id: BITSTAMP_XCH_ID,
                    prc: ask.0.parse().unwrap(),
                    qty: ask.1.parse().unwrap(),
                })
            }

            // Send Bitstamp order book to aggregator thread.
            tx_agg
                .send(Orderbook {
                    spd: 0.0,
                    bids,
                    asks,
                })
                .await
                .map_err(|e| println!("Bitstamp:stream_orderbook: error: {:?}", e))
                .unwrap();
        }
        dbg.then(|| println!("Bitstamp:stream_orderbook: End"));
    }
}

/// An order book.
#[derive(Debug, Clone)]
pub struct Orderbook {
    // Spread for the order book.
    pub spd: f64,
    /// Bids of an order book.
    pub bids: Vec<Bid>,
    /// Asks of an order book.
    pub asks: Vec<Ask>,
}

impl Orderbook {
    /// Returns a new order book.
    pub fn new() -> Self {
        Orderbook {
            spd: 0.0,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
    /// Returns a new order book with vectors initialized with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Orderbook {
            spd: 0.0,
            bids: Vec::with_capacity(capacity),
            asks: Vec::with_capacity(capacity),
        }
    }
    /// Clear the asks and bids.
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }
}

/// An order book `bid`.
#[derive(Debug, Clone)]
pub struct Bid {
    /// An exchange id.
    pub xch_id: u8,
    /// A bid price.
    pub prc: f64,
    /// A bid quantity.
    pub qty: f64,
}

/// An order book `ask`.
#[derive(Debug, Clone)]
pub struct Ask {
    /// An exchange id.
    pub xch_id: u8,
    /// An ask price.
    pub prc: f64,
    /// An ask quantity.
    pub qty: f64,
}

/// The Binanace exchange id.
pub const BINANCE_XCH_ID: u8 = 1;

/// The Binanace exchange id.
pub const BITSTAMP_XCH_ID: u8 = 2;

/// The Binance display name.
pub const BINANCE_NAME: &str = "binance";

/// The Bitstamp display name.
pub const BITSTAMP_NAME: &str = "bitstamp";

#[inline]
pub fn xch_name(id: u8) -> String {
    if id == BINANCE_XCH_ID {
        return BINANCE_NAME.to_owned();
    }
    BITSTAMP_NAME.to_owned()
}
