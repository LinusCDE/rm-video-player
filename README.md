# rm-video-player

Just some random PoC of playing some video on the device.

**DISCLAIMER:** Doing that stuff hurts the feelings of the
eink display. After some time it stops working until reboot
or has some nasty "burn ins" for some time (early offical software
could trigger that, too).  
**Proceed on your own risk.**

A with opkg installed ffmpeg is used to decode a H264 Mp4 file
and send it as rawvideo rgb24 to this application.

That punishes the eink display and tries various ways to display
it.
Also see earlyer commits for alternative versions.

The program currently has to be compiled for a specific video resolution.
