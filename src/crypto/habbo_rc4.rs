/// RC4 implementation for Habbo protocol encryption/decryption
pub struct HabboRC4 {
    i: usize,
    j: usize,
    table: [u8; 256],
}

impl HabboRC4 {
    /// Create a new RC4 cipher with the given key
    pub fn new(key: &[u8]) -> Self {
        let mut rc4 = HabboRC4 {
            i: 0,
            j: 0,
            table: [0; 256],
        };

        let length = key.len();

        // Initialize table with identity values
        while rc4.i < 256 {
            rc4.table[rc4.i] = rc4.i as u8;
            rc4.i += 1;
        }

        rc4.i = 0;
        rc4.j = 0;

        // Key scheduling algorithm
        while rc4.i < 256 {
            rc4.j = (rc4.j + rc4.table[rc4.i] as usize + (key[rc4.i % length] & 0xff) as usize) % 256;
            rc4.swap(rc4.i, rc4.j);
            rc4.i += 1;
        }

        rc4.i = 0;
        rc4.j = 0;

        rc4
    }

    /// Swap two values in the state table
    fn swap(&mut self, a: usize, b: usize) {
        let temp = self.table[a];
        self.table[a] = self.table[b];
        self.table[b] = temp;
    }

    /// Parse and modify the input bytes with RC4 cipher in place
    pub fn parse(&mut self, bytes: &mut [u8]) {
        for index in 0..bytes.len() {
            self.i = (self.i + 1) % 256;
            self.j = (self.j + self.table[self.i] as usize) % 256;
            self.swap(self.i, self.j);

            let k = (self.table[self.i] as usize + self.table[self.j] as usize) % 256;
            bytes[index] = bytes[index] ^ self.table[k];
        }
    }
}
