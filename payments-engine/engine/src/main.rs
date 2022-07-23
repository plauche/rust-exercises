use std::fs::File;
use std::io::BufReader;

use csv::Trim;
use libpayments::client::Client;
use libpayments::transaction::Transaction;
use std::collections::HashMap;
use std::error::Error;
use std::io;

fn process_records(filename: &str) -> Result<HashMap<u16, Client>, Box<dyn Error>> {
    let f = File::open(filename)?;
    let file_reader = BufReader::new(f);
    let mut clients = HashMap::<u16, Client>::new();
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(file_reader);
    for result in reader.deserialize() {
        let record: Transaction = result?;
        match clients.get_mut(&record.client) {
            Some(client) => {
                client.transactions.push(record);
            }
            None => {
                clients.insert(
                    record.client,
                    Client::new(record.client).with_transactions(vec![record]),
                );
            }
        }
    }
    Ok(clients)
}

fn output_record_states(clients: &HashMap<u16, Client>) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_writer(io::stdout());
    for c in clients.values() {
        writer.serialize(c.calculate_state())?;
    }
    Ok(writer.flush()?)
}

fn main() {
    let path = std::env::args().nth(1).expect("No path given");
    let clients = process_records(&path).expect("Failed to parse client records");
    output_record_states(&clients).expect("Failed to output client states");
}
