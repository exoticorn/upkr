;; Example using upkr depacker for screens slideshow
    OPT --syntax=abf
    DEVICE ZXSPECTRUM48,$8FFF

    ORG     $9000
compressed_scr_files:       ; border color byte + upkr-packed .scr file
    DB      1
    INCBIN  "screens/Grongy - ZX Spectrum (2022).scr.upk"
    DB      7
    INCBIN  "screens/Schafft - Poison (2017).scr.upk"
    DB      0
    INCBIN  "screens/diver - Mercenary 4. The Heaven's Devil (2014) (Forever 2014 Olympic Edition, 1).scr.upk"
    DB      6
    INCBIN  "screens/diver - Back to Bjork (2015).scr.upk"
.e:

start:
    di
;     OPT --zxnext
;     nextreg 7,3                 ; ZX Next: switch to 28Mhz
    ld      ix,compressed_scr_files
.slideshow_loop
  ; set BORDER for next image
    ldi     a,(ix)              ; fake: ld a,(ix) : inc ix
    out     (254),a
  ; call unpack of next image directly into VRAM
    ld      de,$4000            ; target VRAM
    exx
  ; IX = packed data, DE' = destination ($4000)
  ; returned IX will point right after the packed data
    call    upkr.unpack
  ; do some busy loop with CPU to delay between images
    ld      bc,$AA00
.delay:
    .8 ex      (sp),ix
    dec     c
    jr      nz,.delay
    djnz    .delay
  ; check if all images were displayed, loop around from first one then
    ld      a,ixl
    cp      low compressed_scr_files.e
    jr      z,start
    jr      .slideshow_loop

  ; include the depacker library, optionally putting probs array buffer near end of RAM
    DEFINE  UPKR_PROBS_ORIGIN $FA00   ; if not defined, array will be put after unpack code
    INCLUDE "../unpack.asm"

    SAVESNA "example.sna",start
