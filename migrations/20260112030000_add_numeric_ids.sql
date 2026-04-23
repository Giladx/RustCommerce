-- Add numeric IDs to all tables for admin UI display
-- Using SERIAL for auto-incrementing integer IDs

-- Stores
ALTER TABLE stores ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_stores_numeric_id ON stores(numeric_id);

-- Categories
ALTER TABLE categories ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_numeric_id ON categories(numeric_id);

-- Brands
ALTER TABLE brands ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_brands_numeric_id ON brands(numeric_id);

-- Brand Categories
ALTER TABLE brand_categories ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_brand_categories_numeric_id ON brand_categories(numeric_id);

-- Products
ALTER TABLE products ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_products_numeric_id ON products(numeric_id);

-- Product Attributes
ALTER TABLE product_attributes ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_attributes_numeric_id ON product_attributes(numeric_id);

-- Product Features
ALTER TABLE product_features ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_features_numeric_id ON product_features(numeric_id);

-- Product Images
ALTER TABLE product_images ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_images_numeric_id ON product_images(numeric_id);

-- Product Combinations
ALTER TABLE product_combinations ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_combinations_numeric_id ON product_combinations(numeric_id);

-- Product Combination Attributes
ALTER TABLE product_combination_attributes ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_combination_attributes_numeric_id ON product_combination_attributes(numeric_id);

-- Customers
ALTER TABLE customers ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_customers_numeric_id ON customers(numeric_id);

-- Orders
ALTER TABLE orders ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_orders_numeric_id ON orders(numeric_id);

-- Order Items
ALTER TABLE order_items ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_items_numeric_id ON order_items(numeric_id);

-- Order Statuses
ALTER TABLE order_statuses ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_statuses_numeric_id ON order_statuses(numeric_id);

-- Order Status History
ALTER TABLE order_status_history ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_status_history_numeric_id ON order_status_history(numeric_id);

-- Order Status Transitions
ALTER TABLE order_status_transitions ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_status_transitions_numeric_id ON order_status_transitions(numeric_id);

-- Invoices
ALTER TABLE invoices ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_numeric_id ON invoices(numeric_id);

-- Credit Slips
ALTER TABLE credit_slips ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_credit_slips_numeric_id ON credit_slips(numeric_id);

-- Credit Slip Items
ALTER TABLE credit_slip_items ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_credit_slip_items_numeric_id ON credit_slip_items(numeric_id);

-- Delivery Slips
ALTER TABLE delivery_slips ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_delivery_slips_numeric_id ON delivery_slips(numeric_id);

-- Shopping Carts
ALTER TABLE shopping_carts ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_shopping_carts_numeric_id ON shopping_carts(numeric_id);

-- Cart Items
ALTER TABLE cart_items ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_cart_items_numeric_id ON cart_items(numeric_id);

-- Gift Cards
ALTER TABLE gift_cards ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_gift_cards_numeric_id ON gift_cards(numeric_id);

-- Gift Card Transactions
ALTER TABLE gift_card_transactions ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_gift_card_transactions_numeric_id ON gift_card_transactions(numeric_id);

-- Discount Rules
ALTER TABLE discount_rules ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_discount_rules_numeric_id ON discount_rules(numeric_id);

-- Discount Conditions
ALTER TABLE discount_conditions ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_discount_conditions_numeric_id ON discount_conditions(numeric_id);

-- Discount Actions
ALTER TABLE discount_actions ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_discount_actions_numeric_id ON discount_actions(numeric_id);

-- Coupons
ALTER TABLE coupons ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_coupons_numeric_id ON coupons(numeric_id);

-- Coupon Usage
ALTER TABLE coupon_usage ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_coupon_usage_numeric_id ON coupon_usage(numeric_id);

-- Chat Messages
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_chat_messages_numeric_id ON chat_messages(numeric_id);

-- Order Messages
ALTER TABLE order_messages ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_messages_numeric_id ON order_messages(numeric_id);

-- Merchandise Returns
ALTER TABLE merchandise_returns ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_merchandise_returns_numeric_id ON merchandise_returns(numeric_id);

-- API Tokens
ALTER TABLE api_tokens ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_tokens_numeric_id ON api_tokens(numeric_id);

-- System Settings
ALTER TABLE system_settings ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_system_settings_numeric_id ON system_settings(numeric_id);
