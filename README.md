# rm-video-player

[![rm1](https://img.shields.io/badge/rM1-supported-green)](https://remarkable.com/store/remarkable)
[![rm2](https://img.shields.io/badge/rM2-unsupported-red)](https://remarkable.com/store/remarkable-2)

Just some random PoC of playing some video on the device.

**DISCLAIMER:** Doing that stuff may hurt the feelings of the
eink display. It may cause some perceived "burn ins" (usually
temporary) or other damage (escpecially on earlier versions of this sw).
**Proceed on your own risk.**

[Old demo videos](https://www.youtube.com/watch?v=JNtU0pDRY98&list=PLiWCGAUWRzf6hZnvxXiJwSw9LSInNvssw&index=2)


The last commits made video playback seemingly rock solid and nice to look at.  
Performance improvements could still be done (cpu at 100% causes a lot of dropped frames if over 720p).  
See src/main.rs for preparing a video for it.

[Reddit post with a far better demo](https://www.reddit.com/r/RemarkableTablet/comments/j91bsq/we_are_paper_people_again_on_the_remarkable_but/)

## reMarkable 2 support

As of now this sw won't work on the rM 2. For that to happen, libremarkable has to be made compatible first, which will require figuring out the inner workings of the new framebuffer.
