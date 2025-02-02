#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::peripherals::RNG;
use esp_hal::rng::Rng;
use esp_hal::{main, Blocking};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use smart_leds::{SmartLedsWrite, RGB8};
use fugit::HertzU32;
use log::info;
use ws2812_spi::Ws2812;

extern crate alloc;

const NUM_LEDS: usize = 144;

const RED: RGB8 = RGB8::new(40, 0, 0);
#[allow(dead_code)]
const ORANGE: RGB8 = RGB8::new(40, 20, 0);
const YELLOW: RGB8 = RGB8::new(40, 40, 0);
const GREEN: RGB8 = RGB8::new(0, 40, 0);
const CYAN: RGB8 = RGB8::new(0, 40, 40);
const BLUE: RGB8 = RGB8::new(0, 0, 40);
const PURPLE: RGB8 = RGB8::new(40, 0, 40);

#[main]
fn main() -> ! {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);

    let pin15 = esp_hal::gpio::Output::new(peripherals.GPIO15, esp_hal::gpio::Level::High);
    let spi = Spi::new(peripherals.SPI2, Config::default().with_frequency(HertzU32::MHz(3)).with_mode(Mode::_0))
        .unwrap().with_mosi(pin15);
    let mut ws = Ws2812::new(spi);
    let leds = [RGB8::default(); 144];
    ws.write(leds).unwrap();

    cat_toy(ws, peripherals.RNG);
    // rainbow(peripherals);
    // matrix(ws, peripherals.RNG);
}

#[allow(dead_code)]
fn cat_toy(mut ws: Ws2812<Spi<'_, Blocking>>, rng: RNG) -> ! {
    let mut leds = [RGB8::default(); NUM_LEDS];
    let colors = [RED, PURPLE, BLUE, CYAN, GREEN, YELLOW, ORANGE];
    let mut rng = Rng::new(rng);
    
    let mut pos = rng.random() % NUM_LEDS as u32;
    let mut direction: i32 = if rng.random() % 2 == 0 { 1 } else { -1 };
    let mut color_iter = colors.into_iter().cycle();
    let mut current_color = color_iter.next().unwrap();
    let use_colors = true;

    loop {
        let distance = rng.random() % (
            if direction > 0 {
                NUM_LEDS as u32 - pos
            } else {
                pos
            }
        );
        info!("{pos}");
        for _ in 0..distance {
            if use_colors {
                leds[pos as usize] = current_color;
            } else {
                leds = [RGB8::default(); NUM_LEDS];
                leds[pos as usize] = RED;
            }
            pos = if direction > 0 { pos + 1} else { pos - 1 };
            ws.write(leds).unwrap();
        }
        current_color = color_iter.next().unwrap();
        direction = direction * -1;

    }
}

#[allow(dead_code)]
fn rainbow(mut ws: Ws2812<Spi<'_, Blocking>>) -> ! {
    let mut leds = [RGB8::default(); NUM_LEDS];
    let colors = [RED, GREEN, BLUE];
    let colors2 = [PURPLE, YELLOW, CYAN];

    let delay = Delay::new();
    let mut led_idx1 = 0;
    let mut led_idx2 = NUM_LEDS / 2;
    let mut color_iter1 = colors.into_iter().cycle();
    let mut color_iter2 = colors2.into_iter().cycle();
    let mut current_color1 = color_iter1.next().unwrap();
    let mut current_color2 = color_iter2.next().unwrap();
    loop {
        info!("writing index {led_idx1} and {led_idx2}");

        leds[led_idx1] = current_color1;
        leds[led_idx2] = current_color2;

        ws.write(leds).unwrap();

        delay.delay_micros(10);
        led_idx1 = if led_idx1 >= NUM_LEDS-1 {
            current_color1 = color_iter1.next().unwrap();
            0
        } else {
            led_idx1 + 1
        };
        led_idx2 = if led_idx2 >= NUM_LEDS-1 {
            current_color2 = color_iter2.next().unwrap();
            0
        } else {
            led_idx2 + 1
        };
    }
}

#[allow(dead_code)]
fn matrix(mut ws: Ws2812<Spi<'_, Blocking>>, _rng: RNG) -> ! {
    const LEAD_LED: RGB8 = RGB8::new(10, 160, 10);
    const SUB: u8 = 1;
    let mut leds = [RGB8::default(); NUM_LEDS];
    let mut head: usize = 0;
    let mut tail: usize = 0;
    
    leds[head] = LEAD_LED;
    ws.write(leds).unwrap();

    loop {
        for i in tail..head {
            let rgb = leds[i].clone();
            let new_rgb = RGB8::new(rgb.r.saturating_sub(SUB), rgb.g.saturating_sub(SUB), rgb.b.saturating_sub(SUB));
            if rgb.g != 0 && new_rgb.g == 0 {
                tail = i;
            }
            leds[i] = new_rgb;
            // info!("i: {i}, head: {head}, tail: {tail}");
        }

        if head == NUM_LEDS - 1 {
            // info!("edge case head: {head}, tail: {tail}");
            let rgb = leds[head].clone();
            let new_rgb = RGB8::new(rgb.r.saturating_sub(SUB), rgb.g.saturating_sub(SUB), rgb.b.saturating_sub(SUB));
            leds[head] = new_rgb;
        }

        ws.write(leds).unwrap();
        if head >= (NUM_LEDS - 1) {
            if head == tail {
                tail = 0;
                head = 0;
            } else {
                tail = tail + 1;
            }
        } else {
            head = head + 1;
            leds[head] = LEAD_LED;
            // info!("new head");
        };
        
        // info!("head: {head}, tail: {tail}");
    }
}