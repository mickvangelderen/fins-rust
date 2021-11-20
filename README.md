
FINS frames have a finite maximum length.
This means that if memory permits we do not need to do a streaming write to the network.
This means that we can serialize messages and deserialize complete messages to and from a buffer and read/write that in one operation.

Serialization should write to memory sequentially.
This means that for headers which include some information derived from the body, like the a length field, we need to be able to compute the length of the body without serializing the body.
I suspect that since no complex (entropy reducing?) serialization, like compression, is involved, we should be able to compute these statistics cheaply which allows us to achieve the above.
Alternatively, we could opt for non-sequential writes and finalize headers after serializing the body.

| sequential write | read once | compute body length         |
| ---------------- | --------- | --------------------------- |
| yes              | no        | during header serialization |
| no               | yes       | during body serialization   |

We should really compare the above aproaches in

1. implementation complexity
2. performance

to figure out which one is superior.

The sequential write method can use the `Write` interface and the body needs to implement some sort of `ByteLength` interface.
The single read method can use

```rust
struct Request {
    body: Body,
}

impl Request {
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize> {
        let (header_buffer, body_buffer) = buffer.split_at_mut(4);
        let body_bytes = self.body.serialize(body_buffer)?;
        buffer.write_exact(u32be::from_u32(body_bytes.try_into().unwrap()).to_bytes())?;
        4 + body_bytes
    }
}
