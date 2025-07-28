use iced::widget::image::Handle;
use ndarray::Array3;

//fn create_grayscale_handle(data: &Array3<f64>) -> Handle {}
pub fn create_rgb_handle(data: &Array3<f64>) -> Handle {
    // trace!("Creating image handle for plot2d, {:?}", data.shape());
    let max = data.iter().fold(-f64::INFINITY, |a, &b| a.max(b));
    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let brightness = |p: f64| {
        let p = ((p - min) / (max - min)) as f32;
        let p = if p.is_nan() { 0.0 } else { p };
        (p * 255.0).round() as u8
    };
    let img: Vec<u8> = data
        .outer_iter()
        .flat_map(|row| {
            row.outer_iter()
                .flat_map(|p| {
                    if p.len() == 1 {
                        let b = brightness(p[0]);
                        [b, b, b, 255]
                    } else if p.len() == 3 {
                        [brightness(p[0]), brightness(p[1]), brightness(p[2]), 255]
                    } else {
                        panic!("unsupported array dimensions")
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Handle::from_rgba(data.dim().0 as u32, data.dim().1 as u32, img)
}
