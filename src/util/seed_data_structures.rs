use serde::Deserialize;
use mongodb::{Client, bson::Document};

/// Data seed options for mongodb instance
/// 
/// The database_name and collection_name are used to specify the database and collection to seed the data into
#[derive(Deserialize)]
pub struct DataSeeder {
    /// The name of the database to seed.
    pub database_name: String,
    /// The name of the collection to seed.
    pub collection_name: String,
    /// The documents to be seeded into the collection.
    pub documents: Vec<Document>,
}

impl DataSeeder {
    /// Creates a new `SeedData` instance with default values.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with empty database name, collection name, and an empty vector of documents.
    pub fn new() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            documents: Vec::new(),
        }
    }

    /// Creates a new `SeedData` instance with specified values.
    ///
    /// # Arguments
    ///
    /// * `database_name` - A string slice representing the name of the database.
    /// * `collection_name` - A string slice representing the name of the collection.
    /// * `documents` - A vector of MongoDB documents to be seeded into the collection.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with the specified database and collection names, and the provided documents.
    pub fn new_in(&self, database_name: &str, collection_name: &str, documents: Vec<Document>) -> Self {
        Self {
            database_name: database_name.to_string(),
            collection_name: collection_name.to_string(),
            documents,
        }
    }

    /// Creates a new `SeedData` instance with specified values using `String` objects.
    ///
    /// # Arguments
    ///
    /// * `database_name` - A `String` reference representing the name of the database.
    /// * `collection_name` - A `String` reference representing the name of the collection.
    /// * `documents` - A vector of MongoDB documents to be seeded into the collection.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with the specified database and collection names, and the provided documents.
    pub fn new_in_with_string(&self, database_name: &String, collection_name: &String, documents: Vec<Document>) -> Self {
        Self {
            database_name: database_name.clone(),
            collection_name: collection_name.clone(),
            documents,
        }
    }
    

    /// Seeds the specified MongoDB collection with the provided documents.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference to the MongoDB client to use for inserting documents.
    ///
    /// # Errors
    ///
    /// Returns an error if any MongoDB operation fails during the seeding process.
    pub async fn seed_document(&self, client: &Client) -> mongodb::error::Result<()> {
        let collection = client.database(&self.database_name).collection(&self.collection_name);
        for document in &self.documents {
            collection.insert_one(document.clone(), None).await?;
        }
        Ok(())
}
}
