//! Anchor model: `product` is the anchor (identity only), every attribute lives in its own table.
//! `sku`, `name`, `brand` and `category` are static (one row per product). `price` is historized:
//! one row per validity period, enforced by `WITHOUT OVERLAPS`.
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Product {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ProductSku {
    Table,
    ProductId,
    Sku,
}

#[derive(DeriveIden)]
enum ProductName {
    Table,
    Name,
}

#[derive(DeriveIden)]
enum ProductBrand {
    Table,
    Brand,
}

#[derive(DeriveIden)]
enum ProductCategory {
    Table,
    Category,
}

/// A static attribute table: the anchor id is both primary key and foreign key.
fn static_attribute(table: DynIden, attribute: DynIden) -> TableCreateStatement {
    Table::create()
        .table(table)
        .if_not_exists()
        .col(big_integer(ProductSku::ProductId).primary_key())
        .col(text(attribute))
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Anchor.
        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .col(big_integer(Product::Id).auto_increment().primary_key())
                    .to_owned(),
            )
            .await?;

        // Static attributes.
        let attributes: [(DynIden, DynIden); 4] = [
            (ProductSku::Table.into_iden(), ProductSku::Sku.into_iden()),
            (ProductName::Table.into_iden(), ProductName::Name.into_iden()),
            (ProductBrand::Table.into_iden(), ProductBrand::Brand.into_iden()),
            (ProductCategory::Table.into_iden(), ProductCategory::Category.into_iden()),
        ];
        for (table, column) in attributes {
            let mut stmt = static_attribute(table.clone(), column);
            stmt.foreign_key(
                ForeignKey::create()
                    .from_tbl(table.clone())
                    .from_col(ProductSku::ProductId)
                    .to_tbl(Product::Table)
                    .to_col(Product::Id),
            );
            manager.create_table(stmt).await?;
        }
        manager
            .create_index(
                Index::create()
                    .table(ProductSku::Table)
                    .col(ProductSku::Sku)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Historized attribute. SeaORM has no daterange type nor `WITHOUT OVERLAPS`, so raw SQL.
        manager
            .get_connection()
            .execute_unprepared(
                "
                -- Required for WITHOUT OVERLAPS on scalar columns (GiST equality on bigint).
                CREATE EXTENSION IF NOT EXISTS btree_gist;

                -- One row per validity period of a product's price; a product never has two
                -- prices at the same time.
                CREATE TABLE product_price
                (
                    product_id BIGINT         NOT NULL,
                    valid_at   DATERANGE      NOT NULL,
                    price      NUMERIC(12, 2) NOT NULL CHECK (price >= 0),
                    currency   CHAR(3)        NOT NULL DEFAULT 'EUR',
                    PRIMARY KEY (product_id, valid_at WITHOUT OVERLAPS),
                    FOREIGN KEY (product_id) REFERENCES product (id)
                );

                -- Anchor and its attributes joined back together, with today's price.
                CREATE VIEW product_current AS
                SELECT p.id, s.sku, n.name, b.brand, c.category, pp.price, pp.currency, pp.valid_at
                FROM product p
                         JOIN product_sku s ON s.product_id = p.id
                         JOIN product_name n ON n.product_id = p.id
                         JOIN product_brand b ON b.product_id = p.id
                         JOIN product_category c ON c.product_id = p.id
                         LEFT JOIN product_price pp ON pp.product_id = p.id AND pp.valid_at @> CURRENT_DATE;
                ",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP VIEW IF EXISTS product_current; DROP TABLE IF EXISTS product_price;")
            .await?;
        for table in [
            ProductCategory::Table.into_iden(),
            ProductBrand::Table.into_iden(),
            ProductName::Table.into_iden(),
            ProductSku::Table.into_iden(),
            Product::Table.into_iden(),
        ] {
            manager.drop_table(Table::drop().table(table).if_exists().to_owned()).await?;
        }
        Ok(())
    }
}
