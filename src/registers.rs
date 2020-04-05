/// Page Definitions
///

/// The hardcoded configuration page
pub const PAGE_CONFIG: u8 = 0;
/// Page of dyanmic statuses
pub const PAGE_STATUS: u8 = 1;
/// Array of premixed actuator outputs in the range -10k..10k
/// see REG_CONFIG_N_ACTUATORS
pub const PAGE_ACTUATORS: u8 = 2;
/// Array of servo output PWM values, in microseconds
/// see REG_CONFIG_N_ACTUATORS
pub const PAGE_SERVOS: u8 = 3;
/// Array of RC input values, in microseconds
pub const PAGE_RAW_RC_INPUT: u8 = 4;
/// Array of scaled RC input values in the range -10k..10k
pub const PAGE_SCALED_RC_INPUT: u8 = 5;
/// Array of raw ADC input values
/// see REG_CONFIG_N_ADC_INPUTS
pub const PAGE_RAW_ADC_INPUT: u8 = 6;
/// Servo PWM rate group information
pub const PAGE_PWM_INFO: u8 = 7;
/// Writeable setup page
pub const PAGE_SETUP: u8 = 50;
/// Actuator control groups page
pub const PAGE_CONTROLS: u8 = 51;
/// Page for writing configuration text to the mixer interpreter
pub const PAGE_MIXERLOAD: u8 = 52;
/// RC input configuration
pub const PAGE_RCIN_CONFIG: u8 = 53;
/// Direct PWM output that bypasses the mixer
pub const PAGE_PWM_DIRECT_OUT: u8 = 54;
/// Failsafe values for PWM outputs: zero values disable output
/// see REG_CONFIG_N_ACTUATORS
pub const PAGE_PWM_FAILSAFES: u8 = 55;
/// Sensors directly connected with px4io, such as RC receiver telemetry
pub const PAGE_LOCAL_SENSORS: u8 = 56;
/// Special page value used for testing and debugging
pub const PAGE_TEST_DEBUG: u8 = 127;

/// Configuration page (PAGE_CONFIG) register offsets:

/// The protocol version the px4io mcu firmware supports
pub const REG_CONFIG_PROTOCOL_VERSION: u8 = 0;
/// Hardware version identifier
pub const REG_CONFIG_HARDWARE_VERSION: u8 = 1;
/// Bootloader version identifier
pub const REG_CONFIG_BOOTLOADER_VERSION: u8 = 2;
/// Maximum data transfer size, in bytes
pub const REG_CONFIG_MAX_TRANSFER: u8 = 3;
/// Maximum number of controls
pub const REG_CONFIG_N_CONTROLS: u8 = 4;
/// Maximum number of actuator outputs
pub const REG_CONFIG_N_ACTUATORS: u8 = 5;
/// Maximum number of RC inputs
pub const REG_CONFIG_N_RC_INPUTS: u8 = 6;
/// Maximum number of ADC inputs
pub const REG_CONFIG_N_ADC_INPUTS: u8 = 7;
/// Maximum number of Relay outputs
pub const REG_CONFIG_N_RELAY_OUTPUTS: u8 = 8;

/// Raw RC Input Page (PAGE_RAW_RC_INPUT) register offsets:

/// Number of currently valid RC channels
pub const REG_RAW_RC_INPUT_COUNT: u8 = 0;


/// Test Debug Page register offsets:

/// Toggle the IOMCU's "amber" LED on and off
pub const REG_DEBUG_TEST_LED: u8 = 0;
