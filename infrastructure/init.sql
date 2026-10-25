-- Required for WITHOUT OVERLAPS on scalar columns (GiST equality on bigint).
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- Atemporal product identity: things that do not change with time.
CREATE TABLE product
(
    id       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    sku      TEXT NOT NULL UNIQUE,
    name     TEXT NOT NULL,
    brand    TEXT NOT NULL,
    category TEXT NOT NULL
);

-- Temporal price: one row per validity period of a product's price.
-- WITHOUT OVERLAPS guarantees a product never has two prices at the same time.
CREATE TABLE product_price
(
    product_id BIGINT         NOT NULL,
    valid_at   DATERANGE      NOT NULL,
    price      NUMERIC(12, 2) NOT NULL CHECK (price >= 0),
    currency   CHAR(3)        NOT NULL DEFAULT 'EUR',
    PRIMARY KEY (product_id, valid_at WITHOUT OVERLAPS),
    FOREIGN KEY (product_id) REFERENCES product (id)
);

-- Sample data
INSERT INTO product (sku, name, brand, category)
VALUES ('LAP-001', 'UltraBook 14', 'Acme', 'laptop'),
       ('PHN-001', 'Pixel Phone X', 'Acme', 'smartphone'),
       ('HDP-001', 'NoiseCancel 700', 'Acme', 'headphones');

INSERT INTO product_price (product_id, valid_at, price)
VALUES (1, daterange('2026-01-01', '2026-06-01'), 1299.00),
       (1, daterange('2026-06-01', '2026-11-01'), 1199.00),
       (1, daterange('2026-11-01', NULL), 1099.00),
       (2, daterange('2026-01-01', NULL), 899.00),
       (3, daterange('2026-01-01', '2026-11-25'), 349.00),
       (3, daterange('2026-11-25', '2026-11-30'), 249.00),
       (3, daterange('2026-11-30', NULL), 349.00);

-- Price of every product as of today.
CREATE VIEW product_current_price AS
SELECT p.id, p.sku, p.name, pp.price, pp.currency, pp.valid_at
FROM product p
         JOIN product_price pp ON pp.product_id = p.id
WHERE pp.valid_at @> CURRENT_DATE;
