-- Create Customers table
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    phone VARCHAR(50),
    password_hash VARCHAR(255), -- Nullable for guests if we support them, or strictly registered
    billing_address JSONB,
    delivery_address JSONB,
    is_guest BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(store_id, email)
);

-- Add customer_id to shopping_carts
ALTER TABLE shopping_carts 
ADD COLUMN IF NOT EXISTS customer_id UUID REFERENCES customers(id) ON DELETE SET NULL;

-- Add customer_id to orders
ALTER TABLE orders 
ADD COLUMN IF NOT EXISTS customer_id UUID REFERENCES customers(id) ON DELETE SET NULL;

-- Index for customer lookups
CREATE INDEX IF NOT EXISTS idx_customers_store_email ON customers(store_id, email);
CREATE INDEX IF NOT EXISTS idx_shopping_carts_customer ON shopping_carts(customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_customer ON orders(customer_id);
