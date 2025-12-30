pub mod cart;
pub mod item;
pub mod totals;

#[cfg(test)]
mod tests;

// Public API
pub use cart::Cart;
pub use item::CartItem;
// CartTotals intentionally NOT re-exported yet (internal)
