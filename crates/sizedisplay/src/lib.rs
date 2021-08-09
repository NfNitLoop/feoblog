/// implements Display for a file size.
///
/// ```
/// use sizedisplay::SizeDisplay;
/// assert_eq!("1023 bytes", SizeDisplay::bytes(1023).to_string());
/// assert_eq!("1.00 KiB", SizeDisplay::bytes(1024).to_string());
/// assert_eq!("42.9 MiB", SizeDisplay::bytes(44_983_910).to_string());
/// assert_eq!("16.0 EiB", SizeDisplay::bytes(u64::MAX).to_string());
/// 
/// // Note: whole-number digits are always displayed:
/// assert_eq!("1022 MiB", SizeDisplay::bytes(1_071_644_672).to_string());
/// ```
pub struct SizeDisplay {
    bytes: u64,

    digits: u8,
    short: bool,
}

impl SizeDisplay {

    /// Constructor.
    pub fn bytes(bytes: u64) -> Self {
        Self {bytes, digits: 3, short: false}
    }

    /// Use a single-character postfix.
    pub fn short(mut self) -> Self {
        self.short = true;
        self
    }

    /// How many significant digits to display.
    /// Default 3.  Note: Whole digits are always displayed.
    /// ```
    /// use sizedisplay::SizeDisplay;
    /// assert_eq!("1.234 KiB", SizeDisplay::bytes(1264).digits(4).to_string());
    /// assert_eq!("1 KiB", SizeDisplay::bytes(1264).digits(0).to_string());
    /// assert_eq!("1.5 KiB", SizeDisplay::bytes(1536).digits(2).to_string());
    /// assert_eq!("1.50 KiB", SizeDisplay::bytes(1536).digits(3).to_string());
    /// ```
    pub fn digits(mut self, digits: u8) -> Self {
        self.digits = digits;
        self
    }
}

const SIZES: &[&str] = &[
    " bytes",
    " KiB",
    " MiB",
    " GiB",
    " TiB",
    " PiB",
    " EiB",
];

const SHORT_SIZES: &[&str] = &[
    "b",
    "K",
    "M",
    "G",
    "T",
    "P",
    "E",
];


impl std::fmt::Display for SizeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 0 is unambiguous.
        if self.bytes == 0 {
            return write!(f, "0");
        }

        let mut postfixes = if self.short {
            SHORT_SIZES.iter().cloned().peekable()
        } else {
            SIZES.iter().cloned().peekable()
        };

        let mut postfix = postfixes.next().unwrap_or("!?");
        // No fractions for bytes:
        if self.bytes < 1024 {
            return write!(f, "{}{}", self.bytes, postfix);
        }

        let mut num = self.bytes as f64;
        while num >= 1024.0 && postfixes.peek().is_some() {
            num = num / 1024.0;
            postfix = postfixes.next().expect("the value we just peeked");
        }

        // Rust's "precision" means digits after the decimal point. 🤦‍♂️
        // So we've got to do math to figure how many digits to show if we
        // already have some.
        // We never want to hide whole digits.
        let wholes = whole_digits(num);
        let frac_digs = if wholes > self.digits as u32 {
            0
        } else {
            self.digits as usize - wholes as usize
        };

        write!(f, "{1:.0$}{2}", frac_digs, num, postfix)
    }
}

fn whole_digits(num: f64) -> u32 {
    num.log10().floor() as u32 + 1
}