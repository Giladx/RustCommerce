-- Chat Messages (Customer Service)
CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    sender VARCHAR(20) NOT NULL CHECK (sender IN ('customer', 'support')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_customer ON chat_messages(customer_id);
CREATE INDEX idx_chat_messages_created ON chat_messages(created_at DESC);

-- Order Messages (Questions from customers about orders)
CREATE TABLE IF NOT EXISTS order_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    is_internal BOOLEAN DEFAULT FALSE,
    reply TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    replied_at TIMESTAMPTZ
);

CREATE INDEX idx_order_messages_order ON order_messages(order_id);
CREATE INDEX idx_order_messages_created ON order_messages(created_at DESC);

-- Merchandise Returns
CREATE TABLE IF NOT EXISTS merchandise_returns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    items JSONB NOT NULL DEFAULT '[]'::jsonb,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'completed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_merchandise_returns_order ON merchandise_returns(order_id);
CREATE INDEX idx_merchandise_returns_status ON merchandise_returns(status);
CREATE INDEX idx_merchandise_returns_created ON merchandise_returns(created_at DESC);
