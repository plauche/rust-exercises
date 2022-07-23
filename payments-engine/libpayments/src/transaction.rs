use serde::{Deserialize, Serialize};

use crate::client::ClientState;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,
}

impl Transaction {
    pub fn process(&self, state: ClientState, log: &[Transaction]) -> ClientState {
        match self.r#type {
            TransactionType::Deposit => self.process_deposit(state),
            TransactionType::Withdrawal => self.process_withdrawal(state),
            TransactionType::Dispute => self.process_dispute(state, log),
            TransactionType::Resolve => self.process_resolve(state, log),
            TransactionType::Chargeback => self.process_chargeback(state, log),
        }
    }

    fn process_withdrawal(&self, mut state: ClientState) -> ClientState {
        if state.available >= self.amount {
            state.available -= self.amount;
            state.total -= self.amount;
        }
        state
    }

    fn process_deposit(&self, mut state: ClientState) -> ClientState {
        state.available += self.amount;
        state.total += self.amount;
        state
    }

    fn process_dispute(&self, mut state: ClientState, log: &[Transaction]) -> ClientState {
        if let Some(disputed_transaction) = find_transaction(log, self.tx) {
            if disputed_transaction.r#type == TransactionType::Withdrawal {
                state.available -= -disputed_transaction.amount;
                state.held += -disputed_transaction.amount;
            } else if disputed_transaction.r#type == TransactionType::Deposit {
                state.available -= disputed_transaction.amount;
                state.held += disputed_transaction.amount;
            }
        }
        state
    }

    fn process_resolve(&self, mut state: ClientState, log: &[Transaction]) -> ClientState {
        let disputed_transaction = find_transaction(log, self.tx);
        let pending_dispute = find_dispute(log, self.tx);
        let prior_chargeback = find_chargeback(log, self.tx);
        if let Some(disputed_transaction) = disputed_transaction {
            if pending_dispute.is_some() && prior_chargeback.is_none() {
                if disputed_transaction.r#type == TransactionType::Withdrawal {
                    state.held += disputed_transaction.amount;
                    state.available -= disputed_transaction.amount;
                } else if disputed_transaction.r#type == TransactionType::Deposit {
                    state.held -= disputed_transaction.amount;
                    state.available += disputed_transaction.amount;
                }
            }
        }
        state
    }

    fn process_chargeback(&self, mut state: ClientState, log: &[Transaction]) -> ClientState {
        let disputed_transaction = find_transaction(log, self.tx);
        let pending_dispute = find_dispute(log, self.tx);
        let prior_resolve = find_resolve(log, self.tx);
        if let Some(disputed_transaction) = disputed_transaction {
            if pending_dispute.is_some() && prior_resolve.is_none() {
                if disputed_transaction.r#type == TransactionType::Withdrawal {
                    state.held += disputed_transaction.amount;
                    state.total += disputed_transaction.amount;
                } else if disputed_transaction.r#type == TransactionType::Deposit {
                    state.held -= disputed_transaction.amount;
                    state.total -= disputed_transaction.amount;
                }
                state.locked = true;
            }
        }
        state
    }
}

fn find_transaction(log: &[Transaction], tx: u32) -> Option<Transaction> {
    log.iter()
        .find(|t| {
            (t.r#type == TransactionType::Withdrawal || t.r#type == TransactionType::Deposit)
                && t.tx == tx
        })
        .copied()
}

fn find_dispute(log: &[Transaction], tx: u32) -> Option<Transaction> {
    log.iter()
        .find(|t| t.r#type == TransactionType::Dispute && t.tx == tx)
        .copied()
}

fn find_resolve(log: &[Transaction], tx: u32) -> Option<Transaction> {
    log.iter()
        .find(|t| t.r#type == TransactionType::Resolve && t.tx == tx)
        .copied()
}

fn find_chargeback(log: &[Transaction], tx: u32) -> Option<Transaction> {
    log.iter()
        .find(|t| t.r#type == TransactionType::Chargeback && t.tx == tx)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientState;
    use crate::transaction::TransactionType;

    fn deposit(tx: u32, amount: f32) -> Transaction {
        Transaction {
            r#type: TransactionType::Deposit,
            client: 0,
            tx,
            amount,
        }
    }

    fn withdrawal(tx: u32, amount: f32) -> Transaction {
        Transaction {
            r#type: TransactionType::Withdrawal,
            client: 0,
            tx,
            amount,
        }
    }

    fn dispute(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Dispute,
            client: 0,
            tx,
            amount: 0.0,
        }
    }

    fn resolve(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Resolve,
            client: 0,
            tx,
            amount: 0.0,
        }
    }

    fn chargeback(tx: u32) -> Transaction {
        Transaction {
            r#type: TransactionType::Chargeback,
            client: 0,
            tx,
            amount: 0.0,
        }
    }

    fn calc_state(transactions: Vec<Transaction>) -> ClientState {
        let mut state = ClientState::default();
        for (position, transaction) in transactions.iter().enumerate() {
            state = transaction.process(state, &transactions[..(position)]);
        }
        state
        // transactions
        //     .iter()
        //     .fold(ClientState::default(), |state, t| {
        //         t.process(state, &transactions)
        //     })
    }

    #[test]
    fn test_deposit_withdrawl_ok() {
        let transactions = vec![deposit(1, 5.0), withdrawal(2, 3.5)];
        let expected_state = ClientState {
            id: 0,
            available: 1.5,
            total: 1.5,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_deposit_dispute_ok() {
        let transactions = vec![deposit(1, 10.0), dispute(1)];
        let expected_state = ClientState {
            id: 0,
            available: 0.0,
            total: 10.0,
            held: 10.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_deposit_dispute_no_match() {
        let transactions = vec![deposit(1, 10.0), dispute(2)];

        let expected_state = ClientState {
            id: 0,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_deposit_dispute_resolve_ok() {
        let transactions = vec![deposit(1, 10.0), dispute(1), resolve(1)];

        let expected_state = ClientState {
            id: 0,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_deposit_dispute_chargeback_ok() {
        let transactions = vec![deposit(1, 10.0), dispute(1), chargeback(1)];

        let expected_state = ClientState {
            id: 0,
            available: 0.0,
            total: 0.0,
            held: 0.0,
            locked: true,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_withdraw_dispute_ok() {
        let transactions = vec![deposit(1, 10.0), withdrawal(2, 5.0), dispute(2)];

        let expected_state = ClientState {
            id: 0,
            available: 10.0,
            total: 5.0,
            held: -5.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_withdraw_dispute_resolve_ok() {
        let transactions = vec![deposit(1, 10.0), withdrawal(2, 5.0), dispute(2), resolve(2)];

        let expected_state = ClientState {
            id: 0,
            available: 5.0,
            total: 5.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_withdraw_dispute_chargeback_ok() {
        let transactions = vec![
            deposit(1, 10.0),
            withdrawal(2, 5.0),
            dispute(2),
            chargeback(2),
        ];

        let expected_state = ClientState {
            id: 0,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: true,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_ignore_dispute_before_transaction() {
        let transactions = vec![dispute(1), deposit(1, 15.0)];

        let expected_state = ClientState {
            id: 0,
            available: 15.0,
            total: 15.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_ignore_resolve_before_dispute() {
        let transactions = vec![deposit(1, 10.0), resolve(1), dispute(1)];
        let expected_state = ClientState {
            id: 0,
            available: 0.0,
            total: 10.0,
            held: 10.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_ignore_chargeback_before_dispute() {
        let transactions = vec![deposit(1, 10.0), chargeback(1), dispute(1)];
        let expected_state = ClientState {
            id: 0,
            available: 0.0,
            total: 10.0,
            held: 10.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_ignore_resolve_after_chargeback() {
        let transactions = vec![deposit(1, 10.0), dispute(1), chargeback(1), resolve(1)];
        let expected_state = ClientState {
            id: 0,
            available: 0.0,
            total: 0.0,
            held: 0.0,
            locked: true,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }

    #[test]
    fn test_ignore_chargeback_after_resolve() {
        let transactions = vec![deposit(1, 10.0), dispute(1), resolve(1), chargeback(1)];
        let expected_state = ClientState {
            id: 0,
            available: 10.0,
            total: 10.0,
            held: 0.0,
            locked: false,
        };
        assert_eq!(calc_state(transactions), expected_state);
    }
}
