use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Debug, Clone)]
enum Sensor {
    Temperature,
    Pressure,
    Humidity,
    Lightning,
    Vibration,
    Smoke,
}

#[allow(unused)]
#[derive(Builder, Debug)]
#[builder(setter(into))]
struct Device {
    id: u64,

    name: String,

    device_type: Sensor,

    #[builder(setter(custom))]
    create_date: DateTime<Utc>,

    #[builder(setter(skip), default = "self.running_time_default()")]
    running_time: u64,

    #[builder(default, setter(into, strip_option))]
    location: Option<String>,

    #[builder(default = "true")]
    is_active: bool,

    #[builder(default = "vec![]", setter(each(name = "data_value")))]
    data: Vec<String>,
}

fn main() -> Result<()> {
    let device = Device::builder()
        .id("29388844402912".parse::<u64>()?)
        .name("temperature sensor")
        .device_type(Sensor::Temperature)
        .create_date("2024-07-01T12:34:56Z")
        .data_value("30.3".into())
        .data_value("26.8".into())
        .build()?;

    println!("{:#?}", device);

    Ok(())
}

impl Device {
    pub fn builder() -> DeviceBuilder {
        DeviceBuilder::default()
    }
}

impl DeviceBuilder {
    pub fn create_date(&mut self, value: &str) -> &mut Self {
        self.create_date = value.parse::<DateTime<Utc>>().ok();
        self
    }

    fn running_time_default(&self) -> u64 {
        match self.create_date {
            Some(date) => {
                let time = Utc::now() - date;
                time.num_days() as u64
            }
            None => 0,
        }
    }
}
