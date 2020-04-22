use stm32f4xx_hal as hal;

use hal::{
    prelude::*,
    i2c::{I2c, Pins},
    stm32::I2C1,
    rcc::Clocks,
};


const I2C_ADDRESS: u8 = 0x68;

pub enum Hours {
    Am(u8),
    Pm(u8),
    H24(u8)
}

pub enum DayOfWeek {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun
}

impl DayOfWeek {
    pub fn idx(&self) -> u8 {
        match self {
            DayOfWeek::Mon => 0,
            DayOfWeek::Tue => 1,
            DayOfWeek::Wed => 2,
            DayOfWeek::Thu => 3,
            DayOfWeek::Fri => 4,
            DayOfWeek::Sat => 5,
            DayOfWeek::Sun => 6,
        }
    }
    pub fn from_idx(idx: u8) -> Option<Self> {
        match idx {
            0 => Some(DayOfWeek::Mon),
            1 => Some(DayOfWeek::Tue),
            2 => Some(DayOfWeek::Wed),
            3 => Some(DayOfWeek::Thu),
            4 => Some(DayOfWeek::Fri),
            5 => Some(DayOfWeek::Sat),
            6 => Some(DayOfWeek::Sun),
            _ => None
        }
    }
}

pub struct DateTime {
    data: [u8; 7],
}

pub struct Ds3231<PINS>
where
    PINS: Pins<I2C1>
{
    i2c: I2c<I2C1, PINS>,
}

impl<PINS> Ds3231<PINS>
where
    PINS: Pins<I2C1>
{
    pub fn new(i2c1: I2C1, pins: PINS, clocks: Clocks) -> Self {
        let i2c = I2c::i2c1(i2c1, pins, 400.khz(), clocks);
        Self {i2c}
    }

    pub fn read_date_time(&mut self) -> DateTime {
        let mut data = [0; 7];
        self.i2c.write_read(0x68, &[0], &mut data).unwrap();
        DateTime {data}
    }

    fn write_bcd(&mut self, cmd: u8, val: u8) {
        let low4 = val % 10;
        let high4 = (val / 10) & 0b1111;
        let to_write = low4 + (high4 << 4);
        self.i2c.write(I2C_ADDRESS, &[cmd, to_write]).unwrap();
    }

    #[allow(dead_code)]
    pub fn set_raw(&mut self, idx: u8, val: u8) {
        self.i2c.write(I2C_ADDRESS, &[idx, val]).unwrap();
    }

    #[allow(dead_code)]
    pub fn set_secs(&mut self, secs: u8) {
        self.write_bcd(0x00, secs);
    }

    #[allow(dead_code)]
    pub fn set_mins(&mut self, secs: u8) {
        self.write_bcd(0x01, secs);
    }

    #[allow(dead_code)]
    pub fn set_hours(&mut self, hours: Hours) {
        match hours {
            Hours::H24(v) => {
                self.write_bcd(0x02, v);
            }
            Hours::Am(v) => {
                self.write_bcd(0x02, v + 40);
            }
            Hours::Pm(v) => {
                self.write_bcd(0x02, v + 60);
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_day_of_week(&mut self, day_idx: u8) {
        self.write_bcd(0x03, day_idx + 1);
    }

    #[allow(dead_code)]
    pub fn set_day(&mut self, day: DayOfWeek) {
        self.write_bcd(0x04, day.idx());
    }

    #[allow(dead_code)]
    pub fn set_year_and_month(&mut self, year: u16, mut month: u8) {
        if year >= 2000 {
            month += 80;
        }
        self.write_bcd(0x05, month);
        self.write_bcd(0x06, (year % 100) as u8);
    }
}

impl DateTime {
    fn decode_bcd(&self, idx: usize) -> u8 {
        (self.data[idx] & 0b1111) + self.data[idx].overflowing_shr(4).0 * 10
    }

    pub fn secs(&self) -> u8 {
        self.decode_bcd(0)
    }

    pub fn mins(&self) -> u8 {
        self.decode_bcd(1)
    }

    pub fn hours(&self) -> Hours {
        let mut val = self.data[2] & 0b1111;
        if (self.data[2] & 0b0100_0000) > 0 {
            if (self.data[2] & 0b0001_0000) > 0 {
                val += 10;
            }
            if (self.data[2] & 0b0010_0000) > 0 {
                return Hours::Pm(val);
            }
            return Hours::Am(val);
        }
        return Hours::H24(((self.data[2] & 0b0011_0000).overflowing_shr(4).0 * 10) + val);
    }

    pub fn day_of_week(&self) -> Option<DayOfWeek> {
        DayOfWeek::from_idx(self.data[3] - 1)
    }

    pub fn day(&self) -> u8 {
        self.decode_bcd(4)
    }

    pub fn month(&self) -> u8 {
        (self.data[5] & 0b1111)
            + (self.data[5] & 0b0111_000).overflowing_shr(4).0 * 10
    }

    pub fn year(&self) -> u16 {
        let mut result = self.decode_bcd(6) as u16;
        if (self.data[5] & 0b1000_0000) > 0 {
            result += 2000;
        } else {
            result += 1900;
        }
        result
    }
}
