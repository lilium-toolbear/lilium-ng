# lilium-database Agent SOP

## Layer Boundary

- `lilium-database` owns database runtime, transactions, connection owners, and raw connection infrastructure.
- SeaORM table metadata lives in `lilium-models`, not `lilium-database`.
- Service crates should use `lilium-models` for entity modules and this crate for connection ownership, transactions, and dedicated PostgreSQL connection types.
- Service crates may depend directly on `sea-orm` for query-builder traits and derive macros; do not add re-export shims here solely to satisfy a macro path.
- Do not add SeaORM re-exports for service convenience. New service code imports SeaORM APIs directly from `sea_orm`.
- Public infrastructure APIs must have doc comments that state when to use them, how callers should use them, and which similar-looking paths are not appropriate.

## Entity Rules

- Add a SeaORM entity in `crates/lilium-models` for a real table before writing service-layer CRUD against that table.
- Register every new entity in the matching domain module, such as `dzmm/mod.rs`, `ingestion/mod.rs`, or `wallet/mod.rs`, and in the crate root when callers need that module directly.
- Match the live schema types, primary keys, nullable fields, and table names. Use `crates/lilium-database/testdata/live_schema_bootstrap/0001_live_schema.sql` as schema evidence.
- Do not create duplicate table-shaped structs in services when a SeaORM entity can model the table.
- Use `FromQueryResult` structs for fixed projections only. They are read models, not a second table model layer.

## Raw Connection Rules

- `LISTEN`/`NOTIFY` and advisory locks require dedicated PostgreSQL connections because their state is physical-connection scoped.
- Do not route listener or lock ownership through normal ORM pool CRUD helpers.
- Test fixture database create/drop/reset SQL is database administration code and may use SQLx/raw SQL.
- Migrations and live-schema bootstrap SQL are schema sources and may remain raw SQL.
