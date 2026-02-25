use tabled::Tabled;

use crate::client::trading::{LockedBalance, UserOrder};
use crate::output::print_table;

#[derive(Tabled)]
struct UserOrderRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Qty")]
    quantity: String,
    #[tabled(rename = "Type")]
    order_type: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn print_user_orders_table(orders: &[UserOrder]) {
    let rows: Vec<UserOrderRow> = orders
        .iter()
        .map(|o| UserOrderRow {
            id: crate::output::truncate(&o.id, 12),
            side: o.side.clone(),
            price: o.price.clone(),
            quantity: o
                .quantity
                .clone()
                .or_else(|| o.size.clone())
                .unwrap_or_else(|| "-".to_string()),
            order_type: o.order_type.clone().unwrap_or_else(|| "-".to_string()),
            status: o.status.clone(),
        })
        .collect();
    print_table(&rows);
}

pub fn print_locked_balance(balance: &LockedBalance) {
    let formatted = balance
        .locked_balance_formatted
        .as_deref()
        .or(balance.locked_balance.as_deref())
        .unwrap_or("0");
    let count = balance.order_count.unwrap_or(0);
    let currency = balance.currency.as_deref().unwrap_or("USDC");
    println!("Locked: {} {}", formatted, currency);
    println!("Open orders: {}", count);
}
