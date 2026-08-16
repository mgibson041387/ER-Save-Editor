use std::io::Error;
use binary_reader::BinaryReader;
use crate::write::write::Write;
use crate::read::read::Read;

pub struct UserData11 {
    unk: [u8;0x10],
    pub regulation: Vec<u8>,
    rest: Vec<u8>,
}

impl Default for UserData11 {
    fn default() -> Self {
        Self { 
            unk: Default::default(), 
            regulation: vec![0; 0x1e9fb0],
            rest: vec![0;0x56050]
        }
    }
}

impl Read for UserData11 {
    fn read(br: &mut BinaryReader) -> Result<UserData11, Error> {
        let mut user_data_11 = UserData11::default();
        user_data_11.unk.copy_from_slice(br.read_bytes(0x10)?);
        user_data_11.regulation.copy_from_slice(br.read_bytes(0x1e9fb0)?);
        user_data_11.rest.copy_from_slice(br.read_bytes(0x56050)?);
        Ok(user_data_11)
    }
}

impl Write for UserData11 {
    fn write(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(self.unk);
        bytes.extend(self.regulation.to_vec());
        bytes.extend(self.rest.to_vec());
        Ok(bytes)
    }
}

impl UserData11 {
    /// The `regulation`/`rest` split is a fixed byte offset that assumes a specific compressed
    /// size for the embedded regulation.bin -- but that size legitimately drifts with every game
    /// patch (it's a compressed blob; compression ratio depends on content). So the split point
    /// isn't reliable as a boundary for where the actual regulation data ends. Concatenating both
    /// fields back together and handing the combined buffer to the regulation parser lets its own
    /// self-describing size fields (in the DCX header) determine the real extent, so this keeps
    /// working regardless of which patch's regulation.bin someone's save was made with. This is
    /// read-only -- `write()` above always emits `regulation` and `rest` back at their original
    /// fixed lengths byte-for-byte, so nothing about the save format's round-trip fidelity changes.
    pub fn regulation_source(&self) -> Vec<u8> {
        let mut combined = self.regulation.clone();
        combined.extend_from_slice(&self.rest);
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_accepts_nonzero_rest_first_byte() {
        // A prior version asserted rest[0] == 0, which turned out to be an unfounded assumption
        // -- a real user's save had a nonzero byte there and was otherwise perfectly valid,
        // getting rejected as "irregular data" for no real reason.
        let mut bytes = vec![0u8; 0x10 + 0x1e9fb0 + 0x56050];
        let rest_start = 0x10 + 0x1e9fb0;
        bytes[rest_start] = 0x42; // nonzero rest[0]

        let mut br = BinaryReader::from_u8(&bytes);
        br.set_endian(binary_reader::Endian::Little);
        let result = UserData11::read(&mut br);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rest[0], 0x42);
    }

    #[test]
    fn regulation_source_concatenates_regulation_and_rest() {
        let mut user_data_11 = UserData11::default();
        user_data_11.regulation = vec![1, 2, 3];
        user_data_11.rest = vec![4, 5];
        assert_eq!(user_data_11.regulation_source(), vec![1, 2, 3, 4, 5]);
    }
}