use crate::domain::discount::discount::{Discount, DiscountStacking};
use crate::domain::discount::rule::DiscountRule;
use crate::domain::money::money::Money;
use crate::domain::money::rounding::round;

pub fn apply_discounts(
    net_total: &Money,
    rules: &[DiscountRule],
) -> Money {
    let mut current = net_total.clone();

    let mut rules = rules.to_vec();
    rules.sort_by_key(|r| r.priority);

    for rule in rules {
        let discount_amount = match &rule.discount {
            Discount::Percentage { rate } => {
                Money::new(
                    round(current.amount() * *rate, current.currency().minor_units()),
                    current.currency(),
                )
            }

            Discount::FixedAmount { amount } => {
                if amount.currency() != current.currency() {
                    continue;
                }
                amount.clone()
            }

            Discount::Threshold { min_spend, reward } => {
                if min_spend.currency() != current.currency()
                    || reward.currency() != current.currency()
                {
                    continue;
                }

                if current.amount() >= min_spend.amount() {
                    reward.clone()
                } else {
                    Money::zero(current.currency())
                }
            }
        };

        // Prevent negative totals
        if discount_amount.amount() > current.amount() {
            current = Money::zero(current.currency());
        } else {
            current = current.subtract(&discount_amount).unwrap();
        }

        if rule.stacking == DiscountStacking::Exclusive {
            break;
        }
    }

    current
}
