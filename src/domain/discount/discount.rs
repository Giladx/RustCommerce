use crate::domain::money::money::Money;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum Discount {
    Percentage {
        rate: Decimal, // 0.10 = 10%
    },
    FixedAmount {
        amount: Money,
    },
    Threshold {
        min_spend: Money,
        reward: Money,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscountStacking {
    Stackable,
    Exclusive,
}
