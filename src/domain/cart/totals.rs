use crate::domain::money::money::Money;

#[derive(Debug, Clone)]
pub struct CartTotals {
    pub net: Money,
    pub vat: Money,
    pub gross: Money,
}
