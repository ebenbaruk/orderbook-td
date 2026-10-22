use crate::interfaces::{OrderBook, Price, Quantity, Side, Update};

pub struct OrderBookImpl {
    // both lists are sorted from lowest to highest price
    bids: Vec<(Price, Quantity)>,

    asks: Vec<(Price, Quantity)>,
}

impl OrderBookImpl {
    // binary search: give the index where the price is,
    // or where it should be added to keep the list sorted
    fn find_index(levels: &Vec<(Price, Quantity)>, price: Price) -> usize {
        let mut low = 0;
        let mut high = levels.len();

        while low < high {
            let middle = (low + high) / 2;

            if levels[middle].0 < price {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    // put a quantity at a price, or add the price if it is not in the list
    fn set_level(levels: &mut Vec<(Price, Quantity)>, price: Price, quantity: Quantity) {
        let i = Self::find_index(levels, price);

        if i < levels.len() && levels[i].0 == price {
            levels[i].1 = quantity;
            return;
        }

        levels.insert(i, (price, quantity));
    }

    // remove a price from the list if it is there
    fn remove_level(levels: &mut Vec<(Price, Quantity)>, price: Price) {
        let i = Self::find_index(levels, price);

        if i < levels.len() && levels[i].0 == price {
            levels.remove(i);
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
        // the best bid is the highest price, so the last one
        if self.bids.len() == 0 {
            return None;
        }

        Some(self.bids[self.bids.len() - 1].0)
    }

    fn get_best_ask(&self) -> Option<Price> {
        // the best ask is the lowest price, so the first one
        if self.asks.len() == 0 {
            return None;
        }

        Some(self.asks[0].0)
    }

    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity> {
        let levels = if side == Side::Bid {
            &self.bids
        } else {
            &self.asks
        };

        let i = Self::find_index(levels, price);

        if i < levels.len() && levels[i].0 == price {
            return Some(levels[i].1);
        }

        None
    }

    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)> {
        let mut levels = Vec::new();

        if side == Side::Bid {
            // bids: start from the end of the list
            for i in 0..self.bids.len() {
                if levels.len() == n {
                    break;
                }

                levels.push(self.bids[self.bids.len() - 1 - i]);
            }
        } else {
            // asks: start from the beginning of the list
            for i in 0..self.asks.len() {
                if levels.len() == n {
                    break;
                }

                levels.push(self.asks[i]);
            }
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
