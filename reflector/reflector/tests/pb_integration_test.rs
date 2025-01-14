
#[test]
fn can_decode_own_encoding() {
    // lg::Msg{}

    // let foo = lg::Foo { foole: 42 };
    // let mut buf = Vec::new();
    // foo.encode(&mut buf).unwrap();
    // let bar = lg::Foo::decode(&mut Cursor::new(buf)).unwrap();
    // assert!(bar.foole == 42);
}

#[test]
fn can_decode_what_c_encoded() {
    // let filename = "../../components/codec/test.bin";
    // let mut f = File::open(filename).unwrap();
    // let metadata = fs::metadata(&filename).expect("unable to read metadata");
    // let mut buffer = vec![0; metadata.len() as usize];
    // f.read(&mut buffer).expect("buffer overflow");
    // let bar = lg::Foo::decode(&mut Cursor::new(buffer)).unwrap();
    // assert!(bar.foole == 42);
}
