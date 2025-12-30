use super::{OrderState, OrderTransitionError};

impl OrderState {
    pub fn place(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::Draft => Ok(OrderState::Placed),
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "Placed".into(),
            }),
        }
    }

    pub fn request_payment(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::Placed => Ok(OrderState::PaymentPending),
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "PaymentPending".into(),
            }),
        }
    }

    pub fn mark_paid(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::PaymentPending => Ok(OrderState::Paid),
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "Paid".into(),
            }),
        }
    }

    pub fn start_fulfillment(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::Paid => Ok(OrderState::FulfillmentPending),
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "FulfillmentPending".into(),
            }),
        }
    }

    pub fn ship_partial(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::FulfillmentPending => Ok(OrderState::PartiallyShipped),
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "PartiallyShipped".into(),
            }),
        }
    }

    pub fn ship_all(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::FulfillmentPending | OrderState::PartiallyShipped => {
                Ok(OrderState::Shipped)
            }
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "Shipped".into(),
            }),
        }
    }

    pub fn cancel(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::Draft | OrderState::Placed | OrderState::PaymentPending => {
                Ok(OrderState::Cancelled)
            }
            _ => Err(OrderTransitionError::AlreadyFinalized),
        }
    }

    pub fn refund(self) -> Result<OrderState, OrderTransitionError> {
        match self {
            OrderState::Paid | OrderState::Shipped | OrderState::PartiallyShipped => {
                Ok(OrderState::Refunded)
            }
            _ => Err(OrderTransitionError::InvalidTransition {
                from: format!("{:?}", self),
                to: "Refunded".into(),
            }),
        }
    }
}
