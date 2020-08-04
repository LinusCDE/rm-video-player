use std::io::stdin;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, sync_channel, SyncSender};
use std::thread;
use libremarkable::framebuffer::*;
use libremarkable::cgmath;

fn main() {
    // Can be done on device with this resolution
    // ffmpeg -i VIDEO_FILE.mp4 -vf realtime -f rawvideo -pix_fmt rgb24 -video_size 426x240 pipe:1 | ./a2-video-player

    const width: u32 = 426;
    const height: u32 = 240;
    const bytes_per_pixel: u32 = 3; // rgb8
    
    const bytes_per_frame: usize = (width * height * bytes_per_pixel) as usize;

    const fb_start_x: i32 = 200;
    const fb_start_y: i32 = 800;

    // Get framebuffer and clear it
    let mut fb = core::Framebuffer::new("/dev/fb0");
    fb.clear();
    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_INIT,
        common::display_temp::TEMP_USE_AMBIENT,
        common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true
    );
    thread::sleep_ms(100);
    
    let (tx, rx): (SyncSender<[u8; bytes_per_frame]>, Receiver<[u8; bytes_per_frame]>) = sync_channel(1); // 1 Buffered frame
    let thread_handle = thread::spawn(move || {
        let fb_area = common::mxcfb_rect { top: fb_start_y as u32, left: fb_start_x as u32, width, height };

        let mut prev_buffer: Option<[u8; bytes_per_frame]> = None;

        let mut counter = 0;
        // Ready
        for buffer in rx {
            counter += 1;
            let mut token_queue: Vec<u32> = vec![];
            let token_queue_size = 10;

            let mut buffer_index: usize = 0;
            let mut fb_x: i32 = fb_start_x;
            let mut fb_y: i32 = fb_start_y;
            
            let mut r;
            let mut g;
            let mut b;
            let mut pr;
            let mut pg;
            let mut pb;
            while buffer_index < buffer.len() {
                r = buffer[buffer_index];
                g = buffer[buffer_index+1];
                b = buffer[buffer_index+2];

                // Reduce depth (produces (256 / 64) shades of each component (floored)
                /*r = r - (r % 64);
                g = g - (r % 64);
                b = b - (r % 64);*/

                // Monochrome
                r = if r < 128 { 0 } else { 255 };
                g = if g < 128 { 0 } else { 255 };
                b = if b < 128 { 0 } else { 255 };
                
                if fb_x >= fb_start_x + width as i32 {
                    fb_y += 1;
                    fb_x = fb_start_x;
                }

                let changed = if let Some(prev_buffer) = prev_buffer {
                    pr = prev_buffer[buffer_index];
                    pg = prev_buffer[buffer_index+1];
                    pb = prev_buffer[buffer_index+2];
                    pr != r || pg != g || pb != b
                } else { true };
                
                buffer_index += 3;


                if changed {
                    fb.write_pixel(cgmath::Point2 { x: fb_x, y: fb_y }, common::color::RGB(r, g, b));

                    //let gray = ((r as f32 + g as f32 + b as f32) / 3.0) as u8;
                    //fb.write_pixel(cgmath::Point2 { x: fb_x, y: fb_y }, common::color::GRAY(gray));
                }
                //fb.write_pixel(cgmath::Point2 { x: fb_x, y: fb_y }, common::color::RGB(0, 0, 0));
                fb_x += 1;
            }


            // Toy with this!
            token_queue.push(fb.partial_refresh(
                &fb_area,
                refresh::PartialRefreshMode::Async,
                common::waveform_mode::WAVEFORM_MODE_DU,
                common::display_temp::TEMP_USE_REMARKABLE_DRAW,
                common::dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
                0,
                false
            ));
            if token_queue.len() == token_queue_size {
                let token = token_queue.remove(0);
                fb.wait_refresh_complete(token);
            }
            /*fb.full_refresh(
                common::waveform_mode::WAVEFORM_MODE_GLD16,
                common::display_temp::TEMP_USE_AMBIENT,
                common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
                0,
                true
            );*/
            //println!("REFRESH");
        }

        println!("RX: Received: {} frames", counter);
    });

    let mut buffer = [0u8; bytes_per_frame];
    let mut succeeded = 0;
    let mut dropped = 0;
    while stdin().read_exact(&mut buffer).is_ok() {
        match tx.try_send(buffer) {
            Ok(_) => succeeded += 1,
            Err(_) => dropped += 1
        }
    }
    println!("TX: Succeeded: {} frames ; Dropped: {} frames", succeeded, dropped);
    drop(tx); // Basically end of scope for sender (all senders out of scope = closed)

    thread_handle.join().unwrap();
}
