use libremarkable::cgmath;
use libremarkable::framebuffer::*;
use std::io::stdin;
use std::io::Read;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    // Can be done on device with this resolution
    // Convert a video (preferrably 720p directly with -f)
    // youtube-dl https://www.youtube.com/watch\?v\=naTxqt1wVxc -o- | ffmpeg -i - -vf transpose=1,format=monow,scale=-1:1280 -pix_fmt monow -f rawvideo /tmp/myvideo_hopefully_640_1280_25fps.raw
    // Probaby compress this with zstd (about 4x for those videos): zstd -k /tmp/myvideo_hopefully_640_1280_25fps.raw
    // Copy to your remarkable: scp /tmp/myvideo_hopefully_640_1280_25fps.raw.zst remarkable:
    // Play it there (remarkables ssh): zstd -cd myvideo_hopefully_640_1280_25fps.raw.zst | ./rm-video-player # (zstd was installed with opkg)
    //
    // If resolution or framerate don't match. Please adjust below. THE WIDTH HAS TO BE DIVIDABLE BY 8 (or you fix that in the source)

    //const width: u32 = 320;
    //const height: u32 = 180;
    const width: u32 = 640;
    const height: u32 = 1280;

    const bytes_per_frame: usize = (width * height / 8) as usize;

    const fb_start_x: i32 = 382;
    const fb_start_y: i32 = 296;

    const frame_duration: Duration = Duration::from_micros(1000000 / 25);

    // Get framebuffer and clear it
    let mut fb = core::Framebuffer::new("/dev/fb0");
    fb.clear();
    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_INIT,
        common::display_temp::TEMP_USE_REMARKABLE_DRAW,
        common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );
    thread::sleep_ms(100);

    let (tx, rx): (
        SyncSender<[u8; bytes_per_frame]>,
        Receiver<[u8; bytes_per_frame]>,
    ) = sync_channel(1); // 1 Buffered frame
    let thread_handle = thread::spawn(move || {
        let fb_area = common::mxcfb_rect {
            top: fb_start_y as u32,
            left: fb_start_x as u32,
            width,
            height,
        };

        let full_area = common::mxcfb_rect {
            top: 0,
            left: 0,
            width: common::DISPLAYWIDTH as u32,
            height: common::DISPLAYHEIGHT as u32,
        };

        // Ready
        let mut counter: u64 = 0;
        for buffer in rx {
            counter += 1;

            //let mut token_queue: Vec<u32> = vec![];
            //let token_queue_size = 50;

            let mut buffer_index: usize = 0;
            let mut fb_x: i32 = 0;
            let mut fb_y: i32 = 0;

            while buffer_index < buffer.len() {
                for sub in 0..8 {
                    let is_1 = (buffer[buffer_index] & (0b10000000 >> sub)) > 0;
                    if fb_x >= width as i32 {
                        fb_y += 1;
                        fb_x = 0;
                    }

                    fb.write_pixel(
                        cgmath::Point2 {
                            x: fb_start_x + fb_x,
                            y: fb_start_y + fb_y,
                        },
                        if is_1 {
                            common::color::BLACK
                        } else {
                            common::color::WHITE
                        },
                    );
                    fb_x += 1;
                }

                buffer_index += 1;

                //fb.write_pixel(cgmath::Point2 { x: fb_x*2, y: fb_y*2 }, common::color::GRAY(gray));
            }

            // Toy with this!
            //token_queue.push(
            fb.partial_refresh(
                &full_area,
                refresh::PartialRefreshMode::Async,
                common::waveform_mode::WAVEFORM_MODE_DU,
                common::display_temp::TEMP_USE_REMARKABLE_DRAW,
                common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
                0,
                false,
            );
            //);

            /*if token_queue.len() == token_queue_size {
                let token = token_queue.remove(0);
                fb.wait_refresh_complete(token);
            }*/
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

    let mut last_frame = SystemTime::now();
    while stdin().read_exact(&mut buffer).is_ok() {
        match tx.try_send(buffer) {
            Ok(_) => succeeded += 1,
            Err(_) => dropped += 1,
        }
        let elapsed = last_frame.elapsed().unwrap();
        if frame_duration > elapsed {
            thread::sleep(frame_duration - elapsed);
        }
        last_frame = last_frame.checked_add(frame_duration).unwrap();
    }
    println!(
        "TX: Succeeded: {} frames ; Dropped: {} frames",
        succeeded, dropped
    );
    drop(tx); // Basically end of scope for sender (all senders out of scope = closed)

    thread_handle.join().unwrap();
}
