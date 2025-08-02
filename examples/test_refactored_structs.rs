use backtesting::{DirectionalTrade, Executable, Order, OrderSide, OrderType, StopManagement};
use chrono::Utc;

fn main() {
    println!("Testing refactored Order, Position, Trade structs with traits");

    // Create an order
    let mut order = Order::new(
        OrderSide::Buy,
        OrderType::Market,
        100.0,
        None,
        None,
        Some(95.0),  // Stop loss
        Some(110.0), // Take profit
        Some("Test Order".to_string()),
    );

    println!("Order created: {:?}", order);
    println!("Order is long: {}", order.is_long());
    println!("Order has SL: {}", order.has_sl());
    println!("Order has TP: {}", order.has_tp());

    // Execute order to create a trade
    let fill_time = Utc::now();
    if let Some(trade) = order.execute(100.0, fill_time, 0) {
        println!("Trade created from order: {:?}", trade);
        println!("Trade is long: {}", trade.is_long());
        println!("Trade size: {}", trade.size());
    }

    // Convert order to position
    if let Some(mut position) = order.to_position(100.0, fill_time) {
        println!("Position created from order: {:?}", position);
        println!("Position is long: {}", position.is_long());

        // Update position price
        position.update_price(105.0);
        println!("Position P&L: {:.2}", position.pl());
        println!("Position P&L %: {:.2}%", position.pl_pct() * 100.0);

        // Check if stops should trigger
        println!(
            "Should trigger SL at 105.0: {}",
            position.should_trigger_sl()
        );
        println!(
            "Should trigger TP at 105.0: {}",
            position.should_trigger_tp()
        );

        // Close position to create trade
        let closed_trade = position.close_all(105.0, Utc::now(), Some(1));
        println!("Trade created from closing position: {:?}", closed_trade);
        println!("Closed trade P&L: {:.2}", closed_trade.pl());
        println!("Closed trade P&L %: {:.2}%", closed_trade.pl_pct() * 100.0);
    }

    println!("All tests completed successfully!");
}
