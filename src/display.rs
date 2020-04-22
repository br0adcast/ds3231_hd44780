use core::convert::Infallible;
use core::fmt::Write;

use embedded_hal::digital::v2::OutputPin;

use stm32f4xx_hal as hal;
use hal::{
    delay::Delay,
};

use heapless::consts::*;
use heapless::String;

use hd44780_driver::{Cursor, CursorBlink, bus::{FourBitBus}, DisplayMode, HD44780};

use super::ds3231;

pub struct Display<RS, EN, D4, D5, D6, D7>
where
    RS: OutputPin<Error = Infallible>,
    EN: OutputPin<Error = Infallible>,
    D4: OutputPin<Error = Infallible>,
    D5: OutputPin<Error = Infallible>,
    D6: OutputPin<Error = Infallible>,
    D7: OutputPin<Error = Infallible>,
{
    lcd: HD44780<Delay, FourBitBus<RS, EN, D4, D5, D6, D7>>,
}

impl<RS, EN, D4, D5, D6, D7> Display<RS, EN, D4, D5, D6, D7>
where
    RS: OutputPin<Error = Infallible>,
    EN: OutputPin<Error = Infallible>,
    D4: OutputPin<Error = Infallible>,
    D5: OutputPin<Error = Infallible>,
    D6: OutputPin<Error = Infallible>,
    D7: OutputPin<Error = Infallible>,
{
    pub fn new(rs: RS, en: EN, d4: D4, d5: D5, d6: D6, d7: D7, delay: Delay) -> Self {
        let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, delay);
        lcd.reset();
        lcd.clear();
        lcd.set_display_mode(
            DisplayMode {
                display: hd44780_driver::Display::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            }
        );
        Self {lcd}
    }

    pub fn draw_date_time(&mut self, date_time: &ds3231::DateTime) {
        let hours_val = match date_time.hours() {
            ds3231::Hours::Am(v) => {
                self.lcd.set_cursor_pos(10);
                self.lcd.write_str("AM").unwrap();
                v
            }
            ds3231::Hours::Pm(v) => {
                self.lcd.set_cursor_pos(10);
                self.lcd.write_str("PM").unwrap();
                v
            }
            ds3231::Hours::H24(v) => v,
        };
        self.draw_time(hours_val, date_time.mins(), date_time.secs());
        self.draw_date(date_time.year(), date_time.month(), date_time.day());
        self.draw_day_of_week(date_time.day_of_week());
    }

    fn draw_time(&mut self, h: u8, m: u8, sec: u8) {
        let mut s: String<U12> = if h < 10 {
            String::from("0")
        } else {
            String::new()
        };
        let mut s2: String<U3> = h.into();
        s.push_str(&s2).unwrap();
        if m < 10 {
            s.push_str(":0").unwrap();
        } else {
            s.push_str(":").unwrap();
        }
        s2 = m.into();
        s.push_str(&s2).unwrap();
        if sec < 10 {
            s.push_str(":0").unwrap();
        } else {
            s.push_str(":").unwrap();
        }
        s2 = sec.into();
        s.push_str(&s2).unwrap();
        s.push_str("  ").unwrap();
        self.lcd.set_cursor_pos(0);
        self.lcd.write_str(s.as_str()).unwrap();
    }

    fn draw_date(&mut self, y: u16, m: u8, d: u8) {
        let mut s: String<U12> = y.into();
        if m < 10 {
            s.push_str("-0").unwrap();
        } else {
            s.push_str("-").unwrap();
        }
        let mut s2: String<U3> = m.into();
        s.push_str(&s2).unwrap();
        if d < 10 {
            s.push_str("-0").unwrap();
        } else {
            s.push_str("-").unwrap();
        }
        s2 = d.into();
        s.push_str(&s2).unwrap();
        self.lcd.set_cursor_pos(40);
        self.lcd.write_str(s.as_str()).unwrap();
    }

    fn draw_day_of_week(&mut self, day: Option<ds3231::DayOfWeek>) {
        const DAY_NAMES: [&'static str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        self.lcd.set_cursor_pos(13);
        match day {
            None => {
                self.lcd.write_str("***").unwrap();
            }
            Some(day) => {
                self.lcd.write_str(DAY_NAMES[day.idx() as usize]).unwrap();
            }
        }
    }
}
