use assert2::{assert, let_assert};
use mongodb::bson::{doc, Document};
use temp_mongo::TempMongo;
use temp_mongo::SeedData;

#[cfg_attr(feature = "tokio-runtime", tokio::main)]
#[cfg_attr(feature = "async-std-runtime", async_std::main)]
async fn main() {
	let_assert!(Ok(mongo) = TempMongo::new().await); //new mongo instance
	println!("Temporary directory: {}", mongo.directory().display()); //directory().display() --> gets path to temp state directory and displays it

	let database = mongo.client().database("test"); // calls database method on mongo instance and passes in "test" as the database name
	let collection = database.collection::<Document>("animals"); // calls collection method on database and passes in "animals" as the collection name

	let_assert!(Ok(dog) = collection.insert_one(doc! { "species": "dog", "cute": "yes", "scary": "usually not" }, None).await); //inserts a document into the collection
	let_assert!(Ok(t_rex) = collection.insert_one(doc! { "species": "T-Rex", "cute": "maybe", "scary": "yes" }, None).await); //inserts a document into the collection


	println!("Inserted \"dog\" with ID: {}", dog.inserted_id.as_object_id().unwrap()); //prints the inserted document's ID
	println!("Inserted \"T-Rex\" with ID: {}", t_rex.inserted_id.as_object_id().unwrap()); //prints the inserted document's ID

	let_assert!(Ok(mut documents) = collection.find(None, None).await); //finds all documents in the collection
	loop {
		let_assert!(Ok(more) = documents.advance().await);
		if !more {
			break;
		}
		let_assert!(Ok(current) = documents.deserialize_current());
		println!("Document: {:#?}", current);
	}


	// Option 2: Using seeder
	// Creating seeding options for mongodb instance
	let mut seed_data = SeedData {
		database_name: "test_db".to_string(),
		collection_name: "test_collection".to_string(),
		documents: vec![
			mongodb::bson::doc! {"name": "Alice", "age": 30},
			mongodb::bson::doc! {"name": "Bob", "age": 25},
		],
	};

	//seeding data into mongodb instance
	match mongo.seed_data(&seed_data).await {
		Ok(_) => println!("Data seeded successfully."),
		Err(e) => println!("Error seeding data: {:?}", e),
	}

	let path = std::path::Path::new("./spreadsheet.xlsx");

	let excel = SeedData::from_excel(path, "Blad1").unwrap();

	seed_data = SeedData {
		database_name: "test_db2".to_string(),
		collection_name: "test_collection2".to_string(),
		documents: excel,
	};

	//seeding data into mongodb instance
	match mongo.seed_data(&seed_data).await {
		Ok(_) => println!("Data seeded successfully."),
		Err(e) => println!("Error seeding data: {:?}", e),
	}

	// Now print the documents
	match mongo.print_documents("test_db2", "test_collection2").await {
		Ok(_) => println!("Data succesfully retrieved."),
		Err(e) => println!("Error retrieving data: {:?}", e),
	};
	
	assert!(let Ok(()) = mongo.kill_and_clean().await); //kills the server and removes the temp state directory

}
