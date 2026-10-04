use super::*;

mod jobs;

#[test]
fn schema_mismatch_preserves_existing_database() {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch("CREATE TABLE marker(value TEXT); INSERT INTO marker VALUES ('preserved'); PRAGMA user_version=99;").unwrap();
    assert!(Database::from_connection(connection).is_err());
}

#[tokio::test]
async fn schema_and_foreign_keys_are_enabled() {
    Database::memory()
        .execute(|connection| {
            let version: i64 =
                connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
            assert_eq!(version, 7);
            assert!(connection
                .execute("INSERT INTO projects VALUES ('missing','p','project')", [])
                .is_err());
            Ok(())
        })
        .await
        .unwrap();
}

#[test]
fn v5_migration_preserves_store_binding_and_grants_existing_uploads_a_lease() {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    for schema in [
        include_str!("../schema.sql"),
        include_str!("../schema_v2.sql"),
        include_str!("../schema_v3.sql"),
        include_str!("../schema_v4.sql"),
        include_str!("../schema_v5.sql"),
    ] {
        connection.execute_batch(schema).unwrap();
    }
    connection
        .execute_batch(
            "INSERT INTO organizations VALUES ('org','organization',1);
         INSERT INTO projects VALUES ('org','project','project');
         INSERT INTO cloud_blobs VALUES ('org','project','existing',7);",
        )
        .unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let key = [7u8; 32];
    connection
        .execute(
            "INSERT INTO cloud_store_binding VALUES (1,?1,?2)",
            rusqlite::params![id, key.as_slice()],
        )
        .unwrap();
    let before = crate::service::identity::now_seconds();
    super::migration::migrate(&mut connection).unwrap();
    let binding: (String, Vec<u8>) = connection
        .query_row(
            "SELECT store_id,key_fingerprint FROM cloud_store_binding",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(binding, (id, key.to_vec()));
    let expiry: u64 = connection
        .query_row(
            "SELECT expires_at FROM cloud_upload_leases WHERE digest='existing'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(expiry >= before + 86_400);
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM cloud_retention_policies", [], |row| {
                row.get::<_, u64>(0)
            })
            .unwrap(),
        0
    );
    super::migration::migrate(&mut connection).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM cloud_upload_leases", [], |row| row
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
}
