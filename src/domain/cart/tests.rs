use rust_decimal_macros::dec;

use super::{cart::Cart, item::CartItem};
use crate::domain::money::{currency::Currency, money::Money, vat::VatRate};

#[test]
fn cart_calculates_vat_and_totals() {
    let mut cart = Cart::new(Currency::EUR, VatRate::new(dec!(0.19)));
    cart.add_item(CartItem {
        product_id: "sku-1".to_string(),
        quantity: 2,
        unit_price: Money::new(dec!(15), Currency::EUR),
    });

    let totals = cart.calculate_totals();

    assert_eq!(totals.net.amount(), dec!(30));
    assert_eq!(totals.vat.amount(), dec!(5.7));
    assert_eq!(totals.gross.amount(), dec!(35.7));
}
