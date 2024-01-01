//! Random utilities.

use std::fmt::Display;


/// Lets you get a Display that will print some bytes as hex.
pub(crate) trait AsHex {
    type Displayer: Display;
    fn as_hex(&self) -> Self::Displayer;
}

impl <'a> AsHex for &'a [u8] {
    type Displayer = AsHexDisplayer<'a>;

    fn as_hex(&self) -> Self::Displayer {
        AsHexDisplayer{ bytes: self }
    }
}

pub(crate) struct AsHexDisplayer<'a> {
    bytes: &'a [u8]
}

impl <'a> Display for AsHexDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AsHex;

    #[test]
    fn example() {
        let bytes = [0u8, 255, 88, 42];
        assert_eq!("00ff582a", format!("{}", bytes.as_slice().as_hex()));
    }
}