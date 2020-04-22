#![no_std]
#![no_main]

mod display;
mod ds3231;

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use panic_semihosting as _;

use hal::{
    delay::Delay,
    interrupt,
    prelude::*,
    rcc::Clocks,
    stm32,
    timer::{Event, Timer},
};
use stm32f4xx_hal as hal;

static CLOCK_UPDATE_REQUIRED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

fn timer2_init(clocks: Clocks, tim2: stm32f4xx_hal::stm32::TIM2) {
    let mut timer = Timer::tim2(tim2, 10.hz(), clocks);
    timer.listen(Event::TimeOut);
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });

    // enable
    unsafe {
        stm32::NVIC::unmask(hal::stm32::Interrupt::TIM2);
    }

    // Enable interrupts
    stm32::NVIC::unpend(hal::stm32::Interrupt::TIM2);
}

fn clock_update_required() -> bool {
    free(|cs| {
        let cell = CLOCK_UPDATE_REQUIRED.borrow(cs);
        let res = cell.get();
        if res {
            cell.replace(false);
        }
        res
    })
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.mhz())
        .sysclk(100.mhz())
        .pclk1(50.mhz())
        .pclk2(50.mhz())
        .freeze();

    let delay = Delay::new(cp.SYST, clocks);

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    // I2c
    let sda = gpiob.pb7.into_alternate_af4().set_open_drain();
    let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
    let mut ds3231_clock = ds3231::Ds3231::new(dp.I2C1, (scl, sda), clocks);

    // hd44780
    let rs = gpioa.pa5.into_push_pull_output();
    let en = gpioa.pa4.into_push_pull_output();
    let b4 = gpioa.pa3.into_push_pull_output();
    let b5 = gpioa.pa2.into_push_pull_output();
    let b6 = gpioa.pa1.into_push_pull_output();
    let b7 = gpioa.pa0.into_push_pull_output();
    let mut display = display::Display::new(rs, en, b4, b5, b6, b7, delay);

    // Timer initialization
    timer2_init(clocks, dp.TIM2);

    loop {
        if clock_update_required() {
            let date_time = ds3231_clock.read_date_time();
            display.draw_date_time(&date_time);
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }

        let cell = CLOCK_UPDATE_REQUIRED.borrow(cs);
        cell.replace(true);
    });
}
