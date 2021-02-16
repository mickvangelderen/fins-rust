use fins_util::unsafe_impl_raw;

#[derive(Debug)]
#[repr(C, align(1))]
pub struct RawHeader {
    /// Information Control Field
    pub icf: u8,

    /// Reserved
    pub rsv: u8,
    
    /// Permissible number of gateways
    pub gct: u8,

    /// Destination Network Address
    pub dna: u8,

    /// Destination Node Address
    pub da1: u8,

    /// Destination Unit Address
    pub da2: u8,

    /// Source Network Address
    pub sna: u8,

    /// Source Node Address
    pub sa1: u8,

    /// Source Unit Address
    pub sa2: u8,

    /// Service ID
    /// Set by process to identify which one it came from.
    pub sid: u8,
}

unsafe_impl_raw!(RawHeader);
