#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderState {
    Draft,
    Placed,
    PaymentPending,
    Paid,
    FulfillmentPending,
    PartiallyShipped,
    Shipped,
    Cancelled,
    Refunded,
    PaymentFailed,
}
