use image::{DynamicImage, GenericImageView, ImageReader};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::Widget,
};

#[derive(Debug, Default)]
pub struct Image {
    pub path: String,
    pub image: DynamicImage,
}

impl Image {
    pub fn new(image_path: String) -> Self {
        Image {
            image: ImageReader::open(image_path.clone())
                .unwrap()
                .decode()
                .unwrap(),
            path: image_path,
        }
    }

    pub fn render_image(&self, area: Rect, buf: &mut Buffer) {
        // change the buffer here
        let (img_w, img_h) = self.image.dimensions();
        let cells_w = area.width as u32;
        let cells_h = area.height as u32;

        for cell_y in 0..cells_h {
            for cell_x in 0..cells_w {
                // map cell to image x coordinate
                let img_x = cell_x * img_w / cells_w;
                // determine top and bottom pixel rows in the image
                let top_row = (cell_y * 2) * img_h / (cells_h * 2);
                let bottom_row = ((cell_y * 2 + 1) * img_h) / (cells_h * 2);

                let pixel_top = self.image.get_pixel(img_x, top_row.min(img_h - 1));
                let pixel_bot = self.image.get_pixel(img_x, bottom_row.min(img_h - 1));

                let pos = Position::new(area.x + cell_x as u16, area.y + cell_y as u16);
                let cell = &mut buf[pos];
                cell.set_char('▀')
                    .set_fg(Color::Rgb(pixel_top[0], pixel_top[1], pixel_top[2]))
                    .set_bg(Color::Rgb(pixel_bot[0], pixel_bot[1], pixel_bot[2]));
            }
        }
    }
}

impl Widget for &mut Image {
    fn render(self, area: Rect, buf: &mut Buffer) {
        /* 1. Leave out one line for the status line at the top
         *  2. One line is exactly two pixels tall, which means pixel row one will be the foreground
         *     of the first
         *     row and pixel row two will be the background of it
         *  3. Leave out another line for the command line
         * */

        // buf[Position::new(1, 0)]
        //     .set_char('▀')
        //     .set_fg(ratatui::style::Color::Red)
        //     .set_bg(ratatui::style::Color::Gray);

        /* Also, the program has to handle images and videos differently */
        // For now, only images will be handled
        self.render_image(area, buf);
    }
}
