
use assert2::{assert, let_assert};
use temp_mongo::TempMongo;

#[cfg_attr(feature = "tokio-runtime", tokio::main)]
#[cfg_attr(feature = "async-std-runtime", async_std::main)]
async fn main() {
    // Start a new mongo instance and print the path of the temporary state directory.
    let_assert!(Ok(mongo) = TempMongo::new().await);
	println!("Temporary directory: {}", mongo.directory().display());

    // Create documents for seeding
    let documents = vec![
        mongodb::bson::doc! {"name": "Alice", "age": 30},
        mongodb::bson::doc! {"name": "Bob", "age": 25},
    ];

    // Create seeding object, which contains the documents, database_name and collection_name
    let prepared_seed_data = mongo.prepare_seed_document("test_documents", "trex", documents);

    // Load documents into the mongodb database 
    match mongo.load_document(&prepared_seed_data).await {
        Ok(_) => println!("Data seeded successfully."),
        Err(e) => println!("Error seeding data: {:?}", e),
    }

    // Prints the documents for the specific database and collection
    match mongo.print_documents("test_documents", "trex").await {
        Ok(_) => println!("Data succesfully retrieved."),
        Err(e) => println!("Error retrieving data: {:?}", e),
    };

    // Kill the mongo server and remove the temporary state directory.
	// This is done if the `mongo` object is dropped automatically,
	// but by calling `kill_and_clean()` explicitly you can do better error reporting.
	assert!(let Ok(()) = mongo.kill_and_clean().await);
}
