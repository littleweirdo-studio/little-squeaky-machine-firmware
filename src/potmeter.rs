use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation},
    gpio::GpioPin,
    peripherals::{Peripherals, ADC1, ADC2},
};

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
        | Pin | 15 |  ADC2_CH4   | 🎛️ Potmeter 1 |
        | Pin | 16 |  ADC2_CH5   | 🎛️ Potmeter 2 |
        | Pin | 01 |  ADC1_CH0   | 🎛️ Potmeter 3 |
        | Pin | 02 |  ADC1_CH1   | 🎛️ Potmeter 4 |
        */
        let peripherals: Peripherals = Peripherals::steal();
        let mut adc1_config = AdcConfig::default();
        let mut adc2_config = AdcConfig::default();

        pot_in1 = adc2_config.enable_pin_with_cal::<GpioPin<15>, AdcCalBasic<ADC2>>(
            peripherals.GPIO15,
            Attenuation::_11dB,
        );

        pot_in2 = adc2_config.enable_pin_with_cal::<GpioPin<16>, AdcCalBasic<ADC2>>(
            peripherals.GPIO16,
            Attenuation::_11dB,
        );

        pot_in3 = adc1_config.enable_pin_with_cal::<GpioPin<1>, AdcCalBasic<ADC1>>(
            peripherals.GPIO1,
            Attenuation::_11dB,
        );

        pot_in4 = adc1_config.enable_pin_with_cal::<GpioPin<2>, AdcCalBasic<ADC1>>(
            peripherals.GPIO2,
            Attenuation::_11dB,
        );

        adc1 = Adc::new(peripherals.ADC1, adc1_config);
        adc2 = Adc::new(peripherals.ADC2, adc2_config);
    }

    loop {
        let pin_value_pot_1: u16 = adc2.read_blocking(&mut pot_in1);
        let pin_value_pot_2: u16 = adc2.read_blocking(&mut pot_in2);
        let pin_value_pot_3: u16 = adc1.read_blocking(&mut pot_in3);
        let pin_value_pot_4: u16 = adc1.read_blocking(&mut pot_in4);
        Timer::after(Duration::from_millis(100)).await;
    }
}
