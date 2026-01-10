-- Order Status System

-- Order Statuses Configuration Table
CREATE TABLE IF NOT EXISTS order_statuses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(20), -- hex color for UI display
    icon VARCHAR(50), -- material icon name
    is_default BOOLEAN DEFAULT FALSE,
    is_final BOOLEAN DEFAULT FALSE, -- cannot transition from final states
    position INTEGER DEFAULT 0, -- display order
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Order Status Transitions (allowed state changes)
CREATE TABLE IF NOT EXISTS order_status_transitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_status_id UUID NOT NULL REFERENCES order_statuses(id) ON DELETE CASCADE,
    to_status_id UUID NOT NULL REFERENCES order_statuses(id) ON DELETE CASCADE,
    requires_confirmation BOOLEAN DEFAULT FALSE,
    confirmation_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(from_status_id, to_status_id)
);

-- Order Status History (audit trail)
CREATE TABLE IF NOT EXISTS order_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    from_status VARCHAR(50),
    to_status VARCHAR(50) NOT NULL,
    changed_by VARCHAR(255), -- user who made the change
    notes TEXT,
    metadata JSONB, -- additional data like tracking numbers, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add status_id to orders table (will reference order_statuses)
ALTER TABLE orders ADD COLUMN IF NOT EXISTS status_id UUID REFERENCES order_statuses(id);

-- Seed default order statuses
INSERT INTO order_statuses (code, name, description, color, icon, is_default, is_final, position) VALUES
('pending', 'Pending', 'Order placed, awaiting payment', '#FFA500', 'pending', true, false, 1),
('confirmed', 'Confirmed', 'Payment confirmed, processing order', '#2196F3', 'check_circle', false, false, 2),
('processing', 'Processing', 'Order is being prepared', '#9C27B0', 'settings', false, false, 3),
('packed', 'Packed', 'Order packed and ready to ship', '#673AB7', 'inventory', false, false, 4),
('shipped', 'Shipped', 'Order shipped to customer', '#3F51B5', 'local_shipping', false, false, 5),
('in_transit', 'In Transit', 'Order is on the way', '#00BCD4', 'flight', false, false, 6),
('out_for_delivery', 'Out for Delivery', 'Order out for delivery today', '#009688', 'delivery_dining', false, false, 7),
('delivered', 'Delivered', 'Order successfully delivered', '#4CAF50', 'done_all', false, true, 8),
('cancelled', 'Cancelled', 'Order cancelled', '#F44336', 'cancel', false, true, 9),
('refunded', 'Refunded', 'Order refunded', '#FF5722', 'money_off', false, true, 10),
('on_hold', 'On Hold', 'Order temporarily on hold', '#FF9800', 'pause_circle', false, false, 11),
('failed', 'Failed', 'Order failed (payment or processing)', '#9E9E9E', 'error', false, true, 12)
ON CONFLICT (code) DO NOTHING;

-- Seed default status transitions (allowed transitions)
INSERT INTO order_status_transitions (from_status_id, to_status_id, requires_confirmation, confirmation_message)
SELECT 
    (SELECT id FROM order_statuses WHERE code = from_code),
    (SELECT id FROM order_statuses WHERE code = to_code),
    requires_conf,
    conf_msg
FROM (VALUES
    -- From Pending
    ('pending', 'confirmed', false, null),
    ('pending', 'cancelled', true, 'Are you sure you want to cancel this order?'),
    ('pending', 'on_hold', false, null),
    
    -- From Confirmed
    ('confirmed', 'processing', false, null),
    ('confirmed', 'cancelled', true, 'Are you sure you want to cancel this confirmed order?'),
    ('confirmed', 'on_hold', false, null),
    
    -- From Processing
    ('processing', 'packed', false, null),
    ('processing', 'cancelled', true, 'Cancel this order being processed?'),
    ('processing', 'on_hold', false, null),
    
    -- From Packed
    ('packed', 'shipped', false, null),
    ('packed', 'processing', false, null),
    
    -- From Shipped
    ('shipped', 'in_transit', false, null),
    ('shipped', 'delivered', false, null),
    
    -- From In Transit
    ('in_transit', 'out_for_delivery', false, null),
    ('in_transit', 'delivered', false, null),
    
    -- From Out for Delivery
    ('out_for_delivery', 'delivered', false, null),
    
    -- From On Hold
    ('on_hold', 'pending', false, null),
    ('on_hold', 'confirmed', false, null),
    ('on_hold', 'cancelled', true, 'Cancel this order on hold?'),
    
    -- From Delivered (limited transitions)
    ('delivered', 'refunded', true, 'Refund this delivered order?'),
    
    -- From Cancelled (can be reactivated in some cases)
    ('cancelled', 'pending', true, 'Reactivate this cancelled order?')
) AS t(from_code, to_code, requires_conf, conf_msg)
WHERE EXISTS (SELECT 1 FROM order_statuses WHERE code = from_code)
  AND EXISTS (SELECT 1 FROM order_statuses WHERE code = to_code)
ON CONFLICT (from_status_id, to_status_id) DO NOTHING;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_order_statuses_code ON order_statuses(code);
CREATE INDEX IF NOT EXISTS idx_order_statuses_position ON order_statuses(position);
CREATE INDEX IF NOT EXISTS idx_order_status_transitions_from ON order_status_transitions(from_status_id);
CREATE INDEX IF NOT EXISTS idx_order_status_transitions_to ON order_status_transitions(to_status_id);
CREATE INDEX IF NOT EXISTS idx_order_status_history_order ON order_status_history(order_id);
CREATE INDEX IF NOT EXISTS idx_order_status_history_created ON order_status_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_orders_status_id ON orders(status_id);
