use super::*;

#[test]
fn test_u32_leb128_encode() {
    let mut buffer = vec![];
    123u32.encode(&mut buffer);
    assert_eq!(buffer, vec![0x7b]);

    let mut buffer = vec![];
    101010u32.encode(&mut buffer);
    assert_eq!(buffer, vec![0x92, 0x95, 0x6]);
}

#[test]
fn test_u32_leb128_decode() {
    assert_eq!(u32::decode(&[0x7b]).unwrap(), (123u32, [].as_ref()));
    assert_eq!(
        u32::decode(&[0x92, 0x95, 0x6, 0x12]).unwrap(),
        (101010u32, [0x12].as_ref())
    );
}

#[test]
fn test_i32_leb128_encode() {
    let mut buffer = vec![];
    (-123456i32).encode(&mut buffer);
    assert_eq!(buffer, vec![0xc0, 0xbb, 0x78]);
}

#[test]
fn test_i32_leb128_decode() {
    assert_eq!(
        i32::decode(&[0xc0, 0xbb, 0x78]).unwrap(),
        (-123456i32, [].as_ref())
    );
}

#[test]
fn test_repeated_decode() {
    let bytes = [0x02, 0x03, 0x01, 0x00, 0x00];
    let (xs, rest): (Repeated<u8>, _) = Repeated::decode(&bytes).unwrap();
    assert_eq!(rest, []);
    assert_eq!(xs.0, bytes);
}

#[test]
fn test_custom_section_decode() {
    let bytes = [
        0x04, // section name size = 4
        0x6E, 0x61, 0x6D, 0x65, // "name"
        0x02, 0x03, 0x01, 0x00, 0x00, // random
    ];
    let (Custom(name, contents), rest) = Custom::decode(&bytes).unwrap();
    assert_eq!(rest, []);
    assert_eq!(name.0.as_str(), "name");
    assert_eq!(contents.0, [0x02, 0x03, 0x01, 0x00, 0x00]);
}
