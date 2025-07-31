use crate::types::OHLCV;
use crate::Result;
use chrono::{DateTime, Utc, NaiveDateTime};
use csv::Reader;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::path::Path;

/// CSV record for OHLCV data with flexible field names
#[derive(Debug, Deserialize)]
pub struct OHLCVRecord {
    #[serde(alias = "Date", alias = "Datetime", alias = "Time")]
    pub timestamp: String,
    
    #[serde(alias = "Open", deserialize_with = "deserialize_number_with_commas")]
    pub open: f64,
    
    #[serde(alias = "High", deserialize_with = "deserialize_number_with_commas")]
    pub high: f64,
    
    #[serde(alias = "Low", deserialize_with = "deserialize_number_with_commas")]
    pub low: f64,
    
    #[serde(alias = "Close", deserialize_with = "deserialize_number_with_commas")]
    pub close: f64,
    
    #[serde(alias = "Volume", default = "default_volume", deserialize_with = "deserialize_number_with_commas")]
    pub volume: f64,
    
    // Optional field for adjusted close (we'll ignore it for backtesting)
    #[serde(alias = "Adj Close", default, deserialize_with = "deserialize_optional_number_with_commas")]
    pub adj_close: Option<f64>,
}

fn default_volume() -> f64 {
    1000000.0
}

/// Custom deserializer for numeric fields that may contain commas
fn deserialize_number_with_commas<'de, D>(deserializer: D) -> std::result::Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let cleaned = s.replace(",", "");
    cleaned.parse().map_err(serde::de::Error::custom)
}

/// Custom deserializer for optional numeric fields that may contain commas
fn deserialize_optional_number_with_commas<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) if !s.is_empty() => {
            let cleaned = s.replace(",", "");
            cleaned.parse().map(Some).map_err(serde::de::Error::custom)
        }
        _ => Ok(None),
    }
}

/// Data loader for OHLCV data from CSV files
pub struct DataLoader;

impl DataLoader {
    /// Load OHLCV data from a CSV file in the data/ directory
    /// 
    /// # Arguments
    /// * `dataset_name` - Name of the CSV file (without .csv extension)
    /// 
    /// # Examples
    /// ```
    /// use backtesting::data::DataLoader;
    /// 
    /// // Loads data/AAPL.csv
    /// let data = DataLoader::load_from_csv("AAPL")?;
    /// ```
    pub fn load_from_csv(dataset_name: &str) -> Result<Vec<OHLCV>> {
        let file_path = format!("data/{}.csv", dataset_name);
        Self::load_from_file(&file_path)
    }
    
    /// Load OHLCV data from a specific file path
    /// 
    /// # Arguments
    /// * `file_path` - Path to the CSV file
    /// 
    /// # Supported CSV formats
    /// - Standard format: timestamp,open,high,low,close,volume
    /// - Yahoo Finance format: Date,Open,High,Low,Close,Volume
    /// - Flexible column ordering and naming
    /// 
    /// # Date formats supported
    /// - ISO 8601: 2023-01-01T00:00:00Z
    /// - Date only: 2023-01-01
    /// - US format: 01/01/2023
    /// - European format: 01-01-2023
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<OHLCV>> {
        let file = File::open(file_path.as_ref())
            .map_err(|e| format!("Failed to open file '{}': {}", file_path.as_ref().display(), e))?;
        
        let mut reader = Reader::from_reader(file);
        let mut data = Vec::new();
        
        for (line_num, result) in reader.deserialize::<OHLCVRecord>().enumerate() {
            let record = result
                .map_err(|e| format!("Failed to parse line {}: {}", line_num + 2, e))?;
            
            let timestamp = Self::parse_timestamp(&record.timestamp)
                .map_err(|e| format!("Failed to parse timestamp '{}' on line {}: {}", 
                    record.timestamp, line_num + 2, e))?;
            
            // Validate OHLCV data
            if record.open <= 0.0 || record.high <= 0.0 || record.low <= 0.0 || record.close <= 0.0 {
                return Err(format!("Invalid price data on line {}: prices must be positive", line_num + 2).into());
            }
            
            if record.high < record.low {
                return Err(format!("Invalid price data on line {}: high ({}) < low ({})", 
                    line_num + 2, record.high, record.low).into());
            }
            
            if record.open > record.high || record.open < record.low ||
               record.close > record.high || record.close < record.low {
                return Err(format!("Invalid price data on line {}: open/close outside high/low range", line_num + 2).into());
            }
            
            data.push(OHLCV {
                timestamp,
                open: record.open,
                high: record.high,
                low: record.low,
                close: record.close,
                volume: record.volume,
            });
        }
        
        if data.is_empty() {
            return Err("No data found in CSV file".into());
        }
        
        // Sort by timestamp to ensure chronological order
        data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(data)
    }
    
    /// Parse various timestamp formats
    fn parse_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
        // Try different timestamp formats
        
        // ISO 8601 with timezone
        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp_str) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // ISO 8601 without timezone (assume UTC)
        if let Ok(dt) = timestamp_str.parse::<DateTime<Utc>>() {
            return Ok(dt);
        }
        
        // Date only formats
        let date_formats = [
            "%Y-%m-%d",           // 2023-01-01
            "%m/%d/%Y",           // 01/01/2023
            "%d/%m/%Y",           // 01/01/2023 (European)
            "%Y/%m/%d",           // 2023/01/01
            "%d-%m-%Y",           // 01-01-2023
            "%b %d, %Y",          // Jul 29, 2025
            "%Y-%m-%d %H:%M:%S",  // 2023-01-01 00:00:00
            "%m/%d/%Y %H:%M:%S",  // 01/01/2023 00:00:00
            "%d/%m/%Y %H:%M:%S",  // 01/01/2023 00:00:00
        ];
        
        for format in &date_formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp_str, format) {
                return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
            }
        }
        
        // Try parsing as date only and add midnight time
        let date_only_formats = [
            "%Y-%m-%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y/%m/%d",
            "%d-%m-%Y",
            "%b %d, %Y",  // Jul 29, 2025
        ];
        
        for format in &date_only_formats {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(timestamp_str, format) {
                let datetime = date.and_hms_opt(0, 0, 0).unwrap();
                return Ok(DateTime::from_naive_utc_and_offset(datetime, Utc));
            }
        }
        
        Err(format!("Unable to parse timestamp: {}", timestamp_str).into())
    }
    
    /// Create sample data for testing (saves to data/sample.csv)
    pub fn create_sample_data() -> Result<()> {
        use std::fs;
        use std::io::Write;
        
        // Ensure data directory exists
        fs::create_dir_all("data")?;
        
        let mut file = File::create("data/sample.csv")?;
        writeln!(file, "Date,Open,High,Low,Close,Volume")?;
        
        let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(365);
        let mut price = 100.0;
        
        for i in 0..252 { // One year of trading days
            let date = start_date + chrono::Duration::days(i);
            
            // Simulate price movement
            let change = (i as f64 * 0.1).sin() * 2.0 + (rand::random::<f64>() - 0.5) * 3.0;
            price += change;
            price = price.max(50.0); // Floor price
            
            let open = price;
            let high = price + (rand::random::<f64>() * 3.0);
            let low = price - (rand::random::<f64>() * 3.0);
            let close = low + (rand::random::<f64>() * (high - low));
            let volume = 1000000.0 + (rand::random::<f64>() * 500000.0);
            
            price = close; // Next day starts at previous close
            
            writeln!(file, "{},{:.2},{:.2},{:.2},{:.2},{:.0}", 
                date.format("%Y-%m-%d"), open, high, low, close, volume)?;
        }
        
        println!("Sample data created at data/sample.csv");
        Ok(())
    }
}