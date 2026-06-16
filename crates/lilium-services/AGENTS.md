# lilium-services Agent SOP

## Module Naming

- Module names in this crate use the domain noun directly, for example `user`, `message`, `account`, and `notification`.
- Do not add `_service` suffixes inside `lilium-services`; the crate name already states the layer.
- Test module names should describe behavior or scope directly. Avoid `*_service_integration` unless the suffix is part of an external protocol name.

## SQL And ORM Rules

- Service-layer database access is ORM-first. Start every CRUD, filter, sort, pagination, aggregate, upsert, and projection change with SeaORM query builders.
- Do not add raw `SELECT`, raw `INSERT`, raw `UPDATE`, or ad-hoc row mapping when SeaORM can express the operation.
- Use `Expr::cust_with_values` for PostgreSQL-specific predicates that require parameters, such as tsvector search or `ILIKE` expressions. Never interpolate user input into custom SQL strings.
- Use `Expr::cust` only for localized, parameter-free SQL fragments inside a SeaORM query builder, such as a column expression or a PostgreSQL catalog predicate.
- Fixed read models must use `#[derive(FromQueryResult)]` with `.select_only()`, `.column_as()`, and `.into_model::<T>()`. Do not hand-roll row getter macros for fixed projections.
- Aggregates must use `.select_only()`, `.column_as()`, and `.into_tuple()` or a `FromQueryResult` model.
- Batch inserts must use `insert_many`. Use `on_conflict`, `on_conflict_do_nothing`, `exec_with_returning_many`, or `exec_without_returning` as required by behavior.
- Upserts must use SeaORM `OnConflict` builders. Preserve existing conflict semantics exactly, including reset-to-null behavior.
- JSONB updates must use SeaORM update builders with `.col_expr()` and a minimal PostgreSQL expression. Do not keep a whole raw `UPDATE` just because one column needs JSONB expression logic.
- Raw SQL is allowed only for connection-level PostgreSQL primitives and catalog checks that are not table CRUD: advisory locks, `pg_locks`, `LISTEN`/`NOTIFY`, and dedicated connection health probes.
- Connection-level primitives must not be hidden behind normal pooled ORM CRUD. They need a dedicated connection owner because lock/listen state is bound to a physical PostgreSQL connection.
- If raw SQL remains in service code, add a short comment explaining why SeaORM cannot represent that operation and keep the raw fragment as narrow as possible.

## Database Context Rules

- Service functions should accept `&impl ConnectionTrait` or a generic `C: ConnectionTrait`. Do not introduce new `DbSession` or SQLx session paths in services.
- Service crates may depend directly on `sea-orm` for query-builder traits, derive macros, and SeaQuery expressions.
- Use `lilium-database` for entity modules, connection ownership, transaction helpers, and dedicated PostgreSQL connection types.
- Do not mix ORM and raw connection ownership in the same helper unless the helper is explicitly about a connection-level PostgreSQL primitive.

## Review Checklist

- Run `rg -n "Statement::from_sql_and_values|from_raw_sql|find_by_statement|query_all_raw!|row_get!|sqlx::query" crates/lilium-services/src -S` before claiming the service raw-SQL migration is complete.
- Classify each remaining match as a connection-level primitive or a concrete SeaORM migration target.
- Run focused service tests for touched modules, then `cargo check -p lilium-services --all-targets`.
