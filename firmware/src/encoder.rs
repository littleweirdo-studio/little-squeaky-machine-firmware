use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    peripherals::Peripherals,
};
use esp_println::println;
use little_weirdo_gui::{Event, MAPPING_SIGNAL};

// Mapping event values shared with the event consumer.
pub const ENCODER_EVENT_ROTATION: u8 = 42;
pub const ENCODER_CLOCKWISE: u16 = 1;
pub const ENCODER_COUNTER_CLOCKWISE: u16 = 0;
const ENCODER_COUNT: usize = 4;
// Most mechanical encoders produce four valid signal transitions per detent.
const TRANSITIONS_PER_DETENT: i8 = 3;

// Index with (previous_state << 2) | current_state, where each state is A:B.
// The signs are reversed to match the physical direction of these encoders.
const QUADRATURE_TRANSITIONS: [i8; 16] = [
    0, -1, 1, 0, // 00 -> 00, 01, 10, 11
    1, 0, 0, -1, // 01 -> 00, 01, 10, 11
    -1, 0, 0, 1, // 10 -> 00, 01, 10, 11
    0, 1, -1, 0, // 11 -> 00, 01, 10, 11
];

fn send_rotation_event(encoder_id: u8, clockwise: bool) {
    let event = Event {
        event_identifier: ENCODER_EVENT_ROTATION,
        id: encoder_id,
        value: if clockwise {
            ENCODER_CLOCKWISE
        } else {
            ENCODER_COUNTER_CLOCKWISE
        },
    };

    // Do not block the polling loop if the mapping queue is full.
    if MAPPING_SIGNAL.try_send(event).is_err() {
        esp_println::println!("encoder event dropped: mapping queue full");
    }
}

#[embassy_executor::task]
pub async fn encoder_task() {
    let encoders_a: [Input; ENCODER_COUNT];
    let encoders_b: [Input; ENCODER_COUNT];

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let input_pin_config = InputConfig::default().with_pull(Pull::Up);
        // A and B pin pairs for encoders 0 through 3.
        encoders_a = [
            Input::new(peripherals.GPIO16, input_pin_config),
            Input::new(peripherals.GPIO3, input_pin_config),
            Input::new(peripherals.GPIO1, input_pin_config),
            Input::new(peripherals.GPIO43, input_pin_config),
        ];
        encoders_b = [
            Input::new(peripherals.GPIO8, input_pin_config),
            Input::new(peripherals.GPIO9, input_pin_config),
            Input::new(peripherals.GPIO2, input_pin_config),
            Input::new(peripherals.GPIO44, input_pin_config),
        ];
    }

    let mut previous_states = [0_u8; ENCODER_COUNT];
    let mut accumulated_transitions = [0_i8; ENCODER_COUNT];
    // Start from the live pin levels so startup does not look like a rotation.
    for encoder_id in 0..ENCODER_COUNT {
        previous_states[encoder_id] = ((encoders_a[encoder_id].is_high() as u8) << 1)
            | encoders_b[encoder_id].is_high() as u8;
    }

    loop {
        for encoder_id in 0..ENCODER_COUNT {
            // Encode the two input levels as a 2-bit quadrature state.
            let current_state = ((encoders_a[encoder_id].is_high() as u8) << 1)
                | encoders_b[encoder_id].is_high() as u8;
            let transition_index = ((previous_states[encoder_id] << 2) | current_state) as usize;
            accumulated_transitions[encoder_id] += QUADRATURE_TRANSITIONS[transition_index];
            previous_states[encoder_id] = current_state;

            // Emit one event after a complete detent; retain partial movement.
            if accumulated_transitions[encoder_id] >= TRANSITIONS_PER_DETENT {
                accumulated_transitions[encoder_id] -= TRANSITIONS_PER_DETENT;
              //  println!("encoder {} event clockwise = true", encoder_id);
                send_rotation_event(encoder_id as u8, true);
            } else if accumulated_transitions[encoder_id] <= -TRANSITIONS_PER_DETENT {
                accumulated_transitions[encoder_id] += TRANSITIONS_PER_DETENT;
             //   println!("encoder {} event clockwise = false", encoder_id);
                send_rotation_event(encoder_id as u8, false);
            }
        }

        // Sample frequently enough to capture each step of the quadrature cycle.
        Timer::after(Duration::from_millis(1)).await;
    }
}
