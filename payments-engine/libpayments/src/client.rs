use crate::transaction::{Transaction, TransactionType};

#[derive(Debug, PartialEq)]
pub struct ClientState {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
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
    fn find_transaction(&self, tx: u32) -> Option<&Transaction> {
        self.transactions.iter().find(|t| {
            (t.r#type == TransactionType::Withdrawal || t.r#type == TransactionType::Deposit)
                && t.tx == tx
        })
    }

    fn find_dispute(&self, tx: u32) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(|t| t.r#type == TransactionType::Dispute && t.tx == tx)
    }

    pub fn calculate_state(&self) -> ClientState {
        let mut available: f32 = 0.0;
        let mut held: f32 = 0.0;
        let mut total: f32 = 0.0;
        let mut locked = false;

        for transaction in &self.transactions {
            match transaction.r#type {
                TransactionType::Deposit => {
                    available += transaction.amount;
                    total += transaction.amount;
                }
                TransactionType::Withdrawal => {
                    if available >= transaction.amount {
                        available -= transaction.amount;
                        total -= transaction.amount;
                    }
                }
                TransactionType::Dispute => {
                    if let Some(disputed_transaction) = self.find_transaction(transaction.tx) {
                        if disputed_transaction.r#type == TransactionType::Withdrawal {
                            available -= -disputed_transaction.amount;
                            held += -disputed_transaction.amount;
                        } else if disputed_transaction.r#type == TransactionType::Deposit {
                            available -= disputed_transaction.amount;
                            held += disputed_transaction.amount;
                        }
                    }
                }
                TransactionType::Resolve => {
                    let disputed_transaction = self.find_transaction(transaction.tx);
                    let pending_dispute = self.find_dispute(transaction.tx);
                    if let Some(disputed_transaction) = disputed_transaction {
                        if let Some(_) = pending_dispute {
                            held -= disputed_transaction.amount;
                            available += disputed_transaction.amount;
                        }
                    }
                }
                TransactionType::Chargeback => {
                    let disputed_transaction = self.find_transaction(transaction.tx);
                    let pending_dispute = self.find_dispute(transaction.tx);
                    if let Some(disputed_transaction) = disputed_transaction {
                        if let Some(_) = pending_dispute {
                            held -= disputed_transaction.amount;
                            total -= disputed_transaction.amount;
                            locked = true;
                        }
                    }
                }
            }
        }

        ClientState {
            id: self.id,
            available,
            held,
            total,
            locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deposit(client: u16, tx: u32, amount: f32) -> Transaction {
        Transaction {
            r#type: TransactionType::Deposit,
            client,
            tx,
            amount,
        }
    }

    fn withdrawal(client: u16, tx: u32, amount: f32) -> Transaction {
        Transaction {
            r#type: TransactionType::Withdrawal,
            client,
            tx,
            amount,
        }
    }

    fn dispute(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx,
            amount: 0.0,
        }
    }

    fn resolve(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Resolve,
            client: 1,
            tx,
            amount: 0.0,
        }
    }

    fn chargeback(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx,
            amount: 0.0,
        }
    }

    #[test]
    fn test_deposit_withdrawl_ok() {
        let client = Client {
            id: 1,
            transactions: vec![deposit(1, 1, 5.0), withdrawal(1, 2, 3.5)],
        };
        let expected_state = ClientState {
            id: 1,
            available: 1.5,
            total: 1.5,
            held: 0.0,
            locked: false,
        };
        let result_state = client.calculate_state();

        assert_eq!(expected_state, result_state);
    }

    #[test]
    fn test_deposit_dispute_ok() {
        let client_id = 1;
        let client =
            Client::new(client_id).with_transactions(vec![deposit(client_id, 1, 10.0), dispute(1)]);

        let expected_state = ClientState {
            id: client_id,
            available: 0.0,
            total: 10.0,
            held: 10.0,
            locked: false,
        };
        assert_eq!(expected_state, client.calculate_state());
    }

    #[test]
    fn test_deposit_dispute_no_match() {
        let client_id = 1;
        let client =
            Client::new(client_id).with_transactions(vec![deposit(client_id, 1, 10.0), dispute(2)]);

        let expected_state = ClientState {
            id: client_id,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(expected_state, client.calculate_state());
    }

    #[test]
    fn test_deposit_dispute_resolve_ok() {
        let client_id = 1;
        let client = Client::new(client_id).with_transactions(vec![
            deposit(client_id, 1, 10.0),
            dispute(1),
            resolve(1),
        ]);

        let expected_state = ClientState {
            id: client_id,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(expected_state, client.calculate_state());
    }

    #[test]
    fn test_deposit_dispute_chargeback_ok() {
        let client_id = 1;
        let client = Client::new(client_id).with_transactions(vec![
            deposit(client_id, 1, 10.0),
            dispute(1),
            chargeback(1),
        ]);

        let expected_state = ClientState {
            id: client_id,
            available: 0.0,
            total: 0.0,
            held: 0.0,
            locked: true,
        };
        assert_eq!(expected_state, client.calculate_state());
    }

    #[test]
    fn test_withdraw_dispute_ok() {
        let client_id = 1;
        let client = Client::new(client_id).with_transactions(vec![
            deposit(client_id, 1, 10.0),
            withdrawal(client_id, 2, 5.0),
            dispute(2),
        ]);

        let expected_state = ClientState {
            id: client_id,
            available: 10.0,
            total: 5.0,
            held: -5.0,
            locked: false,
        };
        assert_eq!(expected_state, client.calculate_state());
    }
}
