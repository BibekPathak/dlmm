use anchor_lang::prelude::*;

pub const BINS_PER_ARRAY: usize = 64;

pub fn bin_id_to_array_start(bin_id: i32) -> i32 {
    let rem = bin_id.rem_euclid(BINS_PER_ARRAY as i32);
    bin_id - rem
}

#[zero_copy]
#[repr(C)]
pub struct Bin {
    pub amount_x: u64,
    pub amount_y: u64,
    pub fee_x: u64,
    pub fee_y: u64,
}

#[account(zero_copy)]
#[repr(C)]
pub struct BinArray {
    pub pool: Pubkey,
    pub start_bin_id: i32,
    pub bump: u8,
    pub reserved: [u8; 3],
    pub bins: [Bin; BINS_PER_ARRAY],
}

impl BinArray {
    pub const LEN: usize = 8
        + 32 // pool
        + 4 // start bin id
        + 1 // bump
        + 3 // reserved padding
        + (32 * BINS_PER_ARRAY); // bins

    pub fn bin_index(&self, bin_id: i32) -> Option<usize> {
        let offset = bin_id.checked_sub(self.start_bin_id)?;
        if offset < 0 || offset as usize >= BINS_PER_ARRAY {
            return None;
        }
        Some(offset as usize)
    }

    pub fn get_bin(&self, bin_id: i32) -> std::result::Result<&Bin, crate::errors::DlmmError> {
        let idx = self.bin_index(bin_id).ok_or(crate::errors::DlmmError::BinIdOutOfBounds)?;
        Ok(&self.bins[idx])
    }

    pub fn get_bin_mut(&mut self, bin_id: i32) -> std::result::Result<&mut Bin, crate::errors::DlmmError> {
        let idx = self.bin_index(bin_id).ok_or(crate::errors::DlmmError::BinIdOutOfBounds)?;
        Ok(&mut self.bins[idx])
    }

    pub fn contains(&self, bin_id: i32) -> bool {
        self.bin_index(bin_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_id_to_array_start() {
        assert_eq!(bin_id_to_array_start(0), 0);
        assert_eq!(bin_id_to_array_start(63), 0);
        assert_eq!(bin_id_to_array_start(64), 64);
        assert_eq!(bin_id_to_array_start(-1), -64);
        assert_eq!(bin_id_to_array_start(-64), -64);
        assert_eq!(bin_id_to_array_start(-65), -128);
    }

    #[test]
    fn test_bin_index() {
        let mut ba = BinArray {
            pool: Pubkey::default(),
            start_bin_id: 0,
            bump: 0,
            reserved: [0; 3],
            bins: [Bin { amount_x: 0, amount_y: 0, fee_x: 0, fee_y: 0 }; BINS_PER_ARRAY],
        };
        ba.bins[5].amount_x = 42;

        assert_eq!(ba.bin_index(0), Some(0));
        assert_eq!(ba.bin_index(5), Some(5));
        assert_eq!(ba.bin_index(63), Some(63));
        assert_eq!(ba.bin_index(64), None);
        assert_eq!(ba.bin_index(-1), None);

        assert_eq!(ba.get_bin(5).unwrap().amount_x, 42);
        assert!(ba.get_bin(100).is_err());
    }

    #[test]
    fn test_bin_index_negative_start() {
        let ba = BinArray {
            pool: Pubkey::default(),
            start_bin_id: -64,
            bump: 0,
            reserved: [0; 3],
            bins: [Bin { amount_x: 0, amount_y: 0, fee_x: 0, fee_y: 0 }; BINS_PER_ARRAY],
        };

        assert_eq!(ba.bin_index(-64), Some(0));
        assert_eq!(ba.bin_index(-1), Some(63));
        assert_eq!(ba.bin_index(0), None);
        assert_eq!(ba.bin_index(-65), None);
    }

    #[test]
    fn test_contains() {
        let ba = BinArray {
            pool: Pubkey::default(),
            start_bin_id: 0,
            bump: 0,
            reserved: [0; 3],
            bins: [Bin { amount_x: 0, amount_y: 0, fee_x: 0, fee_y: 0 }; BINS_PER_ARRAY],
        };
        assert!(ba.contains(0));
        assert!(ba.contains(63));
        assert!(!ba.contains(64));
        assert!(!ba.contains(-1));
    }

    #[test]
    fn test_bin_mut() {
        let mut ba = BinArray {
            pool: Pubkey::default(),
            start_bin_id: 0,
            bump: 0,
            reserved: [0; 3],
            bins: [Bin { amount_x: 0, amount_y: 0, fee_x: 0, fee_y: 0 }; BINS_PER_ARRAY],
        };
        *ba.get_bin_mut(10).unwrap() = Bin { amount_x: 100, amount_y: 200, fee_x: 5, fee_y: 10 };
        let b = ba.get_bin(10).unwrap();
        assert_eq!(b.amount_x, 100);
        assert_eq!(b.amount_y, 200);
        assert_eq!(b.fee_x, 5);
        assert_eq!(b.fee_y, 10);
    }
}
