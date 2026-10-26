#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use postgres_temporal::app::{App, shell};
    use sea_orm::{ConnectOptions, Database};

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".into());
    let mut options = ConnectOptions::new(database_url);
    options.max_connections(5);
    let db = Database::connect(options)
        .await
        .expect("failed to connect to Postgres");

    let conf = get_configuration(None).expect("failed to read leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let db = db.clone();
                move || provide_context(db.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    println!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
