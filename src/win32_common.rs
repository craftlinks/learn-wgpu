pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
}

impl ToWide for &str {
    fn to_wide(&self) -> Vec<u16> {
        let mut result: Vec<u16> = self.encode_utf16().collect();
        result.push(0);
        result
    }
}
