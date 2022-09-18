;; Example using upkr depacker for screens slideshow
    OPT --syntax=abf
    DEVICE ZXSPECTRUM48,$8FFF

    ORG     $9000
  ;; forward example data
compressed_scr_files.fwd:               ; border color byte + upkr-packed .scr file
    DB      1
    INCBIN  "screens/Grongy - ZX Spectrum (2022).scr.upk"
    DB      7
    INCBIN  "screens/Schafft - Poison (2017).scr.upk"
    DB      0
    INCBIN  "screens/diver - Mercenary 4. The Heaven's Devil (2014) (Forever 2014 Olympic Edition, 1).scr.upk"
    DB      6
    INCBIN  "screens/diver - Back to Bjork (2015).scr.upk"
.e:
  ;; backward example data (unpacker goes from the end of the data!)
compressed_scr_files.rwd.e: EQU $-1     ; the final IX will point one byte ahead of "$" here
    INCBIN  "screens.reversed/diver - Back to Bjork (2015).scr.upk"
    DB      6
    INCBIN  "screens.reversed/diver - Mercenary 4. The Heaven's Devil (2014) (Forever 2014 Olympic Edition, 1).scr.upk"
    DB      0
    INCBIN  "screens.reversed/Schafft - Poison (2017).scr.upk"
    DB      7
    INCBIN  "screens.reversed/Grongy - ZX Spectrum (2022).scr.upk"
compressed_scr_files.rwd:               ; border color byte + upkr-packed .scr file (backward)
    DB      1

start:
    di
;     OPT --zxnext
;     nextreg 7,3                       ; ZX Next: switch to 28Mhz

  ;;; FORWARD packed/unpacked data demo
    ld      ix,compressed_scr_files.fwd
.slideshow_loop.fwd:
  ; set BORDER for next image
    ld      a,(ix)
    inc     ix
    out     (254),a
  ; call unpack of next image directly into VRAM
    ld      de,$4000                    ; target VRAM
    exx
  ; IX = packed data, DE' = destination ($4000)
  ; returned IX will point right after the packed data
    call    fwd.upkr.unpack
  ; do some busy loop with CPU to delay between images
    call    delay
  ; check if all images were displayed, loop around from first one then
    ld      a,ixl
    cp      low compressed_scr_files.fwd.e
    jr      nz,.slideshow_loop.fwd

  ;;; BACKWARD packed/unpacked data demo
    ld      ix,compressed_scr_files.rwd
.slideshow_loop.rwd:
  ; set BORDER for next image
    ld      a,(ix)
    dec     ix
    out     (254),a
  ; call unpack of next image directly into VRAM
    ld      de,$5AFF                    ; target VRAM
    exx
  ; IX = packed data, DE' = destination
  ; returned IX will point right ahead of the packed data
    call    rwd.upkr.unpack
  ; do some busy loop with CPU to delay between images
    call    delay
  ; check if all images were displayed, loop around from first one then
    ld      a,ixl
    cp      low compressed_scr_files.rwd.e
    jr      nz,.slideshow_loop.rwd

    jr      start

delay:
    ld      bc,$AA00
.delay:
    .8 ex      (sp),ix
    dec     c
    jr      nz,.delay
    djnz    .delay
    ret

  ; include the depacker library, optionally putting probs array buffer near end of RAM
    DEFINE  UPKR_PROBS_ORIGIN $FA00   ; if not defined, array will be put after unpack code

    MODULE fwd
        INCLUDE "../unpack.asm"
    ENDMODULE

    MODULE rwd
        DEFINE BACKWARDS_UNPACK         ; defined to build backwards unpack
                ; initial IX points at last byte of compressed data
                ; initial DE' points at last byte of unpacked data

        INCLUDE "../unpack.asm"
    ENDMODULE

    SAVESNA "example.sna",start
