use super::{Connection, ServiceError, TransactionBehavior};

pub(super) fn migrate(connection: &mut Connection) -> Result<(), ServiceError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // Read under the same lock as DDL, including concurrent first-start migration attempts.
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if !(0..=7).contains(&version) {
        return Err(ServiceError::Configuration);
    }
    if version == 0 {
        transaction.execute_batch(include_str!("schema.sql"))?;
    }
    if version < 2 {
        transaction.execute_batch(include_str!("schema_v2.sql"))?;
    }
    if version < 3 {
        transaction.execute_batch(include_str!("schema_v3.sql"))?;
    }
    if version < 4 {
        transaction.execute_batch(include_str!("schema_v4.sql"))?;
    }
    if version < 5 {
        transaction.execute_batch(include_str!("schema_v5.sql"))?;
    }
    if version < 6 {
        transaction.execute_batch(include_str!("schema_v6.sql"))?;
    }
    if version < 7 {
        transaction.execute_batch(include_str!("schema_v7.sql"))?;
    }
    transaction.commit()?;
    Ok(())
}
