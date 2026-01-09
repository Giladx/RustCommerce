-- Add migration script here
CREATE TABLE stores (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT,
    currency VARCHAR(3) NOT NULL,
    timezone TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE products (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id),
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);