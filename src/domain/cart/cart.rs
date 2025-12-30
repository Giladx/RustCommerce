use crate::domain::cart::item::CartItem;
use crate::domain::cart::totals::CartTotals;
use crate::domain::discount::apply::apply_discounts;
use crate::domain::discount::rule::DiscountRule;
use crate::domain::money::currency::Currency;
use crate::domain::money::money::Money;
use crate::domain::money::vat::{from_net, VatRate};

#[derive(Debug)]
pub struct Cart {
    pub items: Vec<CartItem>,
    pub discounts: Vec<DiscountRule>,
    pub vat_rate: VatRate,
    pub currency: Currency,
}

impl Cart {
    pub fn new(currency: Currency, vat_rate: VatRate) -> Self {
        Self {
            items: Vec::new(),
            discounts: Vec::new(),
            vat_rate,
            currency,
        }
    }

    pub fn add_item(&mut self, item: CartItem) {
        // Currency validation will be hardened later
        self.items.push(item);
    }

    pub fn add_discount(&mut self, rule: DiscountRule) {
        self.discounts.push(rule);
    }

    pub fn calculate_totals(&self) -> CartTotals {
        // 1️⃣ Sum NET line totals
        let net_total = self.items.iter().fold(
            Money::zero(self.currency),
            |acc, item| acc.add(&item.line_total_net()).unwrap(),
        );

        // 2️⃣ Apply discounts on NET only
        let discounted_net = apply_discounts(&net_total, &self.discounts);

        // 3️⃣ Recalculate VAT AFTER discounts
        let vat_breakdown = from_net(&discounted_net, self.vat_rate);

        CartTotals {
            net: vat_breakdown.net,
            vat: vat_breakdown.vat,
            gross: vat_breakdown.gross,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cart::item::CartItem;
    use crate::domain::discount::discount::{Discount, DiscountStacking};
    use crate::domain::discount::rule::DiscountRule;
    use rust_decimal_macros::dec;

    #[test]
    fn cart_totals_ils_with_vat() {
        let mut cart = Cart::new(Currency::ILS, VatRate::new(dec!(0.17)));

        cart.add_item(CartItem {
            product_id: "A".into(),
            quantity: 2,
            unit_price: Money::new(dec!(100), Currency::ILS),
        });

        cart.add_item(CartItem {
            product_id: "B".into(),
            quantity: 1,
            unit_price: Money::new(dec!(50), Currency::ILS),
        });

        let totals = cart.calculate_totals();

        assert_eq!(totals.net.amount(), dec!(250));
        assert_eq!(totals.vat.amount(), dec!(42.50));
        assert_eq!(totals.gross.amount(), dec!(292.50));
    }

    #[test]
    fn threshold_discount_500_get_150() {
        let mut cart = Cart::new(Currency::ILS, VatRate::new(dec!(0.17)));

        cart.add_item(CartItem {
            product_id: "A".into(),
            quantity: 5,
            unit_price: Money::new(dec!(100), Currency::ILS),
        });

        cart.add_discount(DiscountRule {
            discount: Discount::Threshold {
                min_spend: Money::new(dec!(500), Currency::ILS),
                reward: Money::new(dec!(150), Currency::ILS),
            },
            stacking: DiscountStacking::Exclusive,
            priority: 1,
        });

        let totals = cart.calculate_totals();

        assert_eq!(totals.net.amount(), dec!(350));
        assert_eq!(totals.vat.amount(), dec!(59.50));
        assert_eq!(totals.gross.amount(), dec!(409.50));
    }
}
