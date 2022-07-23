use crate::transaction::Transaction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientState {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Default for ClientState {
    fn default() -> ClientState {
        ClientState {
            id: 0,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    pub id: u16,
    pub transactions: Vec<Transaction>,
}

impl Client {
    pub fn new(id: u16) -> Client {
        Client {
            id,
            transactions: vec![],
        }
    }

    pub fn with_transactions(mut self, transactions: Vec<Transaction>) -> Self {
        self.transactions = transactions;
        self
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn calculate_state(&self) -> ClientState {
        let mut state = ClientState {
            id: self.id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        };

        for transaction in &self.transactions {
            state = transaction.process(state, &self.transactions);
        }

        state
    }
}
