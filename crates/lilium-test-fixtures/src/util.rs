// Test-only deterministic name -> UUID mapping. Keeps user-id literals in tests
// readable (e.g. `test_uuid("u1")`) while satisfying the uuid column type after
// the user-chain migration. Not used by production code.

use uuid::Uuid;

/// Deterministically map a human-readable name to a stable `Uuid` via UUIDv5.
/// The same name always yields the same UUID, so a test can seed
/// `seed_test_users(&["user1"])` and later query by `test_uuid("user1")`.
pub fn test_uuid(name: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes())
}
