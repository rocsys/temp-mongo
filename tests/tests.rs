use assert2::{assert, let_assert};
use mongodb::bson::{doc, Document};
use temp_mongo::TempMongo;

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
