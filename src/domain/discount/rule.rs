use crate::domain::discount::discount::{Discount, DiscountStacking};

#[derive(Debug, Clone)]
pub struct DiscountRule {
    pub discount: Discount,
    pub stacking: DiscountStacking,
    pub priority: u32, // lower = applied first
}
