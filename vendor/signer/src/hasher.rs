
pub type Hash = [u8; 32];

pub trait Hasher {
    fn hash(&self) -> Hash;
}

impl<T> Hasher for T where T: AsRef<[u8]> {
    fn hash(&self) -> Hash {
        let mut result = [0u8; 32];
        let input: &[u8] = self.as_ref();
        tiny_keccak::Keccak::keccak256(input, &mut result);
        result
    }
}