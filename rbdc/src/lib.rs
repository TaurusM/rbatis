use std::collections::HashMap;
use indexmap::IndexMap;

pub mod encode;
pub mod decode;
pub mod db;

///Rbatis Object Notation
pub enum RBON {
    Null,
    String(String),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Bytes(Vec<u8>),
    Map(IndexMap<String, RBON>),
    Struct(String, Vec<(String, RBON)>),
    Array(Vec<RBON>),
    Enum((String, Box<RBON>)),
}

#[cfg(test)]
mod test {
    use crate::RBON;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct A {
        name: String,
        age: i32,
    }

    pub trait QPS {
        fn qps(&self, total: u64);
        fn time(&self, total: u64);
        fn cost(&self);
    }

    impl QPS for std::time::Instant {
        fn qps(&self, total: u64) {
            let time = self.elapsed();
            println!(
                "use QPS: {} QPS/s",
                (total as u128 * 1000000000 as u128 / time.as_nanos() as u128)
            );
        }

        fn time(&self, total: u64) {
            let time = self.elapsed();
            println!(
                "use Time: {:?} ,each:{} ns/op",
                &time,
                time.as_nanos() / (total as u128)
            );
        }

        fn cost(&self) {
            let time = self.elapsed();
            println!("cost:{:?}", time);
        }
    }
    #[macro_export]
    macro_rules! mbench {
    ($total:expr,$body:block) => {
       {
        let now = std::time::Instant::now();
        for _ in 0..$total {
            $body;
        }
        now.time($total);
        now.qps($total);
       }
    };
}
    #[test]
    fn test_bench_ser() {
        let a = A {
            name: "1".to_string(),
            age: 2,
        };
        mbench!(100000, {
        let a = A{
            name: "1".to_string(),
            age: 2
        };
            let b=RBON::Struct("A".to_string(),vec![("name".to_string(),RBON::String("1".to_string())),("age".to_string(),RBON::I32(2))]);
        });
    }
}