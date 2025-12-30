use crate::domain::money::money::Money;
use crate::domain::money::rounding::round;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy)]
pub struct VatRate {
    rate: Decimal, // 0.17 = 17%
}

impl VatRate {
    pub fn new(rate: Decimal) -> Self {
        assert!(rate >= Decimal::ZERO);
        Self { rate }
    }

    pub fn zero() -> Self {
        Self { rate: Decimal::ZERO }
    }

    pub fn rate(&self) -> Decimal {
        self.rate
    }
}

#[derive(Debug)]
pub struct VatBreakdown {
    pub net: Money,
    pub vat: Money,
    pub gross: Money,
}

pub fn from_net(net: &Money, vat_rate: VatRate) -> VatBreakdown {
    let vat_amount = round(
        net.amount() * vat_rate.rate(),
        net.currency().minor_units(),
    );

    let vat = Money::new(vat_amount, net.currency());
    let gross = net.add(&vat).unwrap();

    VatBreakdown {
        net: net.clone(),
        vat,
        gross,
    }
}

pub fn from_gross(gross: &Money, vat_rate: VatRate) -> VatBreakdown {
    if vat_rate.rate() == Decimal::ZERO {
        return VatBreakdown {
            net: gross.clone(),
            vat: Money::zero(gross.currency()),
            gross: gross.clone(),
        };
    }

    let divisor = dec!(1) + vat_rate.rate();
    let net_amount = round(
        gross.amount() / divisor,
        gross.currency().minor_units(),
    );

    let net = Money::new(net_amount, gross.currency());
    let vat = gross.subtract(&net).unwrap();

    VatBreakdown {
        net,
        vat,
        gross: gross.clone(),
    }
}
