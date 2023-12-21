use serde::Deserialize;
use std::fs;
use std::path::Path;
use mongodb::{Client, bson::Document};
use calamine::{open_workbook_auto, DataType, Reader};

/// Data seed options for mongodb instance
/// 
/// The database_name and collection_name are used to specify the database and collection to seed the data into
#[derive(Deserialize)]
pub struct SeedData {
    /// The name of the database to seed.
    pub database_name: String,
    /// The name of the collection to seed.
    pub collection_name: String,
    /// The documents to be seeded into the collection.
    pub documents: Vec<Document>,
}

impl SeedData {
    /// Creates a new `SeedData` instance with default values.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with empty database name, collection name, and an empty vector of documents.
    pub fn new() -> Self {
        Self {
            database_name: String::new(),
            collection_name: String::new(),
            documents: Vec::new(),
        }
    }

    /// Creates a new `SeedData` instance with specified values.
    ///
    /// # Arguments
    ///
    /// * `database_name` - A string slice representing the name of the database.
    /// * `collection_name` - A string slice representing the name of the collection.
    /// * `documents` - A vector of MongoDB documents to be seeded into the collection.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with the specified database and collection names, and the provided documents.
    pub fn new_in(&self, database_name: &str, collection_name: &str, documents: Vec<Document>) -> Self {
        Self {
            database_name: database_name.to_string(),
            collection_name: collection_name.to_string(),
            documents,
        }
    }

    /// Creates a new `SeedData` instance with specified values using `String` objects.
    ///
    /// # Arguments
    ///
    /// * `database_name` - A `String` reference representing the name of the database.
    /// * `collection_name` - A `String` reference representing the name of the collection.
    /// * `documents` - A vector of MongoDB documents to be seeded into the collection.
    ///
    /// # Returns
    ///
    /// Returns an instance of `SeedData` with the specified database and collection names, and the provided documents.
    pub fn new_with_string(&self, database_name: &String, collection_name: &String, documents: Vec<Document>) -> Self {
        Self {
            database_name: database_name.clone(),
            collection_name: collection_name.clone(),
            documents,
        }
    }

    
    /// Reads and parses a seed data file into a `SeedData` instance.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A reference to the path of the file to read from.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the file content is not valid JSON.
    pub fn from_file(&self, file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = fs::read_to_string(file_path)?;
        let seed_data: Self = serde_json::from_str(&file_content)?;
        Ok(seed_data)
    }

    /// Reads an Excel file and returns a vector of MongoDB documents to be used for seeding.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A reference to the path of the Excel file.
    /// * `sheet` - The name of the sheet within the Excel file to read from.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the data is not in the expected format.
    pub fn from_excel(&self, file_path: &Path, sheet: &str) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let mut workbook = open_workbook_auto(file_path)?;
        let range = workbook.worksheet_range(sheet)
            .map_err(|e| e.to_string())?;

        // Find the first non-empty row to use as headers
        let mut rows = range.rows();
        let header_row = rows
            .find(|row| row.iter().any(|cell| !cell.is_empty()))
            .ok_or_else(|| "No non-empty header row found")?;

        // Determine the starting column index based on the first non-empty cell
        let start_col = header_row.iter().position(|cell| !cell.is_empty())
            .ok_or_else(|| "No non-empty header cell found")?;

        // Collect only non-empty headers
        let headers: Vec<String> = header_row.iter()
            .skip(start_col)
            .map(|cell| cell.get_string().unwrap_or_default().to_owned())
            .filter(|header| !header.is_empty())
            .collect();

        let mut documents = Vec::new();
        for row in rows {
            // Skip rows that are entirely empty or that don't have enough cells
            if row.len() <= start_col || row.iter().all(DataType::is_empty) {
                continue;
            }

            let document = Self::row_to_document(&headers, &row[start_col..]);
            documents.push(document);
        }

        Ok(documents)
    }

    /// Seeds the specified MongoDB collection with the provided documents.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference to the MongoDB client to use for inserting documents.
    ///
    /// # Errors
    ///
    /// Returns an error if any MongoDB operation fails during the seeding process.
    pub async fn seed(&self, client: &Client) -> mongodb::error::Result<()> {
        let collection = client.database(&self.database_name).collection(&self.collection_name);
        for document in &self.documents {
            collection.insert_one(document.clone(), None).await?;
        }
        Ok(())
    }

    /// Converts a row of Excel data to a MongoDB document using provided headers.
    ///
    /// # Arguments
    ///
    /// * `headers` - A slice of strings representing the column headers.
    /// * `row` - A slice of `DataType` representing the Excel row data.
    ///
    /// # Returns
    ///
    /// Returns a `mongodb::bson::Document` representing the row of data.
   fn row_to_document(headers: &[String], row: &[DataType]) -> mongodb::bson::Document {
    let mut document = mongodb::bson::doc! {};

    for (header, cell) in headers.iter().zip(row.iter()) {
        let bson_value = match cell {
            DataType::String(value) => {
                mongodb::bson::Bson::String(value.clone())
            },
            DataType::Float(value) => {
                mongodb::bson::Bson::Double(*value)
            },
            DataType::Int(value) => {
                mongodb::bson::Bson::Int64(*value as i64)
            },
            DataType::Bool(value) => {
                mongodb::bson::Bson::Boolean(*value)
            },
            // Handle other DataType variants as needed
            _ => continue, // Skip unknown or empty types
        };

        document.insert(header.clone(), bson_value);
    }

    document
}
}