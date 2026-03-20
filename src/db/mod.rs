//! Database layer with SurrealDB

mod schema;
pub mod repository;

pub use repository::*;

use surrealdb::Surreal;
use surrealdb::engine::local::Db;

pub struct Database {
    /// The underlying SurrealDB instance
    pub inner: Surreal<Db>,
}

#[allow(dead_code)]
impl Database {
    pub async fn in_memory() -> crate::NexusResult<Self> {
        let db: Surreal<Db> = Surreal::init();
        db.connect::<surrealdb::engine::local::Mem>(())
            .await
            .map_err(|e| crate::NexusError::Database(e.to_string()))?;

        db.use_ns("nexus_grafo")
            .use_db("knowledge")
            .await
            .map_err(|e| crate::NexusError::Database(e.to_string()))?;

        schema::init_schema(&db).await?;

        Ok(Self { inner: db })
    }

    pub async fn rocksdb(path: impl AsRef<std::path::Path>) -> crate::NexusResult<Self> {
        let db: Surreal<Db> = Surreal::init();
        let path_str = path.as_ref().to_string_lossy().to_string();

        db.connect::<surrealdb::engine::local::RocksDb>(path_str)
            .await
            .map_err(|e| crate::NexusError::Database(e.to_string()))?;

        db.use_ns("nexus_grafo")
            .use_db("knowledge")
            .await
            .map_err(|e| crate::NexusError::Database(e.to_string()))?;

        schema::init_schema(&db).await?;

        Ok(Self { inner: db })
    }

    pub fn blocks(&self) -> BlockRepository<'_> {
        BlockRepository::new(&self.inner)
    }

    pub fn edges(&self) -> EdgeRepository<'_> {
        EdgeRepository::new(&self.inner)
    }

    pub async fn query(&self, sql: &str) -> crate::NexusResult<surrealdb::Response> {
        self.inner
            .query(sql)
            .await
            .map_err(|e| crate::NexusError::Database(e.to_string()))
    }
}
