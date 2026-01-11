-- Demo Data for RustCommerce
-- Run this to populate the database with sample data

-- More Products
INSERT INTO products (id, store_id, category_id, title, description, slug, is_active) VALUES
('11111111-1111-1111-1111-111111111111', '00000000-0000-0000-0000-000000000001', NULL, 'Wireless Bluetooth Headphones', 'Premium noise-canceling headphones with 30-hour battery life', 'wireless-bluetooth-headphones', true),
('22222222-2222-2222-2222-222222222222', '00000000-0000-0000-0000-000000000001', NULL, 'Smart Fitness Watch', 'Track your health with heart rate monitor, GPS, and sleep tracking', 'smart-fitness-watch', true),
('33333333-3333-3333-3333-333333333333', '00000000-0000-0000-0000-000000000001', NULL, 'Portable Power Bank 20000mAh', 'Fast charging power bank with dual USB ports', 'portable-power-bank-20000mah', true),
('44444444-4444-4444-4444-444444444444', '00000000-0000-0000-0000-000000000001', NULL, 'USB-C Mechanical Keyboard', 'RGB backlit mechanical keyboard with brown switches', 'usb-c-mechanical-keyboard', true),
('55555555-5555-5555-5555-555555555555', '00000000-0000-0000-0000-000000000001', NULL, '4K Webcam', 'Professional webcam with auto-focus and dual microphones', '4k-webcam', true),
('66666666-6666-6666-6666-666666666666', '00000000-0000-0000-0000-000000000001', NULL, 'Wireless Gaming Mouse', 'High-precision 16000 DPI gaming mouse with 7 programmable buttons', 'wireless-gaming-mouse', true),
('77777777-7777-7777-7777-777777777777', '00000000-0000-0000-0000-000000000001', NULL, 'Laptop Stand Aluminum', 'Ergonomic aluminum laptop stand with cooling design', 'laptop-stand-aluminum', true),
('88888888-8888-8888-8888-888888888888', '00000000-0000-0000-0000-000000000001', NULL, 'Phone Tripod with Remote', 'Flexible phone tripod with Bluetooth remote control', 'phone-tripod-with-remote', true)
ON CONFLICT (id) DO NOTHING;

-- More Customers
INSERT INTO customers (store_id, email, full_name, phone, is_guest) VALUES
('00000000-0000-0000-0000-000000000001', 'john.smith@example.com', 'John Smith', '+1-555-0101', false),
('00000000-0000-0000-0000-000000000001', 'sarah.jones@example.com', 'Sarah Jones', '+1-555-0102', false),
('00000000-0000-0000-0000-000000000001', 'mike.wilson@example.com', 'Mike Wilson', '+1-555-0103', false),
('00000000-0000-0000-0000-000000000001', 'emma.brown@example.com', 'Emma Brown', '+1-555-0104', false),
('00000000-0000-0000-0000-000000000001', 'david.lee@example.com', 'David Lee', '+1-555-0105', false),
('00000000-0000-0000-0000-000000000001', 'lisa.garcia@example.com', 'Lisa Garcia', '+1-555-0106', false),
('00000000-0000-0000-0000-000000000001', 'james.taylor@example.com', 'James Taylor', '+1-555-0107', false),
('00000000-0000-0000-0000-000000000001', 'anna.martinez@example.com', 'Anna Martinez', '+1-555-0108', false),
('00000000-0000-0000-0000-000000000001', 'robert.anderson@example.com', 'Robert Anderson', '+1-555-0109', false),
('00000000-0000-0000-0000-000000000001', 'guest123@temp.com', 'Guest User', NULL, true)
ON CONFLICT (email, store_id) DO NOTHING;

-- More Orders
INSERT INTO orders (order_number, store_id, customer_email, customer_name, customer_phone, status, payment_status, currency, subtotal, tax_amount, shipping_amount, discount_amount, total_amount, notes) VALUES
('ORD-DEMO001', '00000000-0000-0000-0000-000000000001', 'john.smith@example.com', 'John Smith', '+1-555-0101', 'Pending', 'unpaid', 'USD', 149.99, 12.75, 9.99, 0.00, 172.73, 'First demo order'),
('ORD-DEMO002', '00000000-0000-0000-0000-000000000001', 'sarah.jones@example.com', 'Sarah Jones', '+1-555-0102', 'Processing', 'paid', 'USD', 299.99, 25.50, 0.00, 30.00, 295.49, 'Express shipping'),
('ORD-DEMO003', '00000000-0000-0000-0000-000000000001', 'mike.wilson@example.com', 'Mike Wilson', '+1-555-0103', 'Shipped', 'paid', 'USD', 89.99, 7.65, 9.99, 0.00, 107.63, 'Standard shipping'),
('ORD-DEMO004', '00000000-0000-0000-0000-000000000001', 'emma.brown@example.com', 'Emma Brown', '+1-555-0104', 'Delivered', 'paid', 'USD', 199.99, 17.00, 9.99, 20.00, 206.98, 'Delivered successfully'),
('ORD-DEMO005', '00000000-0000-0000-0000-000000000001', 'david.lee@example.com', 'David Lee', '+1-555-0105', 'Cancelled', 'refunded', 'USD', 79.99, 6.80, 9.99, 0.00, 96.78, 'Customer requested cancellation'),
('ORD-DEMO006', '00000000-0000-0000-0000-000000000001', 'lisa.garcia@example.com', 'Lisa Garcia', '+1-555-0106', 'Processing', 'paid', 'USD', 349.99, 29.75, 0.00, 50.00, 329.74, 'VIP customer discount'),
('ORD-DEMO007', '00000000-0000-0000-0000-000000000001', 'james.taylor@example.com', 'James Taylor', '+1-555-0107', 'Pending', 'pending', 'USD', 129.99, 11.05, 9.99, 0.00, 151.03, 'Awaiting payment'),
('ORD-DEMO008', '00000000-0000-0000-0000-000000000001', 'anna.martinez@example.com', 'Anna Martinez', '+1-555-0108', 'Shipped', 'paid', 'USD', 249.99, 21.25, 9.99, 25.00, 256.23, 'Birthday gift order')
ON CONFLICT (order_number) DO NOTHING;

-- More Gift Cards
INSERT INTO gift_cards (store_id, code, initial_amount, current_balance, currency, recipient_email, recipient_name, status, message) VALUES
('00000000-0000-0000-0000-000000000001', 'GIFT-WELCOME50', 50.00, 50.00, 'USD', 'john.smith@example.com', 'John Smith', 'active', 'Welcome gift card'),
('00000000-0000-0000-0000-000000000001', 'GIFT-BIRTHDAY100', 100.00, 75.50, 'USD', 'sarah.jones@example.com', 'Sarah Jones', 'active', 'Birthday gift - partially used'),
('00000000-0000-0000-0000-000000000001', 'GIFT-HOLIDAY200', 200.00, 200.00, 'USD', 'mike.wilson@example.com', 'Mike Wilson', 'active', 'Holiday season gift card'),
('00000000-0000-0000-0000-000000000001', 'GIFT-THANKS25', 25.00, 0.00, 'USD', 'emma.brown@example.com', 'Emma Brown', 'redeemed', 'Thank you gift - fully redeemed'),
('00000000-0000-0000-0000-000000000001', 'GIFT-PROMO75', 75.00, 75.00, 'USD', NULL, NULL, 'active', 'Promotional gift card'),
('00000000-0000-0000-0000-000000000001', 'GIFT-VIP500', 500.00, 350.00, 'USD', 'lisa.garcia@example.com', 'Lisa Garcia', 'active', 'VIP customer appreciation'),
('00000000-0000-0000-0000-000000000001', 'GIFT-XMAS150', 150.00, 150.00, 'USD', 'james.taylor@example.com', 'James Taylor', 'active', 'Christmas gift card'),
('00000000-0000-0000-0000-000000000001', 'GIFT-REWARD30', 30.00, 30.00, 'USD', 'anna.martinez@example.com', 'Anna Martinez', 'active', 'Loyalty reward')
ON CONFLICT (code) DO NOTHING;

-- More Invoices
INSERT INTO invoices (order_id, store_id, invoice_number, customer_email, customer_name, subtotal, total_amount, currency, status, notes)
SELECT o.id, o.store_id, 
       CONCAT('INV-', SUBSTRING(MD5(o.order_number) FROM 1 FOR 8)), 
       o.customer_email, o.customer_name,
       o.subtotal, o.total_amount, o.currency, 
       CASE 
         WHEN o.payment_status = 'paid' THEN 'paid'
         WHEN o.status = 'Cancelled' THEN 'cancelled'
         ELSE 'draft'
       END,
       CONCAT('Invoice for ', o.order_number)
FROM orders o
WHERE o.order_number LIKE 'ORD-DEMO%'
ON CONFLICT (invoice_number) DO NOTHING;

-- Credit Slips
INSERT INTO credit_slips (order_id, store_id, slip_number, refund_amount, currency, reason, status, notes)
SELECT o.id, o.store_id,
       CONCAT('CS-', SUBSTRING(MD5(o.order_number) FROM 1 FOR 8)),
       o.total_amount * 0.5,
       o.currency,
       'Partial refund requested',
       'pending',
       CONCAT('Credit slip for ', o.order_number)
FROM orders o
WHERE o.status = 'Cancelled' AND o.order_number LIKE 'ORD-DEMO%'
LIMIT 3
ON CONFLICT (slip_number) DO NOTHING;

-- Delivery Slips
INSERT INTO delivery_slips (order_id, store_id, slip_number, tracking_number, carrier, shipping_address, status, shipped_at, notes)
SELECT o.id, o.store_id,
       CONCAT('DS-', SUBSTRING(MD5(o.order_number) FROM 1 FOR 8)),
       CONCAT('TRACK-', SUBSTRING(MD5(RANDOM()::text) FROM 1 FOR 12)),
       CASE 
         WHEN RANDOM() < 0.5 THEN 'UPS'
         ELSE 'FedEx'
       END,
       COALESCE(o.shipping_address, '{}'::jsonb),
       CASE 
         WHEN o.status = 'Delivered' THEN 'delivered'
         WHEN o.status = 'Shipped' THEN 'in_transit'
         ELSE 'pending'
       END,
       CASE 
         WHEN o.status IN ('Shipped', 'Delivered') THEN NOW() - INTERVAL '2 days'
         ELSE NULL
       END,
       CONCAT('Delivery slip for ', o.order_number)
FROM orders o
WHERE o.status IN ('Shipped', 'Delivered') AND o.order_number LIKE 'ORD-DEMO%'
ON CONFLICT (slip_number) DO NOTHING;

-- Shopping Carts (abandoned and active)
INSERT INTO shopping_carts (store_id, session_id, customer_email, customer_name, status, currency, subtotal, tax_amount, discount_amount, total_amount, notes, expires_at)
VALUES
('00000000-0000-0000-0000-000000000001', 'sess_' || MD5(RANDOM()::text), 'cart1@example.com', 'Cart User 1', 'active', 'USD', 199.99, 17.00, 0.00, 216.99, 'Active cart', NOW() + INTERVAL '7 days'),
('00000000-0000-0000-0000-000000000001', 'sess_' || MD5(RANDOM()::text), 'cart2@example.com', 'Cart User 2', 'abandoned', 'USD', 89.99, 7.65, 0.00, 97.64, 'Abandoned cart', NOW() - INTERVAL '2 days'),
('00000000-0000-0000-0000-000000000001', 'sess_' || MD5(RANDOM()::text), NULL, NULL, 'active', 'USD', 149.99, 12.75, 15.00, 147.74, 'Guest cart with discount', NOW() + INTERVAL '7 days'),
('00000000-0000-0000-0000-000000000001', 'sess_' || MD5(RANDOM()::text), 'cart4@example.com', 'Cart User 4', 'converted', 'USD', 299.99, 25.50, 30.00, 295.49, 'Converted to order', NOW() - INTERVAL '5 days')
ON CONFLICT DO NOTHING;

SELECT 'Demo data inserted successfully!' as status;
