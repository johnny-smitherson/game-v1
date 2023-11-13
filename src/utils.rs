use std::f32::consts::PI;

pub fn cap_2pi(x: f32) -> f32 {
    let mut x = x % (PI * 2.0);
    if x < -PI {
        x += PI * 2.0;
    } else if x > PI {
        x -= PI * 2.0;
    }
    x
}
