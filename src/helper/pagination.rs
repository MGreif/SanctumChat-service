pub struct Pagination {
    pub size: u8,
    pub index: u8,
}

impl Pagination {
    pub fn new(size: Option<u8>, index: Option<u8>) -> Self {
        let mut s: u8 = 10;
        let mut i: u8 = 0;

        if let Some(size) = size {
            s = size
        }

        if let Some(index) = index {
            i = index
        }

        return Self { size: s, index: i }
    }
}