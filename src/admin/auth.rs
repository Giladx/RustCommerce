use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

pub const ADMIN_SESSION_COOKIE: &str = "admin_session";

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

pub async fn login_page() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Login - RustCommerce Admin</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    colors: {
                        dark: {
                            bg: '#0f172a',
                            panel: '#1e293b',
                            border: '#334155'
                        }
                    }
                }
            }
        }
    </script>
</head>
<body class="bg-gray-50 dark:bg-dark-bg text-gray-900 dark:text-gray-100 min-h-screen flex items-center justify-center p-4">
    <div class="w-full max-w-md bg-white dark:bg-dark-panel rounded-2xl shadow-xl border border-gray-200 dark:border-dark-border overflow-hidden">
        <div class="p-8">
            <div class="text-center mb-8">
                <h1 class="text-2xl font-bold mb-2">RustCommerce</h1>
                <p class="text-gray-500 dark:text-gray-400">Sign in to the admin dashboard</p>
            </div>
            
            <form action="/admin/login" method="POST" class="space-y-6">
                <div>
                    <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">Username</label>
                    <input type="text" name="username" required 
                        class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all" />
                </div>
                
                <div>
                    <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">Password</label>
                    <input type="password" name="password" required 
                        class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-dark-border bg-white dark:bg-slate-900 focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all" />
                </div>
                
                <button type="submit" 
                    class="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2.5 rounded-lg transition-colors">
                    Sign In
                </button>
            </form>
        </div>
        <div class="px-8 py-4 bg-gray-50 dark:bg-slate-800/50 border-t border-gray-200 dark:border-dark-border text-center text-xs text-gray-500 dark:text-gray-400">
            Default credentials: admin / admin
        </div>
    </div>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
            document.documentElement.classList.add('dark');
        }
    </script>
</body>
</html>"#;

    axum::response::Html(html)
}

pub async fn login_handler(
    jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    if payload.username == "admin" && payload.password == "admin" {
        let cookie = Cookie::build((ADMIN_SESSION_COOKIE, "true"))
            .path("/")
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .build();

        (jar.add(cookie), Redirect::to("/admin"))
    } else {
        (jar, Redirect::to("/admin/login?error=invalid"))
    }
}

pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    (
        jar.remove(Cookie::from(ADMIN_SESSION_COOKIE)),
        Redirect::to("/admin/login"),
    )
}

pub async fn admin_auth_middleware(
    jar: CookieJar,
    req: Request,
    next: Next,
) -> Response {
    if let Some(cookie) = jar.get(ADMIN_SESSION_COOKIE) {
        if cookie.value() == "true" {
            return next.run(req).await;
        }
    }

    Redirect::to("/admin/login").into_response()
}
