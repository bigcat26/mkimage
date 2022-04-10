# mkimage

## brief

A tool for combining binary files in rust language.

## usage

```shell

./mkimage \
    -o image.bin \
    -f 0xff \
    u-boot.bin,0x100000 \
    kernel.bin,0x400000

```
