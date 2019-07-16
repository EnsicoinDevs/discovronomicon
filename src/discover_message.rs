use bytes::BytesMut;
use std::convert::TryInto;
use tokio::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum Message {
    Get { protocol: String },
    Ping,
    Identity { protocol: String, address: String },
}

impl Message {
    fn get_type(&self) -> MessageType {
        match &self {
            Message::Get { protocol: _ } => MessageType::Get,
            Message::Identity {
                protocol: _,
                address: _,
            } => MessageType::Identity,
            Message::Ping => MessageType::Ping,
        }
    }
}

enum MessageType {
    Get,
    Ping,
    Identity,
}

impl MessageType {
    fn section_count(&self) -> usize {
        match self {
            MessageType::Get => 2,
            MessageType::Ping => 0,
            MessageType::Identity => 4,
        }
    }
    fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::from(
            match self {
                MessageType::Get => "get",
                MessageType::Identity => "identity",
                MessageType::Ping => "ping",
            }
            .as_bytes(),
        );
        bytes.resize(10, 0);
        bytes
    }
}

pub struct MessageCodec {
    message_type: Option<MessageType>,
    payload_section_read: usize,
    next_section_length: usize,
    sections: [BytesMut; 2],
}

#[derive(Debug)]
pub enum MessageError {
    ProtocolTooLong,
    InvalidString,
    InvalidMagic(u16),
    InvalidType(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for MessageError {
    fn from(err: std::io::Error) -> Self {
        MessageError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for MessageError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        MessageError::InvalidString
    }
}

impl MessageCodec {
    pub fn new() -> Self {
        MessageCodec {
            message_type: None,
            payload_section_read: 0,
            next_section_length: 0,
            sections: [BytesMut::new(), BytesMut::new()],
        }
    }
}

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = MessageError;

    fn encode(&mut self, message: Message, buf: &mut BytesMut) -> Result<(), MessageError> {
        buf.extend_from_slice(&(555 as u16).to_be_bytes());
        buf.extend(message.get_type().to_bytes());
        match message {
            Message::Get { protocol: p } => {
                buf.extend_from_slice(&(p.len() as u64).to_be_bytes());
                buf.extend_from_slice(&p.as_bytes());
            }
            Message::Ping => (),
            Message::Identity {
                protocol: p,
                address: a,
            } => {
                buf.extend_from_slice(&(p.len() as u64).to_be_bytes());
                buf.extend_from_slice(&p.as_bytes());
                buf.extend_from_slice(&(a.len() as u64).to_be_bytes());
                buf.extend_from_slice(&a.as_bytes());
            }
        }
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = MessageError;

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if self.message_type.is_none() {
                if buf.len() < 12 {
                    return Ok(None);
                }
                let magic = buf.split_to(2);
                let magic = u16::from_be_bytes(magic.as_ref().try_into().unwrap());
                if magic != 555 {
                    return Err(MessageError::InvalidMagic(magic));
                }
                let message_type = buf.split_to(10);
                let message_type = String::from_utf8(message_type.to_vec())?;
                self.message_type = Some(match message_type.trim_end_matches('\u{0}') {
                    "get" => MessageType::Get,
                    "identity" => MessageType::Identity,
                    "ping" => MessageType::Ping,
                    _ => return Err(MessageError::InvalidType(message_type)),
                });
                self.payload_section_read = 0;
                self.next_section_length = if let Some(MessageType::Ping) = self.message_type {
                    0
                } else {
                    8
                };
            } else {
                if self.payload_section_read == self.message_type.as_ref().unwrap().section_count()
                {
                    return Ok(Some(match self.message_type.take().unwrap() {
                        MessageType::Get => {
                            let protocol = String::from_utf8(self.sections[0].to_vec())?;
                            Message::Get { protocol }
                        }
                        MessageType::Identity => {
                            let protocol = String::from_utf8(self.sections[0].to_vec())?;
                            let address = String::from_utf8(self.sections[1].to_vec())?;
                            Message::Identity { protocol, address }
                        }
                        MessageType::Ping => Message::Ping,
                    }));
                } else if buf.len() >= self.next_section_length {
                    match self.message_type.as_ref().unwrap() {
                        MessageType::Ping => unreachable!(),
                        MessageType::Get => match self.payload_section_read {
                            0 => {
                                self.next_section_length = u64::from_be_bytes(
                                    buf.split_to(8).as_ref().try_into().unwrap(),
                                )
                                    as usize;
                                if self.next_section_length > 100 {
                                    return Err(Self::Error::ProtocolTooLong);
                                }
                            }
                            1 => {
                                self.sections[0] = buf.split_to(self.next_section_length);
                                self.next_section_length = 8;
                            }
                            _ => unreachable!(),
                        },
                        MessageType::Identity => match self.payload_section_read {
                            0 | 2 => {
                                self.next_section_length = u64::from_be_bytes(
                                    buf.split_to(8).as_ref().try_into().unwrap(),
                                )
                                    as usize;
                                if self.payload_section_read == 0 && self.next_section_length > 100
                                {
                                    return Err(Self::Error::ProtocolTooLong);
                                }
                            }
                            1 => {
                                self.sections[0] = buf.split_to(self.next_section_length);
                                self.next_section_length = 8;
                            }
                            3 => {
                                self.sections[1] = buf.split_to(self.next_section_length);
                                self.next_section_length = 8;
                            }
                            _ => unreachable!(),
                        },
                    }
                    self.payload_section_read += 1;
                } else {
                    return Ok(None);
                }
            }
        }
    }
}
