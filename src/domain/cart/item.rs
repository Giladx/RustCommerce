use crate::domain::money::money::Money;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price: Money,
}

impl CartItem {
    pub fn line_total_net(&self) -> Money {
        let quantity = Decimal::from(self.quantity);
        let amount = self.unit_price.amount() * quantity;

        Money::new(amount, self.unit_price.currency())
    }
}
