use std::io::Write;

use crate::*;

pub struct MemoryAreaReadResponse {
    server_node: u8,
    client_node: u8,
    address: MemoryAddress,
    count: u16,
}

impl MemoryAreaReadResponse {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_raw(&RawMemoryAreaReadResponse {
            fins_header: Header {
                icf: InformationControlField::ResponseWithResponse,
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
            Response_header: RawResponseHeader {
                mrc: 0x01,
                src: 0x01,
            },
            Response_body: RawMemoryAreaReadResponseBody {
                address: self.address.serialize(),
                count: u16be::from_u16(self.count),
            },
        })?;

        Ok(())
    }

    pub const fn byte_size() -> usize {
        ::std::mem::size_of::<RawMemoryAreaReadResponse>()
    }
}

#[repr(C, packed)]
struct RawMemoryAreaReadResponseBody {
    address: RawMemoryAddress,
    count: u16be,
}

#[repr(C, packed)]
struct RawMemoryAreaReadResponse {
    fins_header: RawHeader,
    Response_header: RawResponseHeader,
    Response_body: RawMemoryAreaReadResponseBody,
}

unsafe_impl_raw!(RawMemoryAreaReadResponse);
