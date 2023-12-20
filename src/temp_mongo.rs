use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::util::SeedData;
use futures_util::stream::TryStreamExt;

use crate::error::ErrorInner;
use crate::util::{KillOnDrop, TempDir};
use crate::Error;
use std::process::Command;

/// A temporary MongoDB instance.
///
/// All state of the MongoDB instance is stored in a temporary directory.
/// Unless disabled, the temporary directory is deleted when this object is dropped.
pub struct TempMongo {
	tempdir: TempDir,
	socket_path: PathBuf,
	log_path: PathBuf,
	client: mongodb::Client,
	server: KillOnDrop,
}

impl std::fmt::Debug for TempMongo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TempMongo")
			.field("tempdir", &self.tempdir.path())
			.field("socket_path", &self.socket_path())
			.field("log_path", &self.log_path())
			.field("server_pid", &self.server.id())
			.finish_non_exhaustive()
	}
}

impl TempMongo {
	/// Spawn a new MongoDB instance with a temporary state directory.
	pub async fn new() -> Result<Self, Error> {
		Self::from_builder(&TempMongoBuilder::new()).await
	}

	/// Create a builder to customize your [`TempMongo`].
	///
	/// After configuring the desirec options, run [`TempMongoBuilder::spawn()`].
	pub fn builder() -> TempMongoBuilder {
		TempMongoBuilder::new()
	}

	/// Get the PID of the MongoDB process.
	pub fn process_id(&self) -> u32 {
		self.server.id()
	}

	/// Get the path of the temporary state directory.
	pub fn directory(&self) -> &Path {
		self.tempdir.path()
	}

	/// Get the path of the listening socket of the MongoDB instance.
	pub fn socket_path(&self) -> &Path {
		&self.socket_path
	}

	/// Get the path of the log file of the MongoDB instance.
	pub fn log_path(&self) -> &Path {
		&self.log_path
	}

	/// Seed the database with the given data
	pub async fn seed_data(&self, seed_data: &SeedData) -> mongodb::error::Result<()> {
        seed_data.seed(&self.client).await
    }

	/// Get a client for the MongDB instance.
	///
	/// This returns a client by reference,
	/// but it can be cloned and sent to other threads or tasks if needed.
	pub fn client(&self) -> &mongodb::Client {
		&self.client
	}

	/// Enable or disable clean-up of the temporary directory when this object is dropped.
	pub fn set_clean_on_drop(&mut self, clean_on_drop: bool) {
		self.tempdir.set_clean_on_drop(clean_on_drop);
	}

	/// Kill the server and remove the temporary state directory on the filesystem.
	///
	/// Note that the server will also be killed when this object is dropped,
	/// and unless disabled, the temporary state directory will be removed by the [`Drop`] implementation too.
	///
	/// This function ignores the value of `clean_on_drop`.
	/// It also allows for better error handling compared to just dropping the object.
	pub async fn kill_and_clean(mut self) -> Result<(), Error> {
		self.client.shutdown_immediate().await;
		self.server.kill()
			.map_err(ErrorInner::KillServer)?;

		let path = self.tempdir.path().to_owned();
		self.tempdir.close()
			.map_err(|e| ErrorInner::CleanDir(path, e))?;
		Ok(())
	}

	/// Kill the server, but leave the temporary state directory on the filesystem.
	///
	/// Note that the server will also be killed when this object is dropped.
	///
	/// This function ignores the value of `clean_on_drop`.
	/// It also allows for better error handling compared to just dropping the object.
	pub async fn kill_no_clean(mut self) -> Result<(), Error> {
		let _path = self.tempdir.into_path();
		self.client.shutdown_immediate().await;
		self.server.kill()
			.map_err(ErrorInner::KillServer)?;
		Ok(())
	}

	/// Create the temporary directory and spawn a server based on the configuration of the given builder object.
	async fn from_builder(builder: &TempMongoBuilder) -> Result<Self, Error> {
		let tempdir = builder.make_temp_dir().map_err(ErrorInner::MakeTempDir)?; //makes a temporary directory --> handles errors that might occur
		let db_dir = tempdir.path().join("db"); // path for database directory
		let socket_path = tempdir.path().join("mongod.sock"); //unix socket for mongodb database
		let log_path = tempdir.path().join("mongod.log"); //path for logging dir is generated

		//Creation of Database directory
		std::fs::create_dir(&db_dir) 
			.map_err(|e| ErrorInner::MakeDbDir(db_dir.clone(), e))?;

		//Starting mongo DB server with various arguments
		let server = Command::new(builder.get_command())
			.arg("--bind_ip")
			.arg(&socket_path)
			.arg("--dbpath")
			.arg(db_dir)
			.arg("--logpath")
			.arg(&log_path)
			.arg("--nounixsocket")
			.arg("--noauth")
			.spawn()
			.map_err(|e| ErrorInner::SpawnServer(builder.get_command_string(), e))?;
		let server = KillOnDrop::new(server); //wrapped in kill on drop to ensure server is terminated when killOnDop instance is dropped

		//Configuring mongo db client using unix socket path
		// Client is used to interact with monoDB server
		let client_options = mongodb::options::ClientOptions::builder()
			.hosts(vec![mongodb::options::ServerAddress::Unix { path: socket_path.clone() }])
			.connect_timeout(Duration::from_millis(10))
			.build();
		let client = mongodb::Client::with_options(client_options)
			.map_err(|e| ErrorInner::Connect(socket_path.display().to_string(), e))?;

		// ensuring the server is running and accessible
		client.list_databases(None, None).await
			.map_err(|e| ErrorInner::Connect(socket_path.display().to_string(), e))?;

		// new tempMongo instance is returned, uses Self keyword to set struct variables
		Ok(Self {
			tempdir, //represents temporary directory where the mongodb instance stores data
			socket_path, // specify location of the unix socket file through which the mongodb server communicates --> essential part that allows mongoDb client and other tools to connect to the server
			log_path, //path where mongodb server writes logs
			server, // holds the process handler for mongodb server, essential for starting, stopping and restarting server
			client, // client is used to interact programmatically with the mongodb server. Allows database operations following the CRUD model
		})
	}

    /// Method to retrieve and print documents from a specified collection
	pub async fn print_documents(&self, db_name: &str, collection_name: &str) -> mongodb::error::Result<()> {
        let collection = self.client.database(db_name).collection(collection_name);
        
        // Query the collection for all documents
        let mut cursor = collection.find(None, None).await?;
        
        // Iterate over the documents in the cursor and print them
        while let Some(result) = cursor.try_next().await? {
            let document: mongodb::bson::Document = result;
            println!("{:?}", document);
        }

        Ok(())
    }


}

/// Builder for customizing your [`TempMongo`] object.
///
/// After configuring the desirec options, run [`TempMongoBuilder::spawn()`].
#[derive(Debug)]

pub struct TempMongoBuilder {
	/// The parent directory for the temporary directory.
	///
	/// Use the system default if set to `None`.
	parent_directory: Option<PathBuf>,

	/// Clean up the temprorary directory when the [`TempMongo`] object is dropped.
	clean_on_drop: bool,

	/// The mongdb command to execute.
	command: Option<OsString>,
}

impl TempMongoBuilder {
	/// Create a new builder.
	pub fn new() -> Self {
		Self {
			parent_directory: None,
			command: None,
			clean_on_drop: true,
		}
	}

	/// Spawn the MongoDB server and connect to it.
	pub async fn spawn(&self) -> Result<TempMongo, Error> {
		TempMongo::from_builder(self).await
	}

	/// Enable or disable cleaning of the temporary state directory when the [`TempMongo`] object is dropped.
	///
	/// This can also be changed after creation with [`TempMongo::set_clean_on_drop()`].
	pub fn clean_on_drop(mut self, clean_on_drop: bool) -> Self {
		self.clean_on_drop = clean_on_drop;
		self
	}

	/// Overwrite the `mongod` command to run.
	///
	/// Can be used to run a `mongod` binary from an alternative location.
	pub fn mongod_command(mut self, command: impl Into<OsString>) -> Self {
		self.command = Some(command.into());
		self
	}

	/// Get the command to execute to run MongoDB.
	fn get_command(&self) -> &OsStr {
		self.command
			.as_deref()
			.unwrap_or("mongod".as_ref())
	}

	/// Get the command to execute to run MongDB as a string, for diagnostic purposes.
	fn get_command_string(&self) -> String {
		self.get_command().to_string_lossy().into()
	}

	/// Create a temporary directory according to the configuration of the builder.
	fn make_temp_dir(&self) -> std::io::Result<TempDir> {
		match &self.parent_directory {
			Some(dir) => TempDir::new_in(dir, self.clean_on_drop),
			None => TempDir::new(self.clean_on_drop),
		}
	}
}

impl Default for TempMongoBuilder {
	fn default() -> Self {
		Self::new()
	}
}
