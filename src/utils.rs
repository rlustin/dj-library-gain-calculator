pub fn loudness_to_gain(loudness: f32, target: f32) -> f32 {
    return target - loudness;
}

pub fn linear_to_db(linear: f32) -> f32 {
    if linear == 0. {
        // assume 24bit dynamic range
        -144.0
    } else {
        20. * linear.log10()
    }
}

pub fn exit_with_error(message: &str) {
    use std::process;

    println!("{}", message);
    println!("aborting…");

    process::exit(1);
}
