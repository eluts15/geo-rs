### Proof of Concept -- Raspberry PI 2, Model B and GPSD

## Raspberry PI 2, Model B hardware specs
[pin layout](https://www.pi4j.com/1.2/images/j8header-2b.png)
[more info](https://www.pi4j.com/1.2/pins/model-2b-rev1.html)
The PI 2, Model B uses a J8 Header(40-pin, 28 are GPIO pins).


pin1: 3.3v DC power
pin2 & Pin 4: 5v DC power

pin3 & pin5: I2C

pin6: ground

pin8: Tx, UART, GPIO-15
pin10: Rx, UART, GPI-16

pin11: GPIO-0
pin13: GPIO-2
pin15: GPIO-3

## GPS Module (BerryGPS-IMU v4)
[part](https://ozzmaker.com/product/berrygps-imu/)

## Antenna Module (CAM-M8C-0-10)
[part](https://www.mouser.com/ProductDetail/u-blox/CAM-M8C-0?qs=vEM7xhTegWh0Qdx4vzEerw%3D%3D)

## GPS Antenna (ANT-105-SMA )

## Overview of GPSD
[Link to the offical Project Documentation](https://gpsd.io/)

gpsd (GPS service daemon) is a project primarily written in C.
The following is taken directly from the official doc.

```
gpsd is a service daemon that monitors one or more GPSes or AIS receivers attached to a host computer through serial or USB ports,
making all data on the location/course/velocity of the sensors available to be queried on TCP port 2947 of the host computer.
```

There also happens to be a few existing libraries written in Rust:
[gpsd](https://docs.rs/gpsd/latest/gpsd/)
[gpsd_proto](https://docs.rs/gpsd_proto/latest/gpsd_proto/)


## Other
https://ozzmaker.com/forums/topic/nmea-unkown-msg46/
 stty -F /dev/serial0 -echo

## Getting data from /dev/serial0

```
cat /dev/serial0
```

sample output:

```
$GNGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*2E

$GPGSV,1,1,00*79

$GLGSV,1,1,00*65

$GNGLL,,,,,,V,N*7A

$GNRMC,,V,,,,,,,,,,N*4D

$GNVTG,,,,,,,,,N*2E

$GNGGA,,,,,,0,00,99.99,,,,,,*56

$GNGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*2E

$GNGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*2E

$GPGSV,1,1,00*79

$GLGSV,1,1,00*65

$GNGLL,,,,,,V,N*7A
```

## NMEA-0183 Sentences
https://www.rfwireless-world.com/terminology/gps-nmea-sentences

GNGGA: Global positioning system fix data (time, position, fix type data)
Example of GPGGA GPS sentence: 
$GPGGA, 161229.487, 3723.2475, N, 12158.3416, W, 1, 07, 1.0, 9.0, M, , , , 0000*18


GPVTG: Course and speed information relative to the ground
Example of GPVTG GPS sentence: 
$GPVTG, 309.62, T, ,M, 0.13, N, 0.2, K, A*23


GPGSV: The number of GPS satellites in view satellite ID numbers, elevation, azimuth and SNR values.

```
$GPGSV,1,1,01,21,,,09*72                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,01,21,,,08*73                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,01,21,,,12*78                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,01,21,,,19*73                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,01,21,,,18*72                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,01,21,,,16*7C                                                                             
$GLGSV,1,1,00*65                                                                                     
$GPGSV,1,1,00*79
```
