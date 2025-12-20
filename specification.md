## Specification

Given the current GPS location, follow a vector to maintain the current direction.


### TODO
This currently needs be done every time the Pi boots:

```
stty -F /dev/serial0 9600 raw -echo
```

### Methods 

get_current_position()

