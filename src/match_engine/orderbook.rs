#![allow(dead_code)]

use rust_decimal::prelude::*;
use std::collections::{HashMap, LinkedList};

#[derive(Debug, Clone)]
pub enum BidOrAsk {
    Bid,
    Ask,
}

#[derive(Debug)]
pub struct OrderBook {
    asks: HashMap<Decimal, Limit>,
    bids: HashMap<Decimal, Limit>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: HashMap::new(),
            bids: HashMap::new(),
        }
    }

    pub fn fill_market_order(&mut self, market_order: &mut Order) -> Vec<Execution> {
        self.fill_order(market_order, None)
    }

    pub fn fill_limit_order(&mut self, limit_order: &mut Order, price: Decimal) -> Vec<Execution> {
        self.fill_order(limit_order, Some(price))
    }

    pub fn fill_order(
        &mut self,
        order: &mut Order,
        limit_price: Option<Decimal>,
    ) -> Vec<Execution> {
        let mut executions = Vec::new();
        // Vec = >matches
        let limits = match order.bid_or_ask {
            BidOrAsk::Bid => self.ask_limits(limit_price),
            BidOrAsk::Ask => self.bid_limits(limit_price),
        };

        for limit_order in limits {
            let execs = limit_order.fill_order(order);
            executions.extend(execs);

            if order.is_filled() {
                break;
            }
        }

        executions
    }

    // BID (BUY Order) => ASKS => sorted cheapest price
    fn ask_limits(&mut self, limit_price: Option<Decimal>) -> Vec<&mut Limit> {
        let mut limits: Vec<&mut Limit> = match limit_price {
            Some(limit_price) => self
                .asks
                .values_mut()
                .filter(|limit| limit.price <= limit_price)
                .collect::<Vec<&mut Limit>>(),
            None => self.asks.values_mut().collect::<Vec<&mut Limit>>(),
        };
        limits.sort_by(|a, b| a.price.cmp(&b.price));
        limits
    }

    // ASK (SELL Order) => BIDS => sorted highest price
    fn bid_limits(&mut self, limit_price: Option<Decimal>) -> Vec<&mut Limit> {
        let mut limits = match limit_price {
            Some(limit_price) => self
                .bids
                .values_mut()
                .filter(|limit| limit.price >= limit_price)
                .collect::<Vec<&mut Limit>>(),
            None => self.bids.values_mut().collect::<Vec<&mut Limit>>(),
        };
        limits.sort_by(|a, b| b.price.cmp(&a.price));
        limits
    }

    pub fn add_limit_order(&mut self, price: Decimal, order: Order) {
        match order.bid_or_ask {
            BidOrAsk::Bid => match self.bids.get_mut(&price) {
                Some(limit) => limit.add_order(order),
                None => {
                    let mut limit = Limit::new(price);
                    limit.add_order(order);
                    self.bids.insert(price, limit);
                }
            },
            BidOrAsk::Ask => match self.asks.get_mut(&price) {
                Some(limit) => limit.add_order(order),
                None => {
                    let mut limit = Limit::new(price);
                    limit.add_order(order);
                    self.asks.insert(price, limit);
                }
            },
        }
    }
}

#[derive(Debug)]
struct Limit {
    price: Decimal,
    orders: LinkedList<Order>,
}

impl Limit {
    fn new(price: Decimal) -> Limit {
        Limit {
            price,
            orders: LinkedList::new(),
        }
    }

    fn total_volume(&self) -> f64 {
        self.orders.iter().map(|order| order.size).sum()
    }

    fn fill_order(&mut self, market_order: &mut Order) -> Vec<Execution> {
        let mut executions = Vec::new();
        while !market_order.is_filled() && !self.orders.is_empty() {
            let mut limit_order = self.orders.front_mut().unwrap();

            let shares = if market_order.size >= limit_order.size {
                limit_order.size
            } else {
                market_order.size
            };
            executions.push(Execution::new(market_order, shares, self.price));
            executions.push(Execution::new(limit_order, shares, self.price));

            market_order.size -= shares;
            limit_order.size -= shares;

            if limit_order.is_filled() {
                self.orders.pop_front();
            }
        }

        executions
    }

    fn add_order(&mut self, order: Order) {
        self.orders.push_back(order);
    }
}

#[derive(Debug)]
pub struct Order {
    id: u64,
    size: f64,
    bid_or_ask: BidOrAsk,
}

impl Order {
    pub fn new(id: u64, bid_or_ask: BidOrAsk, size: f64) -> Order {
        Order {
            id,
            bid_or_ask,
            size,
        }
    }

    pub fn is_filled(&self) -> bool {
        self.size == 0.0
    }
}

#[derive(Debug)]
pub struct Execution {
    id: u64,
    size: f64,
    bid_or_ask: BidOrAsk,
    price: Decimal,
}

impl Execution {
    pub fn new(order: &Order, size: f64, price: Decimal) -> Execution {
        Execution {
            id: order.id,
            size,
            bid_or_ask: order.bid_or_ask.clone(),
            price,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn orderbook_fill_market_order_ask() {
        let mut orderbook = OrderBook::new();
        orderbook.add_limit_order(dec!(500), Order::new(1, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(100), Order::new(2, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(200), Order::new(3, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(300), Order::new(4, BidOrAsk::Ask, 10.0));

        let mut market_order = Order::new(5, BidOrAsk::Bid, 10.0);
        let executions = orderbook.fill_market_order(&mut market_order);
        println!("{:?}", executions);

        let ask_limits = orderbook.ask_limits(None);
        let matched_limit = ask_limits.get(0).unwrap();
        assert_eq!(matched_limit.price, dec!(100));
        assert!(market_order.is_filled());

        assert!(matched_limit.orders.is_empty());

        println!("{:?}", orderbook.ask_limits(None));
    }

    #[test]
    fn orderbook_fill_limit_order_ask() {
        let mut orderbook = OrderBook::new();
        orderbook.add_limit_order(dec!(500), Order::new(1, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(100), Order::new(2, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(200), Order::new(3, BidOrAsk::Ask, 10.0));
        orderbook.add_limit_order(dec!(300), Order::new(4, BidOrAsk::Ask, 10.0));

        let mut limit_order = Order::new(5, BidOrAsk::Bid, 30.0);
        let executions = orderbook.fill_limit_order(&mut limit_order, dec!(210));
        println!("{:?}", executions);

        assert_eq!(limit_order.size, 10.0);
        println!("{:?}", orderbook.ask_limits(None));
    }

    #[test]
    fn limit_total_volume() {
        let price = dec!(10_000.0);
        let mut limit = Limit::new(price);

        let buy_limit_order_a = Order::new(1, BidOrAsk::Bid, 100.0);
        let buy_limit_order_b = Order::new(2, BidOrAsk::Bid, 100.0);
        limit.add_order(buy_limit_order_a);
        limit.add_order(buy_limit_order_b);

        assert_eq!(limit.total_volume(), 200.0);

        println!("{:?}", limit);
    }

    #[test]
    fn limit_order_multi_fill() {
        let price = dec!(10_000.0);
        let mut limit = Limit::new(price);

        let buy_limit_order_a = Order::new(1, BidOrAsk::Bid, 100.0);
        let buy_limit_order_b = Order::new(2, BidOrAsk::Bid, 100.0);
        limit.add_order(buy_limit_order_a);
        limit.add_order(buy_limit_order_b);

        let mut market_sell_order = Order::new(3, BidOrAsk::Ask, 199.0);
        limit.fill_order(&mut market_sell_order);

        assert!(market_sell_order.is_filled());
        assert_eq!(limit.orders.front().unwrap().size, 1.0);

        println!("{:?}", limit);
    }

    #[test]
    fn limit_order_single_fill() {
        let price = dec!(10_000.0);
        let mut limit = Limit::new(price);

        let buy_limit_order = Order::new(1, BidOrAsk::Bid, 100.0);
        limit.add_order(buy_limit_order);

        let mut market_sell_order = Order::new(2, BidOrAsk::Ask, 99.0);
        limit.fill_order(&mut market_sell_order);

        println!("{:?}", limit);

        assert!(market_sell_order.is_filled());
        assert_eq!(limit.orders.front().unwrap().size, 1.0);
    }
}
