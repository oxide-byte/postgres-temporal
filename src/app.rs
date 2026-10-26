use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[server]
pub async fn list_products() -> Result<Vec<Product>, ServerFnError> {
    use crate::entity::product;
    use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

    let db = expect_context::<DatabaseConnection>();
    let rows = product::Entity::find()
        .order_by_asc(product::Column::Name)
        .all(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|p| Product { id: p.id, sku: p.sku, name: p.name })
        .collect())
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
    let history = Resource::new(
        move || selected.get(),
        |id| async move {
            match id {
                Some(id) => price_history(id).await.map(Some),
                None => Ok(None),
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
                        .map(|res| match res {
                            Err(e) => view! { <p class="mt-6 text-red-600">{e.to_string()}</p> }.into_any(),
                            Ok(None) => ().into_any(),
                            Ok(Some(rows)) => view! { <PriceTable rows /> }.into_any(),
                        })
                }}
            </Suspense>
        </main>
    }
}

#[component]
fn PriceTable(rows: Vec<PriceRow>) -> impl IntoView {
    view! {
        <table class="mt-6 w-full border-collapse text-left">
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
