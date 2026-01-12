-- Discount Rules (Catalog Rules & Cart Rules)
CREATE TABLE IF NOT EXISTS discount_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    rule_type TEXT NOT NULL, -- 'catalog' or 'cart'
    discount_type TEXT NOT NULL, -- 'percentage', 'fixed_amount', 'buy_x_get_y', 'free_shipping'
    discount_value DECIMAL(12, 2) NOT NULL, -- percentage (0-100) or fixed amount
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INT NOT NULL DEFAULT 0, -- lower = higher priority
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    uses_per_customer INT, -- NULL = unlimited
    total_usage_limit INT, -- NULL = unlimited
    current_usage_count INT NOT NULL DEFAULT 0,
    stop_rules_processing BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Discount Conditions (AND logic within a rule)
CREATE TABLE IF NOT EXISTS discount_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES discount_rules(id) ON DELETE CASCADE,
    condition_type TEXT NOT NULL, 
    -- Types: 'subtotal_min', 'subtotal_max', 'quantity_min', 'quantity_max',
    --        'product_in_cart', 'product_category', 'customer_group', 'first_order',
    --        'shipping_country', 'payment_method', 'cart_attribute'
    operator TEXT NOT NULL, -- 'equals', 'not_equals', 'greater_than', 'less_than', 'contains', 'in_list'
    condition_value TEXT NOT NULL, -- JSON for complex conditions
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Discount Actions (Buy X Get Y, etc.)
CREATE TABLE IF NOT EXISTS discount_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES discount_rules(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    -- Types: 'cart_percentage', 'cart_fixed', 'product_percentage', 'product_fixed',
    --        'buy_x_get_y', 'cheapest_free', 'free_shipping'
    action_value DECIMAL(12, 2), -- discount amount/percentage
    max_discount_amount DECIMAL(12, 2), -- cap for percentage discounts
    apply_to TEXT, -- 'cart', 'product', 'shipping', 'cheapest', 'category'
    target_products TEXT, -- JSON array of product IDs
    target_categories TEXT, -- JSON array of category IDs
    buy_quantity INT, -- for buy_x_get_y
    get_quantity INT, -- for buy_x_get_y
    get_discount_percentage DECIMAL(5, 2), -- for buy_x_get_y (e.g., 100 = free)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Coupons / Vouchers
CREATE TABLE IF NOT EXISTS coupons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    code TEXT NOT NULL UNIQUE,
    description TEXT,
    discount_type TEXT NOT NULL, -- 'percentage', 'fixed_cart', 'fixed_product'
    discount_value DECIMAL(12, 2) NOT NULL,
    min_purchase_amount DECIMAL(12, 2),
    max_discount_amount DECIMAL(12, 2), -- cap for percentage
    is_active BOOLEAN NOT NULL DEFAULT true,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    usage_limit_per_coupon INT, -- total uses
    usage_limit_per_customer INT, -- per customer
    current_usage_count INT NOT NULL DEFAULT 0,
    free_shipping BOOLEAN NOT NULL DEFAULT false,
    exclude_sale_items BOOLEAN NOT NULL DEFAULT false,
    individual_use BOOLEAN NOT NULL DEFAULT false, -- cannot combine with other coupons
    allowed_emails TEXT, -- JSON array, NULL = everyone
    excluded_product_ids TEXT, -- JSON array
    allowed_product_ids TEXT, -- JSON array, NULL = all
    excluded_category_ids TEXT, -- JSON array
    allowed_category_ids TEXT, -- JSON array, NULL = all
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Coupon Usage Tracking
CREATE TABLE IF NOT EXISTS coupon_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coupon_id UUID NOT NULL REFERENCES coupons(id) ON DELETE CASCADE,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    customer_email TEXT NOT NULL,
    discount_amount DECIMAL(12, 2) NOT NULL,
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_discount_rules_store ON discount_rules(store_id);
CREATE INDEX idx_discount_rules_active ON discount_rules(is_active, start_date, end_date);
CREATE INDEX idx_discount_conditions_rule ON discount_conditions(rule_id);
CREATE INDEX idx_discount_actions_rule ON discount_actions(rule_id);
CREATE INDEX idx_coupons_store ON coupons(store_id);
CREATE INDEX idx_coupons_code ON coupons(code);
CREATE INDEX idx_coupons_active ON coupons(is_active, start_date, end_date);
CREATE INDEX idx_coupon_usage_coupon ON coupon_usage(coupon_id);
CREATE INDEX idx_coupon_usage_customer ON coupon_usage(customer_email);
