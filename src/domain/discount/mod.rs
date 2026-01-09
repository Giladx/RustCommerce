pub mod apply;
pub mod discount;
pub mod rule;

// Public API
pub use discount::Discount;
pub use rule::DiscountRule;

// Keep stacking internal for now
