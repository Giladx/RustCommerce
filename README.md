# RustCommerce

**RustCommerce** is a high-performance, headless e-commerce backend built with **Rust**, **Axum**, and **PostgreSQL**. It is designed to be fast, type-safe, and scalable, providing a robust foundation for modern e-commerce applications.

## 🚀 Features

### Core Commerce
*   **Multi-Store Support:** Manage multiple storefronts from a single instance.
*   **Product Catalog:**
    *   Products with rich descriptions, pricing, and active status.
    *   **Categories:** Hierarchical organization.
    *   **Brands:** Manage product manufacturers/brands.
    *   **Attributes & Features:** Dynamic product specifications (e.g., Color: Red, CPU: M3).
*   **Shopping Cart:** Persistent and guest carts with session management.
*   **Checkout & Orders:**
    *   Full order lifecycle management.
    *   Configurable order statuses and transitions.
    *   Customer management (Guest & Registered).
    *   Document generation (Invoices, Delivery Slips, Credit Slips).

### Admin Interface
*   **Dashboard:** Server-side rendered admin panel built with **Tailwind CSS**.
*   **Authentication:** Secure login/logout for administrators.
*   **Management:** Full CRUD interface for all catalog and order resources.
*   **API Access:** Built-in API token generator and global security settings.
*   **Documentation:** Integrated interactive API docs with `curl` generator.

### Technical Highlights
*   **Backend:** Rust + Axum (Async, multi-threaded).
*   **Database:** PostgreSQL with `sqlx` (Compile-time checked queries).
*   **Security:**
    *   Optional global API authentication (Bearer / Basic).
    *   Signed cookie sessions for admin.
*   **Architecture:** Clean Domain-Driven Design (DDD) inspired structure.

## 🛠️ Tech Stack

*   **Language:** Rust (2021 Edition)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
*   **Database:** PostgreSQL
*   **ORM / Query Builder:** [SQLx](https://github.com/launchbadge/sqlx)
*   **Serialization:** Serde
*   **Admin UI:** Server-Side HTML + Tailwind CSS (via CDN)

## 🏁 Getting Started

### Prerequisites

*   [Rust & Cargo](https://rustup.rs/) (Latest Stable)
*   PostgreSQL (v14+)
*   `sqlx-cli` (for migrations)
    ```bash
    cargo install sqlx-cli
    ```

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Giladx/RustCommerce.git
    cd RustCommerce
    ```

2.  **Configure Environment:**
    Create a `.env` file in the root directory:
    ```env
    DATABASE_URL=postgres://user:password@localhost:5432/commerce
    RUST_LOG=info
    ```

3.  **Setup Database:**
    Create the database and apply migrations:
    ```bash
    sqlx database create
    sqlx migrate run
    ```

4.  **Run the Server:**
    ```bash
    cargo run
    ```
    The server will start at `http://127.0.0.1:3000`.

## 📖 Usage

### Admin Dashboard
Access the admin panel at **[http://localhost:3000/admin](http://localhost:3000/admin)**.

*   **Default Credentials:**
    *   Username: `admin`
    *   Password: `admin`

### API Documentation
Comprehensive API documentation is available directly within the admin dashboard:
*   Go to **Admin > API Access > API Docs**
*   Or visit: `http://localhost:3000/admin/docs`

The docs include schema details and ready-to-paste `curl` commands for all endpoints.

### API Authentication
By default, the API might be open or secured based on your `system_settings`.
To secure the API:
1.  Log in to the Admin Dashboard.
2.  Navigate to **API Access**.
3.  Enable **"Enforce API Authentication"**.
4.  Generate an API Token.
5.  Use the token in your requests:
    ```bash
    curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:3000/api/products
    ```

## 📂 Project Structure

```
src/
├── admin/          # Admin UI handlers and templates
├── api/            # REST API routes and DTOs
│   ├── repository/ # Database access layer
│   └── routes/     # Route definitions
├── domain/         # Core business logic and structs
├── infra/          # Infrastructure (DB pool, etc.)
├── mw/             # Middleware (Auth, Logging)
└── main.rs         # Application entry point
```

## 🤝 Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## 📄 License

This project is open-source.
