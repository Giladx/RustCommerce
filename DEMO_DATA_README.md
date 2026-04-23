# Demo Data - Categories & Products

This document describes the demo data created for RustCommerce.

## Overview

- **14 Categories** organized in a hierarchical structure
- **25 Products** with realistic pricing and descriptions
- All items linked to the first store in the database

## Categories Structure

### Electronics (4 sub-categories)
- **Computers & Laptops** (3 products)
- **Smartphones & Tablets** (3 products)
- **Tech Accessories** (3 products)

### Clothing & Fashion (2 sub-categories)
- **Men's Clothing** (3 products)
- **Women's Clothing** (2 products)

### Home & Garden (2 sub-categories)
- **Furniture** (3 products)
- **Home Decor** (2 products)

### Books & Media (2 sub-categories)
- **Fiction** (2 products)
- **Non-Fiction** (2 products)

## Products by Category

### Computers & Laptops
1. **MacBook Pro 16" M3 Max** - $2,999.00 (was $3,299.00)
2. **Dell XPS 15 Developer Edition** - $2,499.00 (was $2,799.00)
3. **Lenovo ThinkPad X1 Carbon Gen 11** - $1,799.00

### Smartphones & Tablets
4. **iPhone 15 Pro Max 256GB** - $1,199.00
5. **Samsung Galaxy S24 Ultra** - $1,299.00 (was $1,399.00)
6. **Google Pixel 8 Pro** - $999.00 (was $1,099.00)

### Tech Accessories
7. **USB-C Hub 7-in-1** - $49.99
8. **Wireless Charging Pad 15W** - $29.99
9. **Premium Laptop Sleeve 15"** - $34.99 (was $44.99)

### Men's Clothing
10. **Classic Oxford Shirt - Blue** - $59.00 (was $79.00)
11. **Slim Fit Chinos - Khaki** - $69.00
12. **Lightweight Bomber Jacket** - $129.00 (was $159.00)

### Women's Clothing
13. **Floral Maxi Dress** - $89.00 (was $119.00)
14. **Cashmere Blend Sweater** - $149.00

### Furniture
15. **Ergonomic Office Chair** - $399.00 (was $499.00)
16. **Standing Desk Electric 60"** - $699.00 (was $799.00)
17. **Modern Bookshelf 5-Tier** - $179.00

### Home Decor
18. **Abstract Canvas Print Set** - $129.00 (was $179.00)
19. **Smart LED Floor Lamp** - $89.00 (was $119.00)

### Fiction Books
20. **The Midnight Library** - $16.99
21. **Project Hail Mary** - $18.99 (was $24.99)

### Non-Fiction Books
22. **Atomic Habits** - $19.99
23. **Sapiens: A Brief History** - $21.99 (was $28.00)

## Features Demonstrated

### Pricing Strategy
- **Regular Price**: All products have a base price
- **Compare Price**: 60% of products show a "was" price for promotions
- **Cost Price**: All products include cost data for margin analysis

### Product Descriptions
- Detailed, customer-focused descriptions
- Key features highlighted
- Technical specifications where relevant
- Use cases mentioned

### Price Ranges
- **Budget**: $16.99 - $49.99 (accessories, books)
- **Mid-Range**: $59 - $399 (clothing, furniture accessories)
- **Premium**: $699 - $2,999 (laptops, desks)

## Numeric IDs

All categories and products now have auto-incrementing numeric IDs for easy reference:

- **Categories**: #15 - #29
- **Products**: #11 - #35

You can reference items by their numeric ID in the admin panel (e.g., "Product #23").

## How to Use

### View Products in Admin
1. Navigate to `/admin/products`
2. Products will show with `#ID - Product Name` format
3. Filter by category using the dropdown

### View Categories in Admin
1. Navigate to `/admin/categories`
2. Categories show their product count
3. Edit categories to manage the catalog structure

### Running the Demo Data Script

If you need to reload demo data:

```bash
cd /home/gilad/RustCommerce
export $(grep DATABASE_URL .env | xargs)
psql $DATABASE_URL -f demo_categories_products.sql
```

**Note**: This will add MORE data, not replace existing data. To start fresh, truncate the tables first:

```sql
TRUNCATE products, categories CASCADE;
```

## Next Steps

You can:
1. Add product images to the products
2. Create brands and assign to products
3. Add product attributes (size, color, etc.)
4. Create product combinations/variants
5. Set up discount rules for the products
6. Create coupons for specific categories

Enjoy your fully populated e-commerce catalog! 🎉
