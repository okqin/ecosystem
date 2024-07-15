use anyhow::Result;
use derive_more::FromStr;
use strum::{Display, EnumCount, EnumIter, EnumString, IntoEnumIterator, VariantArray};

#[derive(Debug, EnumString, EnumCount, Display, EnumIter, VariantArray)]
enum MyEnum {
    A,
    B,
    C,
}

fn main() -> Result<()> {
    println!("{:?}", MyEnum::VARIANTS);

    for variant in MyEnum::iter() {
        println!("{}", variant);
    }

    println!("MyEnum count:{}", MyEnum::COUNT);

    let variant = MyEnum::from_str("A")?;
    println!("MyEnum variant:{}", variant);
    Ok(())
}
