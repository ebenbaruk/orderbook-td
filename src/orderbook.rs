use crate::interfaces::{OrderBook, Price, Quantity, Side, Update};

pub struct OrderBookImpl {
    bids: Vec<(Price, Quantity)>,

    asks: Vec<(Price, Quantity)>,
}

impl OrderBookImpl {
    // put a quantity at a price, or add the price if it is not in the list
    fn set_level(levels: &mut Vec<(Price, Quantity)>, price: Price, quantity: Quantity) {
        for i in 0..levels.len() {
            if levels[i].0 == price {
                levels[i].1 = quantity;
                return;
            }
        }

        levels.push((price, quantity));
    }

    // remove a price from the list if it is there
    fn remove_level(levels: &mut Vec<(Price, Quantity)>, price: Price) {
        for i in 0..levels.len() {
            if levels[i].0 == price {
                levels.remove(i);

                return;
            }
        }
    }
}

impl OrderBook for OrderBookImpl {
    fn new() -> Self {
        OrderBookImpl {
            bids: Vec::new(),

            asks: Vec::new(),
        }
    }

    fn apply_update(&mut self, update: Update) {
        match update {
            Update::Set {
                price,
                quantity,
                side,
            } => {
                let levels = if side == Side::Bid {
                    &mut self.bids
                } else {
                    &mut self.asks
                };

                if quantity == 0 {
                    Self::remove_level(levels, price);
                } else {
                    Self::set_level(levels, price, quantity);
                }
            }

            Update::Remove { price, side } => {
                if side == Side::Bid {
                    Self::remove_level(&mut self.bids, price);
                } else {
                    Self::remove_level(&mut self.asks, price);
                }
            }
        }
    }

    fn get_spread(&self) -> Option<Price> {
        let best_bid = self.get_best_bid();
        let best_ask = self.get_best_ask();

        if best_bid.is_none() || best_ask.is_none() {
            return None;
        }

        Some(best_ask.unwrap() - best_bid.unwrap())
    }

    fn get_best_bid(&self) -> Option<Price> {
        // the best bid is the highest price
        if self.bids.len() == 0 {
            return None;
        }

        let mut best = self.bids[0].0;

        for i in 0..self.bids.len() {
            if self.bids[i].0 > best {
                best = self.bids[i].0;
            }
        }
        Some(best)
    }

    fn get_best_ask(&self) -> Option<Price> {
        // the best ask is the lowest price
        if self.asks.len() == 0 {
            return None;
        }

        let mut best = self.asks[0].0;

        for i in 0..self.asks.len() {
            if self.asks[i].0 < best {
                best = self.asks[i].0;
            }
        }
        Some(best)
    }

    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity> {
        let levels = if side == Side::Bid {
            &self.bids
        } else {
            &self.asks
        };

        for i in 0..levels.len() {
            if levels[i].0 == price {
                return Some(levels[i].1);
            }
        }

        None
    }

    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)> {
        let mut sorted = if side == Side::Bid {
            self.bids.clone()
        } else {
            self.asks.clone()
        };

        // sort from lowest to highest price
        sorted.sort();

        if side == Side::Bid {
            // bids: from highest to lowest price
            sorted.reverse();
        }

        let mut levels = Vec::new();

        for i in 0..sorted.len() {
            if levels.len() == n {
                break;
            }

            levels.push(sorted[i]);
        }

        levels
    }

    fn get_total_quantity(&self, side: Side) -> Quantity {
        let levels = if side == Side::Bid {
            &self.bids
        } else {
            &self.asks
        };

        let mut total = 0;

        for i in 0..levels.len() {
            total += levels[i].1;
        }
        total
    }
}
