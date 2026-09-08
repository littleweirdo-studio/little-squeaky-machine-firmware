// use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation},
    peripherals::{ADC1, ADC2, GPIO1, GPIO2, GPIO3, GPIO16, Peripherals},
};
use little_weirdo_gui::{KeyEvent, MAPPING_SIGNAL};
// use smart_leds::RGB8;

#[embassy_executor::task]
pub async fn potmeter_task() {
    let mut pot_in1;
    let mut pot_in2;
    let mut pot_in3;
    let mut pot_in4;
    let mut adc1;
    let mut adc2;

    unsafe {
        /*
        |   | 15 | 03 |  ADC1_CH2 | 🎛️ Potmeter 1 |
            | 09 | 16 |  ADC2_CH5 | 🎛️ Potmeter 2 |
            | 39 | 01 |  ADC1_CH0 | 🎛️ Potmeter 3 |
            | 38 | 02 |  ADC1_CH1 | 🎛️ Potmeter 4 |
        */
        let peripherals: Peripherals = Peripherals::steal();
        let mut adc1_config = AdcConfig::default();
        let mut adc2_config = AdcConfig::default();

        pot_in1 = adc1_config
            .enable_pin_with_cal::<GPIO3, AdcCalBasic<ADC1>>(peripherals.GPIO3, Attenuation::_11dB);

        pot_in2 = adc2_config.enable_pin_with_cal::<GPIO16, AdcCalBasic<ADC2>>(
            peripherals.GPIO16,
            Attenuation::_11dB,
        );

        pot_in3 = adc1_config
            .enable_pin_with_cal::<GPIO1, AdcCalBasic<ADC1>>(peripherals.GPIO1, Attenuation::_11dB);

        pot_in4 = adc1_config
            .enable_pin_with_cal::<GPIO2, AdcCalBasic<ADC1>>(peripherals.GPIO2, Attenuation::_11dB);

        adc1 = Adc::new(peripherals.ADC1, adc1_config);
        adc2 = Adc::new(peripherals.ADC2, adc2_config);
    }

    loop {
        let pin_value_pot_1: u16 = adc1.read_blocking(&mut pot_in1);
        MAPPING_SIGNAL
            .send(KeyEvent {
                event: little_weirdo_gui::EventType::PotMeterChange,
                id: 0,
                value: pin_value_pot_1 / 16,
            })
            .await;
        Timer::after(Duration::from_millis(50)).await;

        let pin_value_pot_2: u16 = adc2.read_blocking(&mut pot_in2);
        MAPPING_SIGNAL
            .send(KeyEvent {
                event: little_weirdo_gui::EventType::PotMeterChange,
                id: 1,
                value: pin_value_pot_2 / 16,
            })
            .await;
        Timer::after(Duration::from_millis(50)).await;

        let pin_value_pot_3: u16 = adc1.read_blocking(&mut pot_in3);
        MAPPING_SIGNAL
            .send(KeyEvent {
                event: little_weirdo_gui::EventType::PotMeterChange,
                id: 2,
                value: pin_value_pot_3 / 16,
            })
            .await;
        Timer::after(Duration::from_millis(50)).await;

        let pin_value_pot_4: u16 = adc1.read_blocking(&mut pot_in4);
        MAPPING_SIGNAL
            .send(KeyEvent {
                event: little_weirdo_gui::EventType::PotMeterChange,
                id: 3,
                value: pin_value_pot_4 / 16,
            })
            .await;
        Timer::after(Duration::from_millis(50)).await;
    }
}
