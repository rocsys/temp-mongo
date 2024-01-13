use assert2::{assert, let_assert};
use mongodb::bson::{doc, Document};
use temp_mongo::TempMongo;
use futures_util::stream::TryStreamExt;

#[cfg_attr(feature = "tokio-runtime", tokio::test)]
#[cfg_attr(feature = "async-std-runtime", async_std::test)]
async fn insert_and_find() {
	let_assert!(Ok(mongo) = TempMongo::new().await);
	let database = mongo.client().database("test");
	let collection = database.collection::<Document>("foo");

	let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
	let_assert!(Some(id) = id.inserted_id.as_object_id());
	let_assert!(Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await);
	assert_eq!(document, doc! { "_id": id, "hello": "world" });

	// Not needed, but shows better errors.
	assert!(let Ok(()) = mongo.kill_and_clean().await);
}

#[tokio::test]
async fn insert_and_find_multiple_instances() {
	let instance_count = 5;

	let handles = (0..instance_count)
		.map(|_| {
			tokio::spawn(async move {
				let_assert!(Ok(mongo) = TempMongo::new().await);
				let database = mongo.client().database("test");
				let collection = database.collection::<Document>("foo");

				let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
				let_assert!(Some(id) = id.inserted_id.as_object_id());
				let_assert!(
					Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await
				);
				assert_eq!(document, doc! { "_id": id, "hello": "world" });
				assert!(let Ok(()) = mongo.kill_and_clean().await);
			})
		})
		.collect::<Vec<_>>();

	for handle in handles {
		let result = handle.await;
		assert!(result.is_ok());
	}
}

#[tokio::test]
async fn insert_and_find_multiple_instances_2() {
	let instance_count = 5;

	let handles = (0..instance_count)
		.map(|_| {
			tokio::spawn(async move {
				let_assert!(Ok(mongo) = TempMongo::new().await);
				let database = mongo.client().database("test");
				let collection = database.collection::<Document>("foo");

				let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
				let_assert!(Some(id) = id.inserted_id.as_object_id());
				let_assert!(
					Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await
				);
				assert_eq!(document, doc! { "_id": id, "hello": "world" });
				assert!(let Ok(()) = mongo.kill_and_clean().await);
			})
		})
		.collect::<Vec<_>>();

	// Await all handles and check for errors.
	for handle in handles {
		let result = handle.await;
		assert!(result.is_ok());
	}
}


#[tokio::test]
async fn seeding_document() {
	let documents = vec![
		mongodb::bson::doc! {"name": "Alice", "age": 30},
		mongodb::bson::doc! {"name": "Bob", "age": 25},
	];

	// Create a new mongo instance and assert it's created successfully
	let mongo = TempMongo::new().await.expect("Failed to create TempMongo instance");

	// Prepare seed document
	let prepared_seed_data = mongo.prepare_seed_document("test_db2", "trex", documents.clone());


	match mongo.seed_document(&prepared_seed_data).await {
		Ok(_) => println!("Data seeded successfully."),
		Err(e) => println!("Error seeding data: {:?}", e),
	}

	// Fetch documents from the database and compare
	let collection: mongodb::Collection<mongodb::bson::Document> = mongo.client().database("test_db2").collection("trex");
	let mut cursor = collection.find(None, None).await.expect("Failed to execute find command");

	// Collect documents from cursor
	let mut fetched_documents = Vec::new();
	while let Some(doc) = cursor.try_next().await.expect("Failed during cursor traversal") {
		fetched_documents.push(doc);
	}

	// Remove '_id' field from the fetched documents
	for doc in &mut fetched_documents {
		doc.remove("_id");
	}

	// Assert that the fetched documents match what was seeded
	assert_eq!(documents, fetched_documents, "The seeded documents do not match the fetched documents");
	assert!(let Ok(()) = mongo.kill_and_clean().await);
}
