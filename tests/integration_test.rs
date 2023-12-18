use pnghider::utils::common::{Pngchunk, PNG_CUSTOMCHUNK};

#[test]
fn creates_png_chunk_content() {
    let mut testchunk: Pngchunk = Default::default();
    let content = "test";
    testchunk.create_from_content(PNG_CUSTOMCHUNK, content.as_bytes().to_vec());
    assert_eq!(&"test".as_bytes().to_vec(), testchunk.content())
}

#[test]
fn creates_png_chunk_length() {
    let mut testchunk: Pngchunk = Default::default();
    let content = "test";
    testchunk.create_from_content(PNG_CUSTOMCHUNK, content.as_bytes().to_vec());
    assert_eq!(16, testchunk.len())
}

#[test]
fn creates_png_chunk_flatten() {
    let mut testchunk: Pngchunk = Default::default();
    let content = "test";
    testchunk.create_from_content(PNG_CUSTOMCHUNK, content.as_bytes().to_vec());
    assert_eq!(
        [0, 0, 0, 4, 13, 37, 13, 37, 116, 101, 115, 116, 108, 61, 134, 51].to_vec(),
        testchunk.flatten()
    )
}
