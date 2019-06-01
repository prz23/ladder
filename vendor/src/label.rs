/// All labels of chain.
///
/// One label composed by two parts. L = [S, T].
/// S: the source of the transaction. T: the target of the transaction.
/// Label must be H256 type, so S and T is u128 type. to make a cross transaction, 
/// we should fill the label in transaction.

// use restc_hex::{ToHex, FromHex};
use std::cmp::PartialEq;

static ETH_MAIN:&str = "";
static ABOS_MAIN:&str = "";
static ABOS_TEST:&str = "";
static ETH_KOVAN:&str = "";
static ETH_ROPSTEN:&str = "";

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

// TODO impl the feature of type,such as string change to tag.
#[derive(Default, Debug)]
struct Tag([u8; 16]);


/// Alias of Chain to diff.
#[derive(Copy, Clone)]
pub enum ChainAlias {
    ETH,
    ABOS,
}