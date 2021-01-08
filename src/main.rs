use clap::{crate_authors, crate_version, Clap};
use libremarkable::framebuffer::*;
use std::io::stdin;
use std::io::Read;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    #[clap(long, short, about = "Width of video (has to be dividable by 8)")]
    width: u16,
    #[clap(long, short, about = "Height of video")]
    height: u16,

    #[clap(short, about = "X Pos")]
    x: u16,
    #[clap(short, about = "Y Pos")]
    y: u16,

    #[clap(long, short, about = "Framerate of the Video")]
    fps: u8,
}

fn main() {
    // Can be done on device with this resolution
    // Convert a video (preferrably 720p directly with -f)
    // youtube-dl https://www.youtube.com/watch\?v\=naTxqt1wVxc -o- | ffmpeg -i - -vf transpose=1,format=monow,scale=-1:1280 -pix_fmt monow -f rawvideo /tmp/myvideo_hopefully_640_1280_25fps.raw
    // Probaby compress this with zstd (about 4x for those videos): zstd -k /tmp/myvideo_hopefully_640_1280_25fps.raw
    // Copy to your remarkable: scp /tmp/myvideo_hopefully_640_1280_25fps.raw.zst remarkable:
    // Play it there (remarkables ssh): zstd -cd myvideo_hopefully_640_1280_25fps.raw.zst | ./rm-video-player # (zstd was installed with opkg)
    //
    // If resolution or framerate don't match. Please adjust below. THE WIDTH HAS TO BE DIVIDABLE BY 8 (or you fix that in the source)

    if libremarkable::device::CURRENT_DEVICE.model != libremarkable::device::Model::Gen1 {
        panic!("Only the reMarkable 1 is supported.");
    }

    let opts: Opts = Opts::parse();

    if opts.width % 8 != 0 {
        panic!("Width has to be dividable by 8!");
    }

    if opts.x + opts.width > common::DISPLAYWIDTH || opts.y + opts.height > common::DISPLAYHEIGHT {
        panic!("Video is not allowed to go out of bounds!");
    }

    let width = opts.width as u32;
    let height = opts.height as u32;
    let fb_start_x: i32 = opts.x as i32;
    let fb_start_y: i32 = opts.y as i32;

    let bytes_per_frame: usize = ((width / 8) * height) as usize;
    let frame_duration: Duration = Duration::from_micros(1000000 / opts.fps as u64);

    // Get framebuffer and clear it
    let mut fb = core::Framebuffer::from_path("/dev/fb0");
    fb.clear();
    fb.full_refresh(
        common::waveform_mode::WAVEFORM_MODE_INIT,
        common::display_temp::TEMP_USE_REMARKABLE_DRAW,
        common::dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );
    thread::sleep(Duration::from_millis(100));

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(1); // 1 Buffered frame
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

            fb.restore_region(
                fb_area,
                &bw_to_fb_data(width as usize, height as usize, &buffer),
            )
            .unwrap();

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

    let mut buffer = vec![0u8; bytes_per_frame];
    let mut succeeded = 0;
    let mut dropped = 0;

    let mut last_frame = SystemTime::now();
    while stdin().read_exact(&mut buffer).is_ok() {
        match tx.try_send(buffer) {
            Ok(_) => succeeded += 1,
            Err(_) => dropped += 1,
        }
        buffer = vec![0u8; bytes_per_frame];

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

/// Converts a BW raw buffer into rM 1 framebuffer data.
/// BW contains one pixel per bit.
/// The reMarkable Framebuffer contains one pixel per 2 byte
/// Using pointers (unsafe) gives a big additional performance boost.
#[inline]
fn bw_to_fb_data(width: usize, height: usize, bw_data: &[u8]) -> Vec<u8> {
    let mut fb_data: Vec<u8> = vec![0u8; (width as usize * 2) * height as usize];
    unsafe {
        let fb_data_ptr = fb_data.as_mut_ptr();
        let mut i = 0;
        for byte in bw_data {
            *fb_data_ptr.add(i + 0) = if byte & 0b10000000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 1) = if byte & 0b10000000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 2) = if byte & 0b01000000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 3) = if byte & 0b01000000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 4) = if byte & 0b00100000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 5) = if byte & 0b00100000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 6) = if byte & 0b00010000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 7) = if byte & 0b00010000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 8) = if byte & 0b00001000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 9) = if byte & 0b00001000 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 10) = if byte & 0b00000100 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 11) = if byte & 0b00000100 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 12) = if byte & 0b00000010 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 13) = if byte & 0b00000010 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 14) = if byte & 0b00000001 == 0 { 0xFF } else { 0x00 };
            *fb_data_ptr.add(i + 15) = if byte & 0b00000001 == 0 { 0xFF } else { 0x00 };
            i += 16;
        }
    }

    fb_data
}
