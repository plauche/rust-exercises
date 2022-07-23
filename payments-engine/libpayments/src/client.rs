use crate::transaction::Transaction;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ClientState {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl ClientState {
    pub fn new(id: u16) -> ClientState {
        ClientState {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
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
        calc_state(&self.transactions)
    }
}

// Helper function for running through transactions and calculating end state
pub fn calc_state(transactions: &[Transaction]) -> ClientState {
    let mut state = ClientState::default();
    for (position, transaction) in transactions.iter().enumerate() {
        state = transaction.process(state, &transactions[..(position)]);
        // Assuming that a locked account should not accept any additional transactions
        if state.locked {
            break;
        }
    }
    state
}
