use allen::{AllenError, Context, Device, Orientation};
use ultraviolet::Vec3;

use crate::managers::debugger::error;

static mut DEVICE: Option<Device> = None;
pub static mut CONTEXT: Option<Context> = None;

pub fn init() -> Result<(), SoundError> {
    unsafe {
        let device = Device::open(None);
        match device {
            None => {
                error("error initializing sound manager\nfailed to get current audio device");
                return Err(SoundError::CurrentDeviceGettingError);
            }
            Some(_) => (),
        }
        let device = device.unwrap();
        let context = device.create_context();
        match context {
            Err(err) => {
                error(&format!(
                    "error initializing sound manager\nfailed to create OpenAL context!\nerr: {}",
                    err
                ));
                return Err(SoundError::ContextCreationError(err));
            }
            Ok(_) => (),
        }
        DEVICE = Some(device);

        let context = context.unwrap();
        context.make_current();
        //context.listener().set_gain(0.8);
        CONTEXT = Some(context);

        return Ok(());
    }
}

pub fn set_listener_position(position: Vec3) {
    let context = take_context();
    let _ = context.listener().set_position(position.into());
    return_context(context)
}

pub fn set_listener_orientation(at: Vec3) {
    let context = take_context();
    let _ = context.listener().set_orientation(Orientation {
        at: at.into(),
        up: [0.0, 1.0, 0.0],
    });
    return_context(context)
}

pub fn set_listener_transform(position: Vec3, at: Vec3) {
    set_listener_position(position);
    set_listener_orientation(at);
}

pub fn take_context() -> Context {
    unsafe {
        return CONTEXT.take().unwrap();
    }
}

pub fn return_context(context: Context) {
    unsafe { CONTEXT = Some(context) }
}

#[derive(Debug)]
pub enum SoundError {
    CurrentDeviceGettingError,
    ContextCreationError(AllenError),
    Not16BitWavFileError,
    NotMonoWavFileError,
    SoundAssetLoadingError,
    BufferCreationFailedError(AllenError),
    SourceCreationFailedError(AllenError),
    WrongEmitterType,
}
