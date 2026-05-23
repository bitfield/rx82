#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Bus {
    pub addr: u16,
    pub data: u8,
    pub dirty: bool,
    pub mem: bool,
    pub write: bool,
}
