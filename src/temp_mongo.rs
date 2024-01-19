use crate::error::ErrorInner;
use crate::util::{DataSeeder, KillOnDrop, PortGenerator, TempDir};
use crate::Error;
use futures_util::stream::TryStreamExt;
use mongodb::bson::Document;
use mongodb::options::{ClientOptions, ServerAddress};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

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
	seed: DataSeeder,
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
	/// Spawn a new MongoDB instance with default port configuration.
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

	/// Prepare seed document row with &str for db name and collection name into mongoDB database instance
	pub fn prepare_seed_document(
		&self,
		database_name: &str,
		collection_name: &str,
		documents: Vec<Document>,
	) -> DataSeeder {
		self.seed.new_in(database_name, collection_name, documents)
	}

	/// Prepare seed document row with &String for db name and collection name into mongoDB database instance
	pub fn prepare_seed_document_string(
		&self,
		database_name: &String,
		collection_name: &String,
		documents: Vec<Document>,
	) -> DataSeeder {
		self.seed
			.new_in_with_string(database_name, collection_name, documents)
	}

	/// Seed document into MongoDB database
	/// # Arguments
	/// * `seed_data` - The seed data to insert into the database
	pub async fn load_document(&self, seed_data: &DataSeeder) -> mongodb::error::Result<()> {
		seed_data.seed_document(&self.client).await
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
		self.server.kill().map_err(ErrorInner::KillServer)?;
		sleep(Duration::from_millis(50)).await;

		let path = self.tempdir.path().to_owned();
		self.tempdir
			.close()
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
		self.server.kill().map_err(ErrorInner::KillServer)?;
		Ok(())
	}

	/// Advanced printing of documents in a collection
	/// # Arguments
	/// * `db_name` - The name of the database
	/// * `collection_name` - The name of the collection
	/// # Errors
	/// Returns an error if any MongoDB operation fails during the printing process.
	pub async fn print_documents(
		&self,
		db_name: &str,
		collection_name: &str,
	) -> mongodb::error::Result<()> {
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
	/// Creates a temporary directory and spawns a MongoDB server based on the configuration
	/// provided by the `TempMongoBuilder` object. This function is designed to be cross-platform,
	/// supporting both Windows and Unix-based systems (Linux/macOS). It configures the MongoDB
	/// server and client differently depending on the operating system to ensure compatibility.
	///
	/// # Arguments
	/// * `builder` - A reference to `TempMongoBuilder` used for configuring the MongoDB instance.
	///
	/// # Returns
	/// A `Result` which, on success, contains the `Self` instance representing the running MongoDB
	/// server and its associated configuration. On failure, it returns an `Error` detailing the issue.
	///
	/// # Errors
	/// This function can return errors related to creating temporary directories, starting the MongoDB
	/// server, and configuring the MongoDB client.
	async fn from_builder(builder: &TempMongoBuilder) -> Result<Self, Error> {
		let tempdir = builder.make_temp_dir().map_err(ErrorInner::MakeTempDir)?;
		let db_dir = tempdir.path().join("db");
		let log_path = tempdir.path().join("mongod.log");
		let seed = DataSeeder::new();

		std::fs::create_dir(&db_dir).map_err(|e| ErrorInner::MakeDbDir(db_dir.clone(), e))?;

		let server_address: String;
		let socket_path: PathBuf;

		#[cfg(windows)]
		{
			server_address = "localhost".to_string();
			socket_path = PathBuf::from(&server_address);
		}
		#[cfg(unix)]
		{
			// For Unix-based systems: Use Unix socket for MongoDB
			server_address = tempdir.path().join("mongod.sock").display().to_string();
			socket_path = PathBuf::from(&server_address);
		}

		let mut port_generator = PortGenerator::new();
		let random_port = port_generator.generate();

		let mongodb_port = random_port.selected_port().ok_or_else(|| {
			let error: ErrorInner = ErrorInner::Port.into();
			eprintln!("Error: {}", error);
			error
		})?;

		//TODO: Add some error handling when spawning the service
		//We might need to hide away the spawning of the server in a new class
		let server = Command::new(builder.get_command())
			.arg("--bind_ip")
			.arg(&server_address)
			.arg("--dbpath")
			.arg(&db_dir)
			.arg("--logpath")
			.arg(&log_path)
			.arg("--noauth")
			.arg("--port")
			.arg(mongodb_port.to_string())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.map_err(|e| ErrorInner::SpawnServer(builder.get_command_string(), e))?;

		let server = KillOnDrop::new(server);

		let mut hosts = Vec::new();

		#[cfg(unix)]
		{
			// For Unix-like systems, use a Unix socket
			hosts.push(ServerAddress::Unix {
				path: socket_path.clone(),
			});

			// Debugging: Print the Unix socket path
			println!(
				"Using Unix socket for MongoDB connection: {:?}",
				socket_path
			);
		}

		#[cfg(windows)]
		{
			hosts.push(ServerAddress::Tcp {
				host: "localhost".parse().unwrap(),
				port: Some(mongodb_port),
			});
		}

		let client_options = ClientOptions::builder()
			.hosts(hosts)
			.connect_timeout(Duration::from_millis(100))
			.direct_connection(true)
			.build();

		let client = mongodb::Client::with_options(client_options.clone())
			.map_err(|e| ErrorInner::Connect(server_address.clone(), e))?;

		client
			.list_databases(None, None)
			.await
			.map_err(|e| ErrorInner::Connect(server_address, e))?;

		Ok(Self {
			tempdir,
			socket_path,
			log_path,
			server,
			client,
			seed,
		})
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
	pub fn get_command(&self) -> &OsStr {
		self.command.as_deref().unwrap_or("mongod".as_ref())
	}

	/// Get the command to execute to run MongDB as a string, for diagnostic purposes.
	pub fn get_command_string(&self) -> String {
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
