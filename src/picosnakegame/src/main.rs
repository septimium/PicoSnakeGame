#![no_std]
#![no_main]
#![allow(unused_imports)]
use core::panic::PanicInfo;
use embassy_executor::Spawner;
use tm1637_embedded_hal::Brightness;
use tm1637_embedded_hal::mappings::DigitBits;
use core::fmt::Write;
use tm1637_embedded_hal::{asynch::TM1637, demo::asynch::Demo};
use tm1637_embedded_hal::mappings::SegmentBits;
use tm1637_embedded_hal::mappings::UpCharBits;
extern crate pcd8544;
use pcd8544::PCD8544;
use pcd8544::{WIDTH, HEIGHT};
use pcd8544::graphics::GraphicsMode;
use embedded_snake::*;
use embedded_graphics::{
    prelude::*,
    pixelcolor::BinaryColor,
    text::Text,
};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_8X13_BOLD;
use embedded_graphics::mono_font::ascii::FONT_4X6;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;
use embassy_rp::gpio::{
    AnyPin, 
    self, 
    Input, 
    Pull, 
    Pin,
    Level,
    Output, 
    OutputOpenDrain,
};
use embassy_rp::peripherals::{
    PIN_0, 
    PIN_1,
    PIN_2,
    PIN_3,
    PIN_4,
    PIN_5,
    PIN_6,
    PIN_7,
    PIN_8,
    PIN_9,
    PIN_10,
    PIN_11,
    PIN_12, 
    PIN_13, 
    PIN_14, 
    PIN_15, 
    PIN_16, 
    PIN_17,
    PIN_18,
    PIN_19,
    PIN_20,
    PIN_21, 
    PIN_22, 
    PIN_23, 
    PIN_24, 
    PIN_25, 
    PIN_26,
    PIN_27, 
    PIN_28, 
    ADC,
    USB,
    SPI0,
    PWM_CH0,
};
use embassy_rp::pwm::{
    Config as PwmConfig, 
    Pwm,
};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::adc::{
    Adc,  
    Channel as ChannelADC, 
    Config as ConfigADC, 
    InterruptHandler as InterruptHandlerADC,
};
use embassy_rp::usb::{
    Driver, 
    InterruptHandler,
};
use embassy_rp::bind_interrupts;
use log::info;
use embassy_time::{
    Delay, 
    Timer, 
};

//CODE STARTS FROM HERE

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerADC;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
	//PERIPHERALS INIT
    let p = embassy_rp::init(Default::default());

    //SPI
    let mut config = spi::Config::default();
    config.frequency = 4_000_000;
    
    //USB DRIVER
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();
    
    //ADC CONFIG
    let mut adc = Adc::new(p.ADC, Irqs, ConfigADC::default());

    //PINS FOR PCD8544 AND INIT (LCD)
    let pcd_clk   = p.PIN_18;
    let pcd_din   = p.PIN_19;
    let pcd_dummy     = p.PIN_4; // dummy pin used as miso pin
    let pcd_dc    = Output::new(p.PIN_20, Level::Low);
    let pcd_ce    = Output::new(p.PIN_17, Level::Low);
    let pcd_rst   = Output::new(p.PIN_16, Level::Low);
    let pcd_light = Output::new(p.PIN_14, Level::High);
    let pcd_spi   = Spi::new(
        p.SPI0,
        pcd_clk,
        pcd_din,
        pcd_dummy,
        p.DMA_CH0,
        p.DMA_CH1,
        config,
    );
    let mut pcd = PCD8544::new(pcd_spi, pcd_dc, pcd_ce, pcd_rst, pcd_light).unwrap();

    //PINS FOR TM1637 AND INIT (SCORE)
    let delay = Delay {};
    let clk = OutputOpenDrain::new(p.PIN_0, Level::Low);
    let dio = OutputOpenDrain::new(p.PIN_1, Level::Low);
    let mut tm = TM1637::builder(clk, dio, delay).build();
    tm.init().await.unwrap();
    tm.write_brightness(Brightness::L7).await.unwrap();
    tm.write_segments_raw(0, &[DigitBits::from_digit(0) as u8]).await.unwrap();
    tm.write_segments_raw(1, &[DigitBits::from_digit(0) as u8]).await.unwrap();
    tm.write_segments_raw(2, &[DigitBits::from_digit(0) as u8]).await.unwrap();
    tm.write_segments_raw(3, &[DigitBits::from_digit(0) as u8]).await.unwrap();

    //BUZZER INIT
    let mut config_pwm: PwmConfig = Default::default();
    config_pwm.top = 0xFFFF;
    config_pwm.compare_a = 0;
    let mut buzzer = Pwm::new_output_a(p.PWM_CH3, p.PIN_22, config_pwm.clone());

    // JOYSTICK INIT
    let mut adc_pin0 = ChannelADC::new_pin(p.PIN_26, Pull::None);
    let mut adc_pin1 = ChannelADC::new_pin(p.PIN_27, Pull::None);

    let style = MonoTextStyle::new(&FONT_8X13_BOLD, BinaryColor::On); // font style
    let style2 = MonoTextStyle::new(&FONT_4X6, BinaryColor::On); // font style

    let seed = SmallRng::seed_from_u64(27052024).next_u64(); // seed for the game (CAN BE CHANGED, I USED THE PM FAIR DATE)
    let foodspeed = 50; // how fast the food changes its position (CAN BE CHANGED)
    let lcdwidth = 84; // lcd width 
    let lcdheight = 48; // lcd height
    let mut game = SnakeGame::<100, BinaryColor, SmallRng>::new( // game init
        lcdwidth, 
        lcdheight, 
        4, 
        4, 
        SmallRng::seed_from_u64(seed),
        BinaryColor::On,
        BinaryColor::On,
        foodspeed,
    );
    let mut direction = Direction::None; // working controls init
    let mut initial_snake_length = 5; // variable used for some conditions
    let mut score = 0; // score init
    loop {
        let x = adc.read(&mut adc_pin0).await.unwrap(); // reading x axis analog values of the joystick
        let y = adc.read(&mut adc_pin1).await.unwrap(); // reading y axis analog values of the joystick
        if x < 0x800 - 2000 {
            game.set_direction(Direction::Left);    // left condition
        } else if x > 0x800 + 2000 {
            game.set_direction(Direction::Right);   // right condition
        }
        if y < 0x800 - 2000 {
            game.set_direction(Direction::Up); // up condition
        } else if y > 0x800 + 2000 {
            game.set_direction(Direction::Down); // down condition
        }
        if direction != Direction::None {
            game.set_direction(direction);
            direction = Direction::None;
        }
        
        if game.snake_grown(initial_snake_length){
            config_pwm.compare_a = 20000; // freq of the buzzer
            buzzer.set_config(&config_pwm);
            initial_snake_length += 1;
            score += 1;
            tm.write_segments_raw(0, &[DigitBits::from_digit(score / (100*10)) as u8]).await.unwrap();
            tm.write_segments_raw(1, &[DigitBits::from_digit((score % (10*10*10)) / 100) as u8]).await.unwrap();
            tm.write_segments_raw(2, &[DigitBits::from_digit((score % 100) / 10) as u8]).await.unwrap();
            tm.write_segments_raw(3, &[DigitBits::from_digit(score % 10) as u8]).await.unwrap();
            Timer::after_millis(20).await; // play for 20ms when snake grows
            config_pwm.compare_a = 0; // set the compare value back to its initial value to stop the sound
            buzzer.set_config(&config_pwm);
        }

        if game.has_eaten_itself()==true && (initial_snake_length>5){ // condition for game over (unfortunately, it works only if you have at least score = 1 :( )
            pcd.clear(BinaryColor::Off).unwrap();
            config_pwm.compare_a = 20000; // freq of the buzzer
            buzzer.set_config(&config_pwm);
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_millis(1500).await;
            config_pwm.compare_a = 0; // set the compare value back to its initial value to stop the sound
            buzzer.set_config(&config_pwm);
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 10)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 9)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 8)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 7)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 6)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 5)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 4)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 3)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 2)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            pcd.clear(BinaryColor::Off).unwrap();
            Text::new("Game over!", Point::new(3, 13), style).draw(&mut pcd).unwrap();
            Text::new("You ate yourself :(", Point::new(3, 23), style2).draw(&mut pcd).unwrap();
            Text::new("(Restarting in 1)", Point::new(8, 40), style2).draw(&mut pcd).unwrap();
            pcd.flush().unwrap();
            Timer::after_secs(1).await;
            tm.write_segments_raw(0, &[DigitBits::from_digit(0) as u8]).await.unwrap();
            tm.write_segments_raw(1, &[DigitBits::from_digit(0) as u8]).await.unwrap();
            tm.write_segments_raw(2, &[DigitBits::from_digit(0) as u8]).await.unwrap();
            tm.write_segments_raw(3, &[DigitBits::from_digit(0) as u8]).await.unwrap();
            game = SnakeGame::<100, BinaryColor, SmallRng>::new(
                lcdwidth, 
                lcdheight, 
                4,
                4,
                SmallRng::seed_from_u64(seed),
                BinaryColor::On,
                BinaryColor::On,
                foodspeed,
            );
            direction = Direction::None;
            initial_snake_length = 5;
            score = 0;
        }
        pcd.clear(BinaryColor::Off).unwrap();
        game.draw(&mut pcd);
        pcd.flush().unwrap();
        Timer::after_millis(100).await; // speed of the game, i recommend leaving it like this (CAN BE CHANGED)
    }
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}