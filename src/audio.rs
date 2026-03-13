
use cpal::{Device, DeviceId, Host, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};

pub struct AudioData(pub Host, pub Device, pub StreamConfig);

pub fn init_audio() -> AudioData {
    let host_id = cpal::available_hosts()[0];
    let host = cpal::host_from_id(host_id).unwrap();

    let output_device = host.default_output_device().expect("Error opening device.");

    let mut supported_configs_range = output_device.supported_output_configs().expect("Unable to get output configs.");
    let supported_config = supported_configs_range.next().expect("Unable to get audio config?").with_sample_rate(44100);
    let output_config: StreamConfig = supported_config.into();

    return AudioData(host, output_device, output_config);
}

pub fn create_output_stream<F>(audio_data: &AudioData, get_sample: F) -> cpal::Stream where F: Fn() -> f32 + Send + 'static {
    let device = &audio_data.1;
    let config = audio_data.2;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = get_sample();
            }
        },
        move |err| {
            eprintln!("Error processing sample: {}", err);
        },
        None // None=blocking, Some(Duration)=timeout
    ).unwrap();
    return stream;
}


