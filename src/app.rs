use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sea_orm::FromQueryResult))]
pub struct Product {
    pub id: i64,
    pub sku: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sea_orm::FromQueryResult))]
pub struct PriceRow {
    pub valid_from: String,
    /// `None` while the price is still open-ended.
    pub valid_to: Option<String>,
    pub price: String,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sea_orm::FromQueryResult))]
pub struct PriceQuote {
    pub price: String,
    pub currency: String,
}

#[server]
pub async fn list_products() -> Result<Vec<Product>, ServerFnError> {
    use crate::entity::{product, product_name, product_sku};
    use sea_orm::{
        DatabaseConnection, EntityTrait, JoinType, QueryOrder, QuerySelect, RelationTrait,
    };

    let db = expect_context::<DatabaseConnection>();
    product::Entity::find()
        .select_only()
        .column(product::Column::Id)
        .column(product_sku::Column::Sku)
        .column(product_name::Column::Name)
        .join(JoinType::InnerJoin, product::Relation::Sku.def())
        .join(JoinType::InnerJoin, product::Relation::Name.def())
        .order_by_asc(product_name::Column::Name)
        .into_model::<Product>()
        .all(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn price_history(product_id: i64) -> Result<Vec<PriceRow>, ServerFnError> {
    use sea_orm::{DatabaseConnection, DbBackend, FromQueryResult, Statement};

    let db = expect_context::<DatabaseConnection>();
    PriceRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT lower(valid_at)::text AS valid_from,
                upper(valid_at)::text AS valid_to,
                price::text           AS price,
                currency::text        AS currency
         FROM product_price
         WHERE product_id = $1
         ORDER BY lower(valid_at)",
        [product_id.into()],
    ))
    .all(&db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn price_on_date(
    product_id: i64,
    date: String,
) -> Result<Option<PriceQuote>, ServerFnError> {
    let db = expect_context::<sea_orm::DatabaseConnection>();
    query_price_on_date(&db, product_id, &date)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
async fn query_price_on_date<C>(
    db: &C,
    product_id: i64,
    date: &str,
) -> Result<Option<PriceQuote>, sea_orm::DbErr>
where
    C: sea_orm::ConnectionTrait + Send + Sync,
{
    use sea_orm::{DbBackend, FromQueryResult, Statement};

    PriceQuote::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT price::text AS price, currency::text AS currency
         FROM product_price
         WHERE product_id = $1 AND valid_at @> $2::date
         LIMIT 1",
        [product_id.into(), date.into()],
    ))
    .one(db)
    .await
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <link rel="stylesheet" id="leptos" href="/pkg/postgres-temporal.css" />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let products = Resource::new(|| (), |_| list_products());
    let (selected, set_selected) = signal(None::<i64>);
    let (selected_date, set_selected_date) = signal(String::new());
    let history = Resource::new(
        move || selected.get(),
        |id| async move {
            match id {
                Some(id) => (Some(id), price_history(id).await.map(Some)),
                None => (None, Ok(None)),
            }
        },
    );

    view! {
        <Title text="Price evolution" />
        <main class="mx-auto max-w-3xl p-6 font-sans text-slate-900">
            <h1 class="mb-6 text-2xl font-bold">"Price evolution"</h1>

            <Suspense fallback=|| view! { <p>"Loading products…"</p> }>
                {move || {
                    products
                        .get()
                        .map(|res| match res {
                            Err(e) => view! { <p class="text-red-600">{e.to_string()}</p> }.into_any(),
                            Ok(items) => {
                                view! {
                                    <label class="block">
                                        <span class="mb-1 block text-sm font-medium">"Product"</span>
                                        <select
                                            class="w-full rounded border border-slate-300 p-2"
                                            on:change=move |ev| {
                                                set_selected.set(event_target_value(&ev).parse().ok())
                                            }
                                        >
                                            <option value="">"Select a product…"</option>
                                            {items
                                                .into_iter()
                                                .map(|p| {
                                                    view! {
                                                        <option value=p.id.to_string()>
                                                            {format!("{} ({})", p.name, p.sku)}
                                                        </option>
                                                    }
                                                })
                                                .collect_view()}
                                        </select>
                                    </label>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>

            <Suspense fallback=|| view! { <p class="mt-6">"Loading prices…"</p> }>
                {move || {
                    history
                        .get()
                        .map(|(history_product_id, res)| {
                            if history_product_id != selected.get() {
                                return view! { <p class="mt-6">"Loading prices…"</p> }.into_any();
                            }

                            match res {
                                Err(e) => view! { <p class="mt-6 text-red-600">{e.to_string()}</p> }.into_any(),
                                Ok(None) => ().into_any(),
                                Ok(Some(rows)) => view! {
                                    <PriceHistory
                                        rows
                                        product_id=history_product_id.unwrap()
                                        selected_date
                                        set_selected_date
                                    />
                                }
                                    .into_any(),
                            }
                        })
                }}
            </Suspense>
        </main>
    }
}

#[component]
fn PriceHistory(
    rows: Vec<PriceRow>,
    product_id: i64,
    selected_date: ReadSignal<String>,
    set_selected_date: WriteSignal<String>,
) -> impl IntoView {
    let price = Resource::new(
        move || (product_id, selected_date.get()),
        |(id, date)| async move {
            let result = if date.is_empty() {
                Ok(None)
            } else {
                price_on_date(id, date.clone()).await
            };
            ((id, date), result)
        },
    );

    view! {
        <div class="mt-6 grid grid-cols-1 items-start gap-6 md:grid-cols-[minmax(0,1fr)_16rem]">
            <div>
                <PriceTable rows />
            </div>
            <section class="rounded border border-slate-200 p-4">
                <label class="block">
                    <span class="mb-1 block text-sm font-medium">"Date"</span>
                    <input
                        type="date"
                        class="w-full rounded border border-slate-300 p-2"
                        prop:value=move || selected_date.get()
                        on:input=move |ev| set_selected_date.set(event_target_value(&ev))
                    />
                </label>
                <div class="mt-4">
                    <Suspense fallback=|| view! {
                        <p class="text-sm text-slate-600">"Loading price…"</p>
                    }>
                        {move || {
                            let date = selected_date.get();
                            if date.is_empty() {
                            view! {
                                <p class="text-sm text-slate-600">"Choose a date to see the price."</p>
                            }
                                .into_any()
                            } else {
                                match price.get() {
                                    Some(((result_product_id, result_date), result))
                                        if result_product_id == product_id && result_date == date =>
                                    {
                                        match result {
                                            Ok(Some(quote)) => view! {
                                                <p class="text-sm">
                                                    <span class="mr-2 font-medium">"Price"</span>
                                                    <span class="tabular-nums">
                                                        {format!("{} {}", quote.price, quote.currency.trim())}
                                                    </span>
                                                </p>
                                            }
                                                .into_any(),
                                            Ok(None) => view! {
                                                <p class="text-sm text-slate-600">"No price for this date."</p>
                                            }
                                                .into_any(),
                                            Err(e) => view! {
                                                <p class="text-sm text-red-600">{e.to_string()}</p>
                                            }
                                                .into_any(),
                                        }
                                    }
                                    _ => view! {
                                        <p class="text-sm text-slate-600">"Loading price…"</p>
                                    }
                                        .into_any(),
                                }
                            }
                        }}
                    </Suspense>
                </div>
            </section>
        </div>
    }
}

#[component]
fn PriceTable(rows: Vec<PriceRow>) -> impl IntoView {
    view! {
        <table class="w-full border-collapse text-left">
            <thead>
                <tr class="border-b-2 border-slate-300">
                    <th class="p-2">"From"</th>
                    <th class="p-2">"To"</th>
                    <th class="p-2 text-right">"Price"</th>
                </tr>
            </thead>
            <tbody>
                {rows
                    .into_iter()
                    .map(|r| {
                        view! {
                            <tr class="border-b border-slate-200">
                                <td class="p-2">{r.valid_from}</td>
                                <td class="p-2">{r.valid_to.unwrap_or_else(|| "now".into())}</td>
                                <td class="p-2 text-right tabular-nums">
                                    {format!("{} {}", r.price, r.currency.trim())}
                                </td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod database_tests {
    use super::query_price_on_date;
    use sea_orm::{ConnectionTrait, Database, TransactionTrait};

    #[tokio::test]
    async fn selected_price_comes_from_postgres_range_lookup() {
        let has_database_url = std::env::var_os("DATABASE_URL").is_some();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".into());
        let db = match Database::connect(&database_url).await {
            Ok(db) => db,
            Err(error) if !has_database_url => {
                eprintln!(
                    "Skipping PostgreSQL lookup test because no database is available: {error}"
                );
                return;
            }
            Err(error) => panic!("Could not connect to DATABASE_URL: {error}"),
        };
        let transaction = db.begin().await.expect("begin test transaction");

        transaction
            .execute_unprepared(
                "CREATE TEMP TABLE product_price (
                     product_id BIGINT NOT NULL,
                     valid_at DATERANGE NOT NULL,
                     price NUMERIC(12, 2) NOT NULL,
                     currency CHAR(3) NOT NULL
                 ) ON COMMIT DROP;
                 INSERT INTO product_price (product_id, valid_at, price, currency)
                 VALUES (1, daterange('2026-01-01', '2026-02-01', '[)'), 10.00, 'EUR'),
                        (1, daterange('2026-02-01', NULL, '[)'), 20.00, 'EUR'),
                        (2, daterange('2026-01-01', '2026-02-01', '[)'), 30.00, 'USD'),
                        (2, daterange('2026-03-01', '2026-04-01', '[)'), 40.00, 'USD');",
            )
            .await
            .expect("create fixture data");

        let start = query_price_on_date(&transaction, 1, "2026-01-01")
            .await
            .expect("query period start");
        assert_eq!(start.map(|quote| quote.price), Some("10.00".into()));

        let end = query_price_on_date(&transaction, 1, "2026-02-01")
            .await
            .expect("query exclusive end and next start");
        assert_eq!(end.map(|quote| quote.price), Some("20.00".into()));

        let open_ended = query_price_on_date(&transaction, 1, "9999-12-31")
            .await
            .expect("query open-ended period");
        assert_eq!(open_ended.map(|quote| quote.price), Some("20.00".into()));

        let changed_product = query_price_on_date(&transaction, 2, "2026-01-15")
            .await
            .expect("query another product");
        assert_eq!(
            changed_product.map(|quote| quote.price),
            Some("30.00".into())
        );

        let gap = query_price_on_date(&transaction, 2, "2026-02-15")
            .await
            .expect("query gap");
        assert!(gap.is_none());

        let missing_product = query_price_on_date(&transaction, 3, "2026-01-15")
            .await
            .expect("query missing product");
        assert!(missing_product.is_none());

        assert!(
            query_price_on_date(&transaction, 1, "2026-02-30")
                .await
                .is_err()
        );
        transaction
            .rollback()
            .await
            .expect("rollback test transaction");
    }
}
