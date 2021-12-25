use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use chrono::prelude::*;

// Data structure for data store
#[derive(Debug)]
pub struct Record {
    pub id:        u32,
    pub timestamp: String,
    pub pair:      String,
    pub mark:      f64
}
impl Record {
    pub fn new(pair: String, mark: f64) -> Self {
        Self { id: 0, timestamp: Local::now().to_string(), pair: pair, mark: mark }
    }
    pub fn from_row(row: String) -> Self {
        let vals = row.split("|");
        let vec = vals.collect::<Vec<&str>>();
    
        Record { 
            id:        vec[0].parse::<u32>().unwrap(), 
            pair:      String::from(vec[1]), 
            mark:      vec[2].parse::<f64>().unwrap(),
            timestamp: String::from(vec[3]), 
        }
    }
    fn to_row(&self) -> String {
        format!("{}|{}|{}|{}\n", self.id, self.pair, self.mark, self.timestamp)
    }
}

/**
 * Creates a new record in the data store
 */
pub fn create_record(file: String, mut record: Record) {
    let path = Path::new(&file);
    let path_display = path.display();

    // Get next auto-increment value
    let last_row = get_last_record(&file);
    let next_id = last_row.id.clone() + 1;
    record.id = next_id;

    let mut file = OpenOptions::new().append(true).create(true).open(&path).expect(
        "cannot open file"
    );
    file.write_all(record.to_row().as_bytes()).expect("insert failed");
    println!("record inserted successfully: {}", path_display);
}

/**
 * Gets the last record from the data store
 * TODO:
 *   - Add filters for pair
 */
pub fn get_last_record(file: &String) -> Record {
    let f = match File::open(file) {
        Err(e) => panic!("couldn't open file: {}", e),
        Ok(file) => file,
    };
    let reader = BufReader::new(f);
    let last_line = reader.lines().last().unwrap().ok().unwrap();
    Record::from_row(last_line)
}

pub fn get_last_n_records(file: String, n: usize, newest_first: bool) -> Vec<Record> {
    let f = match File::open(file) {
        Err(e) => panic!("couldn't open file: {}", e),
        Ok(file) => file,
    };
    let reader = BufReader::new(f);
    let lines = reader.lines();
    
    //println!("Total lines: {}\n", lines.count());
    let mut last_n_lines = Vec::<String>::new();    

    for line in lines {
        let line_str = line.unwrap();
        //println!("{}", line_str);

        // Save the last n rows as iteration happens
        last_n_lines.insert(0, line_str);
        if last_n_lines.len() > n {
            last_n_lines.pop();
        }
    }

    if !newest_first {
        last_n_lines.reverse(); // orders same as in db (top to bottom)
    }

    let records: Vec<Record> = last_n_lines.iter().map(|l| Record::from_row(l.to_string())).collect();
    return records
}
