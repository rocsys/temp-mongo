use serde::Deserialize;
use std::fs;
use std::path::Path;
use mongodb::{Client, bson::Document};
use calamine::{open_workbook_auto, DataType, Reader};


#[derive(Deserialize)]
pub struct SeedData {
    pub database_name: String,
    pub collection_name: String,
    pub documents: Vec<Document>,
}

impl SeedData {
    // Function to read and parse the seed data file
    pub fn from_file(&self, file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = fs::read_to_string(file_path)?;
        let seed_data: Self = serde_json::from_str(&file_content)?;
        Ok(seed_data)
    }

    // Takes in path to excel file and sheet name and returns a vector of documents
    // The documents can then be used to seed the database
    pub fn from_excel(file_path: &Path, sheet: &str) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
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

    // Method to seed the database with the data
    pub async fn seed(&self, client: &Client) -> mongodb::error::Result<()> {
        let collection = client.database(&self.database_name).collection(&self.collection_name);
        for document in &self.documents {
            collection.insert_one(document.clone(), None).await?;
        }
        Ok(())
    }

    // Convert data to correct data type and insert into document as Bson 
    // Uses first non empty row as header to map against the values
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