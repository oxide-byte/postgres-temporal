#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use migration::{Migrator, MigratorTrait};
    use postgres_temporal::app::{App, shell};
    use sea_orm::{ConnectOptions, Database};

    let conf = get_configuration(None).expect("failed to read leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let listener = match bind_server_listener(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "Failed to bind the HTTP server to {addr}: {error}\
                 \nIf the address is already in use, inspect its owner or set LEPTOS_SITE_ADDR to a free address."
            );
            return Err(error.into());
        }
    };

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".into());
    let mut options = ConnectOptions::new(database_url);
    options.max_connections(5);
    let db = Database::connect(options)
        .await
        .expect("failed to connect to Postgres");
    Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

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
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(feature = "ssr")]
async fn bind_server_listener(
    addr: std::net::SocketAddr,
) -> std::io::Result<tokio::net::TcpListener> {
    tokio::net::TcpListener::bind(addr).await
}

#[cfg(not(feature = "ssr"))]
fn main() {}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::bind_server_listener;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn binds_to_an_available_ephemeral_port() {
        let listener = bind_server_listener("127.0.0.1:0".parse().unwrap())
            .await
            .expect("bind an available port");

        assert_ne!(listener.local_addr().unwrap().port(), 0);
    }

    #[tokio::test]
    async fn occupied_port_returns_address_in_use_error() {
        let occupied = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("reserve an ephemeral port");
        let addr = occupied.local_addr().unwrap();

        let error = bind_server_listener(addr)
            .await
            .expect_err("binding an occupied port should fail");

        assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
        assert!(error.to_string().to_lowercase().contains("address"));
    }
}
