//! SeaORM entities for the anchor model. `product` is the anchor; each attribute table is keyed by the
//! anchor id. `product_price` has no entity: its `valid_at` column is a `daterange`, which SeaORM has no
//! type for, so it is read with a raw query.
pub mod product {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "product")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_one = "super::product_sku::Entity")]
        Sku,
        #[sea_orm(has_one = "super::product_name::Entity")]
        Name,
    }

    impl Related<super::product_sku::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Sku.def()
        }
    }

    impl Related<super::product_name::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Name.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod product_sku {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "product_sku")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub product_id: i64,
        pub sku: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::product::Entity",
            from = "Column::ProductId",
            to = "super::product::Column::Id"
        )]
        Product,
    }

    impl Related<super::product::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Product.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod product_name {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "product_name")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub product_id: i64,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::product::Entity",
            from = "Column::ProductId",
            to = "super::product::Column::Id"
        )]
        Product,
    }

    impl Related<super::product::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Product.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
