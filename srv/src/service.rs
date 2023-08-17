use crate::exchanges::Orderbook as InnerOrderbook;
use crate::exchanges::{xch_name, Ask, Bid};
use crate::DBG;
use grpc::keyrock_server::Keyrock;
use grpc::{Empty, Level, Orderbook};
use tokio::sync::broadcast::{self};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};

#[derive(Debug)]
pub struct KeyrockService {
    tx: tokio::sync::broadcast::Sender<Orderbook>,
    // rx_agg: tokio::sync::mpsc::Receiver<InnerOrderbook>,
}

impl KeyrockService {
    /// Returns a new gRPC KeyrockService.
    pub fn new(mut rx_agg: tokio::sync::mpsc::Receiver<InnerOrderbook>) -> Self {
        let dbg = DBG.get().unwrap();

        let (tx, _) = broadcast::channel(64);
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            dbg.then(|| println!("KeyrockService:listen: Start"));
            // keyrock_service.listen_and_forward(rx_grpc).await;
            loop {
                // Listen for the aggregator to send an order book.
                if let Some(a) = rx_agg.recv().await {
                    dbg.then(|| println!("KeyrockService:listen: rx"));

                    // Transform the internal order book to gRPC order book.
                    let b = Orderbook {
                        spread: a.spd,
                        bids: a.bids.into_iter().map(bid_lvl).collect(),
                        asks: a.asks.into_iter().map(ask_lvl).collect(),
                    };

                    // dbg.then(|| println!("KeyrockService: sending: {:?}", b));

                    // Forward the order book to the gRPC stream.
                    // Clone the order book.
                    // Send will fail when there are no subscribers.
                    let _ = tx_clone.send(b.clone());
                }
            }
        });
        KeyrockService { tx }
    }
}

#[async_trait]
impl Keyrock for KeyrockService {
    type SummaryStream = ReceiverStream<Result<Orderbook, Status>>;
    async fn summary(&self, _: Request<Empty>) -> Result<Response<Self::SummaryStream>, Status> {
        let (tx_grpc, rx_grpc) = tokio::sync::mpsc::channel(4);

        let mut rx_agg = self.tx.subscribe();
        tokio::spawn(async move {
            loop {
                // Receive the order book from the internal aggregator.
                let orderbook = rx_agg.recv().await.unwrap();
                // Send the order book to the gRPC stream.
                tx_grpc.send(Ok(orderbook)).await.unwrap();
            }
        });

        // Return a stream result.
        Ok(Response::new(ReceiverStream::new(rx_grpc)))
    }
}

/// Converts a bid to a level.
pub fn bid_lvl(bid: Bid) -> Level {
    Level {
        exchange: xch_name(bid.xch_id),
        price: bid.prc,
        amount: bid.qty,
    }
}

/// Converts an ask to a level.
pub fn ask_lvl(ask: Ask) -> Level {
    Level {
        exchange: xch_name(ask.xch_id),
        price: ask.prc,
        amount: ask.qty,
    }
}
