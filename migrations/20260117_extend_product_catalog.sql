-- Add is_active to categories
ALTER TABLE categories ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Create brands table
CREATE TABLE IF NOT EXISTS brands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    logo_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add brand_id to products
ALTER TABLE products ADD COLUMN IF NOT EXISTS brand_id UUID REFERENCES brands(id) ON DELETE SET NULL;

-- Add pricing fields to products
ALTER TABLE products ADD COLUMN IF NOT EXISTS price DECIMAL(10, 2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS compare_price DECIMAL(10, 2);
ALTER TABLE products ADD COLUMN IF NOT EXISTS cost_price DECIMAL(10, 2);

-- Create product_images table
CREATE TABLE IF NOT EXISTS product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    alt_text VARCHAR(255),
    position INTEGER NOT NULL DEFAULT 0,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create product_combinations table (for variants like size/color)
CREATE TABLE IF NOT EXISTS product_combinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(255),
    quantity INTEGER NOT NULL DEFAULT 0,
    price DECIMAL(10, 2),
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create product_combination_attributes (links combinations to specific attribute values)
CREATE TABLE IF NOT EXISTS product_combination_attributes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    combination_id UUID NOT NULL REFERENCES product_combinations(id) ON DELETE CASCADE,
    attribute_id UUID NOT NULL REFERENCES product_attributes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(combination_id, attribute_id)
);

-- Create brand_categories junction table
CREATE TABLE IF NOT EXISTS brand_categories (
    brand_id UUID NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (brand_id, category_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_brands_slug ON brands(slug);
CREATE INDEX IF NOT EXISTS idx_brands_active ON brands(is_active);
CREATE INDEX IF NOT EXISTS idx_products_brand ON products(brand_id);
CREATE INDEX IF NOT EXISTS idx_product_images_product ON product_images(product_id);
CREATE INDEX IF NOT EXISTS idx_product_images_primary ON product_images(product_id, is_primary);
CREATE INDEX IF NOT EXISTS idx_product_combinations_product ON product_combinations(product_id);
CREATE INDEX IF NOT EXISTS idx_product_combinations_default ON product_combinations(product_id, is_default);
CREATE INDEX IF NOT EXISTS idx_categories_active ON categories(is_active);

-- Seed some demo brands
INSERT INTO brands (name, slug, description, is_active, position) VALUES
('Nike', 'nike', 'Athletic footwear and apparel', true, 1),
('Adidas', 'adidas', 'Sports and lifestyle brand', true, 2),
('Apple', 'apple', 'Consumer electronics', true, 3),
('Samsung', 'samsung', 'Electronics and technology', true, 4),
('Sony', 'sony', 'Electronics and entertainment', true, 5)
ON CONFLICT (slug) DO NOTHING;
