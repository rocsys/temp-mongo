
use assert2::{assert, let_assert};
use temp_mongo::TempMongo;

#[cfg_attr(feature = "tokio-runtime", tokio::main)]
#[cfg_attr(feature = "async-std-runtime", async_std::main)]
async fn main() {
    let_assert!(Ok(mongo) = TempMongo::new().await); //new mongo instance
    println!("Temporary directory: {}", mongo.directory().display()); //directory().display() --> gets path to temp state directory and displays it
    
    let documents = vec![
        mongodb::bson::doc! {"name": "Alice", "age": 30},
        mongodb::bson::doc! {"name": "Bob", "age": 25},
    ];

    let prepared_seed_data = mongo.prepare_seed_document("test_example_1_excel", "trex", documents);

    match mongo.seed_document(&prepared_seed_data).await {
        Ok(_) => println!("Data seeded successfully."),
        Err(e) => println!("Error seeding data: {:?}", e),
    }

    // Advanced printing
    match mongo.print_documents("test_example_1_excel", "trex").await {
        Ok(_) => println!("Data succesfully retrieved."),
        Err(e) => println!("Error retrieving data: {:?}", e),
    };

    assert!(let Ok(()) = mongo.kill_and_clean().await); //kills the server and removes the temp state directory
}
