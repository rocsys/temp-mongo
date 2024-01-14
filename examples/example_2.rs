use assert2::let_assert;
use mongodb::bson::{doc, Document};
use mongodb::error::Result;
use temp_mongo::TempMongo;

#[tokio::main]
async fn main() -> Result<()> {
    let_assert!(Ok(mongo) = TempMongo::new().await);

    let database = mongo.client().database("test_example_2");
    let collection = database.collection::<Document>("foo");

    let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
    let_assert!(Some(id) = id.inserted_id.as_object_id());
    let_assert!(Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await);
    assert_eq!(document, doc! { "_id": id, "hello": "world" });

    // Clean up the temporary MongoDB instance
    if let Err(e) = mongo.kill_and_clean().await {
        panic!("Failed to clean up the temporary MongoDB instance: {:?}", e);
    }

    // Connect to the MongoDB instance with a new client
    let_assert!(Ok(mongo_2) = TempMongo::new().await);
    let new_client = mongo_2.client();
    // Check if the 'test' database still exists

    // Clean up the second temporary MongoDB instance
    if let Err(e) = mongo_2.kill_and_clean().await {
        panic!(
            "Failed to clean up the second temporary MongoDB instance: {:?}",
            e
        );
    }

    Ok(())
}
