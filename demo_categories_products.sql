-- Demo Categories and Products for RustCommerce
-- Run this after migrations to populate the database with sample data

DO $$
DECLARE
    v_store_id UUID;
    v_electronics_id UUID;
    v_computers_id UUID;
    v_phones_id UUID;
    v_accessories_id UUID;
    v_clothing_id UUID;
    v_mens_id UUID;
    v_womens_id UUID;
    v_home_id UUID;
    v_furniture_id UUID;
    v_decor_id UUID;
    v_books_id UUID;
    v_fiction_id UUID;
    v_nonfiction_id UUID;
BEGIN
    -- Get first store
    SELECT id INTO v_store_id FROM stores LIMIT 1;
    
    IF v_store_id IS NULL THEN
        RAISE EXCEPTION 'No stores found. Please create a store first.';
    END IF;

    -- ===== CATEGORIES =====
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Electronics', 'electronics', 'Latest electronics and gadgets', true, NOW())
    RETURNING id INTO v_electronics_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Computers & Laptops', 'computers-laptops', 'Desktop computers, laptops, and accessories', true, NOW())
    RETURNING id INTO v_computers_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Smartphones & Tablets', 'smartphones-tablets', 'Mobile devices and tablets', true, NOW())
    RETURNING id INTO v_phones_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Tech Accessories', 'tech-accessories', 'Cables, chargers, cases, and more', true, NOW())
    RETURNING id INTO v_accessories_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Clothing & Fashion', 'clothing-fashion', 'Trendy apparel for everyone', true, NOW())
    RETURNING id INTO v_clothing_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Mens Clothing', 'mens-clothing', 'Shirts, pants, jackets for men', true, NOW())
    RETURNING id INTO v_mens_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Womens Clothing', 'womens-clothing', 'Dresses, tops, accessories for women', true, NOW())
    RETURNING id INTO v_womens_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Home & Garden', 'home-garden', 'Everything for your home', true, NOW())
    RETURNING id INTO v_home_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Furniture', 'furniture', 'Chairs, tables, sofas, and more', true, NOW())
    RETURNING id INTO v_furniture_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Home Decor', 'home-decor', 'Wall art, lamps, decorative items', true, NOW())
    RETURNING id INTO v_decor_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Books & Media', 'books-media', 'Books, magazines, and digital media', true, NOW())
    RETURNING id INTO v_books_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Fiction', 'fiction', 'Novels, short stories, literature', true, NOW())
    RETURNING id INTO v_fiction_id;
    
    INSERT INTO categories (id, store_id, name, slug, description, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, 'Non-Fiction', 'non-fiction', 'Biographies, history, self-help', true, NOW())
    RETURNING id INTO v_nonfiction_id;
    
    -- ===== PRODUCTS =====
    
    -- Laptops
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_computers_id, 'MacBook Pro 16" M3 Max', 'macbook-pro-16-m3-max', 
            'Powerful laptop with M3 Max chip, 32GB RAM, 1TB SSD. Perfect for professionals and creators.', 
            2999.00, 3299.00, 2400.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_computers_id, 'Dell XPS 15 Developer Edition', 'dell-xps-15-developer', 
            'High-performance laptop with Intel i9, 64GB RAM, Ubuntu pre-installed. Developers dream machine.', 
            2499.00, 2799.00, 2000.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_computers_id, 'Lenovo ThinkPad X1 Carbon Gen 11', 'lenovo-thinkpad-x1-carbon', 
            'Ultra-light business laptop with legendary ThinkPad keyboard. 16GB RAM, 512GB SSD.', 
            1799.00, NULL, 1400.00, true, NOW());
    
    -- Smartphones
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_phones_id, 'iPhone 15 Pro Max 256GB', 'iphone-15-pro-max-256gb', 
            'Latest iPhone with titanium design, A17 Pro chip, and advanced camera system.', 
            1199.00, NULL, 900.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_phones_id, 'Samsung Galaxy S24 Ultra', 'samsung-galaxy-s24-ultra', 
            'Flagship Android phone with S Pen, 200MP camera, and AI features.', 
            1299.00, 1399.00, 1000.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_phones_id, 'Google Pixel 8 Pro', 'google-pixel-8-pro', 
            'Pure Android experience with best-in-class camera and 7 years of updates.', 
            999.00, 1099.00, 750.00, true, NOW());
    
    -- Accessories
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_accessories_id, 'USB-C Hub 7-in-1', 'usbc-hub-7in1', 
            'Essential hub with HDMI, USB 3.0, SD card reader, and 100W PD charging.', 
            49.99, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_accessories_id, 'Wireless Charging Pad 15W', 'wireless-charging-pad-15w', 
            'Fast wireless charger compatible with all Qi devices. Sleek aluminum design.', 
            29.99, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_accessories_id, 'Premium Laptop Sleeve 15"', 'laptop-sleeve-15-premium', 
            'Water-resistant neoprene sleeve with extra pocket. Fits most 15" laptops.', 
            34.99, 44.99, true, NOW());
    
    -- Mens Clothing
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_mens_id, 'Classic Oxford Shirt - Blue', 'classic-oxford-shirt-blue', 
            '100% cotton Oxford shirt. Timeless design, comfortable fit. Machine washable.', 
            59.00, 79.00, 30.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_mens_id, 'Slim Fit Chinos - Khaki', 'slim-fit-chinos-khaki', 
            'Versatile chinos with modern slim fit. Perfect for casual or business casual.', 
            69.00, 35.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_mens_id, 'Lightweight Bomber Jacket', 'lightweight-bomber-jacket', 
            'Modern bomber jacket in nylon. Water-resistant, multiple pockets.', 
            129.00, 159.00, 70.00, true, NOW());
    
    -- Womens Clothing
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_womens_id, 'Floral Maxi Dress', 'floral-maxi-dress', 
            'Flowing maxi dress with beautiful floral print. Perfect for summer events.', 
            89.00, 119.00, 45.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_womens_id, 'Cashmere Blend Sweater', 'cashmere-blend-sweater', 
            'Luxuriously soft sweater with 30% cashmere. Available in 5 colors.', 
            149.00, 80.00, true, NOW());
    
    -- Furniture
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_furniture_id, 'Ergonomic Office Chair', 'ergonomic-office-chair', 
            'Premium mesh office chair with lumbar support and adjustable armrests.', 
            399.00, 499.00, 250.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_furniture_id, 'Standing Desk Electric 60"', 'standing-desk-electric-60', 
            'Height-adjustable desk with memory presets. Solid wood top, electric motor.', 
            699.00, 799.00, 450.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_furniture_id, 'Modern Bookshelf 5-Tier', 'modern-bookshelf-5tier', 
            'Industrial style bookshelf with metal frame. Easy assembly.', 
            179.00, 90.00, true, NOW());
    
    -- Home Decor
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_decor_id, 'Abstract Canvas Print Set', 'abstract-canvas-print-set', 
            'Set of 3 modern abstract prints. Ready to hang, 16x20 inches each.', 
            129.00, 179.00, 60.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_decor_id, 'Smart LED Floor Lamp', 'smart-led-floor-lamp', 
            'App-controlled floor lamp with 16 million colors and voice control.', 
            89.00, 119.00, 50.00, true, NOW());
    
    -- Fiction Books
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_fiction_id, 'The Midnight Library', 'the-midnight-library', 
            'Bestselling novel by Matt Haig. Between life and death, there is a library.', 
            16.99, 8.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_fiction_id, 'Project Hail Mary', 'project-hail-mary', 
            'Sci-fi thriller by Andy Weir, author of The Martian. Space adventure at its finest.', 
            18.99, 24.99, 10.00, true, NOW());
    
    -- Non-Fiction Books
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_nonfiction_id, 'Atomic Habits', 'atomic-habits', 
            'Tiny changes, remarkable results. #1 bestseller on habit formation by James Clear.', 
            19.99, 11.00, true, NOW());
    
    INSERT INTO products (id, store_id, category_id, title, slug, description, price, compare_price, cost_price, is_active, created_at)
    VALUES (gen_random_uuid(), v_store_id, v_nonfiction_id, 'Sapiens: A Brief History', 'sapiens-brief-history', 
            'From animals to gods - how Homo sapiens conquered the world. By Yuval Noah Harari.', 
            21.99, 28.00, 12.00, true, NOW());
    
    RAISE NOTICE 'Successfully created 14 categories and 25 products!';
END $$;
