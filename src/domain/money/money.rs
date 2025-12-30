use crate::domain::money::currency::Currency;
use crate::domain::money::rounding::round;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        let scale = currency.minor_units();
        Self {
            amount: round(amount, scale),
            currency,
        }
    }

    pub fn zero(currency: Currency) -> Self {
        Self::new(Decimal::ZERO, currency)
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
        self.ensure_same_currency(other)?;
        Ok(Money::new(self.amount + other.amount, self.currency))
    }

    pub fn subtract(&self, other: &Money) -> Result<Money, MoneyError> {
        self.ensure_same_currency(other)?;
        Ok(Money::new(self.amount - other.amount, self.currency))
    }

    fn ensure_same_currency(&self, other: &Money) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoneyError {
    #[error("Currency mismatch")]
    CurrencyMismatch,
}
