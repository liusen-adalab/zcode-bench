use serde::{ser::SerializeStruct, Serialize};

pub type MyResult<T, E = MyErr> = std::result::Result<T, E>;

pub struct MyErr {
    err: anyhow::Error,
}

impl From<anyhow::Error> for MyErr {
    fn from(value: anyhow::Error) -> Self {
        Self { err: value }
    }
}

impl From<MyErr> for anyhow::Error {
    fn from(value: MyErr) -> Self {
        value.err
    }
}

impl Serialize for MyErr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Error", 1)?;
        s.serialize_field("msg", &format!("{:?}", self.err))?;
        s.end()
    }
}
