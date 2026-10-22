use crate::interfaces::{OrderBook, Price, Quantity, Side, Update};

pub struct OrderBookImpl {
    // the quantity of a price is at index (price - base_price)
    // a quantity of 0 means there is no level at this price
    bid_quantities: Vec<Quantity>,

    ask_quantities: Vec<Quantity>,

    // the price at index 0, the same for both sides
    base_price: Price,

    // we keep the best prices so we don't have to search for them
    best_bid: Option<Price>,

    best_ask: Option<Price>,
}

impl OrderBookImpl {
    fn is_in_range(&self, price: Price) -> bool {
        let last_price = self.base_price + self.bid_quantities.len() as Price - 1;

        price >= self.base_price && price <= last_price
    }

    // make the lists bigger so that the price fits in them
    fn make_room(&mut self, price: Price) {
        if self.bid_quantities.len() == 0 {
            self.base_price = price;
            self.bid_quantities.push(0);
            self.ask_quantities.push(0);
            return;
        }

        if price < self.base_price {
            // add zeros at the beginning
            let extra = (self.base_price - price) as usize;

            let mut new_bids = Vec::new();
            let mut new_asks = Vec::new();

            for _ in 0..extra {
                new_bids.push(0);
                new_asks.push(0);
            }

            for i in 0..self.bid_quantities.len() {
                new_bids.push(self.bid_quantities[i]);
                new_asks.push(self.ask_quantities[i]);
            }

            self.bid_quantities = new_bids;
            self.ask_quantities = new_asks;
            self.base_price = price;
        }

        let last_price = self.base_price + self.bid_quantities.len() as Price - 1;

        if price > last_price {
            // add zeros at the end
            let extra = (price - last_price) as usize;

            for _ in 0..extra {
                self.bid_quantities.push(0);
                self.ask_quantities.push(0);
            }
        }
    }

    fn set_bid(&mut self, price: Price, quantity: Quantity) {
        if !self.is_in_range(price) {
            self.make_room(price);
        }

        let i = (price - self.base_price) as usize;
        self.bid_quantities[i] = quantity;

        // the best bid is the highest price
        if self.best_bid.is_none() || price > self.best_bid.unwrap() {
            self.best_bid = Some(price);
        }
    }

    fn set_ask(&mut self, price: Price, quantity: Quantity) {
        if !self.is_in_range(price) {
            self.make_room(price);
        }

        let i = (price - self.base_price) as usize;
        self.ask_quantities[i] = quantity;

        // the best ask is the lowest price
        if self.best_ask.is_none() || price < self.best_ask.unwrap() {
            self.best_ask = Some(price);
        }
    }

    fn remove_bid(&mut self, price: Price) {
        if !self.is_in_range(price) {
            return;
        }

        let i = (price - self.base_price) as usize;

        if self.bid_quantities[i] == 0 {
            return;
        }

        self.bid_quantities[i] = 0;

        if self.best_bid == Some(price) {
            // look for the next best bid, below this price
            self.best_bid = None;

            for k in 1..=i {
                let j = i - k;

                if self.bid_quantities[j] > 0 {
                    self.best_bid = Some(self.base_price + j as Price);
                    break;
                }
            }
        }
    }

    fn remove_ask(&mut self, price: Price) {
        if !self.is_in_range(price) {
            return;
        }

        let i = (price - self.base_price) as usize;

        if self.ask_quantities[i] == 0 {
            return;
        }

        self.ask_quantities[i] = 0;

        if self.best_ask == Some(price) {
            // look for the next best ask, above this price
            self.best_ask = None;

            for j in (i + 1)..self.ask_quantities.len() {
                if self.ask_quantities[j] > 0 {
                    self.best_ask = Some(self.base_price + j as Price);
                    break;
                }
            }
        }
    }
}

impl OrderBook for OrderBookImpl {
    fn new() -> Self {
        OrderBookImpl {
            bid_quantities: Vec::new(),

            ask_quantities: Vec::new(),

            base_price: 0,

            best_bid: None,

            best_ask: None,
        }
    }

    fn apply_update(&mut self, update: Update) {
        match update {
            Update::Set {
                price,
                quantity,
                side,
            } => {
                if side == Side::Bid {
                    if quantity == 0 {
                        self.remove_bid(price);
                    } else {
                        self.set_bid(price, quantity);
                    }
                } else {
                    if quantity == 0 {
                        self.remove_ask(price);
                    } else {
                        self.set_ask(price, quantity);
                    }
                }
            }

            Update::Remove { price, side } => {
                if side == Side::Bid {
                    self.remove_bid(price);
                } else {
                    self.remove_ask(price);
                }
            }
        }
    }

    fn get_spread(&self) -> Option<Price> {
        if self.best_bid.is_none() || self.best_ask.is_none() {
            return None;
        }

        Some(self.best_ask.unwrap() - self.best_bid.unwrap())
    }

    fn get_best_bid(&self) -> Option<Price> {
        self.best_bid
    }

    fn get_best_ask(&self) -> Option<Price> {
        self.best_ask
    }

    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity> {
        if !self.is_in_range(price) {
            return None;
        }

        let i = (price - self.base_price) as usize;

        let quantity = if side == Side::Bid {
            self.bid_quantities[i]
        } else {
            self.ask_quantities[i]
        };

        if quantity == 0 {
            return None;
        }

        Some(quantity)
    }

    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)> {
        let mut levels = Vec::new();

        if side == Side::Bid {
            if self.best_bid.is_none() {
                return levels;
            }

            // bids: start from the best bid and go down
            let start = (self.best_bid.unwrap() - self.base_price) as usize;

            for k in 0..=start {
                if levels.len() == n {
                    break;
                }

                let j = start - k;

                if self.bid_quantities[j] > 0 {
                    levels.push((self.base_price + j as Price, self.bid_quantities[j]));
                }
            }
        } else {
            if self.best_ask.is_none() {
                return levels;
            }

            // asks: start from the best ask and go up
            let start = (self.best_ask.unwrap() - self.base_price) as usize;

            for j in start..self.ask_quantities.len() {
                if levels.len() == n {
                    break;
                }

                if self.ask_quantities[j] > 0 {
                    levels.push((self.base_price + j as Price, self.ask_quantities[j]));
                }
            }
        }

        levels
    }

    fn get_total_quantity(&self, side: Side) -> Quantity {
        let quantities = if side == Side::Bid {
            &self.bid_quantities
        } else {
            &self.ask_quantities
        };

        let mut total = 0;

        for i in 0..quantities.len() {
            total += quantities[i];
        }
        total
    }
}
