//! Easy temporary MongoDB instance for unit tests.
//!
//! Use the [`TempMongo`] struct to get a [`mongodb::Client`] that is connected to a temporary MongoDB instance.
//! All state of the spawned MongoDB instance is stored in a temporary directory, which will be cleaned up automatically (unless disabled).
//!
//! On Unix platforms, the client is connected over a Unix socket.
//! Windows support is planned by picking a free TCP port on the loopback adapter.
//!
//! # Example
//! ```
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! use temp_mongo::TempMongo;
//! use mongodb::bson::doc;
//!
//! let mongo = TempMongo::new().await?;
//! println!("Using temporary directory: {}", mongo.directory().display());
//!
//! let client = mongo.client();
//! let collection = client.database("test").collection("animals");
//!
//! collection.insert_one(doc! { "species": "dog", "cute": "yes", "scary": "usually not" }, None).await?;
//! collection.insert_one(doc! { "species": "T-Rex", "cute": "maybe", "scary": "yes" }, None).await?;
//! # Ok(())
//! # }

#![warn(missing_docs)]

mod error;
mod temp_mongo;
mod util;

pub use error::Error;
pub use temp_mongo::TempMongo;
pub use temp_mongo::TempMongoBuilder;
