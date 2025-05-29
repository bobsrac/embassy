#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::clock::{set_hfclk_source, HfclkSource};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::radio::ieee802154::{self, Packet};
use embassy_nrf::{peripherals, radio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use defmt::{info, error};

embassy_nrf::bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_nrf::init(Default::default());

    // assumes LED on P0_15 with active-high polarity
    let mut gpo_led = Output::new(peripherals.P0_15, Level::Low, OutputDrive::Standard);

    let mut radio = ieee802154::Radio::new(peripherals.RADIO, Irqs);
    let mut packet = Packet::new();

    loop {
        packet.copy_from_slice(&[0_u8; 16]);
        gpo_led.set_high();
        set_hfclk_source(HfclkSource::ExternalXtal);
        match radio.try_send(&mut packet).await {
            Ok(_) => info!("try_send({:?}): ", *packet),
            Err(err) => error!("try_send(): {:?}", err),
        }
        set_hfclk_source(HfclkSource::Internal);
        gpo_led.set_low();
        Timer::after_millis(10_000u64).await;
    }
}
