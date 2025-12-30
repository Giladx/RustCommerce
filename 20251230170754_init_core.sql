-- =========================
-- RustCommerce core schema
-- =========================

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =========================
-- STORES (tenants)
-- =========================
CREATE TABLE stores (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    default_currency CHAR(3) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =========================
-- PRODUCTS
-- =========================
CREATE TABLE products (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id    UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_products_store ON products(store_id);

-- =========================
-- PRODUCT VARIANTS
-- =========================
CREATE TABLE product_variants (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku         TEXT,
    price       NUMERIC(14,4) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_variants_product ON product_variants(product_id);

-- =========================
-- PAGES (page builder root)
-- =========================
CREATE TABLE pages (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id    UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    slug        TEXT NOT NULL,
    title       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (store_id, slug)
);

-- =========================
-- PAGE BLOCKS (Elementor-like)
-- =========================
CREATE TABLE page_blocks (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    page_id     UUID NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    block_type  TEXT NOT NULL,
    position    INTEGER NOT NULL,
    props       JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_blocks_page ON page_blocks(page_id);
CREATE INDEX idx_blocks_props ON page_blocks USING GIN (props);

-- =========================
-- ORDERS
-- =========================
CREATE TABLE orders (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id    UUID NOT NULL REFERENCES stores(id),
    status      TEXT NOT NULL,
    total       NUMERIC(14,4) NOT NULL,
    currency    CHAR(3) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_orders_store ON orders(store_id);
