use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"Hello World\n");
    buf.put(&b"goodbye world\n"[..]);
    buf.put_i64(0x0102030405060708);

    println!("{:?}", buf);

    let a = buf.split();
    println!("{:?}", a);

    let mut b = a.freeze();
    let c = b.split_to(12);

    println!("{:?}", b);
    println!("{:?}", c);
    println!("{:?}", buf);

    Ok(())
}
