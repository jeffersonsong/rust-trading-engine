mod match_engine;
use match_engine::engine::{MatchingEngine, TradingPair};
use match_engine::orderbook::{BidOrAsk, Order, OrderBook};

fn main() {
    let buy_order_from_alice = Order::new(BidOrAsk::Bid, 5.5);
    let buy_order_from_bob = Order::new(BidOrAsk::Bid, 2.45);

    let mut order_book = OrderBook::new();
    order_book.add_order(4.4, buy_order_from_alice);
    order_book.add_order(4.4, buy_order_from_bob);

    let sell_order = Order::new(BidOrAsk::Ask, 6.5);
    order_book.add_order(20.0, sell_order);

    //println!("{:?}", order_book);

    let mut engine = MatchingEngine::new();
    let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    engine.add_new_market(pair.clone());

    let buy_order = Order::new(BidOrAsk::Bid, 6.5);
    let eth_pair = TradingPair::new("ETH".to_string(), "USD".to_string());
    engine.place_limit_order(pair, 10.0, buy_order).unwrap();
}
