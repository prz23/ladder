/// All labels of chain.
///
/// One label composed by two parts. L = [S, T].
/// S: the source of the transaction. T: the target of the transaction.
/// Label must be H256 type, so S and T is u128 type. to make a cross transaction, 
/// we should fill the label in transaction.

use restc_hex::{ToHex, FromHex};

static ETH_MAIN = "";
static ABOS_MAIN = "";
static ABOS_TEST = "";
static ETH_KOVAN = "";
static ETH_ROPSTEN = "";

/*
we should wrap it with new type.
The type can change to string.
How to get tag? 
1. A simple way is selecting left half of hash of the name.
2. Self define the tag from 0 to n.
*/

#[derive(Default)]
struct Label {
    from: Tag,
    to: Tag,
}

impl From<[u8; 32]> for Label {
    fn from(data: [u8; 32]) -> Self {
        Label {
            from: Tag(data[0..16]),
            to: Tag(data[16..32]),
        }
    }
}

impl Label {
    /// convert label to hex string
    fn hex(&self) -> String {
        self.to_hex()
    }
    
    /// convert hex string to label
    fn from_hex(&self) -> Result<Label, &str> {
        let data: Vec<u8> = 
        Ok(Label::default())
    }
}

// TODO impl the feature of type,such as string change to tag.
#[derive(Debug, partialEq)]
struct Tag([u8; 16]);

