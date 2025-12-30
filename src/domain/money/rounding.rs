use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

pub fn round(amount: Decimal, scale: u32) -> Decimal {
    amount.round_dp_with_strategy(scale, RoundingStrategy::MidpointAwayFromZero)
}
