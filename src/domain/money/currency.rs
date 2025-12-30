use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    EUR,
    USD,
    ILS,
}

impl Currency {
    pub fn minor_units(self) -> u32 {
        match self {
            Currency::EUR => 2,
            Currency::USD => 2,
            Currency::ILS => 2,
        }
    }
}
