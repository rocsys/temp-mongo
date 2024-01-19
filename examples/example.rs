use assert2::{assert, let_assert};
use mongodb::bson::{doc, Document};
use temp_mongo::TempMongo;

#[cfg_attr(feature = "tokio-runtime", tokio::main)]
#[cfg_attr(feature = "async-std-runtime", async_std::main)]
async fn main() {
   // Start a new mongo instance and print the path of the temporary state directory.
let_assert!(Ok(mongo) = TempMongo::new().await);
	println!("Temporary directory: {}", mongo.directory().display());

    // Create a "test" database with an "animals" collection.
	let database = mongo.client().database("test");
	let collection = database.collection::<Document>("animals");

    // Insert two documents in the created collection and print their IDs.
	let_assert!(Ok(dog) = collection.insert_one(doc! { "species": "dog", "cute": "yes", "scary": "usually not" }, None).await);
	let_assert!(Ok(t_rex) = collection.insert_one(doc! { "species": "T-Rex", "cute": "maybe", "scary": "yes" }, None).await);

    println!("Inserted \"dog\" with ID: {}", dog.inserted_id.as_object_id().unwrap()); 
	println!("Inserted \"T-Rex\" with ID: {}", t_rex.inserted_id.as_object_id().unwrap()); 

    // Find all documents in the collection and print them.
    let_assert!(Ok(mut documents) = collection.find(None, None).await);

    loop {
        let_assert!(Ok(more) = documents.advance().await);
        if !more {
            break;
        }
        let_assert!(Ok(current) = documents.deserialize_current());
        println!("Document: {:#?}", current);
    }

    // Kill the mongo server and remove the temporary state directory.
	// This is done if the `mongo` object is dropped automatically,
	// but by calling `kill_and_clean()` explicitly you can do better error reporting.
	assert!(let Ok(()) = mongo.kill_and_clean().await);

}


