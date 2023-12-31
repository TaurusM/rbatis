use crate::arguments::PgArgumentBuffer;
use crate::types::decode::Decode;
use crate::types::encode::{Encode, IsNull};
use crate::value::{PgValue, PgValueFormat};
use rbdc::Error;
use rbs::Value;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::str::FromStr;
use byteorder::{BigEndian, ReadBytesExt};

/// (timestamp,offset sec)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename = "Timestamptz")]
pub struct Timestamptz(pub i64, pub i32);


impl Display for Timestamptz {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timestamptz({})", fastdate::DateTime::from_timestamp_millis(self.0 as i64).set_offset(self.1))
    }
}

impl From<Timestamptz> for Value {
    fn from(arg: Timestamptz) -> Self {
        Value::Ext("Timestamptz", Box::new(Value::Array(vec![Value::I64(arg.0),Value::I32(arg.1)])))
    }
}

impl Encode for Timestamptz {
    fn encode(self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Error> {
        self.0.encode(buf)
    }
}

impl Decode for Timestamptz {
    fn decode(value: PgValue) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                let mut buf = Cursor::new(value.as_bytes()?);
                // TIME is encoded as the microseconds since midnight
                let us = buf.read_i64::<BigEndian>()?;
                // Binary is UTC time
                let offset_seconds = 0;
                // TIMESTAMP is encoded as the microseconds since the epoch
                let epoch = fastdate::DateTime::from(fastdate::Date {
                    day: 1,
                    mon: 1,
                    year: 2000,
                });
                let v = {
                    if us < 0 {
                        epoch - std::time::Duration::from_micros(-us as u64)
                    } else {
                        epoch + std::time::Duration::from_micros(us as u64)
                    }
                };
                Timestamptz(v.unix_timestamp_millis() , offset_seconds)
            }
            PgValueFormat::Text => {
                let s = value.as_str()?;
                let date = fastdate::DateTime::from_str(s)?;
                Timestamptz(date.unix_timestamp_millis() , date.offset())
            }
        })
    }
}


#[cfg(test)]
mod test {
    use crate::types::timestamptz::Timestamptz;

    #[test]
    fn test_de() {
        let tz = Timestamptz(1, 0);
        let v = rbs::to_value(&tz).unwrap();
        println!("{:?}", v);
        let r: Timestamptz = rbs::from_value(v).unwrap();
        assert_eq!(r, tz);
    }
}