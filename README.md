# rm-video-player

Just some random PoC of playing some video on the device.

**DISCLAIMER:** Doing that stuff hurts the feelings of the
eink display. After some time it stops working until reboot
or has some nasty "burn ins" for some time (early offical software
could trigger that, too).  
**Proceed on your own risk.**

[Demo videos](https://www.youtube.com/watch?v=JNtU0pDRY98&list=PLiWCGAUWRzf6hZnvxXiJwSw9LSInNvssw&index=2)

A, with opkg installed ffmpeg, is used to decode a H264 Mp4 file
and send it as rawvideo rgb24 to this application.

That punishes the eink display and tries various ways to display
it.
Also see earlier commits for alternative versions.

The program currently has to be compiled for a specific video resolution.

(The ffmpeg command is in the first comment of the main method.)
