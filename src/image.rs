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
    pub zoom: f32,
    pub pan_x: i32,
    pub pan_y: i32,
}

impl Image {
    pub fn new(image_path: String) -> Self {
        Image {
            image: ImageReader::open(image_path.clone())
                .unwrap()
                .decode()
                .unwrap(),
            path: image_path,
            zoom: 1.0,
            pan_x: 0,
            pan_y: 0,
        }
    }
    pub fn render_image(&self, area: Rect, buf: &mut Buffer) {
        let (img_w, img_h) = self.image.dimensions();
        let term_cells_w = area.width as u32;
        let term_cells_h = area.height as u32;
        let term_pix_w = term_cells_w;
        let term_pix_h = term_cells_h * 2;

        // 1) compute zoomed source‐rect in image coords
        let src_w = (img_w as f32 / self.zoom).round() as u32;
        let src_h = (img_h as f32 / self.zoom).round() as u32;
        // but don’t let it exceed the image dims:
        let src_w = src_w.min(img_w);
        let src_h = src_h.min(img_h);

        let base_x0 = ((img_w - src_w) / 2) as i32;
        let base_y0 = ((img_h - src_h) / 2) as i32;
        // apply pan, then clamp into [0..=img_dim−src_dim]:
        let max_x0 = (img_w - src_w) as i32;
        let max_y0 = (img_h - src_h) as i32;

        let src_x0 = (base_x0 + self.pan_x).clamp(0, max_x0) as u32;
        let src_y0 = (base_y0 + self.pan_y).clamp(0, max_y0) as u32;

        // 2) same uniform scale to fit that rect into terminal pix
        let scale_x = term_pix_w as f32 / src_w as f32;
        let scale_y = term_pix_h as f32 / src_h as f32;
        let scale = scale_x.min(scale_y);

        let scaled_w = (src_w as f32 * scale).round() as u32;
        let scaled_h = (src_h as f32 * scale).round() as u32;
        let scaled_cells_w = scaled_w;
        let scaled_cells_h = (scaled_h + 1) / 2;

        // 3) center offsets
        let offset_x = ((term_cells_w as i32 - scaled_cells_w as i32) / 2).max(0) as u32;
        let offset_y = ((term_cells_h as i32 - scaled_cells_h as i32) / 2).max(0) as u32;

        // clear letterbox
        for y in 0..term_cells_h {
            for x in 0..term_cells_w {
                let pos = Position::new(area.x + x as u16, area.y + y as u16);
                buf[pos].reset();
            }
        }

        // 4) draw each cell sampling from the zoomed rect
        for cell_y in 0..scaled_cells_h {
            for cell_x in 0..scaled_cells_w {
                let tx = offset_x + cell_x;
                let ty = offset_y + cell_y;

                let fx = (cell_x as f32 + 0.5) * src_w as f32 / scaled_w as f32;
                let fy_top = (cell_y * 2) as f32 * src_h as f32 / scaled_h as f32;
                let fy_bot = (cell_y * 2 + 1) as f32 * src_h as f32 / scaled_h as f32;

                let img_x = src_x0 + fx.floor() as u32;
                let top_y = src_y0 + fy_top.floor() as u32;
                let bot_y = src_y0 + fy_bot.floor() as u32;

                let pixel_top = self.image.get_pixel(img_x, top_y.clamp(0, img_h - 1));
                let pixel_bot = self.image.get_pixel(img_x, bot_y.clamp(0, img_h - 1));

                let pos = Position::new(area.x + tx as u16, area.y + ty as u16);
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
