// use tikv_client::{BoundRange, RawClient};
// use async_trait::async_trait;

// pub struct KvStorage {
//     client: RawClient,
// }
//
// #[async_trait]
// pub trait KvStore {
//     type Error: Send;
//     type Key: Send;
//     type Value: Send;
//     type KvPair: Send;
//
//     async fn get(&self, key: impl Into<Self::Key> + Send) -> Result<Option<Self::Value>, Self::Error>;
//     async fn set(&self, key: impl Into<Self::Key> + Send, value: impl Into<Self::Value> + Send) -> Result<(), Self::Error>;
//     async fn delete(&self, key: impl Into<Self::Key> + Send) -> Result<(), Self::Error>;
//     async fn scan(&self, range: impl Into<BoundRange> + Send, limit: u32) -> Result<Vec<Self::KvPair>, Self::Error>;
// }
//
//
// pub async fn connect() -> Result<KvStorage, tikv_client::Error> {
//     Ok(KvStorage {
//         client: RawClient::new(vec!["127.0.0.1:2379"]).await?
//     })
// }
//
// #[async_trait]
// impl KvStore for KvStorage {
//     type Error = tikv_client::Error;
//     type Key = tikv_client::Key;
//     type Value = tikv_client::Value;
//     type KvPair = tikv_client::KvPair;
//
//     async fn get(&self, key: impl Into<Self::Key> + Send) -> Result<Option<Self::Value>, Self::Error> {
//         self.client.get(key).await
//     }
//
//     async fn set(&self, key: impl Into<Self::Key> + Send, value: impl Into<Self::Value> + Send) -> Result<(), Self::Error> {
//         self.client.put(key, value).await
//     }
//
//     async fn delete(&self, key: impl Into<Self::Key> + Send) -> Result<(), Self::Error> {
//         self.client.delete(key).await
//     }
//
//     async fn scan(&self, range: impl Into<BoundRange> + Send, limit: u32) -> Result<Vec<Self::KvPair>, Self::Error> {
//         self.client.scan(range, limit).await
//     }
// }

// pub trait KvStoreBatch {
//     type Error;
//
//     fn batch_get(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>, Self::Error>;
//     fn batch_set(&self, pairs: &[(&[u8], &[u8])]) -> Result<(), Self::Error>;
//     fn batch_delete(&self, keys: &[&[u8]]) -> Result<(), Self::Error>;
//     fn batch_scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>;
// }
//
// pub trait KvStoreAdvancedScan {
//     type Error;
//
//     fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, Self::Error>;
//     fn scan_keys(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>, Self::Error>;
// }
