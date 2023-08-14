pub mod register_client;
pub mod upload;

#[macro_export]
macro_rules! impl_codec {
    ($req_msg:path, $resp_msg:path) => {
        use tokio_util::codec::LengthDelimitedCodec;

        pub struct ServerCodec {
            len_codec: LengthDelimitedCodec,
        }

        pub struct ClientCodec {
            len_codec: LengthDelimitedCodec,
        }

        impl ServerCodec {
            pub fn new() -> Self {
                Self {
                    len_codec: Default::default(),
                }
            }
        }

        impl ClientCodec {
            pub fn new() -> Self {
                Self {
                    len_codec: Default::default(),
                }
            }
        }

        crate::impl_codec!(ServerCodec, $resp_msg, $req_msg);
        crate::impl_codec!(ClientCodec, $req_msg, $resp_msg);
    };

    ($codec:path, $encode_msg:path, $decode_msg:path) => {
        const _: () = {
            use bytes::{Bytes, BytesMut};
            use tokio_util::codec::{Decoder, Encoder};

            impl Decoder for $codec {
                type Item = $decode_msg;

                type Error = anyhow::Error;

                fn decode(
                    &mut self,
                    src: &mut BytesMut,
                ) -> std::result::Result<Option<Self::Item>, Self::Error> {
                    let bytes = match self.len_codec.decode(src)? {
                        Some(b) => b,
                        None => return Ok(None),
                    };
                    let msg = serde_json::from_slice(&bytes)?;
                    Ok(Some(msg))
                }
            }

            impl Encoder<$encode_msg> for $codec {
                type Error = anyhow::Error;

                fn encode(
                    &mut self,
                    item: $encode_msg,
                    dst: &mut BytesMut,
                ) -> std::result::Result<(), Self::Error> {
                    let msg = serde_json::to_vec(&item).unwrap();
                    self.len_codec.encode(Bytes::copy_from_slice(&msg), dst)?;
                    Ok(())
                }
            }
        };
    };
}
