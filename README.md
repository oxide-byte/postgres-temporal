# Postgres Temporal

Demo of PostgreSQL temporal tables: a tech product catalogue whose **price is a temporal field**, shown in a Leptos
screen (pick a product, see how its price evolved).

Docs: https://www.postgresql.org/docs/19/ddl-temporal-tables.html

## Release dates

| Feature                                                                          | PostgreSQL | Released                      |
|----------------------------------------------------------------------------------|------------|-------------------------------|
| Temporal constraints: `WITHOUT OVERLAPS` (PK / UNIQUE) and `PERIOD` (FOREIGN KEY) | 18         | 2025-09-25                    |
| Temporal tables documentation section                                            | 19         | not yet released (RC1 on 2026-10-15) |

This demo runs on PostgreSQL 19 RC1 (`postgres:19rc1-bookworm`), but it only needs the PostgreSQL 18 features.
PostgreSQL 19 does not list `UPDATE/DELETE ... FOR PORTION OF` in its release notes, so changing a price over a
period of time is done by hand (see below).

## How it works

PostgreSQL supports *application time*: a column holding the period during which a row is true in the real world.
System time (when the row was written) is not supported.

### Schema (`infrastructure/init.sql`)

- `product`: the facts that never change (`sku`, `name`, `brand`, `category`).
- `product_price`: one row per period during which a product had a given price.

```sql
CREATE TABLE product_price
(
    product_id BIGINT         NOT NULL REFERENCES product (id),
    valid_at   DATERANGE      NOT NULL,
    price      NUMERIC(12, 2) NOT NULL,
    currency   CHAR(3)        NOT NULL DEFAULT 'EUR',
    PRIMARY KEY (product_id, valid_at WITHOUT OVERLAPS)
);
```

- `valid_at` is a range, written `[from, to)`: the start is included and the end is excluded. An open end means
  "until further notice". Because the end is excluded, a price ending on 2026-06-01 and the next one starting
  on 2026-06-01 do not overlap.
- `PRIMARY KEY (product_id, valid_at WITHOUT OVERLAPS)` means a product may have many rows, but never two prices at the
  same moment. It is backed by a GiST index, which is why the `btree_gist` extension is enabled.
  It also forbids empty ranges.
- `product_current_price` is a view returning each product's price today (`valid_at @> CURRENT_DATE`).

Query a price at any date:

```sql
SELECT price FROM product_price WHERE product_id = 1 AND valid_at @> DATE '2026-07-01';
```

A foreign key with `PERIOD` is only needed when the referenced table is temporal too. `product` is not, so a plain
foreign key is used.

### Changing a price

There is no `FOR PORTION OF`, so close the current period and insert the new one:

```sql
UPDATE product_price SET valid_at = daterange(lower(valid_at), DATE '2026-12-01')
WHERE product_id = 2 AND upper_inf(valid_at);

INSERT INTO product_price (product_id, valid_at, price)
VALUES (2, daterange('2026-12-01', NULL), 849.00);
```

Overlapping periods are rejected by the primary key.

### Related: Anchor Modeling and sixth normal form

[Anchor Modeling](https://www.anchormodeling.com/) is a database modeling technique built for data that changes over
time. It is based on sixth normal form (6NF): a table holds a key plus at most one attribute, so every piece of
information can change on its own timeline.

- **Anchor:** the identity of an entity, with no attributes (here, `product`).
- **Attribute:** one table per property, holding the value and its validity period (here, `product_price`).
  A *historized* attribute is what this demo calls a temporal value: the same key can have several rows, one per
  period, and the value is only true during that period.
- **Tie:** a table that links anchors, also historizable.

This demo is a partial take on that idea. `product_price` is a historized attribute table, kept apart from the
`product` identity, so a price change never touches anything else. `name`, `brand` and `category` stay together in
`product`, so it is not full 6NF. If those had to be versioned too, each would get its own attribute table with its
own `valid_at` and `WITHOUT OVERLAPS` key.

The cost of 6NF is more tables and more joins. The usual remedy is a view that joins the attributes back together,
like `product_current_price` does for the current price.

### Application

Leptos full-stack: axum serves the pages, and server functions query Postgres through
[SeaORM](https://www.sea-ql.org/SeaORM/). The database code only compiles into the server (`ssr` feature), never into
the wasm bundle.

- `list_products` fills the dropdown, using the SeaORM `product` entity (`src/entity.rs`).
- `price_history(product_id)` returns the rows of `product_price`, ordered by start date, shown in the table.
  An open-ended period displays "now".
- `product_price` has no SeaORM entity: SeaORM has no `daterange` type, so `price_history` reads it with a raw SQL
  statement mapped onto `PriceRow`. Dates and prices are cast to text in SQL.

## Run it

```
cargo install --locked cargo-leptos   # once
just db                               # PostgreSQL 19 RC1 on 127.0.0.1:5432, loaded with init.sql
just serve                            # http://127.0.0.1:3000
```

Build and serve with `cargo leptos` (the Justfile does it). `trunk serve` does not work: it builds a browser-only app,
and this project also has a server binary.

The server reads `DATABASE_URL`, defaulting to `postgres://postgres:password@localhost:5432/postgres`.
The database container is started with `--rm`, so its data is reset on every `just db`.
