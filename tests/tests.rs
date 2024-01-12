use assert2::{assert, let_assert};
use mongodb::bson::{doc, Document};
use temp_mongo::TempMongo;
use std::sync::atomic::{AtomicUsize, Ordering};


#[cfg_attr(feature = "tokio-runtime", tokio::test)]
#[cfg_attr(feature = "async-std-runtime", async_std::test)]
async fn insert_and_find() {
	let_assert!(Ok(mongo) = TempMongo::new().await);
	let database = mongo.client().database("test");
	let collection = database.collection::<Document>("foo");

	let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
	let_assert!(Some(id) = id.inserted_id.as_object_id());
	let_assert!(Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await);
	assert!(document == doc! { "_id": id, "hello": "world" });



	// Not needed, but shows better errors.
	assert!(let Ok(()) = mongo.kill_and_clean().await);
}




#[tokio::test]
async fn insert_and_find_multiple_instances() {
	let instance_count = 5; // The number of TempMongo instances you want to test concurrently

	// Atomic counter for generating unique IDs
	static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

	let handles = (0..instance_count)
		.map(|_| {
			let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst); // Get a unique ID for this thread
			tokio::spawn(async move  {
				println!("Starting thread with ID: {}", thread_id); // Print the thread ID

				let_assert!(Ok(mongo) = TempMongo::new().await);
				let database = mongo.client().database("test");
				let collection = database.collection::<Document>("foo");

				let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
				let_assert!(Some(id) = id.inserted_id.as_object_id());
				let_assert!(Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await);
				assert!(document == doc! { "_id": id, "hello": "world" });

				// Not needed, but shows better errors.
				assert!(let Ok(()) = mongo.kill_and_clean().await);

				// Print a message when the thread is about to finish
				println!("Thread with ID: {} is finishing", thread_id);
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
async fn insert_and_find_multiple_instances_2() {
	let instance_count = 5; // The number of TempMongo instances you want to test concurrently

	// Atomic counter for generating unique IDs
	static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

	let handles = (0..instance_count)
		.map(|_| {
			let thread_id = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst); // Get a unique ID for this thread
			tokio::spawn(async move  {
				println!("Starting thread with ID: {}", thread_id); // Print the thread ID

				let_assert!(Ok(mongo) = TempMongo::new().await);
				let database = mongo.client().database("test");
				let collection = database.collection::<Document>("foo");

				let_assert!(Ok(id) = collection.insert_one(doc! { "hello": "world" }, None).await);
				let_assert!(Some(id) = id.inserted_id.as_object_id());
				let_assert!(Ok(Some(document)) = collection.find_one(doc! { "_id": id }, None).await);
				assert!(document == doc! { "_id": id, "hello": "world" });

				// Not needed, but shows better errors.
				assert!(let Ok(()) = mongo.kill_and_clean().await);

				// Print a message when the thread is about to finish
				println!("Thread with ID: {} is finishing", thread_id);
			})
		})
		.collect::<Vec<_>>();

	// Await all handles and check for errors.
	for handle in handles {
		let result = handle.await;
		assert!(result.is_ok());
	}
}
