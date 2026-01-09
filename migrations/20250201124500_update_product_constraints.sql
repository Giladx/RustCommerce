DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'products' AND constraint_name = 'products_store_id_fkey'
    ) THEN
        ALTER TABLE products DROP CONSTRAINT products_store_id_fkey;
    END IF;

    ALTER TABLE products
        ADD CONSTRAINT products_store_id_fkey
        FOREIGN KEY (store_id)
        REFERENCES stores(id)
        ON DELETE CASCADE;
EXCEPTION
    WHEN undefined_table THEN
        -- tables not created yet; skip
        NULL;
END $$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name = 'products' AND constraint_name = 'products_category_id_fkey'
    ) THEN
        ALTER TABLE products DROP CONSTRAINT products_category_id_fkey;
    END IF;

    ALTER TABLE products
        ADD CONSTRAINT products_category_id_fkey
        FOREIGN KEY (category_id)
        REFERENCES categories(id)
        ON DELETE SET NULL;
EXCEPTION
    WHEN undefined_table THEN
        NULL;
END $$;
