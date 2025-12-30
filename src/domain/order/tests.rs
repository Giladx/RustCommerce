use super::OrderState;

#[test]
fn happy_path_order_flow() {
    let state = OrderState::Draft
        .place().unwrap()
        .request_payment().unwrap()
        .mark_paid().unwrap()
        .start_fulfillment().unwrap()
        .ship_all().unwrap();

    assert_eq!(state, OrderState::Shipped);
}

#[test]
fn cannot_ship_unpaid_order() {
    let result = OrderState::Placed.ship_all();
    assert!(result.is_err());
}

#[test]
fn cannot_cancel_after_shipping() {
    let state = OrderState::Paid
        .start_fulfillment().unwrap()
        .ship_all().unwrap();

    assert!(state.cancel().is_err());
}
