use std::{
    array, error,
    fmt::{self, Formatter},
};

use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc::channel, oneshot},
};

use crate::{
    interface::client,
    io::{AnalogInput, DigitalInput, DigitalOutput, HBridge},
    motor::{ClearCoreMotor, MotorBuilder},
};

pub const STX: u8 = 2;
pub const CR: u8 = 13;
pub const RESULT_IDX: u8 = 3;

const NO_MOTORS: usize = 4;
const NO_DIGITAL_INPUTS: usize = 3;
const NO_ANALOG_INPUTS: usize = 4;
const NO_OUTPUTS: usize = 6;
const NO_HBRIDGE: usize = 2;

const REPLY_IDX: usize = 3;
const FAILED_REPLY: u8 = b'?';

#[derive(Debug)]
pub struct Message {
    pub buffer: Vec<u8>,
    pub response: oneshot::Sender<Vec<u8>>,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl<T: error::Error + Send + Sync + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

pub(crate) fn check_reply(reply: &[u8]) -> Result<(), Error> {
    if reply[REPLY_IDX] == FAILED_REPLY {
        Err(Error {
            message: std::str::from_utf8(reply)?.to_string(),
        })
    } else {
        Ok(())
    }
}

pub(crate) const fn make_prefix(device_type: u8, device_id: u8) -> [u8; 3] {
    [2, device_type, device_id + 48]
}

pub type Motors = [ClearCoreMotor; NO_MOTORS];
pub type HBridges = [HBridge; NO_HBRIDGE];
pub type AnalogInputs = [AnalogInput; NO_ANALOG_INPUTS];
pub type Inputs = Vec<DigitalInput>; //We have a variable number of these due to the IO bank's versatility
pub type Outputs = Vec<DigitalOutput>; //We have a variable number of these due to the IO bank's versatility

#[derive(Clone)]
pub struct ControllerHandle {
    motors: Motors,
    digital_inputs: Inputs,
    analog_inputs: AnalogInputs,
    outputs: Outputs,
    h_bridges: HBridges,
}

impl ControllerHandle {
    pub fn new<T>(addr: T, builder: [MotorBuilder; 4]) -> Self
    where
        T: ToSocketAddrs + Send + 'static,
    {
        let (tx, rx) = channel::<Message>(10);
        tokio::spawn(async move {
            client(addr, rx).await.unwrap();
        });
        let motors = array::from_fn(|i| {
            let builder = builder[i].clone();
            ClearCoreMotor::new(builder.id, builder.scale, tx.clone())
        });

        let digital_inputs = (0..NO_DIGITAL_INPUTS)
            .map(|index| DigitalInput::new(index as u8, tx.clone()))
            .collect();

        let analog_inputs = array::from_fn(|i| AnalogInput::new(i as u8 + 3, tx.clone()));

        let outputs = (0..NO_OUTPUTS)
            .map(|index| DigitalOutput::new(index as u8, tx.clone()))
            .collect();

        let h_bridges = [
            HBridge::new(4, 32700, tx.clone()),
            HBridge::new(5, 32700, tx.clone()),
        ];

        Self {
            motors,
            digital_inputs,
            analog_inputs,
            outputs,
            h_bridges,
        }
    }

    pub fn get_motor(&self, id: usize) -> ClearCoreMotor {
        self.motors[id].clone()
    }

    pub fn get_motors(&self) -> Motors {
        self.motors.clone()
    }

    pub fn get_digital_input(&self, id: usize) -> DigitalInput {
        self.digital_inputs[id].clone()
    }

    pub fn get_digital_inputs(&self) -> Inputs {
        self.digital_inputs.clone()
    }

    pub fn get_analog_input(&self, id: usize) -> AnalogInput {
        self.analog_inputs[id].clone()
    }

    pub fn get_analog_inputs(&self) -> AnalogInputs {
        self.analog_inputs.clone()
    }
    pub fn get_output(&self, id: usize) -> DigitalOutput {
        self.outputs[id].clone()
    }

    pub fn get_outputs(&self) -> Outputs {
        self.outputs.clone()
    }

    pub fn get_h_bridge(&self, id: usize) -> HBridge {
        let idx = id - 4;
        self.h_bridges[idx].clone()
    }

    pub fn get_h_bridges(&self) -> HBridges {
        self.h_bridges.clone()
    }
}
