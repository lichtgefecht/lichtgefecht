pub mod lichtgefecht {
    include!(concat!(env!("OUT_DIR"), "/lichtgefecht.rs"));
}

// use lichtgefecht::what;

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use std::io::Cursor;

    #[test]
    fn can_decode_own_encoding() {
        let foo = lichtgefecht::Foo { foole: 42 };
        let mut buf = Vec::new();
        foo.encode(&mut buf).unwrap();
        let bar = lichtgefecht::Foo::decode(&mut Cursor::new(buf)).unwrap();
        assert!(bar.foole == 42);
    }
}
