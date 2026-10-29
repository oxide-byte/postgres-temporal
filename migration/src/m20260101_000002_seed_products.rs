//! Sample data for the demo.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "
                INSERT INTO product (id) OVERRIDING SYSTEM VALUE VALUES (1), (2), (3);
                SELECT setval(pg_get_serial_sequence('product', 'id'), 3);

                INSERT INTO product_sku (product_id, sku)
                VALUES (1, 'LAP-001'), (2, 'PHN-001'), (3, 'HDP-001');
                INSERT INTO product_name (product_id, name)
                VALUES (1, 'UltraBook 14'), (2, 'Pixel Phone X'), (3, 'NoiseCancel 700');
                INSERT INTO product_brand (product_id, brand)
                VALUES (1, 'Acme'), (2, 'Acme'), (3, 'Acme');
                INSERT INTO product_category (product_id, category)
                VALUES (1, 'laptop'), (2, 'smartphone'), (3, 'headphones');

                INSERT INTO product_price (product_id, valid_at, price)
                VALUES (1, daterange('2026-01-01', '2026-06-01'), 1299.00),
                       (1, daterange('2026-06-01', '2026-11-01'), 1199.00),
                       (1, daterange('2026-11-01', NULL), 1099.00),
                       (2, daterange('2026-01-01', NULL), 899.00),
                       (3, daterange('2026-01-01', '2026-11-25'), 349.00),
                       (3, daterange('2026-11-25', '2026-11-30'), 249.00),
                       (3, daterange('2026-11-30', NULL), 349.00);
                ",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM product_price; DELETE FROM product_category; DELETE FROM product_brand;
                 DELETE FROM product_name; DELETE FROM product_sku; DELETE FROM product;",
            )
            .await?;
        Ok(())
    }
}
