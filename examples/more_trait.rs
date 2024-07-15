use anyhow::Result;
use derive_more::{Add, Deref, Display, From, Into};

#[derive(PartialEq, From, Add, Debug, Deref)]
struct MyInt(i32);

#[derive(PartialEq, From, Into)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, From, Add, Display)]
enum MyEnum {
    #[display("int: {_0}")]
    Int(i32),
    Uint(u32),
    #[display("nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int = MyInt(5) + 6.into();
    println!("my_int: {:?}", my_int);
    println!("my_int + 2: {:?}", my_int.add(2.into()));

    let point: (i32, i32) = Point2D { x: 5, y: 6 }.into();
    println!("point: {:?}", point);

    let my_enum = MyEnum::Int(5) + 6.into();
    println!("MyEnum::Int(5) + 6 = {}", my_enum?);
    println!("{}", MyEnum::Int(15));
    println!("{}", MyEnum::Uint(42));
    println!("{}", MyEnum::Nothing);

    Ok(())
}
