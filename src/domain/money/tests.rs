mod tests {
    use crate::domain::money::currency::Currency;
    use crate::domain::money::money::Money;
    use crate::domain::money::vat::{from_gross, from_net, VatRate};

    use rust_decimal_macros::dec;

    #[test]
    fn vat_from_net_ils() {
        let net = Money::new(dec!(100), Currency::ILS);
        let vat = VatRate::new(dec!(0.17));

        let result = from_net(&net, vat);

        assert_eq!(result.vat.amount(), dec!(17));
        assert_eq!(result.gross.amount(), dec!(117));
    }

    #[test]
    fn vat_from_gross_ils() {
        let gross = Money::new(dec!(117), Currency::ILS);
        let vat = VatRate::new(dec!(0.17));

        let result = from_gross(&gross, vat);

        assert_eq!(result.net.amount(), dec!(100));
        assert_eq!(result.vat.amount(), dec!(17));
    }

    #[test]
    fn zero_vat() {
        let gross = Money::new(dec!(100), Currency::EUR);
        let vat = VatRate::zero();

        let result = from_gross(&gross, vat);

        assert_eq!(result.net.amount(), dec!(100));
        assert_eq!(result.vat.amount(), dec!(0));
    }
}
