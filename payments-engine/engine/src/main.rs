use std::fs::File;
use std::io::BufReader;

use csv::Trim;
use libpayments::client::Client;
use libpayments::transaction::Transaction;
use std::collections::HashMap;

fn main() {
    let f = File::open("example.csv").unwrap();
    let file_reader = BufReader::new(f);
    let mut clients = HashMap::<u16, Client>::new();
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(file_reader);
    for result in reader.deserialize() {
        let record: Transaction = result.unwrap();
        match clients.get_mut(&record.client) {
            Some(client) => {
                client.transactions.push(record);
            }
            None => {
                clients.insert(
                    record.client,
                    Client {
                        id: record.client,
                        transactions: vec![record],
                    },
                );
            }
        }
    }
    for c in clients.values() {
        println!("{:?}", c.calculate_state())
    }
}
