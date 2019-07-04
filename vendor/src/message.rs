use crate::events::*;
use web3::types::H256;

#[derive(Debug, Clone, PartialEq)]
pub enum RelayType {
    Match,
    Request,
    Deposit,
    Withdraw,
    SetAuthorities,
    ExchangeRate,
    LockToken,
    UnlockToken,
}

#[derive(Debug, Clone)]
pub struct RelayMessage {
    /// The hash of transaction.
    pub hash: H256,
    /// The raw data of log.
    pub raw: Vec<u8>,
    /// The type of Message.
    pub ty: RelayType,
}

impl From<MatchEvent> for RelayMessage {
    fn from(event: MatchEvent) -> Self {
        RelayMessage {
            hash: H256::from(0),
            raw: event.to_bytes(),
            ty: RelayType::Match,
        }
    }
}

impl From<RequestEvent> for RelayMessage {
    fn from(event: RequestEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::Request,
        }
    }
}

impl From<DepositEvent> for RelayMessage {
    fn from(event: DepositEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::Deposit,
        }
    }
}

impl From<WithdrawEvent> for RelayMessage {
    fn from(event: WithdrawEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::Withdraw,
        }
    }
}

impl From<AuthorityEvent> for RelayMessage {
    fn from(event: AuthorityEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::SetAuthorities,
        }
    }
}

impl From<ExchangeRateEvent> for RelayMessage {
    fn from(event: ExchangeRateEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::ExchangeRate,
        }
    }
}

impl From<LockTokenEvent> for RelayMessage {
    fn from(event: LockTokenEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::LockToken,
        }
    }
}

impl From<UnlockTokenEvent> for RelayMessage {
    fn from(event: UnlockTokenEvent) -> Self {
        RelayMessage {
            hash: event.tx_hash,
            raw: event.to_bytes(),
            ty: RelayType::UnlockToken,
        }
    }
}

// TODO add macro to simplified code
/*
 #[derive(Expand)]
 pub enum Relay {
     #[event(IngressEvent)]
     Ingress,
 }

 ref structopt
*/