use iced::widget::image::Handle;
use ndarray::Array2;

pub struct ImageVisOptions {}

pub struct ImageVis {
    pub image_handle: Handle,
    vis_options: ImageVisOptions,
}

impl ImageVis {
    pub fn new(data: Array2<f64>) -> Self {
        let y_len = data.shape()[0];
        let x_len = data.shape()[1];
        let img = data
            .outer_iter()
            .flat_map(|row| {
                row.iter()
                    .flat_map(|v| [*v as u8, *v as u8, *v as u8, 255])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let image_handle = Handle::from_rgba(x_len as u32, y_len as u32, img);

        Self {
            image_handle,
            vis_options: ImageVisOptions {},
        }
    }
}
