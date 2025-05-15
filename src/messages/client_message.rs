use bytes::BytesMut;

pub struct ClientMessage {
    header: u16,
    body: BytesMut,
}

impl ClientMessage {
    pub fn new(header: u16, body: BytesMut) -> Self {
        Self { header, body }
    }

    pub fn get_header(&self) -> u16 {
        self.header
    }

    pub fn get_body(&self) -> &BytesMut {
        &self.body
    }

    pub fn get_body_mut(&mut self) -> &mut BytesMut {
        &mut self.body
    }
}