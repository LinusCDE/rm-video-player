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

Since the reMarkable 2, put the handling of the framebuffer into software, this made initial development really hard. Luckily some [work by amazing people](https://github.com/ddvk/remarkable2-framebuffer/), managed to hook into it, making the framebuffer accessible to every app once again.

Sadly, this project will probably really work on the rM 2 anyway. It can technically work, but the handling in software makes especially, complex and huge updates a lot slower. Playing Videos on the rM 2 may at best get 2-3 fps at some medium size where the rM 1 managed somewhere around 10 FPS and that even without some weird async frame glitches the rM 2 has.

So consider this a rM 1-only project as there would have to be some miracle to work properly. That miracle would most likely be [waved](https://github.com/matteodelabre/waved). But not sure when this will happen. Miracles are rare but not impossible. ;)
