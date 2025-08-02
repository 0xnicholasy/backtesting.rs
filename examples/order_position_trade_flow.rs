use backtesting::types::{DirectionalTrade, Executable, StopManagement};
use backtesting::{Order, OrderSide, OrderType};
use chrono::Utc;

fn main() {
    println!("=== Order -> Position -> Trade Flow Demo ===\n");

    // Step 1: Create an Order
    println!("1. Creating a Buy Order with Stop Loss and Take Profit");
    let mut buy_order = Order::new(
        OrderSide::Buy,
        OrderType::Market,
        1000.0,      // Size
        None,        // No limit price (market order)
        None,        // No stop price
        Some(95.0),  // Stop loss at $95
        Some(115.0), // Take profit at $115
        Some("AAPL Long".to_string()),
    );

    println!("   Order Details:");
    println!("   - Side: Buy");
    println!("   - Size: {}", buy_order.size());
    println!("   - Stop Loss: {:?}", buy_order.sl());
    println!("   - Take Profit: {:?}", buy_order.tp());
    println!("   - Status: {:?}", buy_order.status);
    println!("   - Is Long: {}", buy_order.is_long());
    println!("   - Is Contingent: {}", buy_order.is_contingent());

    // Step 2: Execute Order to create Position
    println!("\n2. Executing Order at $100 to create Position");
    let entry_price = 100.0;
    let entry_time = Utc::now();

    // First fill the order
    let _trade = buy_order.fill(buy_order.remaining_size(), entry_price, 0, entry_time);

    if let Some(mut position) = buy_order.to_position(entry_price, entry_time) {
        println!("   Position Created:");
        println!("   - Size: {}", position.size());
        println!("   - Entry Price: ${:.2}", position.entry_price);
        println!("   - Is Long: {}", position.is_long());
        println!("   - Has SL: {}", position.has_sl());
        println!("   - Has TP: {}", position.has_tp());

        // Step 3: Simulate price movements
        println!("\n3. Simulating Price Movements");

        // Price goes up to $105
        position.update_price(105.0);
        println!("   Price moves to $105.00:");
        println!("   - Unrealized P&L: ${:.2}", position.pl());
        println!("   - P&L Percentage: {:.2}%", position.pl_pct() * 100.0);
        println!("   - Should Trigger SL: {}", position.should_trigger_sl());
        println!("   - Should Trigger TP: {}", position.should_trigger_tp());

        // Price goes up to $115 (should trigger TP)
        position.update_price(115.0);
        println!("\n   Price moves to $115.00 (Take Profit level):");
        println!("   - Unrealized P&L: ${:.2}", position.pl());
        println!("   - P&L Percentage: {:.2}%", position.pl_pct() * 100.0);
        println!("   - Should Trigger SL: {}", position.should_trigger_sl());
        println!("   - Should Trigger TP: {}", position.should_trigger_tp());

        // Step 4: Close Position to create Trade
        if position.should_trigger_tp() {
            println!("\n4. Take Profit Triggered - Closing Position");
            let exit_time = Utc::now();
            let closed_trade = position.close_all(115.0, exit_time, Some(1));

            println!("   Trade Closed:");
            println!("   - Entry Price: ${:.2}", closed_trade.entry_price);
            println!(
                "   - Exit Price: ${:.2}",
                closed_trade.exit_price.unwrap_or(0.0)
            );
            println!("   - Size: {}", closed_trade.size());
            println!("   - Is Long: {}", closed_trade.is_long());
            println!("   - Is Closed: {}", closed_trade.is_closed());
            println!("   - Realized P&L: ${:.2}", closed_trade.pl());
            println!("   - P&L Percentage: {:.2}%", closed_trade.pl_pct() * 100.0);
            println!("   - Duration: {:?}", closed_trade.duration());
        }
    }

    // Demonstrate Short Order
    println!("\n=== Short Order Example ===");
    let mut short_order = Order::new(
        OrderSide::Sell,
        OrderType::Market,
        500.0,
        None,
        None,
        Some(105.0), // Stop loss at $105 (higher than entry for short)
        Some(85.0),  // Take profit at $85
        Some("AAPL Short".to_string()),
    );

    println!("Short Order:");
    println!("   - Is Short: {}", short_order.is_short());
    println!("   - Stop Loss: {:?}", short_order.sl());
    println!("   - Take Profit: {:?}", short_order.tp());

    let _short_trade = short_order.fill(short_order.remaining_size(), 100.0, 0, Utc::now());
    if let Some(mut short_position) = short_order.to_position(100.0, Utc::now()) {
        short_position.update_price(90.0); // Price drops to $90
        println!("   Short Position at $90:");
        println!("   - P&L: ${:.2}", short_position.pl());
        println!("   - P&L %: {:.2}%", short_position.pl_pct() * 100.0);
    }

    println!("\n=== Demo Complete ===");
}
