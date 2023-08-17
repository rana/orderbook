use crate::exchanges::*;
use crate::*;
use tokio::sync::mpsc::{Receiver, Sender};

/// An order book aggregator.
#[derive(Debug, Clone)]
pub struct Aggregator {
    /// The merged order book.
    pub mrg: Orderbook,
    /// The last received Binance order book.
    pub bin: Orderbook,
    /// The last received Bitstamp order book.
    pub bit: Orderbook,
}

impl Aggregator {
    /// Returns a new instance of an order book aggregator.
    pub fn new() -> Self {
        Aggregator {
            mrg: Orderbook::with_capacity(BID_ASK_LEN * 2),
            bin: Orderbook::new(),
            bit: Orderbook::new(),
        }
    }

    /// Aggregates order book streams.
    pub async fn aggregate_streams(
        &mut self,
        mut rx_agg: Receiver<Orderbook>,
        tx_grpc: Sender<Orderbook>,
    ) {
        // Get the application-wide debug setting.
        let dbg = DBG.get().unwrap();

        dbg.then(|| println!("Aggregator:aggregate_streams: Start"));
        loop {
            if let Some(orderbook) = rx_agg.recv().await {
                dbg.then(|| {
                    println!("Aggregator:aggregate_streams: rx");
                    // println!("{:?}", &orderbook);
                });

                // Store current order book.
                if orderbook.asks[0].xch_id == BINANCE_XCH_ID {
                    self.bin = orderbook;
                } else {
                    self.bit = orderbook;
                }

                // Merge the order books from each exchange.
                self.mrg.clear();
                self.mrg.bids.append(&mut self.bin.bids.clone());
                self.mrg.asks.append(&mut self.bin.asks.clone());
                self.mrg.bids.append(&mut self.bit.bids.clone());
                self.mrg.asks.append(&mut self.bit.asks.clone());

                // Sort the aggregated order book.
                // Sort bids so that highest price is first.
                // Sort asks so that lowest price is first.
                self.mrg
                    .bids
                    .sort_by(|a, b| b.prc.partial_cmp(&a.prc).unwrap());
                self.mrg
                    .asks
                    .sort_by(|a, b| a.prc.partial_cmp(&b.prc).unwrap());

                // Truncate the aggrgeation to top 10.
                self.mrg.bids.truncate(BID_ASK_LEN);
                self.mrg.asks.truncate(BID_ASK_LEN);

                // Calculate the spread.
                self.mrg.spd = self.mrg.asks[0].prc - self.mrg.bids[0].prc;

                // dbg.then(|| {
                //     println!("Aggregator:aggregate_streams: mrg:{:?}", self.mrg);
                // });

                // Send merged order book to gRPC streaming server.
                if let Err(e) = tx_grpc.send(self.mrg.clone()).await {
                    println!("Aggregator:aggregate_streams: error: {:?}", e);
                }
            }
        }
    }
}
