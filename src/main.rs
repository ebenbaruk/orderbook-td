use crate::{benchmarks::OrderBookBenchmark, orderbook::OrderBookImpl};

mod benchmarks;
mod interfaces;
mod orderbook;

// Objective: Complete the orderbook implementation at ./orderbook.rs and run this file to see how fast it is. Faster implementation wins !

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    println!("Running Naive OrderBook Benchmark...\n");

    let result = OrderBookBenchmark::run::<OrderBookImpl>("OrderBook", 100_000);
    OrderBookBenchmark::print_results(&result);

    println!("\n Competition Goal: Achieve sub-nanosecond operations!");
    println!(" Tips:");
    println!("   - Use cache-friendly data structures");
    println!("   - Consider BTreeMap for sorted access");
    println!("   - Pre-allocate where possible");
    println!("   - Profile with 'cargo flamegraph'");
    println!("   - Use 'cargo bench' for micro-benchmarks");
}

// ============================================================================
// CORRECTNESS TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{
        interfaces::{OrderBook, Side, Update},
        orderbook::OrderBookImpl,
    };

    fn test_basic_operations<T: OrderBook>() {
        let mut ob = T::new();

        // Add bids
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 9950,
            quantity: 150,
            side: Side::Bid,
        });

        // Add asks
        ob.apply_update(Update::Set {
            price: 10050,
            quantity: 80,
            side: Side::Ask,
        });
        ob.apply_update(Update::Set {
            price: 10100,
            quantity: 120,
            side: Side::Ask,
        });

        assert_eq!(ob.get_best_bid(), Some(10000));
        assert_eq!(ob.get_best_ask(), Some(10050));
        assert_eq!(ob.get_spread(), Some(50));
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));
    }

    fn test_updates_and_removes<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));

        // Update quantity
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 200,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(200));

        // Remove via zero quantity
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 0,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);

        // Remove via Remove update
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Remove {
            price: 10000,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);
    }

    fn test_empty_book<T: OrderBook>() {
        let ob = T::new();

        assert_eq!(ob.get_best_bid(), None);
        assert_eq!(ob.get_best_ask(), None);
        assert_eq!(ob.get_spread(), None);

        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);
        assert_eq!(ob.get_quantity_at(10000, Side::Ask), None);

        assert_eq!(ob.get_top_levels(Side::Bid, 5), vec![]);
        assert_eq!(ob.get_top_levels(Side::Ask, 5), vec![]);

        assert_eq!(ob.get_total_quantity(Side::Bid), 0);
        assert_eq!(ob.get_total_quantity(Side::Ask), 0);
    }

    fn test_one_side_empty<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });

        // no ask, so no spread
        assert_eq!(ob.get_best_bid(), Some(10000));
        assert_eq!(ob.get_best_ask(), None);
        assert_eq!(ob.get_spread(), None);
    }

    fn test_top_levels<T: OrderBook>() {
        let mut ob = T::new();

        // add the prices in a random order
        ob.apply_update(Update::Set {
            price: 9900,
            quantity: 30,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 10,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 9950,
            quantity: 20,
            side: Side::Bid,
        });

        ob.apply_update(Update::Set {
            price: 10200,
            quantity: 60,
            side: Side::Ask,
        });
        ob.apply_update(Update::Set {
            price: 10100,
            quantity: 40,
            side: Side::Ask,
        });
        ob.apply_update(Update::Set {
            price: 10150,
            quantity: 50,
            side: Side::Ask,
        });

        // bids: highest price first
        assert_eq!(
            ob.get_top_levels(Side::Bid, 2),
            vec![(10000, 10), (9950, 20)]
        );

        // asks: lowest price first
        assert_eq!(
            ob.get_top_levels(Side::Ask, 2),
            vec![(10100, 40), (10150, 50)]
        );

        // ask for more levels than there are
        assert_eq!(
            ob.get_top_levels(Side::Bid, 10),
            vec![(10000, 10), (9950, 20), (9900, 30)]
        );

        assert_eq!(ob.get_top_levels(Side::Ask, 0), vec![]);
    }

    fn test_total_quantity<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 9950,
            quantity: 150,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 10050,
            quantity: 80,
            side: Side::Ask,
        });

        assert_eq!(ob.get_total_quantity(Side::Bid), 250);
        assert_eq!(ob.get_total_quantity(Side::Ask), 80);

        // update a level, the total changes
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 50,
            side: Side::Bid,
        });
        assert_eq!(ob.get_total_quantity(Side::Bid), 200);

        // remove a level, the total changes
        ob.apply_update(Update::Remove {
            price: 9950,
            side: Side::Bid,
        });
        assert_eq!(ob.get_total_quantity(Side::Bid), 50);
    }

    fn test_best_price_after_remove<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 9950,
            quantity: 150,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 10050,
            quantity: 80,
            side: Side::Ask,
        });
        ob.apply_update(Update::Set {
            price: 10100,
            quantity: 120,
            side: Side::Ask,
        });

        // remove the best bid, the next one becomes the best
        ob.apply_update(Update::Remove {
            price: 10000,
            side: Side::Bid,
        });
        assert_eq!(ob.get_best_bid(), Some(9950));

        // remove the best ask with a zero quantity
        ob.apply_update(Update::Set {
            price: 10050,
            quantity: 0,
            side: Side::Ask,
        });
        assert_eq!(ob.get_best_ask(), Some(10100));

        assert_eq!(ob.get_spread(), Some(150));

        // remove the last bid, the side is empty
        ob.apply_update(Update::Remove {
            price: 9950,
            side: Side::Bid,
        });
        assert_eq!(ob.get_best_bid(), None);
        assert_eq!(ob.get_spread(), None);
    }

    fn test_remove_missing_price<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });

        // removing a price that is not in the book should do nothing
        ob.apply_update(Update::Remove {
            price: 12345,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 12345,
            quantity: 0,
            side: Side::Bid,
        });
        ob.apply_update(Update::Remove {
            price: 10000,
            side: Side::Ask,
        });

        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));
        assert_eq!(ob.get_quantity_at(12345, Side::Bid), None);
        assert_eq!(ob.get_best_bid(), Some(10000));
        assert_eq!(ob.get_best_ask(), None);
    }

    fn test_sides_are_separate<T: OrderBook>() {
        let mut ob = T::new();

        // same price on both sides
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 70,
            side: Side::Ask,
        });

        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));
        assert_eq!(ob.get_quantity_at(10000, Side::Ask), Some(70));

        // removing the bid should not remove the ask
        ob.apply_update(Update::Remove {
            price: 10000,
            side: Side::Bid,
        });

        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);
        assert_eq!(ob.get_quantity_at(10000, Side::Ask), Some(70));
    }

    fn test_crossed_book<T: OrderBook>() {
        let mut ob = T::new();

        // the bid is higher than the ask, the spread is negative
        ob.apply_update(Update::Set {
            price: 10100,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Ask,
        });

        assert_eq!(ob.get_spread(), Some(-100));
    }

    fn test_many_levels<T: OrderBook>() {
        let mut ob = T::new();

        // same prices as the benchmark
        for i in 0..1000 {
            ob.apply_update(Update::Set {
                price: 100000 + i * 10,
                quantity: 1,
                side: Side::Bid,
            });

            ob.apply_update(Update::Set {
                price: 100000 + i * 10,
                quantity: 2,
                side: Side::Ask,
            });
        }

        assert_eq!(ob.get_best_bid(), Some(109990));
        assert_eq!(ob.get_best_ask(), Some(100000));
        assert_eq!(ob.get_spread(), Some(-9990));

        assert_eq!(ob.get_total_quantity(Side::Bid), 1000);
        assert_eq!(ob.get_total_quantity(Side::Ask), 2000);

        assert_eq!(
            ob.get_top_levels(Side::Bid, 3),
            vec![(109990, 1), (109980, 1), (109970, 1)]
        );
    }

    #[test]
    fn test_naive_implementation() {
        test_basic_operations::<OrderBookImpl>();
        test_updates_and_removes::<OrderBookImpl>();
    }

    #[test]
    fn test_empty() {
        test_empty_book::<OrderBookImpl>();
        test_one_side_empty::<OrderBookImpl>();
    }

    #[test]
    fn test_levels_and_totals() {
        test_top_levels::<OrderBookImpl>();
        test_total_quantity::<OrderBookImpl>();
    }

    #[test]
    fn test_removes() {
        test_best_price_after_remove::<OrderBookImpl>();
        test_remove_missing_price::<OrderBookImpl>();
        test_sides_are_separate::<OrderBookImpl>();
    }

    #[test]
    fn test_special_cases() {
        test_crossed_book::<OrderBookImpl>();
        test_many_levels::<OrderBookImpl>();
    }
}
