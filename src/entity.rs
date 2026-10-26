//! SeaORM entities. `product_price` has no entity: its `valid_at` column is a
//! `daterange`, which SeaORM has no type for, so it is read with a raw query.
pub mod product {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "product")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub sku: String,
        pub name: String,
        pub brand: String,
        pub category: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
