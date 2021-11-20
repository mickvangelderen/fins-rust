use std::io::Write;

use crate::*;

pub struct MemoryAreaReadRequest {
    pub server_node: u8,
    pub client_node: u8,
    pub address: MemoryAddress,
    pub count: u16,
}

impl MemoryAreaReadRequest {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_raw(&RawMemoryAreaReadRequest {
            fins_header: Header {
                icf: InformationControlField::RequestWithResponse,
                gct: 0x02,
                destination: MachineAddress {
                    network: 0,
                    node: self.server_node,
                    unit: 0,
                },
                source: MachineAddress {
                    network: 0,
                    node: self.client_node,
                    unit: 0,
                },
                sid: 0,
            }
            .serialize(),
            request_header: RawRequestHeader {
                mrc: 0x01,
                src: 0x01,
            },
            request_body: RawMemoryAreaReadRequestBody {
                address: self.address.serialize(),
                count: u16be::from_u16(self.count),
            },
        })?;

        Ok(())
    }

    pub const fn byte_size() -> usize {
        ::std::mem::size_of::<RawMemoryAreaReadRequest>()
    }
}

#[repr(C, packed)]
struct RawMemoryAreaReadRequestBody {
    address: RawMemoryAddress,
    count: u16be,
}

#[repr(C, packed)]
struct RawMemoryAreaReadRequest {
    fins_header: RawHeader,
    request_header: RawRequestHeader,
    request_body: RawMemoryAreaReadRequestBody,
}

unsafe_impl_raw!(RawMemoryAreaReadRequest);
